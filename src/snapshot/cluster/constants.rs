//! Typed constant values reachable from the object pool.
//!
//! Pool labels are display strings; this graph keeps the values themselves
//! so consumers can render a canonical object's actual contents (instance
//! slots, list elements, map entries, record fields) without guessing a
//! constructor. Every node is keyed by its snapshot reference, which is one
//! namespace across the VM and isolate snapshots, and edges are references,
//! so shared and cyclic structures stay intact.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::model::{ConstantSlot, ConstantSlotValue, SnapshotConstant};

use super::cid::Cids;
use super::instructions::Names;
use super::type_recovery::TypeRecovery;
use super::types::{ParseResult, SnapshotObject, SnapshotObjectKind, SnapshotScalar};

/// Upper bound on graph nodes; the pool of a large app references tens of
/// thousands of objects and only the constants reachable from it matter.
const MAX_NODES: usize = 50_000;

pub(crate) struct ConstantGraphBuilder<'a> {
    pub isolate: &'a ParseResult,
    pub vm: &'a ParseResult,
    pub names: &'a Names<'a>,
    pub types: &'a TypeRecovery<'a>,
    pub cids: &'a Cids,
    /// Serialized instance header words (`Instance::NextFieldOffset`).
    pub instance_header_words: usize,
    /// 32-bit chunks per unboxed bitmap word in the serialized stream.
    pub unboxed_word_u32_chunks: usize,
}

impl ConstantGraphBuilder<'_> {
    pub(crate) fn build(
        &self,
        roots: impl IntoIterator<Item = i32>,
    ) -> BTreeMap<i32, SnapshotConstant> {
        let class_by_cid = self.class_references();
        let mut graph = BTreeMap::new();
        let mut pending = roots.into_iter().collect::<VecDeque<_>>();
        let mut seen = BTreeSet::new();
        while let Some(reference) = pending.pop_front() {
            if graph.len() >= MAX_NODES || !seen.insert(reference) {
                continue;
            }
            let Some(constant) = self.constant(reference, &class_by_cid) else {
                continue;
            };
            pending.extend(children(&constant));
            graph.insert(reference, constant);
        }
        graph
    }

    fn object(&self, reference: i32) -> Option<(&ParseResult, &SnapshotObject)> {
        self.isolate
            .object(reference)
            .map(|object| (self.isolate, object))
            .or_else(|| self.vm.object(reference).map(|object| (self.vm, object)))
    }

    /// Class object reference per class id, from either snapshot.
    fn class_references(&self) -> BTreeMap<i32, i32> {
        let mut classes = BTreeMap::new();
        for snapshot in [self.vm, self.isolate] {
            for object in &snapshot.objects {
                if object.cid != self.cids.class {
                    continue;
                }
                if let Some(SnapshotScalar::Tagged32(class_id)) =
                    snapshot.scalars_of(object).first()
                {
                    classes.insert(*class_id as i32, object.reference);
                }
            }
        }
        classes
    }

    fn constant(
        &self,
        reference: i32,
        class_by_cid: &BTreeMap<i32, i32>,
    ) -> Option<SnapshotConstant> {
        let base = self.cids.base;
        if reference == base.null {
            return Some(SnapshotConstant::Null);
        }
        if reference == base.true_value || reference == base.false_value {
            return Some(SnapshotConstant::Bool {
                value: reference == base.true_value,
            });
        }
        if let Some(value) = self.types.resolved_string(reference) {
            return Some(SnapshotConstant::String {
                value: value.to_owned(),
            });
        }
        let (snapshot, object) = self.object(reference)?;
        let references = snapshot.references_of(object);
        let scalars = snapshot.scalars_of(object);
        let cids = self.cids;
        Some(match object.kind {
            SnapshotObjectKind::Integer => SnapshotConstant::Int {
                value: match scalars.first()? {
                    SnapshotScalar::Tagged64(value) => *value,
                    _ => return None,
                },
            },
            SnapshotObjectKind::Double => SnapshotConstant::Double {
                value: match scalars.first()? {
                    SnapshotScalar::Tagged64(bits) => f64::from_bits(*bits as u64),
                    _ => return None,
                },
            },
            SnapshotObjectKind::Array => SnapshotConstant::List {
                elements: references.get(1..).unwrap_or_default().to_vec(),
                type_arguments: references
                    .first()
                    .and_then(|arguments| self.type_arguments_display(*arguments)),
                immutable: object.cid == cids.immutable_array,
            },
            SnapshotObjectKind::Record => {
                let shape = match scalars.first() {
                    Some(SnapshotScalar::Unsigned(shape)) => *shape,
                    _ => 0,
                };
                SnapshotConstant::Record {
                    fields: references.to_vec(),
                    // `RecordShape`: field count in the low 16 bits, the
                    // field-names index above; names live in a VM table that
                    // is not serialized, so only the named count survives.
                    field_names_index: usize::try_from(shape >> 16).unwrap_or_default(),
                }
            }
            SnapshotObjectKind::Instance => self.instance(snapshot, object, class_by_cid)?,
            SnapshotObjectKind::Standard
                if object.cid == cids.const_map || object.cid == cids.map =>
            {
                // [type_arguments, hash_mask, data, used_data, deleted_keys]
                let data = self.array_elements(*references.get(2)?)?;
                let used = self
                    .integer(*references.get(3)?)
                    .unwrap_or(data.len() as i64);
                let used = usize::try_from(used).unwrap_or_default().min(data.len());
                SnapshotConstant::Map {
                    entries: data[..used]
                        .chunks_exact(2)
                        .map(|pair| (pair[0], pair[1]))
                        .collect(),
                    type_arguments: references
                        .first()
                        .and_then(|arguments| self.type_arguments_display(*arguments)),
                }
            }
            SnapshotObjectKind::Standard
                if object.cid == cids.const_set || object.cid == cids.set =>
            {
                let data = self.array_elements(*references.get(2)?)?;
                let used = self
                    .integer(*references.get(3)?)
                    .unwrap_or(data.len() as i64);
                let used = usize::try_from(used).unwrap_or_default().min(data.len());
                SnapshotConstant::Set {
                    elements: data[..used].to_vec(),
                    type_arguments: references
                        .first()
                        .and_then(|arguments| self.type_arguments_display(*arguments)),
                }
            }
            SnapshotObjectKind::Standard
                if object.cid == cids.type_
                    || object.cid == cids.function_type
                    || object.cid == cids.record_type =>
            {
                SnapshotConstant::Type {
                    display: self.types.recover_type(reference)?.display_name,
                }
            }
            SnapshotObjectKind::Standard if object.cid == cids.closure => {
                // [instantiator TAs, function TAs, delayed TAs, function, context, ...]
                let function = *references.get(3)?;
                SnapshotConstant::Closure {
                    function: self.qualified_name(function),
                }
            }
            _ => return None,
        })
    }

    fn instance(
        &self,
        snapshot: &ParseResult,
        object: &SnapshotObject,
        class_by_cid: &BTreeMap<i32, i32>,
    ) -> Option<SnapshotConstant> {
        let class_reference = class_by_cid.get(&object.cid).copied();
        let class_name = class_reference
            .map(|reference| crate::analysis::readable_snapshot_name(&self.names.name(reference)))
            .filter(|name| !name.is_empty())?;
        let library_uri = class_reference.and_then(|reference| self.names.library_uri(reference));
        let layout = self.cids.layout;
        let bitmap = snapshot
            .instance_bitmaps
            .get(&object.cid)
            .copied()
            .unwrap_or_default();
        let field_words = snapshot
            .clusters
            .iter()
            .find(|cluster| cluster.cid == object.cid && cluster.next_field_words > 0)
            .map(|cluster| usize::try_from(cluster.next_field_words).unwrap_or_default())?;
        let references = snapshot.references_of(object);
        let scalars = snapshot.scalars_of(object);
        let mut next_reference = references.iter();
        let mut next_scalar = scalars.iter();
        let mut slots = Vec::new();
        let unboxed_words = usize::try_from(8 / layout.compressed_word)
            .unwrap_or(1)
            .max(1);
        let mut word = self.instance_header_words;
        while word < field_words {
            let offset = layout.header_bytes
                + (word - self.instance_header_words) as i64 * layout.compressed_word;
            if bitmap & (1u64 << word.min(63)) != 0 {
                // Mirror `fill_skip::instance`: each unboxed bitmap word was
                // written as whole 32-bit chunks; the low chunk of each word
                // carries its bits.
                let width = if (1..unboxed_words).all(|extra| {
                    word + extra < field_words && bitmap & (1u64 << (word + extra).min(63)) != 0
                }) {
                    unboxed_words
                } else {
                    1
                };
                let mut bits = 0u64;
                for part in 0..width {
                    let mut chunk = 0u64;
                    for index in 0..self.unboxed_word_u32_chunks {
                        let value = match next_scalar.next() {
                            Some(SnapshotScalar::Tagged32(value)) => u64::from(*value),
                            _ => 0,
                        };
                        chunk |= value << (32 * index);
                    }
                    // A compressed word carries 32 bits; a native word all.
                    let word_bits = if layout.compressed_word == 4 {
                        chunk & 0xffff_ffff
                    } else {
                        chunk
                    };
                    bits |= word_bits << (32 * part);
                }
                slots.push(ConstantSlot {
                    offset,
                    value: ConstantSlotValue::Unboxed { bits },
                    field: None,
                    field_type: None,
                });
                word += width;
            } else {
                let reference = *next_reference.next()?;
                slots.push(ConstantSlot {
                    offset,
                    value: ConstantSlotValue::Reference { reference },
                    field: None,
                    field_type: None,
                });
                word += 1;
            }
        }
        let is_enum = class_reference
            .and_then(|reference| self.types.class_metadata(reference))
            .is_some_and(|metadata| metadata.is_enum);
        // `_Enum` declares `index` then `_name`; the first String slot is the
        // constant's name.
        let enum_name = is_enum
            .then(|| {
                slots.iter().find_map(|slot| match slot.value {
                    ConstantSlotValue::Reference { reference } => {
                        self.types.resolved_string(reference).map(str::to_owned)
                    }
                    ConstantSlotValue::Unboxed { .. } => None,
                })
            })
            .flatten();
        Some(SnapshotConstant::Instance {
            class_name,
            library_uri,
            enum_name,
            canonical: object.canonical,
            slots,
        })
    }

    fn array_elements(&self, reference: i32) -> Option<Vec<i32>> {
        let (snapshot, object) = self.object(reference)?;
        (object.kind == SnapshotObjectKind::Array).then(|| {
            snapshot
                .references_of(object)
                .get(1..)
                .unwrap_or_default()
                .to_vec()
        })
    }

    fn integer(&self, reference: i32) -> Option<i64> {
        let (snapshot, object) = self.object(reference)?;
        match (object.kind, snapshot.scalars_of(object).first()) {
            (SnapshotObjectKind::Integer, Some(SnapshotScalar::Tagged64(value))) => Some(*value),
            _ => None,
        }
    }

    fn type_arguments_display(&self, reference: i32) -> Option<String> {
        if reference == self.cids.base.null || reference == self.cids.base.empty_type_arguments {
            return None;
        }
        let (snapshot, object) = self.object(reference)?;
        if object.kind != SnapshotObjectKind::TypeArguments {
            return None;
        }
        // [instantiations cache, types...]
        let names = snapshot
            .references_of(object)
            .get(1..)
            .unwrap_or_default()
            .iter()
            .map(|argument| {
                self.types
                    .recover_type(*argument)
                    .map_or_else(|| "dynamic".to_owned(), |type_| type_.display_name)
            })
            .collect::<Vec<_>>();
        (!names.is_empty()).then(|| names.join(", "))
    }

    fn qualified_name(&self, reference: i32) -> Option<String> {
        let name = self.names.name(reference);
        if name.is_empty() {
            return None;
        }
        let owner = self.names.owner_name(reference);
        Some(if owner.is_empty() {
            name
        } else {
            format!("{owner}.{name}")
        })
    }
}

fn children(constant: &SnapshotConstant) -> Vec<i32> {
    match constant {
        SnapshotConstant::List { elements, .. } | SnapshotConstant::Set { elements, .. } => {
            elements.clone()
        }
        SnapshotConstant::Map { entries, .. } => entries
            .iter()
            .flat_map(|(key, value)| [*key, *value])
            .collect(),
        SnapshotConstant::Record { fields, .. } => fields.clone(),
        SnapshotConstant::Instance { slots, .. } => slots
            .iter()
            .filter_map(|slot| match slot.value {
                ConstantSlotValue::Reference { reference } => Some(reference),
                ConstantSlotValue::Unboxed { .. } => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}
