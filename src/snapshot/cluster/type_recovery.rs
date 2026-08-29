use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    Abi, RecoveredClassMetadata, RecoveredFieldMetadata, RecoveredParameter, RecoveredSignature,
    RecoveredSignatureDetails, RecoveredType, RecoveredTypeParameter,
};

use super::cid::Cids;
use super::types::{FunctionType, ParseResult, SnapshotObject, SnapshotObjectKind, SnapshotScalar};

const NULL_REFERENCE: i32 = 1;
const EMPTY_ARRAY_REFERENCE: i32 = 4;
const DYNAMIC_TYPE_REFERENCE: i32 = 7;
const VOID_TYPE_REFERENCE: i32 = 8;
const EMPTY_TYPE_ARGUMENTS_REFERENCE: i32 = 9;
const TRUE_REFERENCE: i32 = 10;
const FALSE_REFERENCE: i32 = 11;

const TYPE_CLASS_ID_SHIFT: u32 = 3;
const MAX_TYPE_DEPTH: usize = 12;

/// Joins canonical const instances to their enum class to recover constant
/// names without a VM oracle.
///
/// Every enum value is a canonical `Instance` whose cid is the enum's class
/// id. `_Enum` declares `index` then `_name`, so the instance's first
/// String-valued reference slot is `_name` — enhanced-enum fields are
/// declared after the base ones and therefore sort later. The String payload
/// is data, not an identifier, so `--obfuscate` leaves it intact.
///
/// Instances are visited in reference order, which is the order the
/// serializer wrote the enum's constants, but ordering is treated as
/// presentation only: no ordinal is claimed from it.
fn recover_enum_values(
    isolate: &ParseResult,
    vm: &ParseResult,
    cids: &Cids,
) -> BTreeMap<i32, Vec<String>> {
    // Only classes the snapshot itself marks as enums (state bit 9) may
    // receive constants. Read from the object graph rather than the named
    // table so tree-shaken/unnamed enum classes are still covered.
    let mut enum_class_ids = BTreeSet::new();
    for snapshot in [isolate, vm] {
        for object in &snapshot.objects {
            if object.cid != cids.class {
                continue;
            }
            let scalars = snapshot.scalars_of(object);
            let Some(SnapshotScalar::Tagged32(class_id)) = scalars.first() else {
                continue;
            };
            let Some(SnapshotScalar::Tagged32(state_bits)) = scalars.get(6) else {
                continue;
            };
            if state_bits & (1 << 9) != 0
                && let Ok(class_id) = i32::try_from(*class_id)
            {
                enum_class_ids.insert(class_id);
            }
        }
    }
    if enum_class_ids.is_empty() {
        return BTreeMap::new();
    }

    let mut values = BTreeMap::<i32, Vec<String>>::new();
    for snapshot in [isolate, vm] {
        for object in &snapshot.objects {
            if !object.canonical
                || object.kind != SnapshotObjectKind::Instance
                || !enum_class_ids.contains(&object.cid)
            {
                continue;
            }
            // First String-valued slot is `_Enum._name`. Strings are resolved
            // through the string table directly: rodata-backed strings are
            // registered there without a corresponding SnapshotObject, and in
            // AOT that is where most string payloads live.
            let name = snapshot.references_of(object).iter().find_map(|slot| {
                snapshot
                    .strings
                    .get(slot)
                    .or_else(|| vm.strings.get(slot))
                    .filter(|name| is_enum_member_name(name))
            });
            if let Some(name) = name {
                let members = values.entry(object.cid).or_default();
                if !members.iter().any(|existing| existing == name) {
                    members.push(name.clone());
                }
            }
        }
    }
    let _ = cids;
    values.retain(|_, members| !members.is_empty());
    values
}

/// Enum constants are Dart identifiers. Requiring that shape keeps unrelated
/// String payloads in an instance's slots from being promoted to members.
fn is_enum_member_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        })
}

pub(super) struct TypeRecovery<'a> {
    isolate: &'a ParseResult,
    vm: &'a ParseResult,
    cids: &'a Cids,
    abi: Abi,
    obfuscation_map: Option<&'a crate::analysis::LoadedObfuscationMap>,
    class_references: BTreeMap<i32, i32>,
    /// Class id -> recovered enum constant names, joined from the canonical
    /// const-instance graph. `_Enum._name` is a plain String payload, so it
    /// survives `--obfuscate` (which renames identifiers, not string data)
    /// and is available without any VM oracle.
    enum_values: BTreeMap<i32, Vec<String>>,
}

impl<'a> TypeRecovery<'a> {
    pub fn new(
        isolate: &'a ParseResult,
        vm: &'a ParseResult,
        cids: &'a Cids,
        abi: Abi,
        obfuscation_map: Option<&'a crate::analysis::LoadedObfuscationMap>,
    ) -> Self {
        let mut class_references = BTreeMap::new();
        for snapshot in [vm, isolate] {
            for (reference, named) in &snapshot.named {
                if named.cid != cids.class {
                    continue;
                }
                let Some(object) = snapshot.object(*reference) else {
                    continue;
                };
                let Some(SnapshotScalar::Tagged32(class_id)) = snapshot.scalars_of(object).first()
                else {
                    continue;
                };
                if let Ok(class_id) = i32::try_from(*class_id) {
                    class_references.insert(class_id, *reference);
                }
            }
        }
        Self {
            isolate,
            vm,
            cids,
            abi,
            obfuscation_map,
            enum_values: recover_enum_values(isolate, vm, cids),
            class_references,
        }
    }

    pub fn signature(&self, signature: &FunctionType) -> RecoveredSignature {
        let named_elements = signature
            .named_parameter_names_ref
            .map(|reference| self.array_elements(reference))
            .unwrap_or_default();
        let parameter_type_references = signature
            .parameter_types_ref
            .map(|reference| self.array_elements(reference))
            .unwrap_or_default();
        let named_names = named_elements
            .iter()
            .take(signature.optional)
            .map(|reference| self.string(*reference).map(str::to_owned))
            .collect::<Vec<_>>();
        let flags_per_element = match self.abi {
            Abi::ArmeabiV7a => 8,
            Abi::Arm64V8a | Abi::X86_64 => 16,
        };
        let mut parameters = Vec::with_capacity(signature.fixed + signature.optional);
        for position in 0..signature.fixed.saturating_add(signature.optional) {
            let is_named = signature.optional_are_named && position >= signature.fixed;
            let optional_index = position.saturating_sub(signature.fixed);
            let name = is_named
                .then(|| named_names.get(optional_index).cloned().flatten())
                .flatten()
                .map(|name| self.restore_name(&name));
            let is_required = if is_named {
                let flag_reference = named_elements
                    .get(signature.optional + optional_index / flags_per_element)
                    .copied();
                let mask = 1i64 << (optional_index % flags_per_element);
                flag_reference
                    .and_then(|reference| self.integer(reference))
                    .is_some_and(|flags| flags & mask != 0)
            } else {
                position < signature.fixed
            };
            let type_reference = parameter_type_references
                .get(signature.implicit + position)
                .copied();
            parameters.push(RecoveredParameter {
                position,
                name,
                declared_type: type_reference.and_then(|reference| self.recover_type(reference)),
                is_named,
                is_required,
            });
        }
        let resolved = RecoveredSignatureDetails {
            return_type: signature
                .result_type_ref
                .and_then(|reference| self.recover_type(reference)),
            parameters,
            type_parameters: signature
                .type_parameters_ref
                .map(|reference| self.type_parameters(reference))
                .unwrap_or_default(),
        };
        RecoveredSignature {
            fixed_parameter_count: signature.fixed,
            optional_parameter_count: signature.optional,
            optional_parameters_are_named: signature.optional_are_named,
            implicit_parameter_count: signature.implicit,
            type_parameters_reference: signature.type_parameters_ref,
            result_type_reference: signature.result_type_ref,
            parameter_types_reference: signature.parameter_types_ref,
            named_parameter_names_reference: signature.named_parameter_names_ref,
            flags: signature.flags,
            packed_type_parameter_counts: signature.packed_type_parameter_counts,
            resolved: Some(resolved),
        }
    }

    pub fn class_metadata(&self, reference: i32) -> Option<RecoveredClassMetadata> {
        let (snapshot, object) = self.object(reference)?;
        if object.cid != self.cids.class {
            return None;
        }
        let references = snapshot.references_of(object);
        let scalars = snapshot.scalars_of(object);
        let class_id = match scalars.first()? {
            SnapshotScalar::Tagged32(value) => i32::try_from(*value).ok()?,
            _ => return None,
        };
        let state_bits = match scalars.get(6)? {
            SnapshotScalar::Tagged32(value) => *value,
            _ => return None,
        };
        let mut type_parameters = references
            .get(8)
            .copied()
            .map(|reference| self.type_parameters(reference))
            .unwrap_or_default();
        let mut super_type = references
            .get(9)
            .copied()
            .and_then(|reference| self.recover_type(reference));
        let mut interfaces: Vec<RecoveredType> = references
            .get(5)
            .copied()
            .map(|reference| {
                self.array_elements(reference)
                    .into_iter()
                    .filter_map(|reference| self.recover_type(reference))
                    .collect()
            })
            .unwrap_or_default();
        contextualize_class_types(&mut type_parameters, &mut super_type, &mut interfaces);
        // Unboxed-field bitmaps turn bare slot offsets into typed
        // placeholders that survive even when the Field names themselves
        // were tree-shaken. Bit w marks word w from the object start
        // (headers included), matching how fill_skip.instance reads them.
        // The Instance cluster's `next_field_words` bounds the class's TRUE
        // field count: walking past it turns bitmap padding or neighbouring-
        // class bits into phantom fields (probe EC-7 gave `E15Vec` four
        // slots for two source fields), and clear bits inside the range are
        // ordinary reference slots the unboxed bitmap never records.
        let (header_words, word_size): (i64, i64) = match self.abi {
            Abi::Arm64V8a => (2, 4),
            Abi::ArmeabiV7a => (1, 4),
            Abi::X86_64 => (2, 8),
        };
        let header_bytes = header_words * word_size;
        let mut instance_slots = Vec::new();
        for snapshot in [self.isolate, self.vm] {
            let Some(bitmap) = snapshot.instance_bitmaps.get(&class_id) else {
                continue;
            };
            // Same cluster that wrote the bitmap carries the field count.
            let next_field_words = snapshot
                .clusters
                .iter()
                .find(|cluster| cluster.cid == class_id && cluster.next_field_words > 0)
                .map(|cluster| i64::from(cluster.next_field_words));
            let field_end = next_field_words.map(|words| words.min(64)).unwrap_or(64);
            for word in header_words..field_end {
                if word < 0 {
                    continue;
                }
                let unboxed = bitmap & (1u64 << word) != 0;
                if !unboxed && next_field_words.is_none() {
                    // Without a trustworthy field count only unboxed bits
                    // mean anything; clear bits could be anything.
                    continue;
                }
                instance_slots.push(crate::model::RecoveredInstanceSlot {
                    offset: header_bytes + (word - header_words) * word_size,
                    is_reference: !unboxed,
                    slot_type: if unboxed {
                        "unboxed_field".to_owned()
                    } else {
                        "reference".to_owned()
                    },
                    field_name: None,
                    field_object_id: None,
                });
            }
            break;
        }
        Some(RecoveredClassMetadata {
            class_id,
            type_parameters,
            super_type,
            interfaces,
            is_abstract: bit(state_bits, 6),
            is_enum: bit(state_bits, 9),
            enum_values: self.enum_values.get(&class_id).cloned().unwrap_or_default(),
            is_transformed_mixin_application: bit(state_bits, 10),
            is_sealed: bit(state_bits, 14),
            is_mixin_class: bit(state_bits, 15),
            is_base: bit(state_bits, 16),
            is_interface: bit(state_bits, 17),
            is_final: bit(state_bits, 18),
            instance_size: None,
            type_arguments_field_offset: None,
            instance_slots,
        })
    }

    pub fn field_metadata(&self, reference: i32) -> Option<RecoveredFieldMetadata> {
        let (snapshot, object) = self.object(reference)?;
        if object.cid != self.cids.field {
            return None;
        }
        let references = snapshot.references_of(object);
        let scalars = snapshot.scalars_of(object);
        let type_reference = references.get(2).copied()?;
        let initializer_reference = references.get(3).copied().unwrap_or(NULL_REFERENCE);
        let kind_bits = match scalars.first()? {
            SnapshotScalar::Tagged32(value) => *value,
            _ => return None,
        };
        let offset_or_field_id_reference = match scalars.get(1) {
            Some(SnapshotScalar::Reference(reference)) => Some(*reference),
            _ => None,
        };
        let mut declared_type = self.recover_type(type_reference);
        let owner_type_parameters = references
            .get(1)
            .copied()
            .and_then(|reference| self.class_type_parameter_names(reference))
            .unwrap_or_default();
        if let Some(declared_type) = &mut declared_type {
            contextualize_type(declared_type, &owner_type_parameters);
        }
        Some(RecoveredFieldMetadata {
            type_reference,
            declared_type,
            initializer_reference,
            offset_or_field_id_reference,
            is_const: bit(kind_bits, 0),
            is_static: bit(kind_bits, 1),
            is_final: bit(kind_bits, 2),
            has_nontrivial_initializer: bit(kind_bits, 3),
            is_late: bit(kind_bits, 10),
            has_initializer: bit(kind_bits, 14),
            instance_field_offset: None,
            static_field_offset: None,
            static_value_object_id: None,
            is_reference: None,
            unboxed_type: None,
        })
    }

    pub fn recover_type(&self, reference: i32) -> Option<RecoveredType> {
        let mut visiting = BTreeSet::new();
        self.recover_type_inner(reference, 0, &mut visiting)
    }

    pub fn scalar_label(&self, reference: i32) -> Option<String> {
        match reference {
            NULL_REFERENCE => Some("null".to_owned()),
            TRUE_REFERENCE => Some("true".to_owned()),
            FALSE_REFERENCE => Some("false".to_owned()),
            _ => self.integer(reference).map(|value| value.to_string()),
        }
    }

    fn recover_type_inner(
        &self,
        reference: i32,
        depth: usize,
        visiting: &mut BTreeSet<i32>,
    ) -> Option<RecoveredType> {
        if depth >= MAX_TYPE_DEPTH || !visiting.insert(reference) {
            return None;
        }
        let result = match reference {
            DYNAMIC_TYPE_REFERENCE => Some(RecoveredType {
                snapshot_reference: reference,
                display_name: "dynamic".to_owned(),
                library_uri: Some("dart:core".to_owned()),
            }),
            VOID_TYPE_REFERENCE => Some(RecoveredType {
                snapshot_reference: reference,
                display_name: "void".to_owned(),
                library_uri: Some("dart:core".to_owned()),
            }),
            NULL_REFERENCE => None,
            _ => {
                if let Some((snapshot, object)) = self.object(reference) {
                    if object.cid == self.cids.type_ {
                        self.nominal_type(snapshot, object, depth, visiting)
                    } else if object.cid == self.cids.function_type {
                        self.function_type(reference, depth, visiting)
                    } else if object.cid == self.cids.type_parameter {
                        self.type_parameter(snapshot, object, depth, visiting)
                    } else if object.cid == self.cids.record_type {
                        self.record_type(snapshot, object, depth, visiting)
                    } else if object.cid == self.cids.future_or {
                        self.future_or_type(snapshot, object, depth, visiting)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        visiting.remove(&reference);
        result
    }

    fn nominal_type(
        &self,
        snapshot: &ParseResult,
        object: &SnapshotObject,
        depth: usize,
        visiting: &mut BTreeSet<i32>,
    ) -> Option<RecoveredType> {
        let flags = match snapshot.scalars_of(object).first()? {
            SnapshotScalar::Unsigned(value) => u64::try_from(*value).ok()?,
            _ => return None,
        };
        let class_id = i32::try_from(flags >> TYPE_CLASS_ID_SHIFT).ok()?;
        let class_reference = self.class_references.get(&class_id).copied()?;
        let raw_name = self.name(class_reference)?;
        let name = self.restore_name(raw_name);
        if name.is_empty() {
            return None;
        }
        let arguments_reference = snapshot.references_of(object).get(2).copied();
        let arguments = arguments_reference
            .map(|reference| self.type_argument_references(reference))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|reference| self.recover_type_inner(reference, depth + 1, visiting))
            .map(|value| value.display_name)
            .collect::<Vec<_>>();
        let mut display_name = if arguments.is_empty() {
            name
        } else {
            format!("{name}<{}>", arguments.join(", "))
        };
        if flags & 1 == 0 && !matches!(display_name.as_str(), "dynamic" | "void" | "Null") {
            display_name.push('?');
        }
        Some(RecoveredType {
            snapshot_reference: object.reference,
            display_name,
            library_uri: self.library_uri(class_reference),
        })
    }

    fn function_type(
        &self,
        reference: i32,
        depth: usize,
        visiting: &mut BTreeSet<i32>,
    ) -> Option<RecoveredType> {
        let signature = self
            .isolate
            .function_types
            .get(&reference)
            .or_else(|| self.vm.function_types.get(&reference))?;
        let return_type = signature
            .result_type_ref
            .and_then(|reference| self.recover_type_inner(reference, depth + 1, visiting))
            .map_or_else(|| "dynamic".to_owned(), |value| value.display_name);
        let parameter_types = signature
            .parameter_types_ref
            .map(|reference| self.array_elements(reference))
            .unwrap_or_default();
        let named_names = signature
            .named_parameter_names_ref
            .map(|reference| self.array_elements(reference))
            .unwrap_or_default();
        let mut fixed = Vec::new();
        for position in 0..signature.fixed {
            fixed.push(
                parameter_types
                    .get(signature.implicit + position)
                    .and_then(|reference| self.recover_type_inner(*reference, depth + 1, visiting))
                    .map_or_else(|| "dynamic".to_owned(), |value| value.display_name),
            );
        }
        let mut optional = Vec::new();
        for position in 0..signature.optional {
            let type_name = parameter_types
                .get(signature.implicit + signature.fixed + position)
                .and_then(|reference| self.recover_type_inner(*reference, depth + 1, visiting))
                .map_or_else(|| "dynamic".to_owned(), |value| value.display_name);
            if signature.optional_are_named {
                let name = named_names
                    .get(position)
                    .and_then(|reference| self.string(*reference))
                    .map(|name| self.restore_name(name))
                    .unwrap_or_else(|| format!("namedArg{position}"));
                optional.push(format!("{type_name} {name}"));
            } else {
                optional.push(type_name);
            }
        }
        let mut parameters = fixed;
        if !optional.is_empty() {
            parameters.push(if signature.optional_are_named {
                format!("{{{}}}", optional.join(", "))
            } else {
                format!("[{}]", optional.join(", "))
            });
        }
        let type_parameters = signature
            .type_parameters_ref
            .map(|reference| self.type_parameters_inner(reference, depth + 1, visiting))
            .unwrap_or_default();
        let generics = render_type_parameters(&type_parameters);
        let nullable = if signature.flags & 1 == 0 { "?" } else { "" };
        Some(RecoveredType {
            snapshot_reference: reference,
            display_name: format!(
                "{return_type} Function{generics}({}){nullable}",
                parameters.join(", ")
            ),
            library_uri: Some("dart:core".to_owned()),
        })
    }

    fn type_parameter(
        &self,
        snapshot: &ParseResult,
        object: &SnapshotObject,
        _depth: usize,
        _visiting: &mut BTreeSet<i32>,
    ) -> Option<RecoveredType> {
        let references = snapshot.references_of(object);
        let scalars = snapshot.scalars_of(object);
        let base = match scalars.first()? {
            SnapshotScalar::Tagged32(value) => usize::try_from(*value).ok()?,
            SnapshotScalar::Uint16(value) => usize::from(*value),
            _ => return None,
        };
        let index = match scalars.get(1)? {
            SnapshotScalar::Tagged32(value) => usize::try_from(*value).ok()?,
            SnapshotScalar::Uint16(value) => usize::from(*value),
            _ => return None,
        };
        let local_index = index.saturating_sub(base);
        let flags = match scalars.get(2)? {
            SnapshotScalar::Byte(value) => *value,
            _ => return None,
        };
        let owner = references.get(2).copied()?;
        let (parameters, library_uri) = if self
            .object(owner)
            .is_some_and(|(_, object)| object.cid == self.cids.function_type)
        {
            let signature = self
                .isolate
                .function_types
                .get(&owner)
                .or_else(|| self.vm.function_types.get(&owner));
            (
                signature
                    .and_then(|signature| signature.type_parameters_ref)
                    .map(|reference| self.type_parameter_names(reference))
                    .unwrap_or_default(),
                None,
            )
        } else if let Some(class_id) = self
            .integer(owner)
            .and_then(|value| i32::try_from(value).ok())
        {
            let class_reference = self.class_references.get(&class_id).copied();
            (
                class_reference
                    .and_then(|reference| self.class_type_parameter_names(reference))
                    .unwrap_or_default(),
                class_reference.and_then(|reference| self.library_uri(reference)),
            )
        } else {
            (Vec::new(), None)
        };
        let mut display_name = parameters
            .get(local_index)
            .cloned()
            .unwrap_or_else(|| format!("T{local_index}"));
        if flags & 1 == 0 {
            display_name.push('?');
        }
        Some(RecoveredType {
            snapshot_reference: object.reference,
            display_name,
            library_uri,
        })
    }

    fn record_type(
        &self,
        snapshot: &ParseResult,
        object: &SnapshotObject,
        depth: usize,
        visiting: &mut BTreeSet<i32>,
    ) -> Option<RecoveredType> {
        let references = snapshot.references_of(object);
        let shape = references
            .get(2)
            .and_then(|reference| self.integer(*reference));
        let field_types = references
            .get(3)
            .copied()
            .map(|reference| self.array_elements(reference))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|reference| self.recover_type_inner(reference, depth + 1, visiting))
            .map(|value| value.display_name)
            .collect::<Vec<_>>();
        // The shape packs the named-field count in its high bits.
        let named_field_count = shape.map_or(0, |value| (value >> 16) as usize);
        // Named record fields precede positional ones; their names live in a
        // dedicated Array-of-String reference. Try the leading references so
        // minor layout shifts still resolve.
        let mut field_names: Vec<String> = Vec::new();
        if named_field_count > 0 {
            for candidate in references.iter().take(2) {
                let elements = self.array_elements(*candidate);
                if elements.len() != named_field_count {
                    continue;
                }
                let names = elements
                    .iter()
                    .filter_map(|element| self.string_value(*element))
                    .collect::<Vec<_>>();
                if names.len() == named_field_count {
                    field_names = names;
                    break;
                }
            }
        }
        let mut display_name = if !field_types.is_empty() {
            let named = field_names
                .iter()
                .zip(field_types.iter())
                .map(|(name, type_name)| format!("{name}: {type_name}"))
                .collect::<Vec<_>>();
            let positional = field_types
                .iter()
                .skip(named.len())
                .cloned()
                .collect::<Vec<_>>();
            let mut parts = named;
            parts.extend(positional);
            let comma = if parts.len() == 1 && !field_names.is_empty() {
                ","
            } else {
                ""
            };
            format!("({}{comma})", parts.join(", "))
        } else {
            "Record".to_owned()
        };
        let flags = snapshot
            .scalars_of(object)
            .last()
            .and_then(|value| match value {
                SnapshotScalar::Byte(value) => Some(*value),
                _ => None,
            });
        if flags.is_some_and(|flags| flags & 1 == 0) {
            display_name.push('?');
        }
        Some(RecoveredType {
            snapshot_reference: object.reference,
            display_name,
            library_uri: Some("dart:core".to_owned()),
        })
    }

    fn future_or_type(
        &self,
        snapshot: &ParseResult,
        object: &SnapshotObject,
        depth: usize,
        visiting: &mut BTreeSet<i32>,
    ) -> Option<RecoveredType> {
        let argument = snapshot
            .references_of(object)
            .last()
            .copied()
            .into_iter()
            .flat_map(|reference| self.type_argument_references(reference))
            .next()
            .and_then(|reference| self.recover_type_inner(reference, depth + 1, visiting))
            .map_or_else(|| "dynamic".to_owned(), |value| value.display_name);
        Some(RecoveredType {
            snapshot_reference: object.reference,
            display_name: format!("FutureOr<{argument}>"),
            library_uri: Some("dart:async".to_owned()),
        })
    }

    fn type_parameters(&self, reference: i32) -> Vec<RecoveredTypeParameter> {
        let mut visiting = BTreeSet::new();
        self.type_parameters_inner(reference, 0, &mut visiting)
    }

    fn type_parameters_inner(
        &self,
        reference: i32,
        depth: usize,
        visiting: &mut BTreeSet<i32>,
    ) -> Vec<RecoveredTypeParameter> {
        if depth >= MAX_TYPE_DEPTH || !visiting.insert(reference) {
            return Vec::new();
        }
        let parameters = self
            .object(reference)
            .filter(|(_, object)| object.cid == self.cids.type_parameters)
            .map(|(snapshot, object)| {
                let references = snapshot.references_of(object);
                let names = references
                    .first()
                    .copied()
                    .map(|reference| self.array_elements(reference))
                    .unwrap_or_default();
                let bounds = references
                    .get(2)
                    .copied()
                    .map(|reference| self.type_argument_references(reference))
                    .unwrap_or_default();
                names
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, reference)| {
                        let name = self.string(reference)?;
                        Some(RecoveredTypeParameter {
                            name: self.restore_name(name),
                            bound: bounds.get(index).copied().and_then(|reference| {
                                self.recover_type_inner(reference, depth + 1, visiting)
                            }),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        visiting.remove(&reference);
        parameters
    }

    fn type_parameter_names(&self, reference: i32) -> Vec<String> {
        let Some((snapshot, object)) = self.object(reference) else {
            return Vec::new();
        };
        if object.cid != self.cids.type_parameters {
            return Vec::new();
        }
        snapshot
            .references_of(object)
            .first()
            .copied()
            .map(|reference| self.array_elements(reference))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|reference| self.string(reference))
            .map(|name| self.restore_name(name))
            .collect()
    }

    fn class_type_parameter_names(&self, reference: i32) -> Option<Vec<String>> {
        let (snapshot, object) = self.object(reference)?;
        let type_parameters_reference = snapshot.references_of(object).get(8).copied()?;
        Some(self.type_parameter_names(type_parameters_reference))
    }

    fn type_argument_references(&self, reference: i32) -> Vec<i32> {
        if reference == EMPTY_TYPE_ARGUMENTS_REFERENCE || reference == NULL_REFERENCE {
            return Vec::new();
        }
        let Some((snapshot, object)) = self.object(reference) else {
            return Vec::new();
        };
        if object.kind != SnapshotObjectKind::TypeArguments {
            return Vec::new();
        }
        let count = snapshot
            .scalars_of(object)
            .first()
            .and_then(|value| match value {
                SnapshotScalar::Unsigned(value) => usize::try_from(*value).ok(),
                _ => None,
            });
        let references = snapshot.references_of(object);
        references
            .get(1..1 + count.unwrap_or_else(|| references.len().saturating_sub(1)))
            .unwrap_or_default()
            .to_vec()
    }

    pub(crate) fn array_elements(&self, reference: i32) -> Vec<i32> {
        if reference == EMPTY_ARRAY_REFERENCE || reference == NULL_REFERENCE {
            return Vec::new();
        }
        let Some((snapshot, object)) = self.object(reference) else {
            return Vec::new();
        };
        if object.kind != SnapshotObjectKind::Array {
            return Vec::new();
        }
        let count = snapshot
            .scalars_of(object)
            .first()
            .and_then(|value| match value {
                SnapshotScalar::Unsigned(value) => usize::try_from(*value).ok(),
                _ => None,
            });
        let references = snapshot.references_of(object);
        references
            .get(1..1 + count.unwrap_or_else(|| references.len().saturating_sub(1)))
            .unwrap_or_default()
            .to_vec()
    }

    fn integer(&self, reference: i32) -> Option<i64> {
        let (snapshot, object) = self.object(reference)?;
        (object.kind == SnapshotObjectKind::Integer)
            .then(|| snapshot.scalars_of(object).first())
            .flatten()
            .and_then(|value| match value {
                SnapshotScalar::Tagged64(value) => Some(*value),
                _ => None,
            })
    }

    fn object(&self, reference: i32) -> Option<(&ParseResult, &SnapshotObject)> {
        if let Some(object) = self.isolate.object(reference) {
            Some((self.isolate, object))
        } else {
            self.vm.object(reference).map(|object| (self.vm, object))
        }
    }

    fn name(&self, reference: i32) -> Option<&str> {
        let object = self
            .isolate
            .named
            .get(&reference)
            .or_else(|| self.vm.named.get(&reference))?;
        self.string(object.name_ref)
    }

    /// Reads a String object by snapshot reference, independent of the
    /// named-object tables.
    fn string_value(&self, reference: i32) -> Option<String> {
        let (snapshot, object) = self.object(reference)?;
        if object.kind != SnapshotObjectKind::String {
            return None;
        }
        Some(crate::snapshot::cluster::types::decode_one_byte_string(
            snapshot.bytes_of(object),
        ))
    }

    fn string(&self, reference: i32) -> Option<&str> {
        self.isolate
            .strings
            .get(&reference)
            .or_else(|| self.vm.strings.get(&reference))
            .map(String::as_str)
    }

    fn library_uri(&self, reference: i32) -> Option<String> {
        let mut current = reference;
        for _ in 0..12 {
            let object = self
                .isolate
                .named
                .get(&current)
                .or_else(|| self.vm.named.get(&current))?;
            if let Some(script_reference) = object.source_uri_ref {
                let script = self
                    .isolate
                    .named
                    .get(&script_reference)
                    .or_else(|| self.vm.named.get(&script_reference))?;
                if let Some(uri) = self.string(script.name_ref) {
                    return Some(self.restore_library_uri(uri));
                }
            }
            if object.cid == self.cids.library {
                return self
                    .string(object.name_ref)
                    .map(|uri| self.restore_library_uri(uri));
            }
            current = object.owner_ref;
            if current < 0 {
                return None;
            }
        }
        None
    }

    fn restore_name(&self, value: &str) -> String {
        let readable = crate::analysis::readable_snapshot_name(value);
        self.obfuscation_map
            .map_or(readable.clone(), |map| map.restore(&readable))
    }

    fn restore_library_uri(&self, value: &str) -> String {
        self.obfuscation_map
            .map_or_else(|| value.to_owned(), |map| map.restore(value))
    }
}

fn contextualize_class_types(
    type_parameters: &mut [RecoveredTypeParameter],
    super_type: &mut Option<RecoveredType>,
    interfaces: &mut [RecoveredType],
) {
    let names = type_parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .collect::<Vec<_>>();
    if names.is_empty() {
        return;
    }
    for parameter in type_parameters {
        if let Some(bound) = &mut parameter.bound {
            contextualize_type(bound, &names);
        }
    }
    if let Some(super_type) = super_type {
        contextualize_type(super_type, &names);
    }
    for interface in interfaces {
        contextualize_type(interface, &names);
    }
}

fn contextualize_type(value: &mut RecoveredType, names: &[String]) {
    for (index, name) in names.iter().enumerate() {
        value.display_name = replace_type_parameter(&value.display_name, index, name);
    }
}

fn replace_type_parameter(value: &str, index: usize, name: &str) -> String {
    let needle = format!("T{index}");
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(position) = remaining.find(&needle) {
        let before = remaining[..position].chars().next_back();
        let after = remaining[position + needle.len()..].chars().next();
        let boundary = |character: Option<char>| {
            character.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
        };
        output.push_str(&remaining[..position]);
        if boundary(before) && boundary(after) {
            output.push_str(name);
        } else {
            output.push_str(&needle);
        }
        remaining = &remaining[position + needle.len()..];
    }
    output.push_str(remaining);
    output
}

fn bit(value: u32, position: u32) -> bool {
    value & (1 << position) != 0
}

fn render_type_parameters(parameters: &[RecoveredTypeParameter]) -> String {
    if parameters.is_empty() {
        return String::new();
    }
    format!(
        "<{}>",
        parameters
            .iter()
            .map(|parameter| {
                parameter.bound.as_ref().map_or_else(
                    || parameter.name.clone(),
                    |bound| format!("{} extends {}", parameter.name, bound.display_name),
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::{bit, render_type_parameters, replace_type_parameter};
    use crate::model::{Abi, RecoveredType, RecoveredTypeParameter};
    use crate::snapshot::cluster::cid::test_cids;
    use crate::snapshot::cluster::types::{
        ClusterHeader, ParseResult, SnapshotObjectKind, SnapshotObjectPayload,
    };

    #[test]
    fn derives_typed_unboxed_instance_slots_from_instance_bitmaps() {
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        // Two consecutive unboxed words (schema-5 bitmap convention shared
        // with fill_skip: bit w = word w from the object start, headers
        // included, so the first field word is bit 2).
        snapshot.insert_object(
            60,
            cids.class,
            false,
            SnapshotObjectKind::Class,
            SnapshotObjectPayload {
                references: Vec::new(),
                scalars: vec![
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(44),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                ],
                bytes: Vec::new(),
            },
        );
        snapshot.instance_bitmaps.insert(44, 0b1100);

        let types = super::TypeRecovery::new(&snapshot, &snapshot, &cids, Abi::Arm64V8a, None);
        let metadata = types.class_metadata(60).expect("class metadata");
        let slots = &metadata.instance_slots;
        assert_eq!(slots.len(), 2);
        // ARM64 compressed layout: 8-byte header, 4-byte field words.
        assert!(!slots[0].is_reference);
        assert_eq!(slots[0].offset, 8);
        assert_eq!(slots[0].slot_type, "unboxed_field");
        assert_eq!(slots[1].offset, 12);
        assert!(!slots[1].is_reference);
    }

    #[test]
    fn bounds_slot_walk_by_cluster_field_count_and_marks_reference_slots() {
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        snapshot.insert_object(
            62,
            cids.class,
            false,
            SnapshotObjectKind::Class,
            SnapshotObjectPayload {
                references: Vec::new(),
                scalars: vec![
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(46),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                ],
                bytes: Vec::new(),
            },
        );
        // Bitmap bit 6 is padding noise beyond the class's true layout
        // (`next_field_words` says header 2 + 3 field words); bits 2/3 are
        // clear reference slots the unboxed bitmap never records, bit 4 is
        // a genuine unboxed word.
        snapshot.instance_bitmaps.insert(46, 0b101_0000);
        snapshot
            .clusters
            .push(crate::snapshot::cluster::types::Cluster {
                cid: 46,
                canonical: false,
                count: 1,
                start_ref: 0,
                next_field_words: 5,
                main_count: 0,
                lengths: Vec::new(),
                predefined_cids: Vec::new(),
                discarded: Vec::new(),
                allocation_values: Vec::new(),
            });

        let types = super::TypeRecovery::new(&snapshot, &snapshot, &cids, Abi::Arm64V8a, None);
        let metadata = types.class_metadata(62).expect("class metadata");
        let slots = &metadata.instance_slots;
        assert_eq!(
            slots.len(),
            3,
            "bits beyond next_field_words must not appear"
        );
        assert_eq!(slots[0].slot_type, "reference");
        assert!(slots[0].is_reference);
        assert!(slots[1].is_reference);
        assert_eq!(slots[2].slot_type, "unboxed_field");
        assert!(!slots[2].is_reference);
        assert_eq!(slots[2].offset, 16);
    }

    #[test]
    fn recovers_enum_member_names_from_canonical_const_instances() {
        use crate::snapshot::cluster::types::SnapshotScalar as Scalar;
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        // Class 900 is marked enum (state bit 9); class 901 is not.
        let class_scalars = |class_id: u32, is_enum: bool| SnapshotObjectPayload {
            references: Vec::new(),
            scalars: vec![
                Scalar::Tagged32(class_id),
                Scalar::Tagged32(0),
                Scalar::Tagged32(0),
                Scalar::Tagged32(0),
                Scalar::Tagged32(0),
                Scalar::Tagged32(0),
                Scalar::Tagged32(if is_enum { 1 << 9 } else { 0 }),
            ],
            bytes: Vec::new(),
        };
        snapshot.insert_object(
            70,
            cids.class,
            false,
            SnapshotObjectKind::Class,
            class_scalars(900, true),
        );
        snapshot.insert_object(
            71,
            cids.class,
            false,
            SnapshotObjectKind::Class,
            class_scalars(901, false),
        );

        // `_Enum` lays out index then _name; the index slot is unboxed, so the
        // first reference slot is the name string.
        snapshot.strings.insert(200, "light".to_owned());
        snapshot.strings.insert(201, "dark".to_owned());
        snapshot.strings.insert(202, "notAnEnumMember".to_owned());
        let instance = |name_ref: i32| SnapshotObjectPayload {
            references: vec![name_ref],
            scalars: vec![Scalar::Tagged32(0)],
            bytes: Vec::new(),
        };
        snapshot.insert_object(300, 900, true, SnapshotObjectKind::Instance, instance(200));
        snapshot.insert_object(301, 900, true, SnapshotObjectKind::Instance, instance(201));
        // Non-enum class instance must never contribute members.
        snapshot.insert_object(302, 901, true, SnapshotObjectKind::Instance, instance(202));
        // Non-canonical instance of the enum class is not a declared constant.
        snapshot.insert_object(303, 900, false, SnapshotObjectKind::Instance, instance(202));

        let types = super::TypeRecovery::new(&snapshot, &snapshot, &cids, Abi::ArmeabiV7a, None);
        let metadata = types.class_metadata(70).expect("enum class metadata");
        assert!(metadata.is_enum);
        assert_eq!(metadata.enum_values, vec!["light", "dark"]);

        let plain = types.class_metadata(71).expect("plain class metadata");
        assert!(!plain.is_enum);
        assert!(
            plain.enum_values.is_empty(),
            "non-enum classes never receive recovered constants"
        );
    }

    #[test]
    fn leaves_instance_slots_empty_without_bitmap_evidence() {
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        snapshot.insert_object(
            61,
            cids.class,
            false,
            SnapshotObjectKind::Class,
            SnapshotObjectPayload {
                references: Vec::new(),
                scalars: vec![
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(45),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                    crate::snapshot::cluster::types::SnapshotScalar::Tagged32(0),
                ],
                bytes: Vec::new(),
            },
        );
        let types = super::TypeRecovery::new(&snapshot, &snapshot, &cids, Abi::Arm64V8a, None);
        let metadata = types.class_metadata(61).expect("class metadata");
        assert!(metadata.instance_slots.is_empty());
    }

    #[test]
    fn decodes_stable_class_and_field_flag_positions() {
        let flags = (1 << 6) | (1 << 9) | (1 << 14) | (1 << 18);
        assert!(bit(flags, 6));
        assert!(bit(flags, 9));
        assert!(bit(flags, 14));
        assert!(bit(flags, 18));
        assert!(!bit(flags, 17));
    }

    #[test]
    fn renders_generic_bounds_from_snapshot_types() {
        let parameters = vec![RecoveredTypeParameter {
            name: "T".to_owned(),
            bound: Some(RecoveredType {
                snapshot_reference: 42,
                display_name: "Object".to_owned(),
                library_uri: Some("dart:core".to_owned()),
            }),
        }];
        assert_eq!(render_type_parameters(&parameters), "<T extends Object>");
    }

    #[test]
    fn replaces_only_contextual_type_parameter_tokens() {
        assert_eq!(
            replace_type_parameter("Map<T0, List<T10>>", 0, "Element"),
            "Map<Element, List<T10>>"
        );
        assert_eq!(
            replace_type_parameter("SomeT0Name<T0?>", 0, "Value"),
            "SomeT0Name<Value?>"
        );
    }
}
