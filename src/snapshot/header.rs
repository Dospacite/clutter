use crate::diagnostic::{ClutterError, Result};
use crate::model::{SnapshotHeader, SnapshotKind};

const MAGIC: [u8; 4] = [0xf5, 0xf5, 0xdc, 0xdc];
const HASH_OFFSET: usize = 0x14;
const FEATURES_OFFSET: usize = 0x34;
const MAX_FEATURE_BYTES: usize = 1024;

pub fn parse(bytes: &[u8]) -> Result<SnapshotHeader> {
    if bytes.len() < FEATURES_OFFSET + 1 {
        return Err(ClutterError::InvalidArtifact(
            "Dart snapshot header is truncated".to_owned(),
        ));
    }
    if bytes[..4] != MAGIC {
        return Err(ClutterError::InvalidArtifact(format!(
            "invalid Dart snapshot magic {}",
            hex::encode(&bytes[..4])
        )));
    }
    let length = read_u64(bytes, 4)?;
    let kind = match read_i64(bytes, 12)? {
        0 => SnapshotKind::Full,
        1 => SnapshotKind::FullCore,
        2 => SnapshotKind::FullJit,
        3 => SnapshotKind::FullAot,
        value => SnapshotKind::Unknown(value),
    };
    let hash_bytes = &bytes[HASH_OFFSET..HASH_OFFSET + 32];
    if !hash_bytes
        .iter()
        .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
    {
        return Err(ClutterError::InvalidArtifact(
            "Dart snapshot hash is malformed".to_owned(),
        ));
    }
    let end = bytes[FEATURES_OFFSET..]
        .iter()
        .take(MAX_FEATURE_BYTES)
        .position(|value| *value == 0)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact(
                "Dart snapshot features string is not terminated".to_owned(),
            )
        })?
        + FEATURES_OFFSET;
    let features = String::from_utf8_lossy(&bytes[FEATURES_OFFSET..end])
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    Ok(SnapshotHeader {
        length,
        kind,
        snapshot_hash: String::from_utf8_lossy(hash_bytes).into_owned(),
        features,
    })
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    let value = bytes.get(offset..offset + 8).ok_or_else(|| {
        ClutterError::InvalidArtifact(format!("snapshot u64 at 0x{offset:x} is truncated"))
    })?;
    Ok(u64::from_le_bytes(value.try_into().expect("eight bytes")))
}

fn read_i64(bytes: &[u8], offset: usize) -> Result<i64> {
    Ok(read_u64(bytes, offset)? as i64)
}
