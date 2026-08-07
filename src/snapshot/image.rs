use crate::diagnostic::{ClutterError, Result};

pub struct CodeImage {
    pub image_virtual_address: u64,
    pub code_offset: u64,
    pub bytes: Vec<u8>,
}

pub fn parse(bytes: &[u8], image_va: u64, pointer_width: usize) -> Result<CodeImage> {
    if !matches!(pointer_width, 4 | 8) {
        return Err(ClutterError::Analysis(format!(
            "unsupported pointer width {pointer_width}"
        )));
    }
    let image_size = read_word(bytes, 0, pointer_width)?;
    let section_offset = read_word(bytes, pointer_width, pointer_width)?;
    if image_size > bytes.len() as u64 {
        return Err(ClutterError::InvalidArtifact(format!(
            "Dart instructions image declares {image_size} bytes but only {} are present",
            bytes.len()
        )));
    }
    let payload_length = read_word(
        bytes,
        section_offset as usize + pointer_width,
        pointer_width,
    )?;
    let code_offset = section_offset
        .checked_add((pointer_width * 5) as u64)
        .ok_or_else(|| ClutterError::InvalidArtifact("code offset overflow".to_owned()))?;
    let code_end = code_offset
        .checked_add(payload_length)
        .ok_or_else(|| ClutterError::InvalidArtifact("instruction payload overflow".to_owned()))?;
    let code = bytes
        .get(code_offset as usize..code_end.min(image_size) as usize)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("instruction payload is out of bounds".to_owned())
        })?
        .to_vec();
    Ok(CodeImage {
        image_virtual_address: image_va,
        code_offset,
        bytes: code,
    })
}

fn read_word(bytes: &[u8], offset: usize, width: usize) -> Result<u64> {
    let data = bytes.get(offset..offset + width).ok_or_else(|| {
        ClutterError::InvalidArtifact(format!("Dart image word at 0x{offset:x} is truncated"))
    })?;
    Ok(if width == 8 {
        u64::from_le_bytes(data.try_into().expect("eight-byte word"))
    } else {
        u64::from(u32::from_le_bytes(data.try_into().expect("four-byte word")))
    })
}
