use object::{Architecture, Object, ObjectSegment, ObjectSymbol, SymbolKind};
use sha2::{Digest, Sha256};

use crate::diagnostic::{ClutterError, Result};
use crate::model::{Abi, SnapshotRegion};

const SNAPSHOT_SYMBOLS: [&str; 4] = [
    "_kDartVmSnapshotData",
    "_kDartVmSnapshotInstructions",
    "_kDartIsolateSnapshotData",
    "_kDartIsolateSnapshotInstructions",
];

pub struct ElfImage<'a> {
    bytes: &'a [u8],
    file: object::File<'a>,
    pub pointer_width: usize,
}

impl<'a> ElfImage<'a> {
    pub fn parse(bytes: &'a [u8], expected_abi: Abi) -> Result<Self> {
        let file = object::File::parse(bytes)?;
        if file.format() != object::BinaryFormat::Elf {
            return Err(ClutterError::InvalidArtifact(
                "libapp.so is not an ELF file".to_owned(),
            ));
        }
        let abi = match file.architecture() {
            Architecture::Aarch64 => Abi::Arm64V8a,
            Architecture::Arm => Abi::ArmeabiV7a,
            Architecture::X86_64 => Abi::X86_64,
            architecture => {
                return Err(ClutterError::Unsupported(format!(
                    "unsupported ELF architecture {architecture:?}"
                )));
            }
        };
        if abi != expected_abi {
            return Err(ClutterError::InvalidArtifact(format!(
                "archive path says {expected_abi}, but ELF machine is {abi}"
            )));
        }
        let pointer_width = if file.is_64() { 8 } else { 4 };
        Ok(Self {
            bytes,
            file,
            pointer_width,
        })
    }

    pub fn snapshot_regions(&self) -> Result<Vec<SnapshotRegion>> {
        let mut regions = Vec::with_capacity(SNAPSHOT_SYMBOLS.len());
        for name in SNAPSHOT_SYMBOLS {
            let symbol = self
                .file
                .symbols()
                .chain(self.file.dynamic_symbols())
                .find(|symbol| {
                    symbol.kind() == SymbolKind::Data && symbol.name().ok() == Some(name)
                })
                .ok_or_else(|| {
                    ClutterError::InvalidArtifact(format!(
                        "required Dart snapshot symbol {name} is missing"
                    ))
                })?;
            let address = symbol.address();
            let size = symbol.size();
            if size == 0 {
                return Err(ClutterError::InvalidArtifact(format!(
                    "Dart snapshot symbol {name} has zero size"
                )));
            }
            let file_offset = self.virtual_address_to_offset(address, size)?;
            let end = file_offset.checked_add(size).ok_or_else(|| {
                ClutterError::InvalidArtifact(format!(
                    "snapshot region {name} overflows its file range"
                ))
            })?;
            let data = self
                .bytes
                .get(file_offset as usize..end as usize)
                .ok_or_else(|| {
                    ClutterError::InvalidArtifact(format!(
                        "snapshot region {name} extends beyond libapp.so"
                    ))
                })?
                .to_vec();
            let digest = Sha256::digest(&data);
            regions.push(SnapshotRegion {
                name: name.to_owned(),
                virtual_address: format!("0x{address:x}"),
                file_offset,
                size,
                sha256: hex::encode(digest),
                data,
            });
        }
        Ok(regions)
    }

    fn virtual_address_to_offset(&self, address: u64, size: u64) -> Result<u64> {
        for segment in self.file.segments() {
            let segment_address = segment.address();
            let segment_size = segment.size();
            let Some(relative) = address.checked_sub(segment_address) else {
                continue;
            };
            if relative > segment_size || size > segment_size.saturating_sub(relative) {
                continue;
            }
            let (file_offset, file_size) = segment.file_range();
            if relative > file_size || size > file_size.saturating_sub(relative) {
                continue;
            }
            return file_offset.checked_add(relative).ok_or_else(|| {
                ClutterError::InvalidArtifact("ELF file offset overflow".to_owned())
            });
        }
        Err(ClutterError::InvalidArtifact(format!(
            "ELF virtual range 0x{address:x}+0x{size:x} is not file-backed"
        )))
    }
}
