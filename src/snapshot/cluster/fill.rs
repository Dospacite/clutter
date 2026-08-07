use crate::diagnostic::{ClutterError, Result};

use super::cid::Profile;
use super::fill_skip;
use super::fill_spec::{self, FillKind, Scalar, Spec};
use super::reader::Reader;
use super::types::{
    Cluster, Code, FunctionType, NamedObject, ObjectPool, ParseResult, PoolValue,
    SnapshotObjectKind, SnapshotObjectPayload, SnapshotScalar,
};

pub fn read(data: &[u8], result: &mut ParseResult, profile: &Profile) -> Result<usize> {
    let mut reader = Reader::at(data, result.fill_start)?;
    let mut instruction_index = 0usize;
    let mut recent = std::collections::VecDeque::with_capacity(5);

    let clusters = result.clusters.clone();
    for (cluster_index, cluster) in clusters.iter().enumerate() {
        let fill_offset = reader.position();
        let spec = fill_spec::for_cid(cluster.cid, &profile.cids, profile.compressed_pointers);
        let outcome = (|| -> Result<()> {
            match spec.kind {
                FillKind::String => read_strings(&mut reader, cluster, result)?,
                FillKind::None => {
                    for (index, reference) in references_for(cluster).enumerate() {
                        let allocation_value = cluster.allocation_values.get(index).copied();
                        insert_object(
                            result,
                            cluster,
                            reference,
                            if allocation_value.is_some() {
                                SnapshotObjectKind::Integer
                            } else {
                                SnapshotObjectKind::Standard
                            },
                            Vec::new(),
                            allocation_value
                                .map(SnapshotScalar::Tagged64)
                                .into_iter()
                                .collect(),
                            Vec::new(),
                        );
                    }
                }
                FillKind::Refs => read_refs(&mut reader, cluster, &spec, result)?,
                FillKind::Double => {
                    for reference in references_for(cluster) {
                        let value = reader.tagged64()?;
                        insert_object(
                            result,
                            cluster,
                            reference,
                            SnapshotObjectKind::Double,
                            Vec::new(),
                            vec![SnapshotScalar::Tagged64(value)],
                            Vec::new(),
                        );
                    }
                }
                FillKind::Code => {
                    read_code(&mut reader, cluster, result, &mut instruction_index)?;
                }
                FillKind::ObjectPool => read_object_pool(&mut reader, cluster, result)?,
                FillKind::Array => fill_skip::array(&mut reader, cluster, result)?,
                FillKind::WeakArray => fill_skip::weak_array(&mut reader, cluster, result)?,
                FillKind::TypedData => {
                    fill_skip::typed_data(&mut reader, cluster, &profile.cids, result)?;
                }
                FillKind::ExceptionHandlers => {
                    fill_skip::exception_handlers(&mut reader, cluster, result)?;
                }
                FillKind::Context => fill_skip::context(&mut reader, cluster, result)?,
                FillKind::TypeArguments => {
                    fill_skip::type_arguments(&mut reader, cluster, result)?;
                }
                FillKind::InlineBytes => fill_skip::inline_bytes(&mut reader, cluster, result)?,
                FillKind::Instance => {
                    fill_skip::instance(
                        &mut reader,
                        cluster,
                        profile.instance_header_words,
                        profile.unboxed_word_u32_chunks,
                        result,
                    )?;
                }
                FillKind::Record => fill_skip::record(&mut reader, cluster, result)?,
                FillKind::ContextScope => {
                    fill_skip::context_scope(&mut reader, cluster, result)?;
                }
                FillKind::Class => {
                    read_classes(&mut reader, cluster, result, &profile.cids)?;
                }
            }
            Ok(())
        })();
        if let Err(error) = outcome {
            let trail = recent
                .iter()
                .map(|(cid, count, fields, start, end)| {
                    format!("CID {cid} ({count} objects, {fields} words) 0x{start:x}..0x{end:x}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ClutterError::InvalidArtifact(format!(
                "snapshot fill cluster {cluster_index} (CID {}, {} objects) at 0x{fill_offset:x}: {error}; preceding clusters: {trail}",
                cluster.cid, cluster.count,
            )));
        }
        if recent.len() == 5 {
            recent.pop_front();
        }
        recent.push_back((
            cluster.cid,
            cluster.count,
            cluster.next_field_words,
            fill_offset,
            reader.position(),
        ));
    }
    result.rebuild_back_references();
    Ok(reader.position())
}

fn read_strings(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    let mut reference = cluster.start_ref;
    for _ in 0..cluster.count {
        let encoded = reader.unsigned()?;
        let length = usize::try_from(encoded >> 1)
            .map_err(|_| ClutterError::InvalidArtifact("string length is invalid".to_owned()))?;
        let two_byte = encoded & 1 != 0;
        let (value, raw) = if two_byte {
            let raw = reader.bytes(length.checked_mul(2).ok_or_else(|| {
                ClutterError::InvalidArtifact("UTF-16 string size overflow".to_owned())
            })?)?;
            let units: Vec<_> = raw
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            (String::from_utf16_lossy(&units), raw.to_vec())
        } else {
            let raw = reader.bytes(length)?;
            (super::types::decode_one_byte_string(raw), raw.to_vec())
        };
        result.strings.insert(reference, value);
        insert_object(
            result,
            cluster,
            reference,
            SnapshotObjectKind::String,
            Vec::new(),
            vec![
                SnapshotScalar::Unsigned(length as i64),
                SnapshotScalar::Byte(u8::from(two_byte)),
            ],
            raw,
        );
        reference = reference.saturating_add(1);
    }
    Ok(())
}

fn read_refs(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    spec: &Spec,
    result: &mut ParseResult,
) -> Result<()> {
    let mut reference = cluster.start_ref;
    for _ in 0..cluster.count {
        let mut name_ref = -1;
        let mut owner_ref = -1;
        let mut signature_ref = -1;
        let mut library_uri_ref = -1;
        let mut function_kind_tag = None;
        let mut instruction_index = None;
        let mut references = Vec::with_capacity(spec.refs);
        for index in 0..spec.refs {
            let value = checked_reference(reader.reference()?)?;
            references.push(value);
            if spec.name_index == Some(index) {
                name_ref = value;
            }
            if spec.owner_index == Some(index) {
                owner_ref = value;
            }
            if spec.signature_index == Some(index) {
                signature_ref = value;
            }
            if cluster.cid == 13 && index == 1 {
                library_uri_ref = value;
            }
        }

        let mut scalars = Vec::with_capacity(spec.scalars.len());
        for (index, scalar) in spec.scalars.iter().enumerate() {
            let value = read_scalar(reader, *scalar)?;
            if spec.function && index == 0 {
                let code_index = match value {
                    SnapshotScalar::Unsigned(value) => value,
                    _ => unreachable!("function code index is an unsigned scalar"),
                };
                instruction_index = code_index
                    .checked_sub(1)
                    .and_then(|value| usize::try_from(value).ok());
            } else if spec.function_type && index == 1 {
                let packed = match value {
                    SnapshotScalar::Tagged32(value) => value,
                    _ => unreachable!("function-type parameter counts are a u32 scalar"),
                };
                let implicit = (packed & 1) as usize;
                let optional_are_named = packed & 2 != 0;
                let mut fixed = ((packed >> 2) & 0x3fff) as usize;
                let optional = ((packed >> 16) & 0x3fff) as usize;
                fixed = fixed.saturating_sub(implicit);
                result.function_types.insert(
                    reference,
                    FunctionType {
                        fixed,
                        optional,
                        optional_are_named,
                        implicit,
                        type_parameters_ref: references.get(2).copied(),
                        result_type_ref: references.get(3).copied(),
                        parameter_types_ref: references.get(4).copied(),
                        named_parameter_names_ref: references.get(5).copied(),
                        flags: scalars.first().and_then(snapshot_byte).unwrap_or_default(),
                        packed_type_parameter_counts: 0,
                    },
                );
            } else if spec.function && index == 1 {
                function_kind_tag = match value {
                    SnapshotScalar::Tagged32(value) => Some(value),
                    _ => None,
                };
            }
            scalars.push(value);
        }
        if let Some(function_type) = result.function_types.get_mut(&reference) {
            function_type.packed_type_parameter_counts = scalars
                .get(2)
                .and_then(snapshot_u16)
                .map(u32::from)
                .unwrap_or_default();
        }

        if spec.name_index.is_some() {
            if library_uri_ref >= 0 {
                result.library_uris.insert(reference, library_uri_ref);
                name_ref = library_uri_ref;
            }
            result.named.insert(
                reference,
                NamedObject {
                    cid: cluster.cid,
                    name_ref,
                    owner_ref,
                    signature_ref,
                    function_kind_tag,
                    instruction_index,
                    source_uri_ref: None,
                },
            );
        }
        insert_object(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Standard,
            references,
            scalars,
            Vec::new(),
        );
        reference = reference.saturating_add(1);
    }
    Ok(())
}

fn read_classes(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
    cids: &super::cid::Cids,
) -> Result<()> {
    resynchronize_class_fill(reader, cluster)?;
    let mut reference = cluster.start_ref;
    let mut previous_class = None;
    let mut recent_class_ids = std::collections::VecDeque::with_capacity(8);
    for index in 0..cluster.count {
        let mut name_ref = -1;
        let mut library_ref = -1;
        let mut source_uri_ref = None;
        let mut references = Vec::with_capacity(13);
        for field in 0..13 {
            let value = reader
                .reference()
                .and_then(checked_reference)
                .map_err(|error| {
                    let previous = previous_class.map_or_else(
                        || "none".to_owned(),
                        |(prior_index, prior_cid, predefined, top_level)| {
                            format!(
                                "{prior_index} (class ID {prior_cid}, predefined={predefined}, top-level={top_level})"
                            )
                        },
                    );
                    let recent = recent_class_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    ClutterError::InvalidArtifact(format!(
                        "class {index}/{} reference {field}/13 after class {previous} (recent class IDs: {recent}): {error}",
                        cluster.count,
                    ))
                })?;
            references.push(value);
            if field == 0 {
                name_ref = value;
            }
            if reference_has_cid(result, value, cids.library) {
                library_ref = value;
            }
            if reference_has_cid(result, value, cids.script) {
                source_uri_ref = Some(value);
            }
        }
        let class_id = reader.i32()?;
        let mut scalars = vec![
            SnapshotScalar::Tagged32(class_id as u32),
            SnapshotScalar::Tagged32(reader.u32()?),
            SnapshotScalar::Tagged32(reader.u32()?),
            SnapshotScalar::Tagged32(reader.u32()?),
            SnapshotScalar::Int16(reader.i16()?),
            SnapshotScalar::Uint16(reader.u16()?),
            SnapshotScalar::Tagged32(reader.u32()?),
        ];

        let predefined = index < cluster.main_count;
        let top_level = i64::from(class_id) >= 1 << 20;
        if predefined || !top_level {
            scalars.push(SnapshotScalar::Unsigned(reader.unsigned()?));
        }
        previous_class = Some((index, class_id, predefined, top_level));
        if recent_class_ids.len() == 8 {
            recent_class_ids.pop_front();
        }
        recent_class_ids.push_back(class_id);
        result.named.insert(
            reference,
            NamedObject {
                cid: cluster.cid,
                name_ref,
                owner_ref: library_ref,
                signature_ref: -1,
                function_kind_tag: None,
                instruction_index: None,
                source_uri_ref,
            },
        );
        insert_object(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Class,
            references,
            scalars,
            Vec::new(),
        );
        reference = reference.saturating_add(1);
    }
    Ok(())
}

fn resynchronize_class_fill(reader: &mut Reader<'_>, cluster: &Cluster) -> Result<()> {
    if cluster.predefined_cids.is_empty() || predefined_classes_match(reader, cluster) {
        return Ok(());
    }

    let current = reader.position();
    const SEARCH_RADIUS: usize = 16 * 1024;
    let start = current.saturating_sub(SEARCH_RADIUS);
    let end = current
        .saturating_add(SEARCH_RADIUS)
        .min(reader.data().len());
    let mut best = None;
    for candidate in start..=end {
        let mut probe = Reader::at(reader.data(), candidate)?;
        if predefined_classes_match(&mut probe, cluster) {
            let distance = current.abs_diff(candidate);
            if best.is_none_or(|(best_distance, _)| distance < best_distance) {
                best = Some((distance, candidate));
            }
        }
    }
    if let Some((_, position)) = best {
        reader.set_position(position)?;
    }
    Ok(())
}

fn predefined_classes_match(reader: &mut Reader<'_>, cluster: &Cluster) -> bool {
    let mut probe = reader.clone();
    for expected in cluster.predefined_cids.iter().take(12) {
        for _ in 0..13 {
            if probe.reference().is_err() {
                return false;
            }
        }
        let Ok(class_id) = probe.i32() else {
            return false;
        };
        if class_id != *expected {
            return false;
        }
        if probe.skip(12 + 2 + 2 + 4).is_err() {
            return false;
        }
        if probe.unsigned().is_err() {
            return false;
        }
    }
    true
}

fn reference_has_cid(result: &ParseResult, reference: i32, cid: i32) -> bool {
    result.clusters.iter().any(|cluster| {
        let Ok(count) = i32::try_from(cluster.count) else {
            return false;
        };
        cluster.cid == cid
            && reference >= cluster.start_ref
            && reference < cluster.start_ref.saturating_add(count)
    })
}

fn read_code(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
    instruction_index: &mut usize,
) -> Result<()> {
    let mut reference = cluster.start_ref;
    for index in 0..cluster.count {
        let main = index < cluster.main_count;
        let discarded = cluster.discarded.get(index).copied().unwrap_or(false);
        let payload_info = if main {
            let payload_info = reader.unsigned()?;
            let current = *instruction_index;
            *instruction_index = instruction_index.saturating_add(1);
            (Some(current), Some(payload_info))
        } else {
            (None, None)
        };
        let (code_index, payload_info) = payload_info;
        let mut references = Vec::new();
        if discarded {
            references.push(checked_reference(reader.reference()?)?);
        } else {
            for _ in 0..6 {
                references.push(checked_reference(reader.reference()?)?);
            }
        }
        let owner_ref = references.first().copied().unwrap_or_default();
        let payload = payload_info.unwrap_or_default() as u64;
        result.codes.push(Code {
            ref_id: reference,
            owner_ref,
            instruction_index: code_index,
            payload_info,
            unchecked_entry_offset: payload_info.map(|_| payload >> 1),
            has_monomorphic_entrypoint: payload_info.is_some() && payload & 1 != 0,
            exception_handlers_ref: references.get(1).copied(),
            pc_descriptors_ref: references.get(2).copied(),
            catch_entry_ref: references.get(3).copied(),
            inlined_functions_ref: references.get(4).copied(),
            code_source_map_ref: references.get(5).copied(),
        });
        insert_object(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Code,
            references,
            payload_info
                .map(SnapshotScalar::Unsigned)
                .into_iter()
                .collect(),
            Vec::new(),
        );
        reference = reference.saturating_add(1);
    }
    Ok(())
}

fn read_object_pool(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    let mut reference = cluster.start_ref;
    for _ in 0..cluster.count {
        let length = usize::try_from(reader.unsigned()?).map_err(|_| {
            ClutterError::InvalidArtifact("object pool length is invalid".to_owned())
        })?;
        let mut entries = Vec::with_capacity(length);
        let mut scalars = vec![SnapshotScalar::Unsigned(length as i64)];
        let mut references = Vec::new();
        for _ in 0..length {
            let bits = reader.byte()?;
            let behavior = bits >> 5;
            let kind = bits & 0x0f;
            let value = match (behavior, kind) {
                (0, 0) => PoolValue::Immediate(reader.tagged64()?),
                (0, 1) => PoolValue::Reference(checked_reference(reader.reference()?)?),
                (0, 2) => PoolValue::Native,
                (1..=4, _) => PoolValue::Empty,
                _ => {
                    return Err(ClutterError::InvalidArtifact(format!(
                        "unsupported object-pool entry bits 0x{bits:02x}"
                    )));
                }
            };
            scalars.push(SnapshotScalar::Byte(bits));
            if let PoolValue::Reference(value) = value {
                references.push(value);
            }
            entries.push(value);
        }
        result.pool.extend(entries.iter().cloned());
        result.object_pools.push(ObjectPool { reference, entries });
        insert_object(
            result,
            cluster,
            reference,
            SnapshotObjectKind::ObjectPool,
            references,
            scalars,
            Vec::new(),
        );
        reference = reference.saturating_add(1);
    }
    Ok(())
}

fn read_scalar(reader: &mut Reader<'_>, scalar: Scalar) -> Result<SnapshotScalar> {
    Ok(match scalar {
        Scalar::Tagged32 => SnapshotScalar::Tagged32(reader.tagged32()?),
        Scalar::Tagged64 => SnapshotScalar::Tagged64(reader.tagged64()?),
        Scalar::Uint16 => SnapshotScalar::Uint16(reader.u16()?),
        Scalar::Unsigned => SnapshotScalar::Unsigned(reader.unsigned()?),
        Scalar::Byte => SnapshotScalar::Byte(reader.byte()?),
        Scalar::Reference => SnapshotScalar::Reference(checked_reference(reader.reference()?)?),
    })
}

fn snapshot_byte(value: &SnapshotScalar) -> Option<u8> {
    match value {
        SnapshotScalar::Byte(value) => Some(*value),
        _ => None,
    }
}

fn snapshot_u16(value: &SnapshotScalar) -> Option<u16> {
    match value {
        SnapshotScalar::Uint16(value) => Some(*value),
        _ => None,
    }
}

fn references_for(cluster: &Cluster) -> impl Iterator<Item = i32> {
    let count = i32::try_from(cluster.count).unwrap_or(i32::MAX);
    cluster.start_ref..cluster.start_ref.saturating_add(count)
}

fn insert_object(
    result: &mut ParseResult,
    cluster: &Cluster,
    reference: i32,
    kind: SnapshotObjectKind,
    references: Vec<i32>,
    scalars: Vec<SnapshotScalar>,
    bytes: Vec<u8>,
) {
    result.insert_object(
        reference,
        cluster.cid,
        cluster.canonical,
        kind,
        SnapshotObjectPayload {
            references,
            scalars,
            bytes,
        },
    );
}

fn checked_reference(value: i64) -> Result<i32> {
    i32::try_from(value).map_err(|_| {
        ClutterError::InvalidArtifact(format!("snapshot reference {value} exceeds i32"))
    })
}
