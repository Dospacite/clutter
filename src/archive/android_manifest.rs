use crate::diagnostic::{ClutterError, Result};
use crate::model::AndroidMetadata;

const RES_XML_TYPE: u16 = 0x0003;
const RES_STRING_POOL_TYPE: u16 = 0x0001;
const RES_XML_START_ELEMENT_TYPE: u16 = 0x0102;
const TYPE_STRING: u8 = 0x03;
const UTF8_FLAG: u32 = 1 << 8;

pub fn parse_binary_xml(bytes: &[u8]) -> Result<AndroidMetadata> {
    let mut metadata = AndroidMetadata::default();
    let mut strings = Vec::new();

    let root_type = read_u16(bytes, 0)?;
    if root_type != RES_XML_TYPE {
        return Err(ClutterError::InvalidArtifact(
            "APK AndroidManifest.xml is not Android binary XML".to_owned(),
        ));
    }
    let root_header_size = usize::from(read_u16(bytes, 2)?);
    let mut offset = root_header_size;

    while offset + 8 <= bytes.len() {
        let chunk_type = read_u16(bytes, offset)?;
        let header_size = usize::from(read_u16(bytes, offset + 2)?);
        let chunk_size = read_u32(bytes, offset + 4)? as usize;
        if chunk_size < header_size || offset.saturating_add(chunk_size) > bytes.len() {
            return Err(ClutterError::InvalidArtifact(format!(
                "invalid Android XML chunk at offset 0x{offset:x}"
            )));
        }

        match chunk_type {
            RES_STRING_POOL_TYPE => {
                strings = parse_string_pool(&bytes[offset..offset + chunk_size])?;
            }
            RES_XML_START_ELEMENT_TYPE if !strings.is_empty() => {
                parse_start_element(&bytes[offset..offset + chunk_size], &strings, &mut metadata)?;
            }
            _ => {}
        }
        offset += chunk_size;
    }

    metadata.permissions.sort();
    metadata.permissions.dedup();
    Ok(metadata)
}

fn parse_string_pool(chunk: &[u8]) -> Result<Vec<String>> {
    let header_size = usize::from(read_u16(chunk, 2)?);
    let string_count = read_u32(chunk, 8)? as usize;
    let flags = read_u32(chunk, 16)?;
    let strings_start = read_u32(chunk, 20)? as usize;
    if header_size + string_count.saturating_mul(4) > chunk.len() || strings_start > chunk.len() {
        return Err(ClutterError::InvalidArtifact(
            "invalid Android string pool bounds".to_owned(),
        ));
    }

    let utf8 = flags & UTF8_FLAG != 0;
    let mut strings = Vec::with_capacity(string_count);
    for index in 0..string_count {
        let relative = read_u32(chunk, header_size + index * 4)? as usize;
        let start = strings_start.checked_add(relative).ok_or_else(|| {
            ClutterError::InvalidArtifact("Android string offset overflow".to_owned())
        })?;
        strings.push(if utf8 {
            decode_utf8_string(chunk, start)?
        } else {
            decode_utf16_string(chunk, start)?
        });
    }
    Ok(strings)
}

fn decode_utf8_string(bytes: &[u8], mut offset: usize) -> Result<String> {
    let (_, consumed) = decode_length8(bytes, offset)?;
    offset += consumed;
    let (byte_len, consumed) = decode_length8(bytes, offset)?;
    offset += consumed;
    let end = offset
        .checked_add(byte_len)
        .ok_or_else(|| ClutterError::InvalidArtifact("Android UTF-8 string overflow".to_owned()))?;
    let data = bytes.get(offset..end).ok_or_else(|| {
        ClutterError::InvalidArtifact("truncated Android UTF-8 string".to_owned())
    })?;
    Ok(String::from_utf8_lossy(data).into_owned())
}

fn decode_utf16_string(bytes: &[u8], mut offset: usize) -> Result<String> {
    let first = read_u16(bytes, offset)?;
    offset += 2;
    let units = if first & 0x8000 != 0 {
        let second = read_u16(bytes, offset)?;
        offset += 2;
        (usize::from(first & 0x7fff) << 16) | usize::from(second)
    } else {
        usize::from(first)
    };
    let byte_len = units.checked_mul(2).ok_or_else(|| {
        ClutterError::InvalidArtifact("Android UTF-16 string overflow".to_owned())
    })?;
    let data = bytes.get(offset..offset + byte_len).ok_or_else(|| {
        ClutterError::InvalidArtifact("truncated Android UTF-16 string".to_owned())
    })?;
    let units: Vec<_> = data
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok(String::from_utf16_lossy(&units))
}

fn decode_length8(bytes: &[u8], offset: usize) -> Result<(usize, usize)> {
    let first = *bytes.get(offset).ok_or_else(|| {
        ClutterError::InvalidArtifact("truncated Android string length".to_owned())
    })?;
    if first & 0x80 == 0 {
        Ok((usize::from(first), 1))
    } else {
        let second = *bytes.get(offset + 1).ok_or_else(|| {
            ClutterError::InvalidArtifact("truncated Android string length".to_owned())
        })?;
        Ok(((usize::from(first & 0x7f) << 8) | usize::from(second), 2))
    }
}

fn parse_start_element(
    chunk: &[u8],
    strings: &[String],
    metadata: &mut AndroidMetadata,
) -> Result<()> {
    if chunk.len() < 36 {
        return Err(ClutterError::InvalidArtifact(
            "truncated Android start-element chunk".to_owned(),
        ));
    }
    let name_index = read_u32(chunk, 20)?;
    let Some(element_name) = string_at(strings, name_index) else {
        return Ok(());
    };
    let attribute_start = usize::from(read_u16(chunk, 24)?);
    let attribute_size = usize::from(read_u16(chunk, 26)?);
    let attribute_count = usize::from(read_u16(chunk, 28)?);
    let base = 16usize.checked_add(attribute_start).ok_or_else(|| {
        ClutterError::InvalidArtifact("Android attribute offset overflow".to_owned())
    })?;

    for index in 0..attribute_count {
        let offset = base
            .checked_add(index.saturating_mul(attribute_size))
            .ok_or_else(|| {
                ClutterError::InvalidArtifact("Android attribute offset overflow".to_owned())
            })?;
        if attribute_size < 20 || offset + 20 > chunk.len() {
            return Err(ClutterError::InvalidArtifact(
                "truncated Android attribute".to_owned(),
            ));
        }
        let name = string_at(strings, read_u32(chunk, offset + 4)?).unwrap_or("");
        let raw = read_u32(chunk, offset + 8)?;
        let data_type = chunk[offset + 15];
        let data = read_u32(chunk, offset + 16)?;
        let string_value = if raw != u32::MAX {
            string_at(strings, raw).map(ToOwned::to_owned)
        } else if data_type == TYPE_STRING {
            string_at(strings, data).map(ToOwned::to_owned)
        } else {
            None
        };

        match (element_name, name) {
            ("manifest", "package") => metadata.package_name = string_value,
            ("manifest", "versionName") => metadata.version_name = string_value,
            ("manifest", "versionCode") => metadata.version_code = Some(u64::from(data)),
            ("uses-sdk", "minSdkVersion") => metadata.min_sdk = Some(data),
            ("uses-sdk", "targetSdkVersion") => metadata.target_sdk = Some(data),
            ("uses-permission", "name") => {
                if let Some(value) = string_value {
                    metadata.permissions.push(value);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn string_at(strings: &[String], index: u32) -> Option<&str> {
    if index == u32::MAX {
        None
    } else {
        strings.get(index as usize).map(String::as_str)
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes.get(offset..offset + 2).ok_or_else(|| {
        ClutterError::InvalidArtifact(format!("truncated u16 at offset 0x{offset:x}"))
    })?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes.get(offset..offset + 4).ok_or_else(|| {
        ClutterError::InvalidArtifact(format!("truncated u32 at offset 0x{offset:x}"))
    })?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}
