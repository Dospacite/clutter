mod cluster;
mod header;
mod image;
mod profiles;

use regex::Regex;

use crate::diagnostic::{ClutterError, Result};
use crate::elf::ElfImage;
use crate::model::{ProfileMatch, SnapshotInfo};

pub use cluster::recover_functions;
pub use image::CodeImage;

pub fn inspect(elf: &ElfImage<'_>, libflutter: Option<&[u8]>) -> Result<SnapshotInfo> {
    let regions = elf.snapshot_regions()?;
    let vm_region = region(&regions, "_kDartVmSnapshotData")?;
    let isolate_region = region(&regions, "_kDartIsolateSnapshotData")?;
    let vm_header = header::parse(&vm_region.data)?;
    let isolate_header = header::parse(&isolate_region.data)?;
    if vm_header.snapshot_hash != isolate_header.snapshot_hash {
        return Err(ClutterError::InvalidArtifact(
            "VM and isolate snapshots use different version hashes".to_owned(),
        ));
    }

    let dart_version = libflutter.and_then(detect_dart_version);
    let profile = profiles::detect(&vm_header.snapshot_hash, dart_version.as_deref());
    Ok(SnapshotInfo {
        dart_version: dart_version.or_else(|| profile.version.map(str::to_owned)),
        profile_id: profile.id,
        profile_match: profile.match_kind,
        vm_header,
        isolate_header,
        regions,
    })
}

pub fn isolate_code(info: &SnapshotInfo, pointer_width: usize) -> Result<CodeImage> {
    let region = region(&info.regions, "_kDartIsolateSnapshotInstructions")?;
    image::parse(
        &region.data,
        parse_hex(&region.virtual_address)?,
        pointer_width,
    )
}

fn region<'a>(
    regions: &'a [crate::model::SnapshotRegion],
    name: &str,
) -> Result<&'a crate::model::SnapshotRegion> {
    regions
        .iter()
        .find(|region| region.name == name)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact(format!("snapshot region {name} is unavailable"))
        })
}

fn detect_dart_version(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let expression = Regex::new(
        r#"(?m)(?:Dart SDK version:\s*)?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.\-]+)?) \((?:stable|beta|dev)\)"#,
    )
    .expect("static Dart version regular expression");
    expression
        .captures(&text)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_owned())
}

fn parse_hex(value: &str) -> Result<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|error| ClutterError::Analysis(format!("invalid address {value}: {error}")))
}

pub fn is_supported(info: &SnapshotInfo) -> bool {
    !matches!(info.profile_match, ProfileMatch::Unknown)
}
