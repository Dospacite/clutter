use crate::diagnostic::{ClutterError, Result};

use super::cid::Cids;
use super::reader::Reader;
use super::types::{
    Cluster, ExceptionHandler, ExceptionHandlers, ParseResult, SnapshotObjectKind,
    SnapshotObjectPayload, SnapshotScalar,
};

pub fn inline_bytes(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    for reference in references_for(cluster) {
        let length = length(reader, "inline byte length")?;
        let bytes = reader.bytes(length)?.to_vec();
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::MetadataBytes,
            Vec::new(),
            vec![SnapshotScalar::Unsigned(length as i64)],
            bytes,
        );
    }
    Ok(())
}

pub fn array(reader: &mut Reader<'_>, cluster: &Cluster, result: &mut ParseResult) -> Result<()> {
    for reference in references_for(cluster) {
        let count = length(reader, "array length")?;
        let type_arguments = checked_reference(reader.reference()?)?;
        let mut object_references = vec![type_arguments];
        object_references.extend(read_references(reader, count)?);
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Array,
            object_references,
            vec![SnapshotScalar::Unsigned(count as i64)],
            Vec::new(),
        );
    }
    Ok(())
}

pub fn weak_array(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    for reference in references_for(cluster) {
        let count = length(reader, "weak array length")?;
        let object_references = read_references(reader, count)?;
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::WeakArray,
            object_references,
            vec![SnapshotScalar::Unsigned(count as i64)],
            Vec::new(),
        );
    }
    Ok(())
}

pub fn typed_data(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    cids: &Cids,
    result: &mut ParseResult,
) -> Result<()> {
    let element_size = typed_data_element_size(cluster.cid, cids);
    for reference in references_for(cluster) {
        let count = length(reader, "typed data length")?;
        let byte_count = count.checked_mul(element_size).ok_or_else(|| {
            ClutterError::InvalidArtifact("typed data byte length overflow".to_owned())
        })?;
        let bytes = reader.bytes(byte_count)?.to_vec();
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::TypedData,
            Vec::new(),
            vec![
                SnapshotScalar::Unsigned(count as i64),
                SnapshotScalar::Unsigned(element_size as i64),
            ],
            bytes,
        );
    }
    Ok(())
}

pub fn exception_handlers(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    for reference in references_for(cluster) {
        let packed = length(reader, "exception handler flags")?;
        let count = packed >> 1;
        let handled_types_ref = checked_reference(reader.reference()?)?;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(ExceptionHandler {
                handler_pc_offset: reader.u32()?,
                outer_try_index: reader.i16()?,
                needs_stack_trace: reader.byte()? != 0,
                has_catch_all: reader.byte()? != 0,
                is_generated: reader.byte()? != 0,
            });
        }
        result.exception_handlers.insert(
            reference,
            ExceptionHandlers {
                reference,
                has_async_handler: packed & 1 != 0,
                handled_types_ref,
                entries,
            },
        );
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::ExceptionHandlers,
            vec![handled_types_ref],
            vec![SnapshotScalar::Unsigned(packed as i64)],
            Vec::new(),
        );
    }
    Ok(())
}

pub fn context(reader: &mut Reader<'_>, cluster: &Cluster, result: &mut ParseResult) -> Result<()> {
    for reference in references_for(cluster) {
        let count = length(reader, "context length")?;
        let parent = checked_reference(reader.reference()?)?;
        let mut object_references = vec![parent];
        object_references.extend(read_references(reader, count)?);
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Context,
            object_references,
            vec![SnapshotScalar::Unsigned(count as i64)],
            Vec::new(),
        );
    }
    Ok(())
}

pub fn type_arguments(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    for reference in references_for(cluster) {
        let count = length(reader, "type argument length")?;
        let hash = reader.i32()?;
        let nullability = reader.unsigned()?;
        let instantiations = checked_reference(reader.reference()?)?;
        let mut object_references = vec![instantiations];
        object_references.extend(read_references(reader, count)?);
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::TypeArguments,
            object_references,
            vec![
                SnapshotScalar::Unsigned(count as i64),
                SnapshotScalar::Tagged32(hash as u32),
                SnapshotScalar::Unsigned(nullability),
            ],
            Vec::new(),
        );
    }
    Ok(())
}

pub fn instance(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    header_words: usize,
    unboxed_word_u32_chunks: usize,
    result: &mut ParseResult,
) -> Result<()> {
    let bitmap = reader.unsigned()? as u64;
    result.instance_bitmaps.insert(cluster.cid, bitmap);
    let field_count = usize::try_from(cluster.next_field_words)
        .unwrap_or_default()
        .saturating_sub(header_words);
    for (object_index, reference) in references_for(cluster).enumerate() {
        let mut object_references = Vec::new();
        let mut scalars = Vec::new();
        for index in 0..field_count {
            let word = header_words + index;
            if bitmap & (1u64 << word.min(63)) != 0 {
                for _ in 0..unboxed_word_u32_chunks {
                    scalars.push(SnapshotScalar::Tagged32(reader.u32()?));
                }
            } else {
                let offset = reader.position();
                let value = reader.reference()?;
                let value = i32::try_from(value).map_err(|_| {
                    ClutterError::InvalidArtifact(format!(
                        "instance CID {} object {object_index} field word {word} at \
                         0x{offset:x} decoded invalid reference {value} \
                         (bitmap 0x{bitmap:016x})",
                        cluster.cid
                    ))
                })?;
                object_references.push(value);
            }
        }
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Instance,
            object_references,
            scalars,
            Vec::new(),
        );
    }
    Ok(())
}

pub fn record(reader: &mut Reader<'_>, cluster: &Cluster, result: &mut ParseResult) -> Result<()> {
    for reference in references_for(cluster) {
        let shape = reader.unsigned()?;
        let count = usize::try_from(shape & 0xffff)
            .map_err(|_| ClutterError::InvalidArtifact("record shape is invalid".to_owned()))?;
        let object_references = read_references(reader, count)?;
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::Record,
            object_references,
            vec![SnapshotScalar::Unsigned(shape)],
            Vec::new(),
        );
    }
    Ok(())
}

pub fn context_scope(
    reader: &mut Reader<'_>,
    cluster: &Cluster,
    result: &mut ParseResult,
) -> Result<()> {
    for reference in references_for(cluster) {
        let count = length(reader, "context scope length")?;
        let implicit = reader.byte()?;
        let object_references = read_references(
            reader,
            count.checked_mul(7).ok_or_else(|| {
                ClutterError::InvalidArtifact("context scope size overflow".to_owned())
            })?,
        )?;
        insert(
            result,
            cluster,
            reference,
            SnapshotObjectKind::ContextScope,
            object_references,
            vec![
                SnapshotScalar::Unsigned(count as i64),
                SnapshotScalar::Byte(implicit),
            ],
            Vec::new(),
        );
    }
    Ok(())
}

fn references_for(cluster: &Cluster) -> impl Iterator<Item = i32> {
    let count = i32::try_from(cluster.count).unwrap_or(i32::MAX);
    cluster.start_ref..cluster.start_ref.saturating_add(count)
}

fn insert(
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

fn read_references(reader: &mut Reader<'_>, count: usize) -> Result<Vec<i32>> {
    (0..count)
        .map(|_| reader.reference().and_then(checked_reference))
        .collect()
}

fn checked_reference(value: i64) -> Result<i32> {
    i32::try_from(value).map_err(|_| {
        ClutterError::InvalidArtifact(format!("snapshot reference {value} exceeds i32"))
    })
}

fn length(reader: &mut Reader<'_>, label: &str) -> Result<usize> {
    usize::try_from(reader.unsigned()?)
        .map_err(|_| ClutterError::InvalidArtifact(format!("{label} is negative or too large")))
}

fn typed_data_element_size(cid: i32, cids: &Cids) -> usize {
    if cid == 1 || cid == cids.typed_data {
        return 1;
    }
    let index = (cid - cids.typed_data_first) / 4;
    const SIZES: [usize; 14] = [1, 1, 1, 2, 2, 4, 4, 8, 8, 4, 8, 16, 16, 16];
    usize::try_from(index)
        .ok()
        .and_then(|index| SIZES.get(index).copied())
        .unwrap_or(1)
}
