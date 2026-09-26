//! Decodes AOT catch-entry move maps (`Code::catch_entry_moves_maps`).
//!
//! When an exception reaches a handler, the VM copies each value the catch
//! block needs from wherever the throwing site kept it into the frame slot
//! the catch block reads (`ExecuteCatchEntryMoves` in exceptions.cc). The
//! compiler records one move list per call site inside a try, keyed by the
//! call's return-address pc offset (`FlowGraphCompiler::RecordCatchEntryMoves`),
//! and `CatchEntryMovesMapBuilder` shares common suffixes between lists.
//!
//! The stream is written with `BaseWriteStream::Raw`: each entry header is
//! `pc_offset, prefix_length, suffix_length, suffix_offset` as
//! target-`intptr_t` variable-length values, followed by the prefix moves
//! written from the back, each `src, dest_and_kind` as `int32_t` values.

use crate::model::{CatchEntryMove, CatchEntryMoves, CatchMoveSource};

use super::reader::Reader;

/// Entries in a single map are bounded by the calls in one function; the cap
/// only rejects corrupt input.
const MAX_ENTRIES: usize = 1 << 16;
const MAX_MOVES: usize = 1 << 12;

struct Header {
    position: usize,
    pc_offset: i64,
    suffix_length: usize,
    /// Stream position of the entry holding the shared suffix; `-1` when
    /// the entry has none.
    suffix_offset: i64,
    prefix: Vec<(i32, i32)>,
}

/// Decodes every entry of a move map. `native_word` is the target's
/// `intptr_t` width in bytes. Returns `None` for a stream that does not
/// follow the format exactly.
pub(super) fn decode(bytes: &[u8], native_word: i64) -> Option<Vec<CatchEntryMoves>> {
    let mut reader = Reader::at(bytes, 0).ok()?;
    let word = |reader: &mut Reader<'_>| -> Option<i64> {
        if native_word == 4 {
            reader.i32().ok().map(i64::from)
        } else {
            reader.tagged64().ok()
        }
    };
    let mut headers = Vec::new();
    while reader.position() < bytes.len() {
        if headers.len() >= MAX_ENTRIES {
            return None;
        }
        let position = reader.position();
        let pc_offset = word(&mut reader)?;
        let prefix_length = usize::try_from(word(&mut reader)?).ok()?;
        let suffix_length = usize::try_from(word(&mut reader)?).ok()?;
        let suffix_offset = word(&mut reader)?;
        if prefix_length > MAX_MOVES || suffix_length > MAX_MOVES {
            return None;
        }
        let mut prefix = Vec::with_capacity(prefix_length);
        for _ in 0..prefix_length {
            prefix.push((reader.i32().ok()?, reader.i32().ok()?));
        }
        headers.push(Header {
            position,
            pc_offset,
            suffix_length,
            suffix_offset,
            prefix,
        });
    }
    let by_position = headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header.position, index))
        .collect::<std::collections::BTreeMap<_, _>>();
    headers
        .iter()
        .map(|header| {
            // ReadCompressedCatchEntryMovesSuffix: this entry's prefix (stored
            // back to front) followed by the moves of the entry at
            // `suffix_offset`, recursively, until `suffix_length` is covered.
            let total = header.prefix.len().checked_add(header.suffix_length)?;
            let mut raw = Vec::with_capacity(total);
            let mut current = header;
            let mut remaining = total;
            loop {
                let own = remaining.checked_sub(current.suffix_length)?;
                if own > current.prefix.len() {
                    return None;
                }
                raw.extend(current.prefix[..own].iter().rev().copied());
                remaining = current.suffix_length;
                if remaining == 0 {
                    break;
                }
                let position = usize::try_from(current.suffix_offset).ok()?;
                current = &headers[*by_position.get(&position)?];
            }
            Some(CatchEntryMoves {
                pc_offset: u32::try_from(header.pc_offset).ok()?,
                moves: raw
                    .into_iter()
                    .map(|(source, destination_and_kind)| move_of(source, destination_and_kind))
                    .collect::<Option<Vec<_>>>()?,
            })
        })
        .collect()
}

fn move_of(source: i32, destination_and_kind: i32) -> Option<CatchEntryMove> {
    let kind = match destination_and_kind & 0xf {
        0 => CatchMoveSource::Constant,
        1 => CatchMoveSource::Tagged,
        2 => CatchMoveSource::Float,
        3 => CatchMoveSource::Double,
        4 => CatchMoveSource::Float32x4,
        5 => CatchMoveSource::Float64x2,
        6 => CatchMoveSource::Int32x4,
        7 => CatchMoveSource::Int64Pair,
        8 => CatchMoveSource::Int64,
        9 => CatchMoveSource::Int32,
        10 => CatchMoveSource::Uint32,
        _ => return None,
    };
    // `index_to_pair_slot`: odd indices are nonnegative slots.
    let pair_slot = |index: i32| {
        if index & 1 != 0 {
            index >> 1
        } else {
            -(index >> 1)
        }
    };
    let (source, source_high) = if kind == CatchMoveSource::Int64Pair {
        (pair_slot(source & 0xffff), Some(pair_slot((source >> 16) & 0xffff)))
    } else {
        (source, None)
    };
    Some(CatchEntryMove {
        kind,
        source,
        source_high,
        destination: destination_and_kind >> 4,
    })
}

#[cfg(test)]
mod tests {
    use super::decode;
    use crate::model::CatchMoveSource;

    /// Dart's signed variable-length encoding (`Write<T>` with the
    /// `kEndByteMarker` of 192).
    fn write(output: &mut Vec<u8>, mut value: i64) {
        while !(-64..=63).contains(&value) {
            output.push((value & 0x7f) as u8);
            value >>= 7;
        }
        output.push((value + 192) as u8);
    }

    fn entry(output: &mut Vec<u8>, header: [i64; 4], prefix: &[(i64, i64)]) {
        for value in header {
            write(output, value);
        }
        for (source, destination_and_kind) in prefix {
            write(output, *source);
            write(output, *destination_and_kind);
        }
    }

    #[test]
    fn decodes_the_empty_move_list_the_arm64_fixture_uses() {
        // pc 0xb8, no prefix, no suffix, suffix offset -1.
        let decoded = decode(&[0x38, 0xc1, 0xc0, 0xc0, 0xbf], 8).expect("well formed");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].pc_offset, 0xb8);
        assert!(decoded[0].moves.is_empty());
    }

    #[test]
    fn shared_suffixes_expand_in_order() {
        // Entry A at pc 0x40: one own move (tagged slot 2 -> slot 5).
        // Entry B at pc 0x80: its own move (constant pool[7] -> slot 6)
        // then A's moves as a shared suffix.
        let mut bytes = Vec::new();
        entry(&mut bytes, [0x40, 1, 0, -1], &[(2, (5 << 4) | 1)]);
        entry(&mut bytes, [0x80, 1, 1, 0], &[(7, 6 << 4)]);
        let decoded = decode(&bytes, 8).expect("well formed");
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].pc_offset, 0x40);
        assert_eq!(decoded[0].moves.len(), 1);
        assert_eq!(decoded[0].moves[0].kind, CatchMoveSource::Tagged);
        assert_eq!((decoded[0].moves[0].source, decoded[0].moves[0].destination), (2, 5));
        let b = &decoded[1];
        assert_eq!(b.pc_offset, 0x80);
        assert_eq!(b.moves.len(), 2);
        assert_eq!(b.moves[0].kind, CatchMoveSource::Constant);
        assert_eq!((b.moves[0].source, b.moves[0].destination), (7, 6));
        assert_eq!((b.moves[1].source, b.moves[1].destination), (2, 5));
    }

    #[test]
    fn prefixes_are_stored_back_to_front_and_pairs_split() {
        let mut bytes = Vec::new();
        // Written order is reversed: the first move is last in the stream.
        let pair = (3 << 16) | 4; // hi index 3 -> slot 1, lo index 4 -> slot -2
        entry(&mut bytes, [0x10, 2, 0, -1], &[(pair, (9 << 4) | 7), (1, (8 << 4) | 3)]);
        let decoded = decode(&bytes, 4).expect("well formed");
        let moves = &decoded[0].moves;
        assert_eq!(moves[0].kind, CatchMoveSource::Double);
        assert_eq!((moves[0].source, moves[0].destination), (1, 8));
        assert_eq!(moves[1].kind, CatchMoveSource::Int64Pair);
        assert_eq!((moves[1].source, moves[1].source_high), (-2, Some(1)));
        assert_eq!(moves[1].destination, 9);
    }

    #[test]
    fn rejects_dangling_suffixes_and_unknown_kinds() {
        let mut bytes = Vec::new();
        entry(&mut bytes, [0x10, 0, 1, 99], &[]);
        assert!(decode(&bytes, 8).is_none());
        let mut bytes = Vec::new();
        entry(&mut bytes, [0x10, 1, 0, -1], &[(1, (2 << 4) | 15)]);
        assert!(decode(&bytes, 8).is_none());
    }
}
