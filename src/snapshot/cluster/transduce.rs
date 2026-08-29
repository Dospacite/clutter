use std::collections::{BTreeSet, VecDeque};
use std::io::Read;

use flate2::read::{GzDecoder, ZlibDecoder};

use crate::model::{EvidenceConfidence, RecoveredString, RecoveredStringSource};

use super::types::{ParseResult, SnapshotObjectKind, SnapshotScalar};

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_XOR_BYTES: usize = 16 * 1024;
const MAX_TRANSFORM_DEPTH: usize = 3;
const MAX_STATES_PER_OBJECT: usize = 12;
const MIN_TEXT_SCORE: i32 = 10;
const MIN_XOR_SCORE: i32 = 24;

#[derive(Clone)]
struct State {
    bytes: Vec<u8>,
    transforms: Vec<String>,
}

pub fn recover(snapshot: &ParseResult, partition: &str) -> Vec<RecoveredString> {
    let mut recovered = snapshot
        .strings
        .iter()
        .filter_map(|(reference, value)| {
            // P4: huge control-char allocations (e.g. obfuscated `allocateOneByteString`
            // noise) otherwise pollute `lib/main.dart` with inline literals. Keep
            // bounded human-readable heap strings; the raw bytes stay reachable
            // via `aot.snapshotRef` and `reports/coverage.json`.
            if value.len() > 1024 || value.contains('\0') {
                return None;
            }
            if value
                .chars()
                .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
                && text_score(value) < MIN_TEXT_SCORE
            {
                return None;
            }
            Some(RecoveredString {
                value: value.clone(),
                source: RecoveredStringSource::SnapshotObject,
                file_offset: None,
                snapshot_reference: Some(format!("{partition}:{reference}")),
                transform: None,
                confidence: None,
            })
        })
        .collect::<Vec<_>>();

    for (reference, value) in &snapshot.strings {
        recover_carrier(
            value.as_bytes(),
            Carrier::EncodedString,
            &format!("{partition}:{reference}"),
            &mut recovered,
        );
    }

    for object in &snapshot.objects {
        if object.kind != SnapshotObjectKind::TypedData
            || typed_data_element_size(snapshot.scalars_of(object)) != Some(1)
        {
            continue;
        }
        recover_carrier(
            snapshot.bytes_of(object),
            Carrier::TypedData,
            &format!("{partition}:{}", object.reference),
            &mut recovered,
        );
    }

    recovered.sort_by(|left, right| {
        left.value
            .cmp(&right.value)
            .then(left.source.cmp(&right.source))
            .then(left.snapshot_reference.cmp(&right.snapshot_reference))
    });
    recovered.dedup_by(|left, right| left.value == right.value);
    recovered
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Carrier {
    EncodedString,
    TypedData,
}

fn typed_data_element_size(scalars: &[SnapshotScalar]) -> Option<usize> {
    match scalars.get(1) {
        Some(SnapshotScalar::Unsigned(value)) => usize::try_from(*value).ok(),
        _ => None,
    }
}

fn recover_carrier(
    input: &[u8],
    carrier: Carrier,
    reference: &str,
    recovered: &mut Vec<RecoveredString>,
) {
    if !(4..=MAX_INPUT_BYTES).contains(&input.len()) {
        return;
    }

    let mut pending = VecDeque::from([State {
        bytes: input.to_vec(),
        transforms: Vec::new(),
    }]);
    let mut visited = BTreeSet::from([input.to_vec()]);
    let mut processed = 0usize;

    while let Some(state) = pending.pop_front() {
        if processed >= MAX_STATES_PER_OBJECT {
            break;
        }
        processed += 1;

        let may_emit_direct = carrier == Carrier::TypedData || !state.transforms.is_empty();
        let mut has_direct_text = false;
        if may_emit_direct {
            if let Some(value) = utf8_text(&state.bytes) {
                has_direct_text = true;
                emit(
                    value,
                    carrier,
                    reference,
                    with_decoder(&state.transforms, "utf8"),
                    EvidenceConfidence::High,
                    recovered,
                );
            }
            if let Some((value, encoding)) = utf16_text(&state.bytes) {
                has_direct_text = true;
                emit(
                    value,
                    carrier,
                    reference,
                    with_decoder(&state.transforms, encoding),
                    EvidenceConfidence::High,
                    recovered,
                );
            }
        }

        // A normal Dart String is already plaintext. Trying all XOR keys
        // against every literal manufactures case/punctuation variants. An
        // encoded String becomes eligible only after a structural decoder
        // (for example Base64) has exposed an opaque byte payload.
        if !has_direct_text
            && (carrier == Carrier::TypedData || !state.transforms.is_empty())
            && state.transforms.len() <= 1
            && let Some((key, value)) = best_single_byte_xor(&state.bytes)
        {
            let mut transforms = state.transforms.clone();
            transforms.push(format!("xor(key=0x{key:02x})"));
            transforms.push("utf8".to_owned());
            emit(
                value,
                carrier,
                reference,
                transforms.join(" -> "),
                EvidenceConfidence::Medium,
                recovered,
            );
        }

        if state.transforms.len() >= MAX_TRANSFORM_DEPTH {
            continue;
        }
        for (name, bytes) in reversible_transforms(&state.bytes) {
            if bytes.is_empty() || bytes.len() > MAX_OUTPUT_BYTES || !visited.insert(bytes.clone())
            {
                continue;
            }
            let mut transforms = state.transforms.clone();
            transforms.push(name.to_owned());
            pending.push_back(State { bytes, transforms });
        }
    }
}

fn reversible_transforms(bytes: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let mut output = Vec::with_capacity(3);
    if let Some(decoded) = decode_hex(bytes) {
        output.push(("hex", decoded));
    }
    if let Some(decoded) = decode_base64(bytes) {
        output.push(("base64", decoded));
    }
    if bytes.starts_with(&[0x1f, 0x8b])
        && let Some(decoded) = decompress_gzip(bytes)
    {
        output.push(("gzip", decoded));
    } else if looks_like_zlib(bytes)
        && let Some(decoded) = decompress_zlib(bytes)
    {
        output.push(("zlib", decoded));
    }
    output
}

fn emit(
    value: String,
    carrier: Carrier,
    reference: &str,
    transform: String,
    confidence: EvidenceConfidence,
    recovered: &mut Vec<RecoveredString>,
) {
    if text_score(&value) < MIN_TEXT_SCORE {
        return;
    }
    let source = if carrier == Carrier::TypedData
        && !transform.contains(" -> ")
        && matches!(transform.as_str(), "utf8" | "utf16le" | "utf16be")
    {
        RecoveredStringSource::SnapshotTypedData
    } else {
        RecoveredStringSource::SnapshotTransduction
    };
    recovered.push(RecoveredString {
        value,
        source,
        file_offset: None,
        snapshot_reference: Some(reference.to_owned()),
        transform: Some(transform),
        confidence: Some(confidence),
    });
}

fn with_decoder(transforms: &[String], decoder: &str) -> String {
    if transforms.is_empty() {
        decoder.to_owned()
    } else {
        format!("{} -> {decoder}", transforms.join(" -> "))
    }
}

fn utf8_text(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes)
        .ok()
        .filter(|value| text_score(value) >= MIN_TEXT_SCORE)
        .map(str::to_owned)
}

fn utf16_text(bytes: &[u8]) -> Option<(String, &'static str)> {
    if bytes.len() < 8 || bytes.len() % 2 != 0 {
        return None;
    }
    let pair_count = bytes.len() / 2;
    let zero_even = bytes.iter().step_by(2).filter(|byte| **byte == 0).count();
    let zero_odd = bytes
        .iter()
        .skip(1)
        .step_by(2)
        .filter(|byte| **byte == 0)
        .count();
    let has_le_bom = bytes.starts_with(&[0xff, 0xfe]);
    let has_be_bom = bytes.starts_with(&[0xfe, 0xff]);
    let little_endian = if has_le_bom || zero_odd * 4 >= pair_count {
        true
    } else if has_be_bom || zero_even * 4 >= pair_count {
        false
    } else {
        return None;
    };
    let start = usize::from(has_le_bom || has_be_bom) * 2;
    let units = bytes[start..].chunks_exact(2).map(|pair| {
        if little_endian {
            u16::from_le_bytes([pair[0], pair[1]])
        } else {
            u16::from_be_bytes([pair[0], pair[1]])
        }
    });
    let value = char::decode_utf16(units)
        .collect::<Result<String, _>>()
        .ok()?;
    (text_score(&value) >= MIN_TEXT_SCORE)
        .then_some((value, if little_endian { "utf16le" } else { "utf16be" }))
}

fn decode_hex(bytes: &[u8]) -> Option<Vec<u8>> {
    (bytes.len() >= 8 && bytes.len() % 2 == 0 && bytes.iter().all(u8::is_ascii_hexdigit))
        .then(|| hex::decode(bytes).ok())
        .flatten()
}

fn decode_base64(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 8 || bytes.len() > MAX_INPUT_BYTES || !bytes.is_ascii() {
        return None;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if padding > 2 || bytes[..bytes.len().saturating_sub(padding)].contains(&b'=') {
        return None;
    }
    let data_len = bytes.len().checked_sub(padding)?;
    let has_encoding_evidence = padding > 0
        || bytes[..data_len]
            .iter()
            .any(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'/'));
    if !has_encoding_evidence {
        // Ordinary camelCase and snake_case identifiers are valid members of
        // the unpadded Base64 alphabet surprisingly often. Requiring a digit,
        // standard-alphabet marker, or padding prevents those names from
        // becoming decoder/XOR false positives.
        return None;
    }
    if (padding > 0 && bytes.len() % 4 != 0)
        || (padding == 1 && data_len % 4 != 3)
        || (padding == 2 && data_len % 4 != 2)
        || (padding == 0 && data_len % 4 == 1)
    {
        return None;
    }

    let mut accumulator = 0u32;
    let mut bit_count = 0u8;
    let mut output = Vec::with_capacity(data_len.saturating_mul(3) / 4);
    for byte in &bytes[..data_len] {
        let value = base64_value(*byte)?;
        accumulator = (accumulator << 6) | u32::from(value);
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            output.push(((accumulator >> bit_count) & 0xff) as u8);
            accumulator &= if bit_count == 0 {
                0
            } else {
                (1u32 << bit_count) - 1
            };
        }
    }
    (accumulator == 0 && !output.is_empty()).then_some(output)
}

fn base64_value(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

fn looks_like_zlib(bytes: &[u8]) -> bool {
    let Some((&cmf, rest)) = bytes.split_first() else {
        return false;
    };
    let Some(&flags) = rest.first() else {
        return false;
    };
    cmf & 0x0f == 8 && (u16::from(cmf) * 256 + u16::from(flags)) % 31 == 0
}

fn decompress_gzip(bytes: &[u8]) -> Option<Vec<u8>> {
    read_bounded(GzDecoder::new(bytes))
}

fn decompress_zlib(bytes: &[u8]) -> Option<Vec<u8>> {
    read_bounded(ZlibDecoder::new(bytes))
}

fn read_bounded(reader: impl Read) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    reader
        .take((MAX_OUTPUT_BYTES + 1) as u64)
        .read_to_end(&mut output)
        .ok()?;
    (output.len() <= MAX_OUTPUT_BYTES).then_some(output)
}

fn best_single_byte_xor(bytes: &[u8]) -> Option<(u8, String)> {
    if !(8..=MAX_XOR_BYTES).contains(&bytes.len()) {
        return None;
    }
    let mut best: Option<(i32, u8, String)> = None;
    for key in 1..=u8::MAX {
        let decoded = bytes.iter().map(|byte| byte ^ key).collect::<Vec<_>>();
        let Ok(value) = std::str::from_utf8(&decoded) else {
            continue;
        };
        let score = text_score(value);
        if score < MIN_XOR_SCORE || !strong_xor_shape(value) {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|(best_score, _, _)| score > *best_score)
        {
            best = Some((score, key, value.to_owned()));
        }
    }
    best.map(|(_, key, value)| (key, value))
}

fn strong_xor_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.chars().any(char::is_whitespace)
        || has_structural_marker(value)
        || [
            "http", "api", "token", "secret", "error", "message", "user", "pass", "flutter",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn text_score(value: &str) -> i32 {
    let count = value.chars().count();
    if !(4..=MAX_OUTPUT_BYTES).contains(&count)
        || value.contains('\0')
        || value.contains(char::REPLACEMENT_CHARACTER)
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return i32::MIN;
    }

    let alphabetic = value.chars().filter(|value| value.is_alphabetic()).count();
    let digits = value.chars().filter(|value| value.is_ascii_digit()).count();
    let whitespace = value.chars().filter(|value| value.is_whitespace()).count();
    let unique = value.chars().collect::<BTreeSet<_>>().len();
    let structured = has_structural_marker(value);
    let mostly_words = alphabetic.saturating_mul(2) >= count;
    if value
        .chars()
        .any(|character| !is_common_text_character(character))
        || alphabetic < 4
        || unique < 3
        || (count >= 16 && unique.saturating_mul(8) < count)
        || !(whitespace > 0 || structured || (count >= 5 && mostly_words))
    {
        return i32::MIN;
    }

    let mut score =
        alphabetic.min(20) as i32 + digits.min(5) as i32 + (whitespace.min(8) * 2) as i32;
    if count >= 8 {
        score += 2;
    }
    if mostly_words {
        score += 5;
    }
    if structured {
        score += 18;
    }
    let lower = value.to_ascii_lowercase();
    if [
        "http", "api", "token", "secret", "error", "message", "user", "pass", "flutter",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        score += 10;
    }
    score
}

fn has_structural_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ws://")
        || lower.starts_with("wss://")
        || lower.contains("://")
        || (value.starts_with('{') && value.ends_with('}'))
        || (value.starts_with('[') && value.ends_with(']'))
        || looks_like_path(value)
        || (value.contains('@') && value.contains('.'))
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/')
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(
                    character,
                    '/' | '\\' | '.' | '_' | '-' | ':' | '?' | '&' | '=' | '%' | '#'
                )
        })
        && (value.starts_with('/')
            || value.contains("./")
            || value
                .split('/')
                .any(|segment| segment.contains('.') && segment.len() >= 3))
}

fn is_common_text_character(value: char) -> bool {
    value.is_ascii_graphic()
        || value.is_ascii_whitespace()
        || matches!(
            value as u32,
            0x00a0..=0x024f // Latin supplements and extensions
                | 0x0370..=0x052f // Greek and Cyrillic
                | 0x0590..=0x08ff // Hebrew and Arabic
                | 0x0900..=0x097f // Devanagari
                | 0x2000..=0x206f // general punctuation
                | 0x3000..=0x30ff // CJK punctuation, Hiragana, Katakana
                | 0x3400..=0x9fff // CJK ideographs
                | 0xac00..=0xd7af // Hangul
        )
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::{GzEncoder, ZlibEncoder};

    use super::{decode_base64, recover};
    use crate::model::RecoveredStringSource;
    use crate::snapshot::cluster::types::{
        ClusterHeader, ParseResult, SnapshotObjectKind, SnapshotObjectPayload, SnapshotScalar,
    };

    fn empty_snapshot() -> ParseResult {
        ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        })
    }

    fn insert_bytes(snapshot: &mut ParseResult, reference: i32, bytes: Vec<u8>) {
        snapshot.insert_object(
            reference,
            100,
            false,
            SnapshotObjectKind::TypedData,
            SnapshotObjectPayload {
                references: Vec::new(),
                scalars: vec![
                    SnapshotScalar::Unsigned(bytes.len() as i64),
                    SnapshotScalar::Unsigned(1),
                ],
                bytes,
            },
        );
    }

    #[test]
    fn decodes_standard_and_url_safe_base64_without_a_runtime() {
        assert_eq!(
            decode_base64(b"aHR0cHM6Ly9leGFtcGxlLmludmFsaWQvYXBp").unwrap(),
            b"https://example.invalid/api"
        );
        assert_eq!(decode_base64(b"YWJjZA==").unwrap(), b"abcd");
        assert!(decode_base64(b"invalid=").is_none());
        assert!(decode_base64(b"didGainFocus").is_none());
        assert!(decode_base64(b"version_name").is_none());
    }

    #[test]
    fn recovers_encoded_strings_and_records_the_transform() {
        let mut snapshot = empty_snapshot();
        snapshot
            .strings
            .insert(7, "aHR0cHM6Ly9leGFtcGxlLmludmFsaWQvYXBp".to_owned());
        let recovered = recover(&snapshot, "isolate");
        let value = recovered
            .iter()
            .find(|value| value.value == "https://example.invalid/api")
            .unwrap();
        assert_eq!(value.source, RecoveredStringSource::SnapshotTransduction);
        assert_eq!(value.snapshot_reference.as_deref(), Some("isolate:7"));
        assert_eq!(value.transform.as_deref(), Some("base64 -> utf8"));
    }

    #[test]
    fn recovers_gzip_and_single_byte_xor_typed_data() {
        let mut snapshot = empty_snapshot();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"compressed recovery message").unwrap();
        insert_bytes(&mut snapshot, 10, encoder.finish().unwrap());

        let key = 0x5a;
        insert_bytes(
            &mut snapshot,
            11,
            b"service endpoint disabled"
                .iter()
                .map(|byte| byte ^ key)
                .collect(),
        );

        let recovered = recover(&snapshot, "isolate");
        assert!(recovered.iter().any(|value| {
            value.value == "compressed recovery message"
                && value.transform.as_deref() == Some("gzip -> utf8")
        }));
        assert!(recovered.iter().any(|value| {
            value.value == "service endpoint disabled"
                && value.transform.as_deref() == Some("xor(key=0x5a) -> utf8")
        }));
    }

    #[test]
    fn recovers_utf16_hex_and_zlib_carriers() {
        let mut snapshot = empty_snapshot();
        let utf16 = "typed data recovery message"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        insert_bytes(&mut snapshot, 20, utf16);
        snapshot
            .strings
            .insert(21, hex::encode("hex recovery message"));

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"zlib recovery message").unwrap();
        insert_bytes(&mut snapshot, 22, encoder.finish().unwrap());

        let recovered = recover(&snapshot, "isolate");
        assert!(recovered.iter().any(|value| {
            value.value == "typed data recovery message"
                && value.transform.as_deref() == Some("utf16le")
        }));
        assert!(recovered.iter().any(|value| {
            value.value == "hex recovery message"
                && value.transform.as_deref() == Some("hex -> utf8")
        }));
        assert!(recovered.iter().any(|value| {
            value.value == "zlib recovery message"
                && value.transform.as_deref() == Some("zlib -> utf8")
        }));
    }

    #[test]
    fn ignores_binary_tables_that_do_not_have_strong_text_evidence() {
        let mut snapshot = empty_snapshot();
        insert_bytes(&mut snapshot, 12, (0..=u8::MAX).collect());
        assert!(
            recover(&snapshot, "isolate")
                .iter()
                .all(|value| value.snapshot_reference.as_deref() != Some("isolate:12"))
        );
    }

    #[test]
    fn does_not_xor_plain_snapshot_strings_into_false_variants() {
        let mut snapshot = empty_snapshot();
        snapshot
            .strings
            .insert(13, "ordinary service error message".to_owned());
        let recovered = recover(&snapshot, "isolate")
            .into_iter()
            .filter(|value| value.snapshot_reference.as_deref() == Some("isolate:13"))
            .collect::<Vec<_>>();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].source, RecoveredStringSource::SnapshotObject);
    }

    #[test]
    fn does_not_xor_plain_typed_data_after_direct_decoding() {
        let mut snapshot = empty_snapshot();
        insert_bytes(&mut snapshot, 14, b"ordinary typed data message".to_vec());
        let recovered = recover(&snapshot, "isolate")
            .into_iter()
            .filter(|value| value.snapshot_reference.as_deref() == Some("isolate:14"))
            .collect::<Vec<_>>();
        assert_eq!(recovered.len(), 1);
        assert_eq!(
            recovered[0].source,
            RecoveredStringSource::SnapshotTypedData
        );
        assert_eq!(recovered[0].transform.as_deref(), Some("utf8"));
    }
}
