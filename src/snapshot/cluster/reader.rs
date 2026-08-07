use crate::diagnostic::{ClutterError, Result};

#[derive(Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn at(data: &'a [u8], position: usize) -> Result<Self> {
        if position > data.len() {
            return Err(ClutterError::InvalidArtifact(format!(
                "snapshot offset 0x{position:x} exceeds {} bytes",
                data.len()
            )));
        }
        Ok(Self { data, position })
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) -> Result<()> {
        if position > self.data.len() {
            return Err(ClutterError::InvalidArtifact(format!(
                "snapshot offset 0x{position:x} exceeds {} bytes",
                self.data.len()
            )));
        }
        self.position = position;
        Ok(())
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn byte(&mut self) -> Result<u8> {
        let value = self.data.get(self.position).copied().ok_or_else(|| {
            ClutterError::InvalidArtifact(format!(
                "snapshot byte at 0x{:x} is truncated",
                self.position
            ))
        })?;
        self.position += 1;
        Ok(value)
    }

    pub fn bytes(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self.position.checked_add(length).ok_or_else(|| {
            ClutterError::InvalidArtifact("snapshot byte range overflow".to_owned())
        })?;
        let value = self.data.get(self.position..end).ok_or_else(|| {
            ClutterError::InvalidArtifact(format!(
                "snapshot range 0x{:x}..0x{end:x} is truncated",
                self.position
            ))
        })?;
        self.position = end;
        Ok(value)
    }

    pub fn skip(&mut self, length: usize) -> Result<()> {
        self.bytes(length).map(|_| ())
    }

    pub fn u16(&mut self) -> Result<u16> {
        self.tagged32().map(|value| value as u16)
    }

    pub fn i16(&mut self) -> Result<i16> {
        self.tagged32().map(|value| value as i16)
    }

    pub fn u32(&mut self) -> Result<u32> {
        self.tagged32()
    }

    pub fn i32(&mut self) -> Result<i32> {
        self.tagged32().map(|value| value as i32)
    }

    pub fn unsigned(&mut self) -> Result<i64> {
        let mut byte = self.byte()?;
        if byte > 0x7f {
            return Ok(i64::from(byte) - 128);
        }
        let mut result = 0i64;
        let mut shift = 0u32;
        loop {
            result |= i64::from(byte) << shift;
            shift += 7;
            byte = self.byte()?;
            if byte > 0x7f {
                result |= i64::from(byte - 128) << shift;
                return Ok(result);
            }
            if shift >= 63 {
                return Err(ClutterError::InvalidArtifact(
                    "snapshot unsigned VLE exceeds 63 bits".to_owned(),
                ));
            }
        }
    }

    pub fn tagged32(&mut self) -> Result<u32> {
        let mut byte = self.byte()?;
        if byte > 0x7f {
            return Ok(u32::from(byte).wrapping_sub(192));
        }
        let mut result = 0u32;
        let mut shift = 0u32;
        loop {
            result |= u32::from(byte) << shift;
            shift += 7;
            byte = self.byte()?;
            if byte > 0x7f {
                result |= u32::from(byte).wrapping_sub(192) << shift;
                return Ok(result);
            }
            if shift >= 28 {
                return Err(ClutterError::InvalidArtifact(
                    "snapshot tagged32 VLE exceeds 32 bits".to_owned(),
                ));
            }
        }
    }

    pub fn tagged64(&mut self) -> Result<i64> {
        let mut byte = self.byte()?;
        if byte > 0x7f {
            return Ok(i64::from(byte) - 192);
        }
        let mut result = 0i64;
        let mut shift = 0u32;
        loop {
            result |= i64::from(byte) << shift;
            shift += 7;
            byte = self.byte()?;
            if byte > 0x7f {
                result |= (i64::from(byte) - 192) << shift;
                return Ok(result);
            }
            if shift >= 63 {
                return Err(ClutterError::InvalidArtifact(
                    "snapshot tagged64 VLE exceeds 64 bits".to_owned(),
                ));
            }
        }
    }

    pub fn reference(&mut self) -> Result<i64> {
        let mut result = 0i64;
        for _ in 0..5 {
            let byte = self.byte()? as i8;
            result = i64::from(byte) + (result << 7);
            if byte < 0 {
                return Ok(result + 128);
            }
        }
        Err(ClutterError::InvalidArtifact(
            "snapshot reference exceeds five bytes".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn reads_single_byte_unsigned() {
        let mut reader = Reader::at(&[128, 255], 0).unwrap();
        assert_eq!(reader.unsigned().unwrap(), 0);
        assert_eq!(reader.unsigned().unwrap(), 127);
    }

    #[test]
    fn reads_reference_encoding() {
        let mut reader = Reader::at(&[0x81], 0).unwrap();
        assert_eq!(reader.reference().unwrap(), 1);
    }

    #[test]
    fn reads_bounded_snapshot_scalars() {
        let bytes = [0x34, 0xe4, 0xbe];
        let mut reader = Reader::at(&bytes, 0).unwrap();
        assert_eq!(reader.u16().unwrap(), 0x1234);
        assert_eq!(reader.i16().unwrap(), -2);
    }
}
