//! Direct Dart clustered-snapshot reader.
//!
//! The binary grammar follows the Dart VM serializer. The implementation was
//! independently ported to Rust with reference to the Dart SDK and the
//! BSD-3-Clause `zboralski/unflutter` parser.

mod alloc;
mod cid;
mod dispatch;
mod fill;
mod fill_skip;
mod fill_spec;
mod instructions;
mod reader;
mod rodata;
mod transduce;
mod type_recovery;
mod types;

use crate::diagnostic::{ClutterError, Result};
use crate::model::{
    Abi, RecoveredDeclaration, RecoveredFunction, RecoveredString, Scope, SnapshotEvidence,
    SnapshotInfo,
};
use crate::snapshot::CodeImage;

pub struct Recovery {
    pub functions: Vec<RecoveredFunction>,
    pub declarations: Vec<RecoveredDeclaration>,
    pub ownership_obfuscated: bool,
    pub snapshot_evidence: SnapshotEvidence,
    pub dispatch_table: Option<crate::model::RecoveredDispatchTable>,
    pub snapshot_strings: Vec<RecoveredString>,
}

pub fn recover_functions(
    info: &SnapshotInfo,
    code: &CodeImage,
    abi: Abi,
    pointer_width: usize,
    scope: Scope,
    application_package: Option<&str>,
    obfuscation_map: Option<&crate::analysis::LoadedObfuscationMap>,
) -> Result<Recovery> {
    let profile = cid::profile_for(info, pointer_width)?;
    let vm_data = region_data(info, "_kDartVmSnapshotData")?;
    let isolate_data = region_data(info, "_kDartIsolateSnapshotData")?;

    let vm = parse_snapshot(
        vm_data,
        &profile,
        true,
        info.vm_header.length + 4,
        pointer_width,
    )?;
    let isolate = parse_snapshot(
        isolate_data,
        &profile,
        false,
        info.isolate_header.length + 4,
        pointer_width,
    )?;
    let table = instructions::parse_table(
        isolate_data,
        &isolate.header,
        info.isolate_header.length + 4,
        pointer_width,
    )?;
    instructions::resolve(
        &isolate,
        &vm,
        &profile.cids,
        &table,
        code,
        instructions::ResolveOptions {
            abi,
            scope,
            application_package,
            obfuscation_map,
        },
    )
}

fn parse_snapshot(
    data: &[u8],
    profile: &cid::Profile,
    is_vm: bool,
    snapshot_size: u64,
    pointer_width: usize,
) -> Result<types::ParseResult> {
    let start = data[0x34..]
        .iter()
        .take(1024)
        .position(|value| *value == 0)
        .map(|relative| relative + 0x35)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("snapshot features string is not terminated".to_owned())
        })?;
    let mut result = alloc::scan(data, start, profile, is_vm)?;
    let fill_end = fill::read(data, &mut result, profile)?;
    let snapshot_end = usize::try_from(snapshot_size)
        .unwrap_or(data.len())
        .min(data.len());
    if let Some(first_code_reference) = result
        .clusters
        .iter()
        .find(|cluster| cluster.cid == profile.cids.code)
        .map(|cluster| cluster.start_ref)
    {
        result.dispatch_table_code_indices =
            dispatch::find_table(data, fill_end, snapshot_end, first_code_reference);
    }
    if !profile.compressed_pointers {
        rodata::extract_strings(data, &mut result, profile, snapshot_size, pointer_width)?;
    }
    Ok(result)
}

fn region_data<'a>(info: &'a SnapshotInfo, name: &str) -> Result<&'a [u8]> {
    info.regions
        .iter()
        .find(|region| region.name == name)
        .map(|region| region.data.as_slice())
        .ok_or_else(|| {
            ClutterError::InvalidArtifact(format!("snapshot region {name} is unavailable"))
        })
}
