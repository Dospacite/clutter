use crate::diagnostic::{ClutterError, Result};

use super::cid::{Cids, Profile};
use super::reader::Reader;
use super::types::{Cluster, ClusterHeader, ParseResult};

const MAX_OBJECTS: usize = 20_000_000;
const MAX_CLUSTERS: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AllocKind {
    Simple,
    CanonicalSet,
    String,
    Mint,
    Array,
    WeakArray,
    TypeArguments,
    Class,
    Code,
    ObjectPool,
    ReadOnlyData,
    ExceptionHandlers,
    Context,
    ContextScope,
    Record,
    TypedData,
    Instance,
    Empty,
}

pub fn scan(data: &[u8], start: usize, profile: &Profile, is_vm: bool) -> Result<ParseResult> {
    let mut reader = Reader::at(data, start)?;
    let header = ClusterHeader {
        num_base_objects: reader.unsigned()?,
        num_objects: reader.unsigned()?,
        num_clusters: reader.unsigned()?,
        instruction_table_length: reader.unsigned()?,
        instruction_table_data_offset: reader.unsigned()?,
    };
    let cluster_count = checked_count(header.num_clusters, MAX_CLUSTERS, "cluster")?;
    let object_count = checked_count(header.num_objects, MAX_OBJECTS, "object")?;
    let mut result = ParseResult::new(header);
    result.clusters.reserve(cluster_count);

    let mut next_ref = result
        .header
        .num_base_objects
        .checked_add(1)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("snapshot base object count overflows i32".to_owned())
        })?;

    for _ in 0..cluster_count {
        let tags = reader.tagged32()?;
        let cid = ((tags >> 12) & 0x000f_ffff) as i32;
        let canonical = tags & (1 << 1) != 0;
        let mut cluster = Cluster::new(cid, canonical, next_ref);
        read_alloc(
            &mut reader,
            &mut cluster,
            &profile.cids,
            profile.compressed_pointers,
            is_vm,
        )?;
        next_ref = next_ref
            .checked_add(i32::try_from(cluster.count).map_err(|_| {
                ClutterError::InvalidArtifact("cluster count exceeds i32".to_owned())
            })?)
            .ok_or_else(|| {
                ClutterError::InvalidArtifact("snapshot reference count overflow".to_owned())
            })?;
        result.clusters.push(cluster);
    }

    if next_ref.saturating_sub(1) as usize > object_count {
        return Err(ClutterError::InvalidArtifact(format!(
            "cluster allocation assigned {} references for {object_count} objects",
            next_ref.saturating_sub(1)
        )));
    }
    result.fill_start = reader.position();
    Ok(result)
}

fn read_alloc(
    reader: &mut Reader<'_>,
    cluster: &mut Cluster,
    cids: &Cids,
    compressed_pointers: bool,
    is_vm: bool,
) -> Result<()> {
    match classify(cluster.cid, cids) {
        AllocKind::Simple => {
            cluster.count = read_count(reader, "simple objects")?;
        }
        AllocKind::CanonicalSet => {
            cluster.count = read_count(reader, "canonical objects")?;
            if cluster.canonical {
                skip_canonical_set(reader, cluster.count)?;
            }
        }
        AllocKind::String if compressed_pointers => {
            cluster.count = read_count(reader, "strings")?;
            cluster.lengths.reserve(cluster.count);
            for _ in 0..cluster.count {
                cluster
                    .lengths
                    .push(checked_usize(reader.unsigned()?, "encoded string length")?);
            }
            if cluster.canonical && !is_vm {
                skip_canonical_set(reader, cluster.count)?;
            }
        }
        AllocKind::String => {
            cluster.count = read_count(reader, "read-only strings")?;
            for _ in 0..cluster.count {
                cluster
                    .lengths
                    .push(checked_usize(reader.unsigned()?, "string offset delta")?);
            }
            if cluster.canonical && cluster.cid == cids.string {
                skip_canonical_set(reader, cluster.count)?;
            }
        }
        AllocKind::Mint => {
            cluster.count = read_count(reader, "mint values")?;
            cluster.allocation_values.reserve(cluster.count);
            for _ in 0..cluster.count {
                cluster.allocation_values.push(reader.tagged64()?);
            }
        }
        AllocKind::Array
        | AllocKind::WeakArray
        | AllocKind::ObjectPool
        | AllocKind::ExceptionHandlers
        | AllocKind::Context
        | AllocKind::ContextScope
        | AllocKind::Record
        | AllocKind::TypedData => {
            cluster.count = read_count(reader, "variable-length objects")?;
            for _ in 0..cluster.count {
                cluster
                    .lengths
                    .push(checked_usize(reader.unsigned()?, "variable object length")?);
            }
        }
        AllocKind::TypeArguments => {
            cluster.count = read_count(reader, "type arguments")?;
            for _ in 0..cluster.count {
                cluster
                    .lengths
                    .push(checked_usize(reader.unsigned()?, "type argument length")?);
            }
            if cluster.canonical {
                skip_canonical_set(reader, cluster.count)?;
            }
        }
        AllocKind::Class => read_class_alloc(reader, cluster, cids)?,
        AllocKind::Code => read_code_alloc(reader, cluster)?,
        AllocKind::ReadOnlyData => {
            cluster.count = read_count(reader, "read-only objects")?;
            for _ in 0..cluster.count {
                cluster
                    .lengths
                    .push(checked_usize(reader.unsigned()?, "read-only offset")?);
            }
        }
        AllocKind::Instance => {
            cluster.count = read_count(reader, "instances")?;
            cluster.next_field_words = reader.tagged32()? as i32;
            reader.tagged32()?;
        }
        AllocKind::Empty => {
            cluster.count = 0;
        }
    }
    Ok(())
}

fn read_class_alloc(reader: &mut Reader<'_>, cluster: &mut Cluster, cids: &Cids) -> Result<()> {
    let mut predefined = read_count(reader, "predefined classes")?;
    if predefined > cids.predefined_count as usize {
        predefined = read_count(reader, "predefined classes after total prefix")?;
    }
    cluster.main_count = predefined;
    for _ in 0..predefined {
        cluster.predefined_cids.push(reader.tagged32()? as i32);
    }
    let new_classes = read_count(reader, "new classes")?;
    cluster.count = predefined.checked_add(new_classes).ok_or_else(|| {
        ClutterError::InvalidArtifact("class allocation count overflow".to_owned())
    })?;
    Ok(())
}

fn read_code_alloc(reader: &mut Reader<'_>, cluster: &mut Cluster) -> Result<()> {
    let main = read_count(reader, "code objects")?;
    cluster.main_count = main;
    cluster.discarded.reserve(main);
    for _ in 0..main {
        cluster.discarded.push(reader.tagged32()? & (1 << 3) != 0);
    }
    let deferred = read_count(reader, "deferred code objects")?;
    for _ in 0..deferred {
        cluster.discarded.push(reader.tagged32()? & (1 << 3) != 0);
    }
    cluster.count = main.checked_add(deferred).ok_or_else(|| {
        ClutterError::InvalidArtifact("code allocation count overflow".to_owned())
    })?;
    Ok(())
}

fn skip_canonical_set(reader: &mut Reader<'_>, count: usize) -> Result<()> {
    let table_length = checked_usize(reader.unsigned()?, "canonical table length")?;
    if table_length > MAX_OBJECTS.saturating_mul(16) {
        return Err(ClutterError::InvalidArtifact(format!(
            "canonical table length {table_length} exceeds limit"
        )));
    }
    let first = checked_usize(reader.unsigned()?, "canonical first element")?;
    if first > count {
        return Err(ClutterError::InvalidArtifact(format!(
            "canonical first element {first} exceeds object count {count}"
        )));
    }
    for _ in first..count {
        reader.unsigned()?;
    }
    Ok(())
}

fn classify(cid: i32, cids: &Cids) -> AllocKind {
    if matches_cid(
        cid,
        &[cids.string, cids.one_byte_string, cids.two_byte_string],
    ) {
        AllocKind::String
    } else if cid == cids.mint {
        AllocKind::Mint
    } else if matches_cid(
        cid,
        &[cids.double, cids.float32x4, cids.int32x4, cids.float64x2],
    ) {
        AllocKind::Simple
    } else if matches_cid(cid, &[cids.array, cids.immutable_array]) {
        AllocKind::Array
    } else if cid == cids.weak_array {
        AllocKind::WeakArray
    } else if cid == cids.type_arguments {
        AllocKind::TypeArguments
    } else if matches_cid(
        cid,
        &[
            cids.type_,
            cids.function_type,
            cids.record_type,
            cids.type_parameter,
        ],
    ) {
        AllocKind::CanonicalSet
    } else if cid == cids.class {
        AllocKind::Class
    } else if cid == cids.code {
        AllocKind::Code
    } else if cid == cids.object_pool {
        AllocKind::ObjectPool
    } else if matches_cid(
        cid,
        &[
            cids.pc_descriptors,
            cids.code_source_map,
            cids.compressed_stack_maps,
        ],
    ) {
        AllocKind::ReadOnlyData
    } else if cid == cids.exception_handlers {
        AllocKind::ExceptionHandlers
    } else if cid == cids.context {
        AllocKind::Context
    } else if cid == cids.context_scope {
        AllocKind::ContextScope
    } else if cid == cids.record {
        AllocKind::Record
    } else if is_typed_data(cid, cids) {
        AllocKind::TypedData
    } else if cid == cids.weak_serialization_reference {
        AllocKind::Empty
    } else if matches_cid(
        cid,
        &[
            cids.function,
            cids.type_parameters,
            cids.closure_data,
            cids.ffi_trampoline_data,
            cids.field,
            cids.script,
            cids.library,
            cids.namespace,
            cids.kernel_program_info,
            cids.patch_class,
            cids.sentinel,
            cids.single_target_cache,
            cids.unlinked_call,
            cids.monomorphic_smiable_call,
            cids.call_site_data,
            cids.ic_data,
            cids.megamorphic_cache,
            cids.subtype_test_cache,
            cids.loading_unit,
            cids.language_error,
            cids.unhandled_exception,
            cids.library_prefix,
            cids.closure,
            cids.external_typed_data,
            cids.typed_data_view,
            cids.growable_array,
            cids.capability,
            cids.receive_port,
            cids.send_port,
            cids.stack_trace,
            cids.suspend_state,
            cids.regexp,
            cids.weak_property,
            cids.weak_reference,
            cids.future_or,
            cids.user_tag,
            cids.transferable_typed_data,
            cids.map,
            cids.const_map,
            cids.set,
            cids.const_set,
        ],
    ) {
        AllocKind::Simple
    } else if cid == cids.instance || cid >= cids.predefined_count {
        AllocKind::Instance
    } else {
        AllocKind::Simple
    }
}

pub fn is_typed_data(cid: i32, cids: &Cids) -> bool {
    cid == cids.typed_data
        || (cid >= cids.typed_data_first
            && cid < cids.byte_data_view
            && (cid - cids.typed_data_first) % 4 == 0)
        || cid == 1
}

fn matches_cid(value: i32, candidates: &[i32]) -> bool {
    candidates.contains(&value)
}

fn read_count(reader: &mut Reader<'_>, label: &str) -> Result<usize> {
    checked_count(reader.unsigned()?, MAX_OBJECTS, label)
}

fn checked_count(value: i64, maximum: usize, label: &str) -> Result<usize> {
    let value = checked_usize(value, label)?;
    if value > maximum {
        return Err(ClutterError::InvalidArtifact(format!(
            "{label} count {value} exceeds limit {maximum}"
        )));
    }
    Ok(value)
}

fn checked_usize(value: i64, label: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| {
        ClutterError::InvalidArtifact(format!("{label} value {value} is negative or too large"))
    })
}
