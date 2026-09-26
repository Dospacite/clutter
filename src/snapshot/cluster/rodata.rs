use crate::diagnostic::{ClutterError, Result};

use super::cid::Profile;
use super::types::{ParseResult, SnapshotObjectKind};

const DATA_ALIGNMENT: u64 = 64;
const MAX_STRING_CODE_UNITS: usize = 16 * 1024 * 1024;

pub fn extract_objects(
    data: &[u8],
    result: &mut ParseResult,
    profile: &Profile,
    snapshot_size: u64,
    pointer_width: usize,
) -> Result<()> {
    let image_start = round_up(snapshot_size, DATA_ALIGNMENT);
    let object_alignment_shift = if pointer_width == 4 { 3 } else { 4 };
    for cluster in result.clusters.clone() {
        let is_string = cluster.cid == profile.cids.string;
        let is_metadata = matches!(cluster.cid, cid if cid == profile.cids.pc_descriptors
            || cid == profile.cids.code_source_map
            || cid == profile.cids.compressed_stack_maps);
        if !is_string && !is_metadata {
            continue;
        }
        let mut running_offset = 0u64;
        for (index, delta) in cluster.lengths.iter().enumerate() {
            running_offset = running_offset
                .checked_add((*delta as u64) << object_alignment_shift)
                .ok_or_else(|| {
                    ClutterError::InvalidArtifact("Dart RO-data object offset overflow".to_owned())
                })?;
            let Some(position) = image_start
                .checked_add(running_offset)
                .and_then(|value| usize::try_from(value).ok())
            else {
                continue;
            };
            let reference = cluster.start_ref.saturating_add(index as i32);
            if is_string {
                if let Some((cid, length, payload)) = string_header(data, position, pointer_width) {
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
                continue;
            }
            let Some((bytes, header_value)) = metadata_bytes(
                data,
                position,
                pointer_width,
                cluster.cid,
                cluster.cid == profile.cids.compressed_stack_maps,
            ) else {
                continue;
            };
            if let Ok(object_index) = result
                .objects
                .binary_search_by_key(&reference, |object| object.reference)
            {
                let start = result.object_bytes.len();
                result.object_bytes.extend_from_slice(bytes);
                let object = &mut result.objects[object_index];
                object.kind = SnapshotObjectKind::MetadataBytes;
                object.byte_range = start..result.object_bytes.len();
                let scalar_start = result.object_scalars.len();
                result
                    .object_scalars
                    .push(super::types::SnapshotScalar::Unsigned(header_value as i64));
                object.scalar_range = scalar_start..result.object_scalars.len();
            }
        }
    }
    Ok(())
}

fn metadata_bytes<'a>(
    data: &'a [u8],
    position: usize,
    pointer_width: usize,
    expected_cid: i32,
    compressed_stack_maps: bool,
) -> Option<(&'a [u8], usize)> {
    let tags = match pointer_width {
        4 => u32::from_le_bytes(
            data.get(position..position.checked_add(4)?)?
                .try_into()
                .ok()?,
        ) as u64,
        8 => u64::from_le_bytes(
            data.get(position..position.checked_add(8)?)?
                .try_into()
                .ok()?,
        ),
        _ => return None,
    };
    let cid = if pointer_width == 4 {
        ((tags >> 12) & 0x000f_ffff) as i32
    } else {
        ((tags >> 16) & 0xffff) as i32
    };
    if cid != expected_cid {
        return None;
    }
    let header_end = position.checked_add(pointer_width)?;
    if compressed_stack_maps {
        let flags = u32::from_le_bytes(
            data.get(header_end..header_end.checked_add(4)?)?
                .try_into()
                .ok()?,
        );
        let length = (flags >> 2) as usize;
        let payload = header_end.checked_add(4)?;
        Some((
            data.get(payload..payload.checked_add(length)?)?,
            flags as usize,
        ))
    } else {
        let length_end = header_end.checked_add(pointer_width)?;
        let length = match pointer_width {
            4 => u32::from_le_bytes(data.get(header_end..length_end)?.try_into().ok()?) as usize,
            8 => usize::try_from(u64::from_le_bytes(
                data.get(header_end..length_end)?.try_into().ok()?,
            ))
            .ok()?,
            _ => return None,
        };
        Some((
            data.get(length_end..length_end.checked_add(length)?)?,
            length,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::metadata_bytes;

    #[test]
    fn reads_arm32_read_only_metadata_headers() {
        let mut data = vec![0u8; 64];
        let cid = 25u32;
        data[8..12].copy_from_slice(&(cid << 12).to_le_bytes());
        data[12..16].copy_from_slice(&3u32.to_le_bytes());
        data[16..19].copy_from_slice(&[0x21, 0x42, 0x63]);
        assert_eq!(
            metadata_bytes(&data, 8, 4, 25, false),
            Some((&[0x21, 0x42, 0x63][..], 3))
        );
        assert_eq!(metadata_bytes(&data, 8, 4, 24, false), None);
        data.truncate(18);
        assert_eq!(metadata_bytes(&data, 8, 4, 25, false), None);
    }

    #[test]
    fn reads_read_only_compressed_stack_map_flags_and_payload() {
        let mut data = vec![0u8; 24];
        data[0..4].copy_from_slice(&(26u32 << 12).to_le_bytes());
        let flags = (2u32 << 2) | 1;
        data[4..8].copy_from_slice(&flags.to_le_bytes());
        data[8..10].copy_from_slice(&[0xaa, 0xbb]);
        assert_eq!(
            metadata_bytes(&data, 0, 4, 26, true),
            Some((&[0xaa, 0xbb][..], flags as usize))
        );
    }
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
