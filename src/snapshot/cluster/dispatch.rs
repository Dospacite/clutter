use super::reader::Reader;

const RECENT_COUNT: usize = 64;
const MAX_REPEAT: i64 = 63;
const INDEX_BASE: i64 = 64;
const MAX_DISPATCH_ENTRIES: usize = 20_000_000;

/// Locate and decode the dispatch table at the end of the snapshot roots.
///
/// Root-list sizes are VM-build dependent. Instead of copying those unstable
/// counts, search the post-cluster tail for the uniquely constrained dispatch
/// stream: it must contain the Code cluster's first reference, decode exactly
/// the declared number of entries, and end exactly at the snapshot boundary.
pub fn find_table(
    data: &[u8],
    fill_end: usize,
    snapshot_end: usize,
    first_code_reference: i32,
) -> Vec<Option<usize>> {
    if fill_end >= snapshot_end || snapshot_end > data.len() || first_code_reference <= 0 {
        return Vec::new();
    }
    for start in (fill_end..snapshot_end).rev() {
        if let Some(table) =
            decode_candidate(data, start, snapshot_end, first_code_reference as i64)
        {
            return table;
        }
    }
    Vec::new()
}

fn decode_candidate(
    data: &[u8],
    start: usize,
    end: usize,
    expected_first_code_reference: i64,
) -> Option<Vec<Option<usize>>> {
    let mut reader = Reader::at(data.get(..end)?, start).ok()?;
    let length = usize::try_from(reader.unsigned().ok()?).ok()?;
    if length == 0 || length > MAX_DISPATCH_ENTRIES {
        return None;
    }
    if reader.unsigned().ok()? != expected_first_code_reference {
        return None;
    }
    // A one-byte repeat can expand to at most 63 table entries.
    let remaining = end.saturating_sub(reader.position());
    if length
        > remaining
            .saturating_mul(MAX_REPEAT as usize)
            .saturating_add(1)
    {
        return None;
    }

    let mut table = Vec::with_capacity(length);
    let mut recent = [None; RECENT_COUNT];
    let mut recent_index = 0usize;
    let mut value = None;
    let mut repeat_count = 0usize;
    while table.len() < length {
        if repeat_count > 0 {
            table.push(value);
            repeat_count -= 1;
            continue;
        }
        let encoded = reader.tagged64().ok()?;
        if encoded == 0 {
            value = None;
        } else if encoded < 0 {
            let recent_slot = usize::try_from(!encoded).ok()?;
            value = *recent.get(recent_slot)?;
            value?;
        } else if encoded <= MAX_REPEAT {
            repeat_count = usize::try_from(encoded - 1).ok()?;
        } else {
            let code_index = usize::try_from(encoded - INDEX_BASE).ok()?;
            value = Some(code_index);
            recent[recent_index] = value;
            recent_index = (recent_index + 1) & (RECENT_COUNT - 1);
        }
        table.push(value);
    }
    (repeat_count == 0 && reader.position() == end).then_some(table)
}

#[cfg(test)]
mod tests {
    use super::find_table;

    fn write_unsigned(output: &mut Vec<u8>, mut value: i64) {
        while value > 127 {
            output.push((value as u8) & 0x7f);
            value >>= 7;
        }
        output.push((value + 128) as u8);
    }

    fn write_signed(output: &mut Vec<u8>, mut value: i64) {
        while !(-64..=63).contains(&value) {
            output.push((value as u8) & 0x7f);
            value >>= 7;
        }
        output.push((value + 192) as u8);
    }

    #[test]
    fn finds_repeat_recent_and_null_dispatch_entries() {
        let mut bytes = vec![0x11, 0x22, 0x33];
        write_unsigned(&mut bytes, 7);
        write_unsigned(&mut bytes, 42);
        write_signed(&mut bytes, 65); // Code index 1.
        write_signed(&mut bytes, 2); // Two more copies.
        write_signed(&mut bytes, 0); // Null.
        write_signed(&mut bytes, 1); // One more null.
        write_signed(&mut bytes, 66); // Code index 2.
        write_signed(&mut bytes, -1); // Recent slot zero: Code index 1.

        assert_eq!(
            find_table(&bytes, 0, bytes.len(), 42),
            vec![Some(1), Some(1), Some(1), None, None, Some(2), Some(1)]
        );
    }

    #[test]
    fn rejects_a_tail_for_a_different_code_cluster() {
        let mut bytes = Vec::new();
        write_unsigned(&mut bytes, 1);
        write_unsigned(&mut bytes, 41);
        write_signed(&mut bytes, 65);
        assert!(find_table(&bytes, 0, bytes.len(), 42).is_empty());
    }
}
