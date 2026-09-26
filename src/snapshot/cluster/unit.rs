//! Deferred loading units (`libapp.so-N.part.so`).
//!
//! A unit snapshot is deserialized against its parent
//! (`UnitDeserializationRoots` in app_snapshot.cc): its base objects are the
//! parent's entire reference array, so its own references continue after
//! the parent's. The deferred functions' Code and Function objects were
//! serialized in the root with placeholder entry points; the unit supplies
//! their instructions (in its own image and instructions table) and source
//! maps, and patches global-pool entries with objects it adds.

use crate::diagnostic::{ClutterError, Result};

use super::cid::Profile;
use super::reader::Reader;
use super::types::{InstructionTable, ParseResult, PoolValue, SnapshotObjectPayload};

/// A root Code object whose instructions this unit carries.
#[derive(Clone, Debug)]
pub(super) struct UnitCode {
    pub code_ref: i32,
    pub payload_info: i64,
    pub code_source_map_ref: i32,
}

pub(super) struct UnitSnapshot {
    pub objects: ParseResult,
    pub codes: Vec<UnitCode>,
    /// `(global pool index, reference)` entries the unit overwrites.
    pub pool_patches: Vec<(usize, i32)>,
    pub table: InstructionTable,
}

/// Parses a unit's isolate snapshot data. `parent_objects` is the parent's
/// total reference count (base objects included) and `global_pool_length`
/// the parent's global object pool length.
pub(super) fn parse(
    data: &[u8],
    profile: &Profile,
    pointer_width: usize,
    parent_objects: i64,
    global_pool_length: usize,
) -> Result<UnitSnapshot> {
    let header = crate::snapshot::header::parse(data)?;
    let snapshot_size = header.length + 4;
    let features_end = data[0x34..]
        .iter()
        .take(1024)
        .position(|value| *value == 0)
        .map(|relative| relative + 0x35)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("unit features string is not terminated".to_owned())
        })?;
    // `WriteUnitSnapshot` writes the program hash before the clusters.
    let mut reader = Reader::at(data, features_end)?;
    reader.tagged32()?;
    let mut objects = super::alloc::scan(data, reader.position(), profile, false, false)?;
    if objects.header.num_base_objects != parent_objects {
        return Err(ClutterError::InvalidArtifact(format!(
            "loading unit expects {} base objects but its parent has {parent_objects}",
            objects.header.num_base_objects
        )));
    }
    let fill_end = super::fill::read(data, &mut objects, profile)?;

    // UnitDeserializationRoots::ReadRoots.
    let mut reader = Reader::at(data, fill_end)?;
    let deferred_start = reader.unsigned()?;
    let deferred_count = reader.unsigned()?;
    if deferred_count < 0 || deferred_count > 1 << 20 {
        return Err(ClutterError::InvalidArtifact(format!(
            "loading unit claims {deferred_count} deferred Code objects"
        )));
    }
    let mut codes = Vec::new();
    for offset in 0..deferred_count {
        let code_ref = i32::try_from(deferred_start + offset).map_err(|_| {
            ClutterError::InvalidArtifact("deferred Code reference exceeds i32".to_owned())
        })?;
        if i64::from(code_ref) > parent_objects {
            return Err(ClutterError::InvalidArtifact(
                "deferred Code reference is not a parent object".to_owned(),
            ));
        }
        let payload_info = reader.unsigned()?;
        let code_source_map_ref = i32::try_from(reader.reference()?).map_err(|_| {
            ClutterError::InvalidArtifact("code source map reference exceeds i32".to_owned())
        })?;
        codes.push(UnitCode {
            code_ref,
            payload_info,
            code_source_map_ref,
        });
    }
    let mut pool_patches = Vec::new();
    let mut index = usize::try_from(reader.unsigned()?).unwrap_or(usize::MAX);
    while index < global_pool_length {
        let reference = i32::try_from(reader.reference()?).map_err(|_| {
            ClutterError::InvalidArtifact("pool patch reference exceeds i32".to_owned())
        })?;
        pool_patches.push((index, reference));
        let step = usize::try_from(reader.unsigned()?).unwrap_or(usize::MAX);
        if step == 0 {
            return Err(ClutterError::InvalidArtifact(
                "loading unit pool patch does not advance".to_owned(),
            ));
        }
        index = index.saturating_add(step);
    }

    if !profile.compressed_pointers {
        super::rodata::extract_objects(data, &mut objects, profile, snapshot_size, pointer_width)?;
    }
    let table = super::instructions::parse_table(data, &objects.header, snapshot_size, pointer_width)?;
    Ok(UnitSnapshot {
        objects,
        codes,
        pool_patches,
        table,
    })
}

/// The parent's object graph with the unit's objects, source maps and pool
/// patches applied, as the VM holds it after loading the unit.
pub(super) fn merge(parent: &ParseResult, unit: &UnitSnapshot) -> ParseResult {
    let mut merged = parent.clone();
    let child = &unit.objects;
    for object in &child.objects {
        merged.insert_object(
            object.reference,
            object.cid,
            object.canonical,
            object.kind,
            SnapshotObjectPayload {
                references: child.references_of(object).to_vec(),
                scalars: child.scalars_of(object).to_vec(),
                bytes: child.bytes_of(object).to_vec(),
            },
        );
    }
    merged.objects.sort_by_key(|object| object.reference);
    merged.strings.extend(child.strings.clone());
    merged.named.extend(child.named.clone());
    merged.library_uris.extend(child.library_uris.clone());
    merged.function_types.extend(child.function_types.clone());
    merged.exception_handlers.extend(child.exception_handlers.clone());
    merged.instance_bitmaps.extend(child.instance_bitmaps.clone());
    merged.codes.extend(child.codes.iter().cloned());
    for unit_code in &unit.codes {
        if let Some(code) = merged
            .codes
            .iter_mut()
            .find(|code| code.ref_id == unit_code.code_ref)
        {
            code.code_source_map_ref = Some(unit_code.code_source_map_ref);
            code.payload_info = Some(unit_code.payload_info);
            code.unchecked_entry_offset = Some((unit_code.payload_info >> 1) as u64);
            code.has_monomorphic_entrypoint = unit_code.payload_info & 1 == 1;
        }
    }
    let global_pool = merged
        .roots
        .as_ref()
        .and_then(|roots| roots.named.get("global_object_pool"))
        .copied();
    if let Some(pool) = merged
        .object_pools
        .iter_mut()
        .find(|pool| Some(pool.reference) == global_pool)
    {
        for (index, reference) in &unit.pool_patches {
            if let Some(entry) = pool.entries.get_mut(*index) {
                *entry = PoolValue::Reference(*reference);
            }
        }
    }
    merged.rebuild_back_references();
    merged
}

/// Loading-unit id from `libapp.so-N.part.so`.
pub fn unit_id(path: &str) -> Option<u32> {
    path.rsplit('/')
        .next()?
        .strip_prefix("libapp.so-")?
        .strip_suffix(".part.so")?
        .parse()
        .ok()
}
