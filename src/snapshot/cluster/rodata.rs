use crate::diagnostic::{ClutterError, Result};

use super::cid::Profile;
use super::types::ParseResult;

const DATA_ALIGNMENT: u64 = 64;
const MAX_STRING_CODE_UNITS: usize = 16 * 1024 * 1024;

pub fn extract_strings(
    data: &[u8],
    result: &mut ParseResult,
    profile: &Profile,
    snapshot_size: u64,
    pointer_width: usize,
) -> Result<()> {
    let image_start = round_up(snapshot_size, DATA_ALIGNMENT);
    let cluster = result
        .clusters
        .iter()
        .find(|cluster| cluster.cid == profile.cids.string)
        .cloned();
    let Some(cluster) = cluster else {
        return Ok(());
    };

    let mut running_offset = 0u64;
    let mut reference = cluster.start_ref;
    let object_alignment_shift = if pointer_width == 4 { 3 } else { 4 };
    for delta in &cluster.lengths {
        running_offset = running_offset
            .checked_add((*delta as u64) << object_alignment_shift)
            .ok_or_else(|| {
                ClutterError::InvalidArtifact("Dart RO-data string offset overflow".to_owned())
            })?;
        let position = image_start
            .checked_add(running_offset)
            .and_then(|value| usize::try_from(value).ok());
        if let Some(position) = position
            && let Some((cid, length, payload)) = string_header(data, position, pointer_width)
        {
            let one_byte = cid == profile.cids.one_byte_string;
            let two_byte = cid == profile.cids.two_byte_string;
            if (one_byte || two_byte) && length <= MAX_STRING_CODE_UNITS {
                let byte_length = if two_byte {
                    length.saturating_mul(2)
                } else {
                    length
                };
                if let Some(raw) = data.get(payload..payload.saturating_add(byte_length)) {
                    let value = if two_byte {
                        let units = raw
                            .chunks_exact(2)
                            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                            .collect::<Vec<_>>();
                        String::from_utf16_lossy(&units)
                    } else {
                        super::types::decode_one_byte_string(raw)
                    };
                    result.strings.insert(reference, value);
                }
            }
        }
        reference = reference.saturating_add(1);
    }
    Ok(())
}

fn string_header(
    data: &[u8],
    position: usize,
    pointer_width: usize,
) -> Option<(i32, usize, usize)> {
    match pointer_width {
        4 => {
            let tags = u32::from_le_bytes(data.get(position..position + 4)?.try_into().ok()?);
            let length_word =
                u32::from_le_bytes(data.get(position + 8..position + 12)?.try_into().ok()?);
            Some((
                ((tags >> 12) & 0x000f_ffff) as i32,
                (length_word >> 1) as usize,
                position + 12,
            ))
        }
        8 => {
            let tags = u64::from_le_bytes(data.get(position..position + 8)?.try_into().ok()?);
            let length_word =
                u64::from_le_bytes(data.get(position + 8..position + 16)?.try_into().ok()?);
            Some((
                ((tags >> 16) & 0xffff) as i32,
                usize::try_from(length_word >> 1).ok()?,
                position + 16,
            ))
        }
        _ => None,
    }
}

fn round_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment).saturating_mul(alignment)
}
