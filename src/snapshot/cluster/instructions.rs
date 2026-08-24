use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;

use crate::diagnostic::{ClutterError, Result};
use crate::model::{
    Abi, CodeSourceMapEntry, CodeSourceMapOperation, RecoveredCodeMetadata, RecoveredDeclaration,
    RecoveredDeclarationKind, RecoveredDispatchRun, RecoveredDispatchTable,
    RecoveredExceptionHandler, RecoveredFunction, RecoveredFunctionKind, RecoveredNameSource,
    RecoveredPcDescriptor, RecoveredSignatureSource, Scope, SnapshotEvidence,
};
use crate::snapshot::CodeImage;

use super::cid::Cids;
use super::type_recovery::TypeRecovery;
use super::types::{CompressedStackMap, InstructionEntry, InstructionTable, ParseResult};

#[derive(Clone)]
struct Range {
    code_ref: i32,
    owner_ref: i32,
    pc_offset: u32,
    stack_map_offset: u32,
    size: u32,
}

pub(super) struct ResolveOptions<'a> {
    pub abi: Abi,
    pub scope: Scope,
    pub application_package: Option<&'a str>,
    pub obfuscation_map: Option<&'a crate::analysis::LoadedObfuscationMap>,
}

pub fn parse_table(
    data: &[u8],
    header: &super::types::ClusterHeader,
    snapshot_size: u64,
    pointer_width: usize,
) -> Result<InstructionTable> {
    if header.instruction_table_data_offset <= 0 {
        return Err(ClutterError::InvalidArtifact(
            "snapshot does not contain an instruction table offset".to_owned(),
        ));
    }
    let data_image_start = round_up(snapshot_size, 64);
    let object_offset = data_image_start
        .checked_add(header.instruction_table_data_offset as u64)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("instruction table offset overflow".to_owned())
        })?;
    let string_header_size = if pointer_width == 4 { 12 } else { 16 };
    let payload_offset = object_offset
        .checked_add(string_header_size)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("instruction table payload overflow".to_owned())
        })? as usize;
    let header_bytes = data
        .get(payload_offset..payload_offset + 16)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("instruction table header is truncated".to_owned())
        })?;
    let canonical_stack_map_offset =
        u32::from_le_bytes(header_bytes[..4].try_into().expect("four bytes"));
    let length = u32::from_le_bytes(header_bytes[4..8].try_into().expect("four bytes")) as usize;
    let first_code =
        u32::from_le_bytes(header_bytes[8..12].try_into().expect("four bytes")) as usize;
    if first_code > length {
        return Err(ClutterError::InvalidArtifact(format!(
            "instruction table first-code index {first_code} exceeds length {length}"
        )));
    }
    let retained_length = length.saturating_sub(first_code);
    if header.instruction_table_length > 0
        && retained_length != header.instruction_table_length as usize
    {
        return Err(ClutterError::InvalidArtifact(format!(
            "instruction table has {retained_length} retained entries ({length} total, first retained {first_code}) at object 0x{object_offset:x} (payload 0x{payload_offset:x}), but the snapshot header declares {}",
            header.instruction_table_length,
        )));
    }
    let entries_offset = payload_offset + 16;
    let entries_size = length.checked_mul(8).ok_or_else(|| {
        ClutterError::InvalidArtifact("instruction table byte size overflow".to_owned())
    })?;
    let entries_bytes = data
        .get(entries_offset..entries_offset + entries_size)
        .ok_or_else(|| {
            ClutterError::InvalidArtifact("instruction table entries are truncated".to_owned())
        })?;
    let entries = entries_bytes
        .chunks_exact(8)
        .map(|entry| InstructionEntry {
            pc_offset: u32::from_le_bytes(entry[..4].try_into().expect("four bytes")),
            stack_map_offset: u32::from_le_bytes(entry[4..].try_into().expect("four bytes")),
        })
        .collect::<Vec<_>>();
    let mut stack_maps = BTreeMap::new();
    let mut stack_map_offsets = entries
        .iter()
        .map(|entry| entry.stack_map_offset)
        .filter(|offset| *offset != 0)
        .collect::<std::collections::BTreeSet<_>>();
    if canonical_stack_map_offset != 0 {
        stack_map_offsets.insert(canonical_stack_map_offset);
    }
    for stack_map_offset in stack_map_offsets {
        let Some(header_offset) = payload_offset.checked_add(stack_map_offset as usize) else {
            continue;
        };
        let Some(header) = data.get(header_offset..header_offset.saturating_add(4)) else {
            continue;
        };
        let flags_and_size = u32::from_le_bytes(header.try_into().expect("four bytes"));
        let payload_size = (flags_and_size >> 2) as usize;
        let Some(payload) = data.get(
            header_offset.saturating_add(4)
                ..header_offset.saturating_add(4).saturating_add(payload_size),
        ) else {
            continue;
        };
        let global_table = flags_and_size & 1 != 0;
        let uses_global_table = flags_and_size & 2 != 0;
        stack_maps.insert(
            stack_map_offset,
            CompressedStackMap {
                offset: stack_map_offset,
                global_table,
                uses_global_table,
                payload: payload.to_vec(),
                entry_count: count_stack_map_entries(payload, global_table, uses_global_table),
            },
        );
    }
    Ok(InstructionTable {
        first_code,
        entries,
        canonical_stack_map_offset: (canonical_stack_map_offset != 0)
            .then_some(canonical_stack_map_offset),
        stack_maps,
    })
}

fn count_stack_map_entries(payload: &[u8], global_table: bool, uses_global_table: bool) -> usize {
    if global_table && uses_global_table {
        return 0;
    }
    let mut cursor = 0usize;
    let mut entries = 0usize;
    while cursor < payload.len() {
        if !global_table && read_uleb128(payload, &mut cursor).is_none() {
            break;
        }
        if uses_global_table {
            if read_uleb128(payload, &mut cursor).is_none() {
                break;
            }
        } else {
            let Some(spill_bits) = read_uleb128(payload, &mut cursor) else {
                break;
            };
            let Some(non_spill_bits) = read_uleb128(payload, &mut cursor) else {
                break;
            };
            let bitmap_bytes = spill_bits
                .saturating_add(non_spill_bits)
                .div_ceil(8)
                .try_into()
                .unwrap_or(usize::MAX);
            let Some(next) = cursor.checked_add(bitmap_bytes) else {
                break;
            };
            if next > payload.len() {
                break;
            }
            cursor = next;
        }
        entries += 1;
    }
    entries
}

fn read_uleb128(bytes: &[u8], cursor: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    let mut shift = 0u32;
    loop {
        let byte = *bytes.get(*cursor)?;
        *cursor = (*cursor).saturating_add(1);
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
}

pub fn resolve(
    isolate: &ParseResult,
    vm: &ParseResult,
    cids: &Cids,
    table: &InstructionTable,
    image: &CodeImage,
    options: ResolveOptions<'_>,
) -> Result<super::Recovery> {
    let mut ranges = Vec::new();
    let mut claimed_slots = std::collections::BTreeSet::new();
    for (function_ref, function) in &isolate.named {
        let Some(slot) = function.instruction_index else {
            continue;
        };
        let Some(entry) = table.entries.get(slot) else {
            continue;
        };
        claimed_slots.insert(slot);
        ranges.push(Range {
            code_ref: *function_ref,
            owner_ref: *function_ref,
            pc_offset: entry.pc_offset,
            stack_map_offset: entry.stack_map_offset,
            size: 0,
        });
    }
    for code in &isolate.codes {
        let Some(index) = code.instruction_index else {
            continue;
        };
        let slot = table.first_code.checked_add(index).ok_or_else(|| {
            ClutterError::InvalidArtifact("instruction table slot overflow".to_owned())
        })?;
        let entry = table.entries.get(slot).ok_or_else(|| {
            ClutterError::InvalidArtifact(format!(
                "code reference {} maps to missing instruction slot {slot}",
                code.ref_id
            ))
        })?;
        if !claimed_slots.insert(slot) {
            continue;
        }
        ranges.push(Range {
            code_ref: code.ref_id,
            owner_ref: code.owner_ref,
            pc_offset: entry.pc_offset,
            stack_map_offset: entry.stack_map_offset,
            size: 0,
        });
    }
    for (slot, entry) in table.entries.iter().enumerate() {
        if !claimed_slots.insert(slot) {
            continue;
        }
        ranges.push(Range {
            code_ref: synthetic_code_reference(slot),
            owner_ref: -1,
            pc_offset: entry.pc_offset,
            stack_map_offset: entry.stack_map_offset,
            size: 0,
        });
    }
    ranges.sort_by_key(|range| range.pc_offset);
    assign_sizes(
        &mut ranges,
        u32::try_from(image.code_offset + image.bytes.len() as u64).unwrap_or(u32::MAX),
    );

    let names = Names::new(isolate, vm);
    let types = TypeRecovery::new(isolate, vm, cids, options.abi, options.obfuscation_map);
    let initializer_fields = field_initializer_fields(isolate, cids, &names);
    let application_package = select_application_package(
        options.application_package,
        ranges.iter().filter_map(|range| {
            let attribution_ref = initializer_fields
                .get(&range.owner_ref)
                .copied()
                .unwrap_or(range.owner_ref);
            restore_library_uri(names.library_uri(attribution_ref), options.obfuscation_map)
        }),
    );
    let ownership_obfuscated = options.obfuscation_map.is_none()
        && library_ownership_is_obfuscated(
            ranges.iter().filter_map(|range| {
                let attribution_ref = initializer_fields
                    .get(&range.owner_ref)
                    .copied()
                    .unwrap_or(range.owner_ref);
                names.library_uri(attribution_ref)
            }),
            application_package.as_deref(),
        );
    let effective_scope = if ownership_obfuscated && options.scope != Scope::All {
        Scope::All
    } else {
        options.scope
    };
    let mut symbols = BTreeMap::new();
    for range in &ranges {
        let address = image.image_virtual_address + u64::from(range.pc_offset);
        let symbol = {
            let raw_function = names.name(range.owner_ref);
            if raw_function.is_empty() {
                crate::analysis::disassembly::Symbol::code_boundary(address)
            } else {
                let function = restore_snapshot_name(&raw_function, options.obfuscation_map);
                let attribution_ref = initializer_fields
                    .get(&range.owner_ref)
                    .copied()
                    .unwrap_or(range.owner_ref);
                let owner = restore_snapshot_name(
                    &names.owner_name(attribution_ref),
                    options.obfuscation_map,
                );
                let display = if owner.is_empty() {
                    function
                } else {
                    format!("{owner}.{function}")
                };
                let library_uri = restore_library_uri(
                    names.library_uri(attribution_ref),
                    options.obfuscation_map,
                );
                let result_class = isolate
                    .named
                    .get(&range.owner_ref)
                    .and_then(|function| function.function_kind_tag)
                    .and_then(RecoveredFunctionKind::from_raw_tag)
                    .filter(|kind| *kind == RecoveredFunctionKind::Constructor)
                    .and_then(|_| (!owner.is_empty()).then_some(owner));
                crate::analysis::disassembly::Symbol::new(
                    display,
                    library_uri,
                    application_package.as_deref(),
                )
                .with_code_identity(address, 0, crate::model::DirectCallResolution::ExactEntry)
                .with_result_class(result_class)
            }
        };
        insert_preferred_symbol(&mut symbols, address, symbol.clone());
        if let Some(unchecked_offset) = range_unchecked_entry_offset(isolate, range)
            && unchecked_offset > 0
            && unchecked_offset < u64::from(range.size)
        {
            insert_preferred_symbol(
                &mut symbols,
                address.saturating_add(unchecked_offset),
                symbol.with_code_identity(
                    address,
                    unchecked_offset,
                    crate::model::DirectCallResolution::UncheckedEntry,
                ),
            );
        }
    }
    let refs_with_code = ranges
        .iter()
        .map(|range| range.owner_ref)
        .collect::<std::collections::BTreeSet<_>>();
    let object_pool_labels = isolate
        .object_pools
        .first()
        .map(|pool| {
            object_pool_labels(
                isolate,
                &names,
                &types,
                pool,
                options.obfuscation_map,
                cids,
            )
        });
    let dispatch_target_labels = isolate
        .dispatch_table_code_indices
        .iter()
        .map(|code_index| {
            code_index.and_then(|code_index| {
                dispatch_target(code_index, table, image, &symbols).map(|(_, label, _)| label)
            })
        })
        .collect::<Vec<_>>();
    let dispatch_class_ids = class_ids(isolate);
    let dispatch_table = recover_dispatch_table(options.abi, isolate, table, image, &symbols);
    let closure_parents = recover_closure_parents(isolate, cids);
    let context = FunctionRecoveryContext {
        abi: options.abi,
        isolate,
        names: &names,
        types: &types,
        image,
        symbols: &symbols,
        scope: effective_scope,
        application_package: application_package.as_deref(),
        obfuscation_map: options.obfuscation_map,
        object_pool_labels: object_pool_labels.as_deref(),
        dispatch_target_labels: &dispatch_target_labels,
        dispatch_class_ids: &dispatch_class_ids,
        initializer_fields: &initializer_fields,
        table,
        static_bit: calibrate_static_bit(&names, isolate, cids),
        closure_parents: &closure_parents,
    };
    let recovered = ranges
        .into_par_iter()
        .map_init(
            || None,
            |disassembler, range| {
                if disassembler.is_none() {
                    *disassembler = Some(crate::analysis::disassembly::Disassembler::new(
                        context.abi,
                    )?);
                }
                recover_range(
                    range,
                    &context,
                    disassembler.as_ref().expect("initialized above"),
                )
            },
        )
        .collect::<Result<Vec<_>>>()?;
    let mut functions = recovered.into_iter().flatten().collect::<Vec<_>>();
    functions.sort_by_key(|function| parse_address(&function.address).unwrap_or(u64::MAX));
    let declarations = recover_declarations(
        isolate,
        vm,
        cids,
        &refs_with_code,
        effective_scope,
        &options,
        &types,
    );
    let mut snapshot_strings = super::transduce::recover(vm, "vm");
    snapshot_strings.extend(super::transduce::recover(isolate, "isolate"));
    Ok(super::Recovery {
        application_package,
        functions,
        declarations,
        ownership_obfuscated,
        snapshot_evidence: summarize_snapshot(vm, isolate, table),
        dispatch_table,
        snapshot_strings,
    })
}

struct FunctionRecoveryContext<'a> {
    abi: Abi,
    isolate: &'a ParseResult,
    names: &'a Names<'a>,
    types: &'a TypeRecovery<'a>,
    image: &'a CodeImage,
    symbols: &'a BTreeMap<u64, crate::analysis::disassembly::Symbol>,
    scope: Scope,
    application_package: Option<&'a str>,
    obfuscation_map: Option<&'a crate::analysis::LoadedObfuscationMap>,
    object_pool_labels: Option<&'a [String]>,
    dispatch_target_labels: &'a [Option<String>],
    dispatch_class_ids: &'a [usize],
    initializer_fields: &'a BTreeMap<i32, i32>,
    table: &'a InstructionTable,
    static_bit: Option<u32>,
    /// Closure-function reference -> lexically enclosing function reference,
    /// recovered from serialized `ClosureData.parent_function` edges.
    closure_parents: &'a BTreeMap<i32, i32>,
}

/// Maps each closure body Function reference to its lexically enclosing
/// Function reference using the serialized `ClosureData.parent_function` edge.
/// In full AOT snapshots a ClosureData serializes refs as
/// `[parent_function, closure]`, and every named closure body reaches its
/// ClosureData through the fourth serialized Function reference (`data_`),
/// so the parent link survives even when debug info does not.
fn recover_closure_parents(isolate: &ParseResult, cids: &Cids) -> BTreeMap<i32, i32> {
    let mut parent_of_data = BTreeMap::new();
    for object in &isolate.objects {
        if object.cid != cids.closure_data {
            continue;
        }
        if let [parent_function, _closure] = isolate.references_of(object)
            && *parent_function >= 0
        {
            parent_of_data.insert(object.reference, *parent_function);
        }
    }
    let mut parents = BTreeMap::new();
    for object in &isolate.objects {
        if object.cid != cids.function {
            continue;
        }
        if let Some(data_ref) = isolate.references_of(object).get(3).copied()
            && let Some(parent_ref) = parent_of_data.get(&data_ref).copied()
        {
            parents.insert(object.reference, parent_ref);
        }
    }
    parents
}

/// Returns the enclosing member's restored name when the code-range owner is
/// a named closure with a snapshot-proven parent link. Only closure bodies
/// appear in `parents`, so a hit doubles as the closure-kind check.
fn lexical_parent(
    isolate: &ParseResult,
    parents: &BTreeMap<i32, i32>,
    owner_ref: i32,
    names: &Names<'_>,
) -> Option<String> {
    isolate.named.get(&owner_ref)?;
    let parent_name = names.name(*parents.get(&owner_ref)?);
    (!parent_name.is_empty()).then_some(parent_name)
}

/// Derives member staticness from the serialized `Function::kind_tag_`.
/// Kinds whose staticness is fixed by definition are answered directly; all
/// others consult the calibrated static bit when one was proven.
fn derive_is_static(
    tag: Option<u32>,
    kind: Option<RecoveredFunctionKind>,
    static_bit: Option<u32>,
) -> Option<bool> {
    use RecoveredFunctionKind as K;
    match kind {
        Some(K::Constructor | K::ImplicitGetter | K::ImplicitSetter | K::Closure) => Some(false),
        Some(K::ImplicitStaticGetter) => Some(true),
        _ => match (static_bit, tag) {
            (Some(position), Some(tag)) => Some((tag >> position) & 1 == 1),
            _ => None,
        },
    }
}

/// The static flag inside `Function::kind_tag_` sits above version-dependent
/// recognizer-kind bits, so its position varies across Dart releases.
/// Calibrate it per snapshot instead of hardcoding: top-level functions
/// (owner class `::`) must read 1, constructors and implicit instance
/// accessors must read 0. A position is trusted only when exactly one
/// candidate satisfies every constraint.
fn calibrate_static_bit(_names: &Names<'_>, isolate: &ParseResult, cids: &Cids) -> Option<u32> {
    let mut constraints: Vec<(u32, bool)> = Vec::new();
    for object in isolate.named.values() {
        if object.cid != cids.function {
            continue;
        }
        let Some(tag) = object.function_kind_tag else {
            continue;
        };
        // Factories make constructors ambiguous (generative = instance,
        // factory = static), and the `::` top-level owner class serializes
        // without a readable name. Only implicit field accessors have
        // definition-fixed staticness.
        let expected = match tag & 0x1f {
            6 | 7 => Some(false),
            8 => Some(true),
            _ => None,
        };
        if let Some(expected) = expected {
            constraints.push((tag, expected));
        }
    }
    if constraints.is_empty() {
        return None;
    }
    let valid: Vec<u32> = (7..u32::BITS)
        .filter(|&position| {
            constraints
                .iter()
                .all(|(tag, expected)| ((tag >> position) & 1 == 1) == *expected)
        })
        .collect();
    if std::env::var("CLUTTER_DEBUG_STATIC_BIT").is_ok() {
        eprintln!(
            "static-bit candidates: {valid:?} from {} constraints",
            constraints.len()
        );
    }
    if valid.len() != 1 {
        return None;
    }
    Some(valid[0])
}

fn recover_range(
    range: Range,
    context: &FunctionRecoveryContext<'_>,
    disassembler: &crate::analysis::disassembly::Disassembler,
) -> Result<Option<RecoveredFunction>> {
    let raw_function = context.names.name(range.owner_ref);
    let attribution_ref = context
        .initializer_fields
        .get(&range.owner_ref)
        .copied()
        .unwrap_or(range.owner_ref);
    let raw_owner = context.names.owner_name(attribution_ref);
    let function = restore_snapshot_name(&raw_function, context.obfuscation_map);
    let owner = restore_snapshot_name(&raw_owner, context.obfuscation_map);
    let library_uri = restore_library_uri(
        context.names.library_uri(attribution_ref),
        context.obfuscation_map,
    );
    let signature = context
        .isolate
        .named
        .get(&range.owner_ref)
        .and_then(|function| context.isolate.function_types.get(&function.signature_ref))
        .map(|signature| context.types.signature(signature));
    let parameter_count = signature.as_ref().map(|signature| {
        signature
            .fixed_parameter_count
            .saturating_add(signature.optional_parameter_count)
    });
    let owner_object = context.isolate.named.get(&range.owner_ref);
    let function_kind_tag = owner_object.and_then(|function| function.function_kind_tag);
    let kind = function_kind_tag.and_then(RecoveredFunctionKind::from_raw_tag);
    let is_static = derive_is_static(function_kind_tag, kind, context.static_bit);
    if !include_library(
        library_uri.as_deref(),
        context.scope,
        context.application_package,
    ) {
        return Ok(None);
    }
    let address = context.image.image_virtual_address + u64::from(range.pc_offset);
    let relative = u64::from(range.pc_offset).checked_sub(context.image.code_offset);
    let Some(relative) = relative.and_then(|value| usize::try_from(value).ok()) else {
        return Ok(None);
    };
    let size = usize::try_from(range.size).unwrap_or_default();
    let Some(bytes) = context
        .image
        .bytes
        .get(relative..relative.saturating_add(size).min(context.image.bytes.len()))
    else {
        return Ok(None);
    };
    let disassembly = disassembler.analyze(
        address,
        bytes,
        context.symbols,
        parameter_count,
        context.object_pool_labels,
        Some(&crate::analysis::disassembly::DispatchTableAnalysis {
            origin_element: dispatch_origin(context.abi),
            targets: context.dispatch_target_labels,
            class_ids: context.dispatch_class_ids,
        })
        .filter(|analysis| !analysis.targets.is_empty() && !analysis.class_ids.is_empty()),
    )?;
    let fallback = format!("sub_{:x}", range.pc_offset);
    let is_synthetic = function.is_empty();
    let snapshot_name = (!is_synthetic && raw_function != function).then_some(raw_function);
    let unmapped_function =
        restore_snapshot_name(snapshot_name.as_deref().unwrap_or(&function), None);
    let unmapped_owner = restore_snapshot_name(&raw_owner, None);
    let map_restored = context.obfuscation_map.is_some()
        && (unmapped_function != function || unmapped_owner != owner);
    let code_metadata = code_metadata(context.isolate, context.table, &range);
    // Inline stack transitions carry the snapshot reference of each inlined
    // Function; resolve them to named callees for reporting and rendering.
    let mut inlined_callees: Vec<crate::model::RecoveredInlineFunction> = Vec::new();
    if let Some(metadata) = code_metadata.as_ref() {
        for entry in &metadata.code_source_map {
            if entry.operation != crate::model::CodeSourceMapOperation::PushFunction {
                continue;
            }
            let Some(reference) = entry.function_reference else {
                continue;
            };
            let raw = context.names.name(reference);
            if raw.is_empty() {
                continue;
            }
            let name = restore_snapshot_name(&raw, context.obfuscation_map);
            let already_listed = inlined_callees
                .iter()
                .any(|callee| callee.name == name && callee.source_location.is_none());
            if already_listed || inlined_callees.len() >= 64 {
                continue;
            }
            inlined_callees.push(crate::model::RecoveredInlineFunction {
                name,
                library_uri: restore_library_uri(
                    context.names.library_uri(reference),
                    context.obfuscation_map,
                ),
                source_location: entry.source_line.map(|line| {
                    crate::model::RecoveredSourceLocation {
                        path: "snapshot:inline".to_owned(),
                        line: u64::try_from(line).ok(),
                        column: None,
                        end_line: None,
                        end_column: None,
                    }
                }),
                call_location: None,
                address: String::new(),
                size: 0,
            });
        }
    }
    let internal_source_line = code_metadata.as_ref().and_then(|metadata| {
        metadata
            .code_source_map
            .iter()
            .filter(|entry| entry.inline_depth == 0)
            .find_map(|entry| entry.source_line)
            .and_then(|line| u64::try_from(line).ok())
    });
    // The last source line the body itself touches approximates the
    // declaration's end line, which lets the renderer nest closures whose
    // start line falls inside this member's span.
    let internal_end_line = code_metadata.as_ref().and_then(|metadata| {
        metadata
            .code_source_map
            .iter()
            .filter(|entry| entry.inline_depth == 0)
            .filter_map(|entry| entry.source_line)
            .filter_map(|line| u64::try_from(line).ok())
            .max()
    });
    let source_location = internal_source_line.map(|line| crate::model::RecoveredSourceLocation {
        path: library_uri
            .clone()
            .unwrap_or_else(|| "snapshot:internal-code-source-map".to_owned()),
        line: Some(line),
        column: None,
        end_line: internal_end_line,
        end_column: None,
    });
    Ok(Some(RecoveredFunction {
        code_reference: range.code_ref,
        code_alias_references: Vec::new(),
        name: if is_synthetic { fallback } else { function },
        name_source: if is_synthetic {
            RecoveredNameSource::Synthetic
        } else if map_restored {
            RecoveredNameSource::ObfuscationMap
        } else {
            RecoveredNameSource::Snapshot
        },
        snapshot_name,
        obfuscated_name: map_restored.then(|| qualified_name(&unmapped_owner, &unmapped_function)),
        owner: (!owner.is_empty()).then_some(owner),
        library_uri,
        source_location,
        inlined_functions: inlined_callees,
        kind,
        is_static,
        signature,
        signature_source: parameter_count
            .is_some()
            .then_some(RecoveredSignatureSource::SnapshotFunction),
        parameter_count,
        lexical_parent: lexical_parent(
            context.isolate,
            context.closure_parents,
            range.owner_ref,
            context.names,
        ),
        vm_evidence: None,
        address: format!("0x{address:x}"),
        size: range.size.into(),
        code_metadata,
        machine_code: disassembly.evidence,
        instructions: disassembly.instructions,
        control_flow: disassembly.control_flow,
        semantic_statements: disassembly.semantic_statements,
        statements: disassembly.statements,
    }))
}

/// Links ownerless `init:<field>` functions back to their Field snapshot
/// object. Modern AOT snapshots retain initializer code as a Function, but
/// the Function itself can have no owner/library edge. The Field retains both
/// the semantic owner and, depending on SDK layout, a direct initializer
/// reference. Exact raw-name matching is a conservative fallback for layouts
/// where that reference is reset after precompilation.
fn field_initializer_fields(
    isolate: &ParseResult,
    cids: &Cids,
    names: &Names<'_>,
) -> BTreeMap<i32, i32> {
    let initializer_functions = isolate
        .named
        .iter()
        .filter_map(|(reference, object)| {
            (object.cid == cids.function
                && object
                    .function_kind_tag
                    .and_then(RecoveredFunctionKind::from_raw_tag)
                    == Some(RecoveredFunctionKind::FieldInitializer))
            .then_some((*reference, names.name(*reference)))
        })
        .collect::<Vec<_>>();
    let mut initializers_by_name = BTreeMap::<String, Vec<i32>>::new();
    for (reference, name) in &initializer_functions {
        if !name.is_empty() {
            initializers_by_name
                .entry(name.clone())
                .or_default()
                .push(*reference);
        }
    }

    let mut fields_by_initializer = BTreeMap::new();
    for (field_ref, field) in &isolate.named {
        if field.cid != cids.field {
            continue;
        }

        // Field serialization stores name, owner, type, and initializer
        // function in that order when the initializer edge is retained.
        if let Some(object) = isolate.object(*field_ref)
            && let Some(initializer_ref) = isolate.references_of(object).get(3).copied()
            && initializer_functions
                .iter()
                .any(|(reference, _)| *reference == initializer_ref)
        {
            fields_by_initializer.insert(initializer_ref, *field_ref);
            continue;
        }

        let field_name = names.name(*field_ref);
        if field_name.is_empty() {
            continue;
        }
        let initializer_name = format!("init:{field_name}");
        if let Some(candidates) = initializers_by_name.get(&initializer_name)
            && let [initializer_ref] = candidates.as_slice()
        {
            fields_by_initializer.insert(*initializer_ref, *field_ref);
        }
    }
    fields_by_initializer
}

fn recover_declarations(
    isolate: &ParseResult,
    vm: &ParseResult,
    cids: &Cids,
    refs_with_code: &std::collections::BTreeSet<i32>,
    scope: Scope,
    options: &ResolveOptions<'_>,
    types: &TypeRecovery<'_>,
) -> Vec<RecoveredDeclaration> {
    let names = Names::new(isolate, vm);
    if std::env::var("CLUTTER_DEBUG_DECLS").is_ok() {
        let mut counts = std::collections::BTreeMap::new();
        for object in isolate.named.values() {
            if object.cid == cids.class {
                *counts.entry("class").or_insert(0) += 1;
            } else if object.cid == cids.field {
                *counts.entry("field").or_insert(0) += 1;
            } else if object.cid == cids.function {
                *counts.entry("function").or_insert(0) += 1;
            }
        }
        eprintln!("decl debug: {counts:?}");
    }
    let mut declarations = Vec::new();
    for (reference, object) in &isolate.named {
        let kind = if object.cid == cids.class {
            RecoveredDeclarationKind::Class
        } else if object.cid == cids.field {
            RecoveredDeclarationKind::Field
        } else if object.cid == cids.function {
            RecoveredDeclarationKind::Function
        } else {
            continue;
        };
        let raw_name = names.name(*reference);
        if raw_name.is_empty() {
            continue;
        }
        let library_uri =
            restore_library_uri(names.library_uri(*reference), options.obfuscation_map);
        if !include_library(library_uri.as_deref(), scope, options.application_package) {
            continue;
        }
        let name = restore_snapshot_name(&raw_name, options.obfuscation_map);
        if name.is_empty() {
            continue;
        }
        let raw_owner = names.owner_name(*reference);
        let owner = restore_snapshot_name(&raw_owner, options.obfuscation_map);
        let signature = isolate
            .function_types
            .get(&object.signature_ref)
            .map(|signature| types.signature(signature));
        declarations.push(RecoveredDeclaration {
            snapshot_reference: *reference,
            vm_object_id: None,
            kind,
            name: name.clone(),
            snapshot_name: (raw_name != name).then_some(raw_name),
            owner: (!owner.is_empty()).then_some(owner),
            library_uri,
            source_location: None,
            function_kind: object
                .function_kind_tag
                .and_then(RecoveredFunctionKind::from_raw_tag),
            signature,
            vm_evidence: None,
            class_metadata: (kind == RecoveredDeclarationKind::Class)
                .then(|| types.class_metadata(*reference))
                .flatten(),
            field_metadata: (kind == RecoveredDeclarationKind::Field)
                .then(|| types.field_metadata(*reference))
                .flatten(),
            has_code: refs_with_code.contains(reference),
        });
    }
    declarations.sort_by(|left, right| {
        left.library_uri
            .cmp(&right.library_uri)
            .then(left.kind.label().cmp(right.kind.label()))
            .then(left.owner.cmp(&right.owner))
            .then(left.name.cmp(&right.name))
            .then(left.snapshot_reference.cmp(&right.snapshot_reference))
    });
    declarations
}

fn assign_sizes(ranges: &mut [Range], code_end: u32) {
    let mut next_offset = code_end;
    for index in (0..ranges.len()).rev() {
        if ranges[index].pc_offset < next_offset {
            ranges[index].size = next_offset - ranges[index].pc_offset;
            next_offset = ranges[index].pc_offset;
        } else if ranges[index].pc_offset == next_offset {
            let duplicate_size = ranges.get(index + 1).map_or(0, |next| next.size);
            ranges[index].size = duplicate_size;
        }
    }
}

fn insert_preferred_symbol(
    symbols: &mut BTreeMap<u64, crate::analysis::disassembly::Symbol>,
    address: u64,
    symbol: crate::analysis::disassembly::Symbol,
) {
    match symbols.get(&address) {
        Some(existing) if existing.semantic_name || !symbol.semantic_name => {}
        _ => {
            symbols.insert(address, symbol);
        }
    }
}

fn range_code<'a>(isolate: &'a ParseResult, range: &Range) -> Option<&'a super::types::Code> {
    isolate
        .codes
        .iter()
        .find(|code| code.ref_id == range.code_ref)
        .or_else(|| {
            isolate
                .codes
                .iter()
                .find(|code| code.owner_ref == range.owner_ref)
        })
}

fn range_unchecked_entry_offset(isolate: &ParseResult, range: &Range) -> Option<u64> {
    range_code(isolate, range)?.unchecked_entry_offset
}

fn code_metadata(
    isolate: &ParseResult,
    table: &InstructionTable,
    range: &Range,
) -> Option<RecoveredCodeMetadata> {
    let code = range_code(isolate, range)?;
    let handlers = code
        .exception_handlers_ref
        .and_then(|reference| isolate.exception_handlers.get(&reference));
    let stack_map = table.stack_maps.get(&range.stack_map_offset);
    Some(RecoveredCodeMetadata {
        stack_map_offset: stack_map.map_or(range.stack_map_offset, |map| map.offset),
        stack_map_payload_bytes: stack_map.map_or(0, |map| map.payload.len()),
        stack_map_entries: stack_map.map_or(0, |map| map.entry_count),
        stack_map_uses_global_table: stack_map.is_some_and(|map| map.uses_global_table),
        stack_map_is_global_table: stack_map.is_some_and(|map| map.global_table),
        payload_info: code.payload_info,
        unchecked_entry_offset: code.unchecked_entry_offset,
        has_monomorphic_entrypoint: code.has_monomorphic_entrypoint,
        catch_entry_reference: code.catch_entry_ref,
        inlined_functions_reference: code.inlined_functions_ref,
        pc_descriptors_reference: code.pc_descriptors_ref,
        pc_descriptors: decode_pc_descriptors(isolate, code),
        code_source_map_reference: code.code_source_map_ref,
        code_source_map: decode_code_source_map(isolate, code),
        exception_handlers_reference: code.exception_handlers_ref,
        handled_types_reference: handlers.map(|handlers| handlers.handled_types_ref),
        has_async_exception_handler: handlers.is_some_and(|handlers| handlers.has_async_handler),
        exception_handlers: handlers
            .map(|handlers| {
                handlers
                    .entries
                    .iter()
                    .map(|handler| RecoveredExceptionHandler {
                        handler_pc_offset: handler.handler_pc_offset,
                        outer_try_index: handler.outer_try_index,
                        needs_stack_trace: handler.needs_stack_trace,
                        has_catch_all: handler.has_catch_all,
                        is_generated: handler.is_generated,
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// Mirrors Dart's `Function::DropImplicitCallPrefix`: the implicit dynamic
/// invocation selector `dyn:implicit:call` reports as the canonical
/// `dyn:call`; every other selector is unchanged.
fn drop_implicit_call_prefix(name: &str) -> &str {
    if name == "dyn:implicit:call" {
        "dyn:call"
    } else {
        name
    }
}

/// Renders a dynamic-call pool label from an UnlinkedCall object. The
/// selector survives obfuscation because it lives in serialized CallSiteData,
/// not in symbol names. Arity comes from the args descriptor array
/// (`[typeArgsLen, count, size, positionalCount, …]`, dart_entry.h) when it
/// is present in the snapshot.
fn unlinked_call_label(isolate: &ParseResult, reference: i32) -> Option<String> {
    let object = isolate.object(reference)?;
    let selector = drop_implicit_call_prefix(
        isolate
            .strings
            .get(&isolate.named.get(&reference)?.name_ref)?,
    );
    let arity = isolate
        .references_of(object)
        .get(1)
        .copied()
        .filter(|descriptor| *descriptor >= 0)
        .and_then(|descriptor| isolate.object(descriptor))
        .and_then(|array| {
            let scalars = isolate.scalars_of(array);
            let type_args_len = scalars.first().and_then(snapshot_scalar_value);
            let count = scalars.get(1).and_then(snapshot_scalar_value)?;
            Some(count.saturating_sub(type_args_len.unwrap_or_default()))
        });
    Some(match arity {
        Some(arity) => format!("dynamicCall(\"{selector}\", arity={arity})"),
        None => format!("dynamicCall(\"{selector}\")"),
    })
}

fn snapshot_scalar_value(scalar: &super::types::SnapshotScalar) -> Option<i64> {
    match scalar {
        super::types::SnapshotScalar::Unsigned(value) => Some(*value),
        super::types::SnapshotScalar::Tagged32(value) => Some(i64::from(*value)),
        super::types::SnapshotScalar::Tagged64(value) => Some(*value),
        super::types::SnapshotScalar::Uint16(value) => Some(i64::from(*value)),
        super::types::SnapshotScalar::Int16(value) => Some(i64::from(*value)),
        super::types::SnapshotScalar::Byte(value) => Some(i64::from(*value)),
        super::types::SnapshotScalar::Reference(_) => None,
    }
}

fn object_pool_labels(
    isolate: &ParseResult,
    names: &Names<'_>,
    types: &TypeRecovery<'_>,
    pool: &super::types::ObjectPool,
    obfuscation_map: Option<&crate::analysis::LoadedObfuscationMap>,
    cids: &Cids,
) -> Vec<String> {
    let code_owners = isolate
        .codes
        .iter()
        .map(|code| (code.ref_id, code.owner_ref))
        .collect::<BTreeMap<_, _>>();
    // Class objects carry their class id as the first scalar; canonical
    // instances are then labeled with their concrete class so downstream
    // lifting gains receiver provenance (`snapshotInstance(Product)`).
    let mut class_names_by_cid: BTreeMap<i32, String> = BTreeMap::new();
    for object in &isolate.objects {
        if object.kind != super::types::SnapshotObjectKind::Class {
            continue;
        }
        let Some(super::types::SnapshotScalar::Tagged32(class_id)) =
            isolate.scalars_of(object).first()
        else {
            continue;
        };
        let name = restore_snapshot_name(&names.name(object.reference), None);
        if !name.is_empty() {
            class_names_by_cid.entry(*class_id as i32).or_insert(name);
        }
    }
    pool.entries
        .iter()
        .enumerate()
        .map(|(index, entry)| match entry {
            super::types::PoolValue::Reference(reference) => {
                if let Some(value) = types.scalar_label(*reference) {
                    value
                } else if let Some(value) = isolate.strings.get(reference) {
                    abbreviated_pool_string(value)
                } else if let Some(object) = isolate.object(*reference)
                    && object.canonical
                    && matches!(
                        object.kind,
                        super::types::SnapshotObjectKind::Instance
                            | super::types::SnapshotObjectKind::Record
                    )
                    && let Some(class_name) = class_names_by_cid
                        .get(&object.cid)
                        .filter(|name| !name.is_empty())
                    && names.name(*reference).is_empty()
                {
                    format!("snapshotInstance({class_name})")
                } else if let Some(object) = isolate.object(*reference)
                    && object.cid == cids.unlinked_call
                    && let Some(label) = unlinked_call_label(isolate, *reference)
                {
                    label
                } else {
                    let named_reference = code_owners.get(reference).copied().unwrap_or(*reference);
                    let name = restore_snapshot_name(&names.name(named_reference), obfuscation_map);
                    if name.is_empty() {
                        let nested_strings = nested_pool_strings(isolate, *reference);
                        if nested_strings.is_empty() {
                            format!("snapshotRef({reference})")
                        } else {
                            format!(
                                "snapshotRef({reference}) nestedStrings[{}]",
                                nested_strings.join(", ")
                            )
                        }
                    } else {
                        let owner = restore_snapshot_name(
                            &names.owner_name(named_reference),
                            obfuscation_map,
                        );
                        if owner.is_empty() {
                            name
                        } else {
                            format!("{owner}.{name}")
                        }
                    }
                }
            }
            super::types::PoolValue::Immediate(value) => value.to_string(),
            super::types::PoolValue::Native => format!("nativePoolEntry({index})"),
            super::types::PoolValue::Empty => format!("resetPoolEntry({index})"),
        })
        .collect()
}

fn nested_pool_strings(isolate: &ParseResult, root: i32) -> Vec<String> {
    const MAX_DEPTH: usize = 4;
    const MAX_NODES: usize = 96;
    const MAX_STRINGS: usize = 16;

    let mut pending = std::collections::VecDeque::from([(root, 0usize)]);
    let mut visited = BTreeSet::new();
    let mut values = BTreeSet::new();
    while let Some((reference, depth)) = pending.pop_front() {
        if visited.len() >= MAX_NODES || values.len() >= MAX_STRINGS || !visited.insert(reference) {
            continue;
        }
        if let Some(value) = isolate.strings.get(&reference) {
            if !value.is_empty() && value.chars().all(|character| !character.is_control()) {
                values.insert(abbreviated_pool_string(value));
            }
            continue;
        }
        if depth >= MAX_DEPTH {
            continue;
        }
        let Some(object) = isolate.object(reference) else {
            continue;
        };
        // These are value-container edges. Deliberately avoid Standard/Class/
        // Code objects, whose metadata/owner edges would spread attribution
        // across unrelated libraries.
        if !matches!(
            object.kind,
            super::types::SnapshotObjectKind::Instance
                | super::types::SnapshotObjectKind::Array
                | super::types::SnapshotObjectKind::WeakArray
                | super::types::SnapshotObjectKind::Record
                | super::types::SnapshotObjectKind::Context
        ) {
            continue;
        }
        pending.extend(
            isolate
                .references_of(object)
                .iter()
                .copied()
                .filter(|reference| *reference >= 0)
                .map(|reference| (reference, depth + 1)),
        );
    }
    values.into_iter().collect()
}

fn dispatch_origin(abi: Abi) -> usize {
    match abi {
        Abi::Arm64V8a => 4096,
        Abi::ArmeabiV7a => 1023,
        Abi::X86_64 => 16,
    }
}

fn class_ids(isolate: &ParseResult) -> Vec<usize> {
    let mut ids = std::collections::BTreeSet::new();
    for object in &isolate.objects {
        if object.kind != super::types::SnapshotObjectKind::Class {
            continue;
        }
        let Some(super::types::SnapshotScalar::Tagged32(class_id)) =
            isolate.scalars_of(object).first()
        else {
            continue;
        };
        if *class_id < (1 << 20)
            && let Ok(class_id) = usize::try_from(*class_id)
        {
            ids.insert(class_id);
        }
    }
    ids.into_iter().collect()
}

fn dispatch_target(
    code_index: usize,
    table: &InstructionTable,
    image: &CodeImage,
    symbols: &BTreeMap<u64, crate::analysis::disassembly::Symbol>,
) -> Option<(String, String, Option<String>)> {
    let instruction_slot = code_index.checked_sub(1)?;
    let entry = table.entries.get(instruction_slot)?;
    let address = image
        .image_virtual_address
        .saturating_add(u64::from(entry.pc_offset));
    let symbol = symbols.get(&address);
    Some((
        format!("0x{address:x}"),
        symbol
            .map(|symbol| symbol.label.clone())
            .unwrap_or_else(|| format!("sub_{address:x}")),
        symbol.and_then(|symbol| symbol.library_uri.clone()),
    ))
}

fn recover_dispatch_table(
    abi: Abi,
    isolate: &ParseResult,
    table: &InstructionTable,
    image: &CodeImage,
    symbols: &BTreeMap<u64, crate::analysis::disassembly::Symbol>,
) -> Option<RecoveredDispatchTable> {
    let entries = &isolate.dispatch_table_code_indices;
    if entries.is_empty() {
        return None;
    }
    let mut runs = Vec::new();
    let mut start = 0usize;
    while start < entries.len() {
        let code_index = entries[start];
        let mut end = start + 1;
        while end < entries.len() && entries[end] == code_index {
            end += 1;
        }
        let target = code_index.and_then(|index| dispatch_target(index, table, image, symbols));
        runs.push(RecoveredDispatchRun {
            start_index: start,
            length: end - start,
            code_index,
            instruction_slot: code_index.and_then(|index| index.checked_sub(1)),
            target_address: target.as_ref().map(|target| target.0.clone()),
            target: target.as_ref().map(|target| target.1.clone()),
            target_library_uri: target.and_then(|target| target.2),
        });
        start = end;
    }
    Some(RecoveredDispatchTable {
        abi,
        origin_element: dispatch_origin(abi),
        entry_count: entries.len(),
        non_null_entries: entries.iter().filter(|entry| entry.is_some()).count(),
        unique_code_indices: entries
            .iter()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        runs,
    })
}

fn abbreviated_pool_string(value: &str) -> String {
    const MAX_CHARS: usize = 160;
    let mut characters = value.chars();
    let prefix = characters.by_ref().take(MAX_CHARS).collect::<String>();
    if characters.next().is_some() {
        format!("{prefix:?}…")
    } else {
        format!("{prefix:?}")
    }
}

fn decode_pc_descriptors(
    isolate: &ParseResult,
    code: &super::types::Code,
) -> Vec<RecoveredPcDescriptor> {
    let Some(bytes) = code
        .pc_descriptors_ref
        .and_then(|reference| isolate.object(reference))
        .map(|object| isolate.bytes_of(object))
        .filter(|bytes| !bytes.is_empty())
    else {
        return Vec::new();
    };
    let mut cursor = 0usize;
    let mut pc_offset = 0i64;
    let mut entries = Vec::new();
    while cursor < bytes.len() {
        let Some(metadata) = read_sleb128(bytes, &mut cursor) else {
            break;
        };
        let Some(pc_delta) = read_sleb128(bytes, &mut cursor) else {
            break;
        };
        pc_offset = pc_offset.saturating_add(pc_delta);
        let metadata = metadata as u32;
        let kind_index = metadata & 0x7;
        let kind = [
            "deopt",
            "ic_call",
            "unoptimized_static_call",
            "runtime_call",
            "osr_entry",
            "rewind",
            "other",
            "unknown",
        ][kind_index as usize]
            .to_owned();
        entries.push(RecoveredPcDescriptor {
            pc_offset: u32::try_from(pc_offset).unwrap_or_default(),
            kind,
            try_index: i32::try_from((metadata >> 3) & 0x3ff).unwrap_or_default() - 1,
            yield_index: i32::try_from(metadata >> 13).unwrap_or_default() - 1,
        });
    }
    entries
}

fn read_sleb128(bytes: &[u8], cursor: &mut usize) -> Option<i64> {
    let mut value = 0u64;
    let mut shift = 0u32;
    let mut byte;
    loop {
        byte = *bytes.get(*cursor)?;
        *cursor = (*cursor).saturating_add(1);
        value |= u64::from(byte & 0x7f) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
        if shift >= 64 {
            return None;
        }
    }
    if shift < 64 && byte & 0x40 != 0 {
        value |= u64::MAX << shift;
    }
    Some(value as i64)
}

fn decode_code_source_map(
    isolate: &ParseResult,
    code: &super::types::Code,
) -> Vec<CodeSourceMapEntry> {
    let Some(bytes) = code
        .code_source_map_ref
        .and_then(|reference| isolate.object(reference))
        .map(|object| isolate.bytes_of(object))
        .filter(|bytes| !bytes.is_empty())
    else {
        return Vec::new();
    };
    let inlined_functions = code
        .inlined_functions_ref
        .and_then(|reference| isolate.object(reference))
        .map(|object| isolate.references_of(object))
        .unwrap_or_default();
    decode_code_source_map_bytes(bytes, inlined_functions)
}

fn decode_code_source_map_bytes(
    bytes: &[u8],
    inlined_functions: &[i32],
) -> Vec<CodeSourceMapEntry> {
    let Ok(mut reader) = super::reader::Reader::at(bytes, 0) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    let mut pc_offset = 0u32;
    // CodeSourceMapReader and DebugInfoPosition initialize each line register
    // to TokenPosition::kNoSource (-1), not zero.
    let mut lines = vec![-1i64];
    while reader.position() < bytes.len() {
        let Ok(encoded) = reader.i32() else {
            break;
        };
        let opcode = (encoded as u32 & 0x7) as u8;
        let argument = encoded >> 3;
        let mut function_reference = None;
        let operation = match opcode {
            0 => {
                if let Some(line) = lines.last_mut() {
                    *line = line.saturating_add(i64::from(argument));
                }
                CodeSourceMapOperation::ChangePosition
            }
            1 => {
                pc_offset = pc_offset.saturating_add(argument.max(0) as u32);
                CodeSourceMapOperation::AdvancePc
            }
            2 => {
                function_reference = usize::try_from(argument)
                    .ok()
                    .and_then(|index| inlined_functions.get(index.saturating_add(1)))
                    .copied();
                lines.push(-1);
                CodeSourceMapOperation::PushFunction
            }
            3 => {
                if lines.len() > 1 {
                    lines.pop();
                }
                CodeSourceMapOperation::PopFunction
            }
            4 => CodeSourceMapOperation::NullCheck,
            _ => CodeSourceMapOperation::Unknown,
        };
        entries.push(CodeSourceMapEntry {
            pc_offset,
            operation,
            argument,
            inline_depth: lines.len().saturating_sub(1),
            source_line: lines.last().copied().filter(|line| *line > 0),
            function_reference,
        });
        if operation == CodeSourceMapOperation::Unknown {
            break;
        }
    }
    entries
}

fn summarize_snapshot(
    vm: &ParseResult,
    isolate: &ParseResult,
    table: &InstructionTable,
) -> SnapshotEvidence {
    let mut evidence = SnapshotEvidence {
        vm_objects: vm.objects.len(),
        isolate_objects: isolate.objects.len(),
        instruction_table_entries: table.entries.len(),
        instruction_entries_with_stack_maps: table
            .entries
            .iter()
            .filter(|entry| entry.stack_map_offset != 0)
            .count(),
        compressed_stack_maps: table.stack_maps.len(),
        stack_map_entries: table.stack_maps.values().map(|map| map.entry_count).sum(),
        canonical_stack_map_entries: table
            .canonical_stack_map_offset
            .and_then(|offset| table.stack_maps.get(&offset))
            .map_or(0, |map| map.entry_count),
        dispatch_table_entries: isolate.dispatch_table_code_indices.len(),
        dispatch_table_code_entries: isolate
            .dispatch_table_code_indices
            .iter()
            .filter(|entry| entry.is_some())
            .count(),
        dispatch_table_unique_code_indices: isolate
            .dispatch_table_code_indices
            .iter()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        ..SnapshotEvidence::default()
    };
    let mut scalar_checksum = 0u64;
    for snapshot in [vm, isolate] {
        evidence.reference_edges += snapshot.object_references.len();
        evidence.reverse_reference_targets += snapshot.reverse_reference_target_count();
        evidence.object_pools += snapshot.object_pools.len();
        evidence.object_pool_entries += snapshot
            .object_pools
            .iter()
            .map(|pool| {
                scalar_checksum ^= pool.reference as u64;
                pool.entries
                    .iter()
                    .map(|entry| match entry {
                        super::types::PoolValue::Reference(value) => *value as u64,
                        super::types::PoolValue::Immediate(value) => *value as u64,
                        super::types::PoolValue::Native => 1,
                        super::types::PoolValue::Empty => 0,
                    })
                    .for_each(|value| scalar_checksum = scalar_checksum.rotate_left(5) ^ value);
                pool.entries.len()
            })
            .sum::<usize>();
        evidence.code_objects += snapshot.codes.len();
        evidence.instance_layout_bitmaps += snapshot.instance_bitmaps.len();
        evidence.exception_handler_tables += snapshot.exception_handlers.len();
        evidence.exception_handlers += snapshot
            .exception_handlers
            .values()
            .map(|handlers| {
                scalar_checksum ^= handlers.reference as u64;
                handlers.entries.len()
            })
            .sum::<usize>();
        for object in &snapshot.objects {
            *evidence.cluster_cids.entry(object.cid).or_default() += 1;
            *evidence
                .object_kinds
                .entry(snapshot_kind_label(object.kind).to_owned())
                .or_default() += 1;
            evidence.canonical_objects += usize::from(object.canonical);
            let scalars = snapshot.scalars_of(object);
            evidence.scalar_fields += scalars.len();
            if object.kind == super::types::SnapshotObjectKind::MetadataBytes {
                evidence.metadata_payloads += 1;
                evidence.metadata_bytes += snapshot.bytes_of(object).len();
            }
            for scalar in scalars {
                let value = match scalar {
                    super::types::SnapshotScalar::Unsigned(value) => *value as u64,
                    super::types::SnapshotScalar::Tagged32(value) => u64::from(*value),
                    super::types::SnapshotScalar::Tagged64(value) => *value as u64,
                    super::types::SnapshotScalar::Uint16(value) => u64::from(*value),
                    super::types::SnapshotScalar::Int16(value) => *value as u64,
                    super::types::SnapshotScalar::Byte(value) => u64::from(*value),
                    super::types::SnapshotScalar::Reference(value) => *value as u64,
                };
                scalar_checksum = scalar_checksum.rotate_left(7) ^ value;
            }
        }
    }
    std::hint::black_box(scalar_checksum);
    evidence
}

fn snapshot_kind_label(kind: super::types::SnapshotObjectKind) -> &'static str {
    match kind {
        super::types::SnapshotObjectKind::Standard => "standard",
        super::types::SnapshotObjectKind::String => "string",
        super::types::SnapshotObjectKind::Integer => "integer",
        super::types::SnapshotObjectKind::Double => "double",
        super::types::SnapshotObjectKind::Code => "code",
        super::types::SnapshotObjectKind::ObjectPool => "object_pool",
        super::types::SnapshotObjectKind::Array => "array",
        super::types::SnapshotObjectKind::WeakArray => "weak_array",
        super::types::SnapshotObjectKind::TypedData => "typed_data",
        super::types::SnapshotObjectKind::ExceptionHandlers => "exception_handlers",
        super::types::SnapshotObjectKind::Context => "context",
        super::types::SnapshotObjectKind::TypeArguments => "type_arguments",
        super::types::SnapshotObjectKind::MetadataBytes => "metadata_bytes",
        super::types::SnapshotObjectKind::Instance => "instance",
        super::types::SnapshotObjectKind::Record => "record",
        super::types::SnapshotObjectKind::ContextScope => "context_scope",
        super::types::SnapshotObjectKind::Class => "class",
    }
}

fn parse_address(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

fn restore_snapshot_name(
    value: &str,
    obfuscation_map: Option<&crate::analysis::LoadedObfuscationMap>,
) -> String {
    let readable = crate::analysis::readable_snapshot_name(value);
    obfuscation_map.map_or(readable.clone(), |map| map.restore(&readable))
}

fn restore_library_uri(
    value: Option<String>,
    obfuscation_map: Option<&crate::analysis::LoadedObfuscationMap>,
) -> Option<String> {
    value.map(|value| obfuscation_map.map_or(value.clone(), |map| map.restore(&value)))
}

fn select_application_package(
    explicit: Option<&str>,
    library_uris: impl IntoIterator<Item = String>,
) -> Option<String> {
    explicit.map(str::to_owned).or_else(|| {
        let library_uris = library_uris.into_iter().collect::<BTreeSet<_>>();
        crate::analysis::choose_application_package(&library_uris)
    })
}

fn qualified_name(owner: &str, name: &str) -> String {
    if owner.is_empty() {
        name.to_owned()
    } else {
        format!("{owner}.{name}")
    }
}

fn synthetic_code_reference(slot: usize) -> i32 {
    i32::MIN.saturating_add(i32::try_from(slot).unwrap_or(i32::MAX))
}

fn library_ownership_is_obfuscated(
    library_uris: impl IntoIterator<Item = String>,
    application_package: Option<&str>,
) -> bool {
    let application_prefix = application_package.map(|package| format!("package:{package}/"));
    let library_uris = library_uris
        .into_iter()
        .filter(|uri| !uri.is_empty())
        .collect::<BTreeSet<_>>();
    let mut recognized = 0usize;
    let mut opaque = 0usize;
    let mut compact_opaque = 0usize;
    let mut application = 0usize;
    for uri in library_uris {
        if application_prefix
            .as_deref()
            .is_some_and(|prefix| uri.starts_with(prefix))
        {
            application += 1;
        }
        if uri.starts_with("dart:")
            || uri.starts_with("package:")
            || uri.starts_with("file:")
            || uri.starts_with("org-dartlang-")
        {
            recognized += 1;
        } else {
            opaque += 1;
            if looks_like_compact_obfuscated_library(&uri) {
                compact_opaque += 1;
            }
        }
    }
    let opaque_dominates = opaque >= 8 && opaque > recognized.saturating_mul(2);
    let compact_obfuscation_signature =
        compact_opaque >= 8 && compact_opaque.saturating_mul(4) >= opaque.saturating_mul(3);
    opaque_dominates && (application > 0 || compact_obfuscation_signature)
}

fn looks_like_compact_obfuscated_library(uri: &str) -> bool {
    (1..=8).contains(&uri.len())
        && uri
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, b'_' | b'$'))
}

fn include_library(uri: Option<&str>, scope: Scope, application_package: Option<&str>) -> bool {
    match scope {
        Scope::All => true,
        Scope::Packages => uri
            .is_some_and(|uri| uri.starts_with("package:") && !uri.starts_with("package:flutter/")),
        Scope::App => application_package.is_some_and(|package| {
            uri.is_some_and(|uri| uri.starts_with(&format!("package:{package}/")))
        }),
    }
}

struct Names<'a> {
    isolate: &'a ParseResult,
    strings: BTreeMap<i32, &'a str>,
}

impl<'a> Names<'a> {
    fn new(isolate: &'a ParseResult, vm: &'a ParseResult) -> Self {
        let mut strings: BTreeMap<_, _> = vm
            .strings
            .iter()
            .map(|(reference, value)| (*reference, value.as_str()))
            .collect();
        strings.extend(
            isolate
                .strings
                .iter()
                .map(|(reference, value)| (*reference, value.as_str())),
        );
        Self { isolate, strings }
    }

    fn name(&self, reference: i32) -> String {
        self.isolate
            .named
            .get(&reference)
            .and_then(|object| self.strings.get(&object.name_ref))
            .copied()
            .unwrap_or_default()
            .to_owned()
    }

    fn owner_name(&self, reference: i32) -> String {
        self.isolate
            .named
            .get(&reference)
            .map_or_else(String::new, |object| self.name(object.owner_ref))
    }

    fn library_uri(&self, reference: i32) -> Option<String> {
        let mut current = reference;
        for _ in 0..12 {
            let object = self.isolate.named.get(&current)?;
            if let Some(script_ref) = object.source_uri_ref
                && let Some(script) = self.isolate.named.get(&script_ref)
                && let Some(uri) = self.strings.get(&script.name_ref)
            {
                return Some((*uri).to_owned());
            }
            if object.cid == 13 {
                return self
                    .strings
                    .get(&object.name_ref)
                    .map(|value| (*value).to_owned());
            }
            current = object.owner_ref;
            if current < 0 {
                return None;
            }
        }
        None
    }
}

fn round_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment).saturating_mul(alignment)
}

#[cfg(test)]
mod tests {
    use super::{
        Names, Range, assign_sizes, count_stack_map_entries, decode_code_source_map_bytes,
        drop_implicit_call_prefix, library_ownership_is_obfuscated, lexical_parent,
        nested_pool_strings, object_pool_labels, parse_table, recover_closure_parents,
        select_application_package,
    };
    use crate::model::CodeSourceMapOperation;
    use crate::snapshot::cluster::cid::test_cids;
    use crate::snapshot::cluster::type_recovery::TypeRecovery;
    use crate::snapshot::cluster::types::{
        ClusterHeader, NamedObject, ParseResult, SnapshotObjectKind, SnapshotObjectPayload,
    };

    #[test]
    fn recovers_closure_lexical_parents_from_closure_data_edges() {
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        snapshot.strings.insert(10, "outer".to_owned());
        snapshot.strings.insert(11, "closureA".to_owned());
        snapshot.strings.insert(12, "standalone".to_owned());
        let named_function = |name_ref: i32| NamedObject {
            cid: cids.function,
            name_ref,
            owner_ref: -1,
            signature_ref: -1,
            function_kind_tag: None,
            instruction_index: None,
            source_uri_ref: None,
        };
        // Ref 21 encloses closure body 22; 24 is a closure whose parent edge
        // was nulled out during serialization; 21 itself is a plain member.
        snapshot.named.insert(21, named_function(10));
        snapshot.named.insert(22, named_function(11));
        snapshot.named.insert(24, named_function(12));
        // Full AOT ClosureData serializes refs as [parent_function, closure].
        snapshot.insert_object(
            30,
            cids.closure_data,
            false,
            SnapshotObjectKind::Standard,
            SnapshotObjectPayload {
                references: vec![21, 22],
                scalars: Vec::new(),
                bytes: Vec::new(),
            },
        );
        snapshot.insert_object(
            31,
            cids.closure_data,
            false,
            SnapshotObjectKind::Standard,
            SnapshotObjectPayload {
                references: vec![-1, 24],
                scalars: Vec::new(),
                bytes: Vec::new(),
            },
        );
        // Each named closure body reaches its ClosureData through the fourth
        // serialized Function reference (the data_ edge).
        let function_payload = |data_ref: i32| SnapshotObjectPayload {
            references: vec![-1, -1, -1, data_ref],
            scalars: Vec::new(),
            bytes: Vec::new(),
        };
        snapshot.insert_object(22, cids.function, false, SnapshotObjectKind::Standard, function_payload(30));
        snapshot.insert_object(24, cids.function, false, SnapshotObjectKind::Standard, function_payload(31));

        let parents = recover_closure_parents(&snapshot, &cids);
        assert_eq!(parents.get(&22), Some(&21));
        assert_eq!(parents.get(&24), None);
        assert_eq!(parents.get(&21), None);

        let names = Names::new(&snapshot, &snapshot);
        assert_eq!(
            lexical_parent(&snapshot, &parents, 22, &names).as_deref(),
            Some("outer")
        );
        assert_eq!(lexical_parent(&snapshot, &parents, 24, &names), None);
        assert_eq!(lexical_parent(&snapshot, &parents, 21, &names), None);
    }

    fn write_bounded_i32(output: &mut Vec<u8>, mut value: i32) {
        while !(-64..=63).contains(&value) {
            output.push((value as u8) & 0x7f);
            value >>= 7;
        }
        output.push((value + 192) as u8);
    }

    #[test]
    fn recovers_strings_nested_in_value_object_graphs() {
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        snapshot.insert_object(
            10,
            200,
            true,
            SnapshotObjectKind::Instance,
            SnapshotObjectPayload {
                references: vec![11],
                scalars: Vec::new(),
                bytes: Vec::new(),
            },
        );
        snapshot.strings.insert(11, "nested value".to_owned());

        assert_eq!(
            nested_pool_strings(&snapshot, 10),
            vec!["\"nested value\"".to_owned()]
        );
    }

    #[test]
    fn labels_unlinked_call_pool_entries_with_dynamic_selectors() {
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        snapshot.strings.insert(10, "isEmpty".to_owned());
        // UntaggedCallSiteData serializes [target_name, args_descriptor].
        snapshot.named.insert(
            20,
            NamedObject {
                cid: cids.unlinked_call,
                name_ref: 10,
                owner_ref: -1,
                signature_ref: -1,
                function_kind_tag: None,
                instruction_index: None,
                source_uri_ref: None,
            },
        );
        snapshot.insert_object(
            20,
            cids.unlinked_call,
            false,
            SnapshotObjectKind::Standard,
            SnapshotObjectPayload {
                references: vec![10, 21],
                scalars: Vec::new(),
                bytes: Vec::new(),
            },
        );
        // ArgsDescriptor array [typeArgsLen=0, count=2, size, positionalCount=2].
        snapshot.insert_object(
            21,
            cids.immutable_array,
            false,
            SnapshotObjectKind::Array,
            SnapshotObjectPayload {
                references: vec![-1, -1, -1, -1],
                scalars: vec![
                    crate::snapshot::cluster::types::SnapshotScalar::Unsigned(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Unsigned(2),
                    crate::snapshot::cluster::types::SnapshotScalar::Unsigned(4),
                    crate::snapshot::cluster::types::SnapshotScalar::Unsigned(2),
                ],
                bytes: Vec::new(),
            },
        );
        let pool = super::super::types::ObjectPool {
            reference: 30,
            entries: vec![
                super::super::types::PoolValue::Reference(20),
                super::super::types::PoolValue::Empty,
            ],
        };
        let names = Names::new(&snapshot, &snapshot);
        let types = TypeRecovery::new(
            &snapshot,
            &snapshot,
            &cids,
            crate::model::Abi::Arm64V8a,
            None,
        );
        let labels = object_pool_labels(&snapshot, &names, &types, &pool, None, &cids);
        assert_eq!(labels[0], "dynamicCall(\"isEmpty\", arity=2)");
        assert_eq!(labels[1], "resetPoolEntry(1)");
    }

    #[test]
    fn drops_implicit_dynamic_call_prefixes() {
        assert_eq!(
            drop_implicit_call_prefix("dyn:implicit:call"),
            "dyn:call"
        );
        assert_eq!(drop_implicit_call_prefix("isEmpty"), "isEmpty");
    }

    #[test]
    fn infers_application_package_from_restored_snapshot_uris() {
        let restored = [
            "package:simple_app_obfuscated/main.dart".to_owned(),
            "package:simple_app_obfuscated/models.dart".to_owned(),
        ];

        assert_eq!(
            select_application_package(None, restored.clone()).as_deref(),
            Some("simple_app_obfuscated")
        );
        assert_eq!(
            select_application_package(Some("pre_scanned"), restored).as_deref(),
            Some("pre_scanned")
        );
    }

    fn write_source_map_op(output: &mut Vec<u8>, opcode: i32, argument: i32) {
        write_bounded_i32(output, argument.wrapping_shl(3) | opcode);
    }

    #[test]
    fn assigns_duplicate_code_ranges_consistently() {
        let mut ranges = vec![
            Range {
                code_ref: 1,
                owner_ref: 1,
                pc_offset: 10,
                stack_map_offset: 0,
                size: 0,
            },
            Range {
                code_ref: 2,
                owner_ref: 2,
                pc_offset: 10,
                stack_map_offset: 0,
                size: 0,
            },
            Range {
                code_ref: 3,
                owner_ref: 3,
                pc_offset: 20,
                stack_map_offset: 0,
                size: 0,
            },
        ];
        assign_sizes(&mut ranges, 30);
        assert_eq!(ranges[0].size, 10);
        assert_eq!(ranges[1].size, 10);
        assert_eq!(ranges[2].size, 10);
    }

    #[test]
    fn detects_when_obfuscation_hides_library_ownership() {
        let mut obfuscated = vec!["package:example/main.dart".to_owned()];
        obfuscated.extend((0..12).map(|index| format!("opaque{index}")));
        assert!(library_ownership_is_obfuscated(obfuscated, Some("example")));

        let mut fully_obfuscated = vec![
            "dart:core".to_owned(),
            "package:flutter/widgets.dart".to_owned(),
        ];
        fully_obfuscated.extend((0..12).map(|index| format!("x{index:02}")));
        assert!(library_ownership_is_obfuscated(
            fully_obfuscated.clone(),
            Some("ffi")
        ));
        assert!(library_ownership_is_obfuscated(fully_obfuscated, None));

        assert!(!library_ownership_is_obfuscated(
            [
                "package:example/main.dart".to_owned(),
                "package:flutter/widgets.dart".to_owned(),
                "dart:core".to_owned(),
            ],
            Some("example"),
        ));

        let relative_sources = (0..12)
            .map(|index| format!("feature_{index}.dart"))
            .collect::<Vec<_>>();
        assert!(!library_ownership_is_obfuscated(relative_sources, None));
    }

    #[test]
    fn accepts_dwarf_tables_with_discarded_prefix_entries() {
        let header = ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 2,
            instruction_table_data_offset: 64,
        };
        let mut data = vec![0u8; 80 + 16 + 4 * 8];
        data[84..88].copy_from_slice(&4u32.to_le_bytes());
        data[88..92].copy_from_slice(&2u32.to_le_bytes());
        for (index, offset) in [10u32, 20, 30, 40].into_iter().enumerate() {
            let start = 96 + index * 8;
            data[start..start + 4].copy_from_slice(&offset.to_le_bytes());
        }

        let table = parse_table(&data, &header, 0, 8).unwrap();
        assert_eq!(table.first_code, 2);
        assert_eq!(table.entries.len(), 4);
        assert_eq!(table.entries[3].pc_offset, 40);
    }

    #[test]
    fn parses_canonical_and_per_code_stack_maps() {
        let header = ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 1,
            instruction_table_data_offset: 64,
        };
        // The Data object starts at byte 80. Its 16-byte header and one
        // 8-byte entry put the first packed map at relative offset 24.
        let mut data = vec![0u8; 128];
        data[80..84].copy_from_slice(&24u32.to_le_bytes());
        data[84..88].copy_from_slice(&1u32.to_le_bytes());
        data[88..92].copy_from_slice(&0u32.to_le_bytes());
        data[96..100].copy_from_slice(&12u32.to_le_bytes());
        data[100..104].copy_from_slice(&24u32.to_le_bytes());
        // A global table with two one-byte bitmaps.
        let payload = [1u8, 7, 0xaa, 0, 8, 0x55];
        let flags_and_size = ((payload.len() as u32) << 2) | 1;
        data[104..108].copy_from_slice(&flags_and_size.to_le_bytes());
        data[108..114].copy_from_slice(&payload);

        let table = parse_table(&data, &header, 0, 8).unwrap();
        assert_eq!(table.canonical_stack_map_offset, Some(24));
        let canonical = table.stack_maps.get(&24).unwrap();
        assert!(canonical.global_table);
        assert_eq!(canonical.entry_count, 2);
    }

    #[test]
    fn counts_all_compressed_stack_map_encodings() {
        // Two inlined maps: pc delta, spill bits, non-spill bits, bitmap.
        assert_eq!(
            count_stack_map_entries(&[1, 2, 6, 0xaa, 4, 0, 8, 0x55], false, false),
            2
        );
        // Two maps referencing offsets in the canonical global table.
        assert_eq!(count_stack_map_entries(&[1, 3, 4, 7], false, true), 2);
        // Two canonical entries: spill bits, non-spill bits, bitmap.
        assert_eq!(
            count_stack_map_entries(&[1, 7, 0xaa, 0, 8, 0x55], true, false),
            2
        );
        assert_eq!(count_stack_map_entries(&[1, 2], true, true), 0);
        assert_eq!(count_stack_map_entries(&[1, 9, 0xaa], true, false), 0);
    }

    #[test]
    fn decodes_source_map_line_origin_pc_and_inline_stack() {
        let mut bytes = Vec::new();
        write_source_map_op(&mut bytes, 0, 11); // -1 + 11 = line 10
        write_source_map_op(&mut bytes, 1, 5);
        write_source_map_op(&mut bytes, 2, 0);
        write_source_map_op(&mut bytes, 0, 21); // new inline line: -1 + 21 = 20
        write_source_map_op(&mut bytes, 4, 7);
        write_source_map_op(&mut bytes, 3, 0);
        write_source_map_op(&mut bytes, 0, -2); // root line: 10 - 2 = 8

        // Array objects retain their type-arguments reference at index zero.
        let entries = decode_code_source_map_bytes(&bytes, &[0, 42]);
        assert_eq!(entries.len(), 7);
        assert_eq!(entries[0].source_line, Some(10));
        assert_eq!(entries[1].pc_offset, 5);
        assert_eq!(entries[2].operation, CodeSourceMapOperation::PushFunction);
        assert_eq!(entries[2].function_reference, Some(42));
        assert_eq!(entries[3].inline_depth, 1);
        assert_eq!(entries[3].source_line, Some(20));
        assert_eq!(entries[4].operation, CodeSourceMapOperation::NullCheck);
        assert_eq!(entries[6].inline_depth, 0);
        assert_eq!(entries[6].source_line, Some(8));
    }

    #[test]
    fn rejects_dwarf_table_when_retained_count_disagrees() {
        let header = ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 3,
            instruction_table_data_offset: 64,
        };
        let mut data = vec![0u8; 80 + 16 + 4 * 8];
        data[84..88].copy_from_slice(&4u32.to_le_bytes());
        data[88..92].copy_from_slice(&2u32.to_le_bytes());

        let error = parse_table(&data, &header, 0, 8).unwrap_err();
        assert!(error.to_string().contains("2 retained entries"));
    }
}
