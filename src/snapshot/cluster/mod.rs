//! Direct Dart clustered-snapshot reader.
//!
//! The binary grammar follows the Dart VM serializer. The implementation was
//! independently ported to Rust with reference to the Dart SDK and the
//! BSD-3-Clause `zboralski/unflutter` parser.

mod alloc;
mod catch_moves;
mod cid;
mod constants;
mod dispatch;
mod fill;
mod fill_skip;
mod fill_spec;
mod instructions;
mod reader;
mod rodata;
mod root_names;
mod roots;
mod transduce;
mod type_recovery;
mod types;
mod unit;
mod vm_stub_names;

pub use unit::unit_id;

pub(crate) use instructions::source_bands_from_metadata;

use crate::diagnostic::{ClutterError, Result};
use crate::model::{
    Abi, RecoveredDeclaration, RecoveredFunction, RecoveredString, Scope, SnapshotEvidence,
    SnapshotInfo,
};
use crate::snapshot::CodeImage;

pub struct Recovery {
    pub application_package: Option<String>,
    pub root_library_uri: Option<String>,
    pub functions: Vec<RecoveredFunction>,
    /// Declarations in the output scope.
    pub declarations: Vec<RecoveredDeclaration>,
    /// Declarations of every scope, used as enrichment evidence only.
    pub evidence_declarations: Vec<RecoveredDeclaration>,
    pub ownership_obfuscated: bool,
    pub snapshot_evidence: SnapshotEvidence,
    pub dispatch_table: Option<crate::model::RecoveredDispatchTable>,
    /// Class-dispatch rows, class ids and class-name maps for resolving
    /// dispatch-table calls in the final semantic pass.
    pub dispatch_analysis: crate::analysis::disassembly::DispatchTableData,
    pub snapshot_strings: Vec<RecoveredString>,
    pub snapshot_roots: Option<crate::model::SnapshotRootEvidence>,
    /// Typed values reachable from the object pool, by snapshot reference.
    pub constants: std::collections::BTreeMap<i32, crate::model::SnapshotConstant>,
    /// Deferred loading units recovered against this snapshot.
    pub loading_units: Vec<crate::model::LoadingUnitRecovery>,
}

/// A deferred loading unit's isolate snapshot data and instructions.
pub struct DeferredUnitImage {
    pub id: u32,
    pub path: String,
    pub data: Vec<u8>,
    pub code: CodeImage,
}

#[allow(clippy::too_many_arguments)]
pub fn recover_functions(
    info: &SnapshotInfo,
    code: &CodeImage,
    abi: Abi,
    pointer_width: usize,
    scope: Scope,
    application_package: Option<&str>,
    obfuscation_map: Option<&crate::analysis::LoadedObfuscationMap>,
    units: &[DeferredUnitImage],
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
        false,
    )?;
    let isolate = parse_snapshot(
        isolate_data,
        &profile,
        false,
        info.isolate_header.length + 4,
        pointer_width,
        info.isolate_header.snapshot_hash == "ace654289f5abc240509fc941453ebc5",
    )?;
    let table = instructions::parse_table(
        isolate_data,
        &isolate.header,
        info.isolate_header.length + 4,
        pointer_width,
    )?;
    let options = || instructions::ResolveOptions {
        abi,
        scope,
        application_package,
        obfuscation_map,
        instance_header_words: profile.instance_header_words,
        unboxed_word_u32_chunks: profile.unboxed_word_u32_chunks,
        async_modifier_shift: instructions::async_modifier_shift(&info.isolate_header.snapshot_hash),
    };
    let mut recovery =
        instructions::resolve(&isolate, &vm, &profile.cids, &table, code, options())?;
    let parent_objects = isolate.header.num_objects;
    let global_pool_length = isolate
        .roots
        .as_ref()
        .and_then(|roots| roots.named.get("global_object_pool"))
        .and_then(|reference| {
            isolate
                .object_pools
                .iter()
                .find(|pool| pool.reference == *reference)
        })
        .map_or(0, |pool| pool.entries.len());
    for deferred in units {
        let recovered = unit::parse(
            &deferred.data,
            &profile,
            pointer_width,
            parent_objects,
            global_pool_length,
        )
        .and_then(|unit| {
            let merged = unit::merge(&isolate, &unit);
            instructions::resolve_unit(&merged, &vm, &profile.cids, &unit, &deferred.code, options())
        });
        match recovered {
            Ok(unit_recovery) => {
                recovery.functions.extend(unit_recovery.functions.into_iter().map(|mut function| {
                    function.loading_unit = Some(deferred.id);
                    function
                }));
                recovery.functions.sort_by_key(|function| {
                    u64::from_str_radix(function.address.trim_start_matches("0x"), 16)
                        .unwrap_or(u64::MAX)
                });
                recovery.loading_units.push(crate::model::LoadingUnitRecovery {
                    id: deferred.id,
                    path: deferred.path.clone(),
                    functions: unit_recovery_count(&recovery, deferred.id),
                    error: None,
                });
            }
            Err(error) => recovery.loading_units.push(crate::model::LoadingUnitRecovery {
                id: deferred.id,
                path: deferred.path.clone(),
                functions: 0,
                error: Some(error.to_string()),
            }),
        }
    }
    Ok(recovery)
}

fn unit_recovery_count(recovery: &Recovery, id: u32) -> usize {
    recovery
        .functions
        .iter()
        .filter(|function| function.loading_unit == Some(id))
        .count()
}

fn parse_snapshot(
    data: &[u8],
    profile: &cid::Profile,
    is_vm: bool,
    snapshot_size: u64,
    pointer_width: usize,
    exact_3122: bool,
) -> Result<types::ParseResult> {
    let start = data[0x34..]
        .iter()
        .take(1024)
        .position(|value| *value == 0)
        .map(|relative| relative + 0x35)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("snapshot features string is not terminated".to_owned())
        })?;
    let mut result = alloc::scan(data, start, profile, is_vm, true)?;
    let fill_end = fill::read(data, &mut result, profile)?;
    let snapshot_end = usize::try_from(snapshot_size)
        .unwrap_or(data.len())
        .min(data.len());
    if is_vm && let Some(stubs) = vm_stub_names::for_minor(profile.minor) {
        result.vm_stub_names =
            roots::vm_stub_names(data, fill_end, snapshot_end, stubs, &result, &profile.cids)
                .unwrap_or_default();
    }
    if !is_vm {
        // The exact 3.12.2 layout is verified end to end, so a mismatch is a
        // hard error. Other releases use their generated root list and keep
        // it only when it leads to the same dispatch table and the root
        // CIDs check out; otherwise the snapshot is analyzed without roots.
        let roots = if exact_3122 {
            Some(roots::parse_3122(data, fill_end, snapshot_end)?)
        } else {
            roots::parse(
                data,
                fill_end,
                snapshot_end,
                profile.root_names,
                profile.shared_field_table,
                profile.minor,
            )
            .ok()
        };
        result.roots = roots;
    }
    if let Some(first_code_reference) = result
        .clusters
        .iter()
        .find(|cluster| cluster.cid == profile.cids.code)
        .map(|cluster| cluster.start_ref)
    {
        result.dispatch_table_code_indices =
            dispatch::find_table(data, fill_end, snapshot_end, first_code_reference);
        if let Some(roots) = result.roots.as_ref() {
            let checked = dispatch::decode_at(
                data,
                roots.dispatch_offset,
                snapshot_end,
                first_code_reference,
            )
            .ok_or_else(|| {
                ClutterError::InvalidArtifact(
                    "snapshot root layout does not lead to its dispatch table".to_owned(),
                )
            })
            .and_then(|at_roots| {
                if at_roots == result.dispatch_table_code_indices {
                    Ok(())
                } else {
                    Err(ClutterError::InvalidArtifact(
                        "snapshot root and dispatch decoders disagree".to_owned(),
                    ))
                }
            })
            .and_then(|()| roots::validate(roots, &result, &profile.cids));
            match checked {
                Ok(()) => {}
                Err(error) if exact_3122 => return Err(error),
                Err(_) => result.roots = None,
            }
        }
    } else if !exact_3122 {
        result.roots = None;
    }
    if !profile.compressed_pointers {
        rodata::extract_objects(data, &mut result, profile, snapshot_size, pointer_width)?;
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
