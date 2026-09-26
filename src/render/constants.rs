//! Renders snapshot constants from the decoded constant graph.
//!
//! Pool labels such as `snapshotInstance(Product@412)` and `snapshotRef(412)`
//! name a snapshot object; when the constant graph decoded that object, the
//! renderer prints its actual contents instead of inventing a constructor
//! call. Instances render through `aot.constObject` with their slots keyed by
//! the field names the class declarations prove, because the snapshot keeps
//! field values, not the constructor arguments that produced them.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::model::{
    ConstantSlotValue, RecoveredDeclarationKind, RecoveredProgram, SnapshotConstant,
};

use super::dart::{clean_symbol, dart_identifier, dart_string};

/// Nesting depth past which constants render as `aot.snapshotRef(N)`.
const MAX_DEPTH: usize = 4;
/// Elements, entries or slots printed before the rest is elided.
const MAX_ITEMS: usize = 24;

#[derive(Default)]
pub(super) struct ConstantIndex {
    constants: BTreeMap<i32, SnapshotConstant>,
    /// `(class, byte offset) -> field name` for instance fields.
    fields: BTreeMap<(String, i64), String>,
    superclasses: BTreeMap<String, String>,
    /// Initial field-table values by field id, from the snapshot roots.
    initial_fields: Vec<i32>,
    shared_initial_fields: Vec<i32>,
}

impl ConstantIndex {
    pub(super) fn new(program: &RecoveredProgram) -> Self {
        let mut fields = BTreeMap::new();
        let mut superclasses = BTreeMap::new();
        for declaration in &program.declarations {
            match declaration.kind {
                RecoveredDeclarationKind::Field => {
                    let (Some(owner), Some(offset)) = (
                        declaration.owner.as_ref(),
                        declaration
                            .field_metadata
                            .as_ref()
                            .filter(|metadata| !metadata.is_static)
                            .and_then(|metadata| metadata.instance_field_offset),
                    ) else {
                        continue;
                    };
                    fields
                        .entry((owner.clone(), offset))
                        .or_insert_with(|| declaration.name.clone());
                }
                RecoveredDeclarationKind::Class => {
                    if let Some(parent) = declaration
                        .class_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.super_type.as_ref())
                    {
                        let parent = parent
                            .display_name
                            .split('<')
                            .next()
                            .unwrap_or_default()
                            .to_owned();
                        superclasses.insert(declaration.name.clone(), parent);
                    }
                }
                _ => {}
            }
        }
        Self {
            constants: program.constants.clone(),
            fields,
            superclasses,
            initial_fields: program
                .snapshot_roots
                .as_ref()
                .map(|roots| roots.initial_field_references.clone())
                .unwrap_or_default(),
            shared_initial_fields: program
                .snapshot_roots
                .as_ref()
                .map(|roots| roots.shared_initial_field_references.clone())
                .unwrap_or_default(),
        }
    }

    fn field_name(&self, class_name: &str, offset: i64) -> Option<&str> {
        let mut current = class_name.to_owned();
        for _ in 0..64 {
            if let Some(name) = self.fields.get(&(current.clone(), offset)) {
                return Some(name);
            }
            current = self.superclasses.get(&current)?.clone();
        }
        None
    }

    /// Renders the constant `reference`, or `None` when the graph does not
    /// hold it.
    pub(super) fn render(&self, reference: i32) -> Option<String> {
        self.constants.get(&reference)?;
        Some(self.render_node(reference, 0, &mut BTreeSet::new()))
    }

    fn render_node(&self, reference: i32, depth: usize, active: &mut BTreeSet<i32>) -> String {
        let fallback = || format!("aot.snapshotRef({reference})");
        let Some(constant) = self.constants.get(&reference) else {
            return fallback();
        };
        let composite = matches!(
            constant,
            SnapshotConstant::List { .. }
                | SnapshotConstant::Map { .. }
                | SnapshotConstant::Set { .. }
                | SnapshotConstant::Record { .. }
                | SnapshotConstant::Instance {
                    enum_name: None,
                    ..
                }
        );
        if composite && (depth >= MAX_DEPTH || !active.insert(reference)) {
            return fallback();
        }
        let mut child = |reference: i32| self.render_node(reference, depth + 1, active);
        let rendered = match constant {
            SnapshotConstant::Null => "null".to_owned(),
            SnapshotConstant::Bool { value } => value.to_string(),
            SnapshotConstant::Int { value } => value.to_string(),
            SnapshotConstant::Double { value } => double_literal(*value),
            SnapshotConstant::String { value } => dart_string(value),
            SnapshotConstant::Type { display } => type_literal(display),
            SnapshotConstant::Closure { function } => match function {
                Some(function) => format!("aot.constClosure({})", dart_string(function)),
                None => fallback(),
            },
            SnapshotConstant::List {
                elements,
                type_arguments,
                ..
            } => format!(
                "const {}[{}]",
                type_prefix(type_arguments.as_deref(), 1),
                items(elements.iter().map(|&element| child(element)))
            ),
            SnapshotConstant::Set {
                elements,
                type_arguments,
            } => format!(
                "const {}{{{}}}",
                type_prefix(type_arguments.as_deref(), 1),
                items(elements.iter().map(|&element| child(element)))
            ),
            SnapshotConstant::Map {
                entries,
                type_arguments,
            } => {
                let prefix = type_prefix(type_arguments.as_deref(), 2);
                let prefix = if entries.is_empty() && prefix.is_empty() {
                    "<dynamic, dynamic>".to_owned()
                } else {
                    prefix
                };
                format!(
                    "const {prefix}{{{}}}",
                    items(entries.iter().map(|&(key, value)| format!(
                        "{}: {}",
                        child(key),
                        child(value)
                    )))
                )
            }
            SnapshotConstant::Record {
                fields,
                field_names_index,
            } => {
                if *field_names_index != 0 {
                    // Named record fields exist but their names are not
                    // serialized; positional rendering would misstate the
                    // shape.
                    fallback()
                } else {
                    let body = items(fields.iter().map(|&field| child(field)));
                    if fields.len() == 1 {
                        format!("({body},)")
                    } else {
                        format!("({body})")
                    }
                }
            }
            SnapshotConstant::Instance {
                class_name,
                enum_name: Some(enum_name),
                ..
            } => format!(
                "{}.{}",
                dart_identifier(&clean_symbol(class_name)),
                dart_identifier(enum_name)
            ),
            SnapshotConstant::Instance {
                class_name, slots, ..
            } => {
                let entries = slots.iter().map(|slot| {
                    let key = slot
                        .field
                        .as_deref()
                        .or_else(|| self.field_name(class_name, slot.offset))
                        .map(|name| name.to_owned())
                        .unwrap_or_else(|| format!("_slot_{:x}", slot.offset));
                    let value = match slot.value {
                        ConstantSlotValue::Reference { reference } => child(reference),
                        ConstantSlotValue::Unboxed { bits } => {
                            unboxed_literal(bits, slot.field_type.as_deref())
                        }
                    };
                    format!("{}: {value}", dart_string(&key))
                });
                format!(
                    "aot.constObject({}, <String, Object?>{{{}}})",
                    dart_string(&clean_symbol(class_name)),
                    items(entries)
                )
            }
        };
        if composite {
            active.remove(&reference);
        }
        rendered
    }
}

fn items(values: impl Iterator<Item = String>) -> String {
    let mut rendered = Vec::new();
    let mut elided = 0usize;
    for value in values {
        if rendered.len() < MAX_ITEMS {
            rendered.push(value);
        } else {
            elided += 1;
        }
    }
    if elided > 0 {
        rendered.push(format!("/* {elided} more */"));
    }
    rendered.join(", ")
}

/// `<T>` or `<K, V>` when the recovered type arguments are Dart-safe.
fn type_prefix(type_arguments: Option<&str>, arity: usize) -> String {
    let Some(arguments) = type_arguments else {
        return String::new();
    };
    // Strip exactly the outer brackets: `<String, List<String>>` keeps the
    // nested list's closing bracket.
    let arguments = arguments.trim();
    let arguments = arguments
        .strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
        .unwrap_or(arguments)
        .trim();
    let safe = !arguments.is_empty()
        && arguments.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '_' | '$' | '<' | '>' | ',' | ' ' | '?' | '.')
        })
        && balanced_angles(arguments)
        && top_level_arity(arguments) == arity
        // Obfuscated class names can be keywords (`is`, `in`).
        && !arguments
            .split(|character: char| !(character.is_ascii_alphanumeric() || matches!(character, '_' | '$')))
            .any(|token| super::dart::DART_RESERVED_WORDS.contains(&token));
    if safe {
        format!("<{arguments}>")
    } else {
        String::new()
    }
}

/// Whether every `<` closes in order and nothing closes unopened.
fn balanced_angles(arguments: &str) -> bool {
    let mut depth = 0i32;
    for character in arguments.chars() {
        match character {
            '<' => depth += 1,
            '>' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

fn top_level_arity(arguments: &str) -> usize {
    let mut depth = 0i32;
    let mut count = 1;
    for character in arguments.chars() {
        match character {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

fn type_literal(display: &str) -> String {
    let base = display.trim_end_matches('?');
    if !base.is_empty()
        && base.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        })
        && !base.starts_with(|character: char| character.is_ascii_digit())
        // Obfuscated class names can be keywords (`is`, `in`).
        && !super::dart::DART_RESERVED_WORDS.contains(&base)
    {
        base.to_owned()
    } else {
        format!("aot.constType({})", dart_string(display))
    }
}

fn double_literal(value: f64) -> String {
    if value.is_nan() {
        "double.nan".to_owned()
    } else if value.is_infinite() {
        if value > 0.0 {
            "double.infinity".to_owned()
        } else {
            "double.negativeInfinity".to_owned()
        }
    } else {
        let text = format!("{value:?}");
        if text.contains(['.', 'e', 'E']) {
            text
        } else {
            format!("{text}.0")
        }
    }
}

/// Unboxed slots carry raw bits. A declared `double` or `int` field decides
/// the reading. Without one, small integers and doubles with a short decimal
/// form are unambiguous in practice; anything else keeps its bits.
fn unboxed_literal(bits: u64, declared_type: Option<&str>) -> String {
    match declared_type.map(|value| value.trim_end_matches('?')) {
        Some("double") => return double_literal(f64::from_bits(bits)),
        Some("int") => return (bits as i64).to_string(),
        _ => {}
    }
    let signed = bits as i64;
    if (-(1i64 << 32)..(1i64 << 32)).contains(&signed) {
        return signed.to_string();
    }
    let value = f64::from_bits(bits);
    if value.is_finite() && value.abs() >= 1e-9 && value.abs() < 1e15 {
        let text = double_literal(value);
        if text.len() <= 12 {
            return text;
        }
    }
    format!("aot.unboxedBits(0x{bits:x})")
}

thread_local! {
    static ACTIVE: RefCell<Option<Arc<ConstantIndex>>> = const { RefCell::new(None) };
}

/// Makes `index` the constant graph for rendering on this thread until the
/// returned guard drops.
pub(super) fn activate(index: Arc<ConstantIndex>) -> ActiveGuard {
    let previous = ACTIVE.with(|active| active.borrow_mut().replace(index));
    ActiveGuard { previous }
}

pub(super) struct ActiveGuard {
    previous: Option<Arc<ConstantIndex>>,
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        ACTIVE.with(|active| *active.borrow_mut() = previous);
    }
}

/// Renders the value the snapshot's initial field table holds for a static
/// field, when the constant graph decoded it. Lazily initialized fields hold
/// the VM sentinel, which is not a constant and yields `None`.
pub(super) fn render_static_initial(shared: bool, id: i64) -> Option<String> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let index = active.as_ref()?;
        let table = if shared {
            &index.shared_initial_fields
        } else {
            &index.initial_fields
        };
        index.render(*table.get(usize::try_from(id).ok()?)?)
    })
}

/// Renders `reference` from the active graph.
pub(super) fn render_reference(reference: i32) -> Option<String> {
    ACTIVE.with(|active| active.borrow().as_ref()?.render(reference))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_type_names_stay_quoted() {
        assert_eq!(type_literal("Widget"), "Widget");
        assert_eq!(type_literal("is"), "aot.constType('is')");
        assert_eq!(type_literal("in?"), "aot.constType('in?')");
    }

    #[test]
    fn type_prefixes_keep_nested_generic_brackets() {
        assert_eq!(
            type_prefix(Some("<String, List<String>>"), 2),
            "<String, List<String>>"
        );
        assert_eq!(type_prefix(Some("<List<Map<String, int>>>"), 1), "<List<Map<String, int>>>");
        assert_eq!(type_prefix(Some("<String, List<String>"), 2), "");
        assert_eq!(type_prefix(Some("<a>b<c>"), 1), "");
        assert_eq!(type_prefix(Some("<is>"), 1), "");
    }

    use crate::model::ConstantSlot;

    fn index(constants: Vec<(i32, SnapshotConstant)>) -> ConstantIndex {
        ConstantIndex {
            constants: constants.into_iter().collect(),
            ..ConstantIndex::default()
        }
    }

    #[test]
    fn static_initial_values_come_from_the_matching_field_table() {
        let mut graph = index(vec![
            (10, SnapshotConstant::Bool { value: true }),
            (
                11,
                SnapshotConstant::String {
                    value: "shared".to_owned(),
                },
            ),
        ]);
        graph.initial_fields = vec![99, 10];
        graph.shared_initial_fields = vec![11];
        let _active = activate(Arc::new(graph));
        assert_eq!(render_static_initial(false, 1).as_deref(), Some("true"));
        assert_eq!(render_static_initial(true, 0).as_deref(), Some("'shared'"));
        // The sentinel (or any undecoded object) is not a value.
        assert_eq!(render_static_initial(false, 0), None);
        assert_eq!(render_static_initial(false, 7), None);
        assert_eq!(render_static_initial(false, -1), None);
    }

    #[test]
    fn renders_instances_with_proven_field_names() {
        let mut graph = index(vec![
            (
                10,
                SnapshotConstant::Instance {
                    class_name: "Product".to_owned(),
                    library_uri: None,
                    enum_name: None,
                    canonical: true,
                    slots: vec![
                        ConstantSlot {
                            offset: 8,
                            value: ConstantSlotValue::Reference { reference: 11 },
                            field: None,
                            field_type: None,
                        },
                        ConstantSlot {
                            offset: 12,
                            value: ConstantSlotValue::Unboxed {
                                bits: 19.99f64.to_bits(),
                            },
                            field: None,
                            field_type: None,
                        },
                        ConstantSlot {
                            offset: 20,
                            value: ConstantSlotValue::Unboxed {
                                bits: (0.1f64 / 3.0).to_bits(),
                            },
                            field: Some("alpha".to_owned()),
                            field_type: Some("double".to_owned()),
                        },
                    ],
                },
            ),
            (
                11,
                SnapshotConstant::String {
                    value: "Lamp".to_owned(),
                },
            ),
        ]);
        graph
            .fields
            .insert(("Item".to_owned(), 8), "name".to_owned());
        graph
            .superclasses
            .insert("Product".to_owned(), "Item".to_owned());
        assert_eq!(
            graph.render(10).as_deref(),
            Some(
                "aot.constObject('Product', <String, Object?>{'name': 'Lamp', '_slot_c': 19.99, \
                 'alpha': 0.03333333333333333})"
            )
        );
    }

    #[test]
    fn renders_enums_lists_and_cycles() {
        let graph = index(vec![
            (
                1,
                SnapshotConstant::List {
                    elements: vec![2, 1],
                    type_arguments: Some("<Color>".to_owned()),
                    immutable: true,
                },
            ),
            (
                2,
                SnapshotConstant::Instance {
                    class_name: "Color".to_owned(),
                    library_uri: None,
                    enum_name: Some("red".to_owned()),
                    canonical: true,
                    slots: Vec::new(),
                },
            ),
        ]);
        assert_eq!(
            graph.render(1).as_deref(),
            Some("const <Color>[Color.red, aot.snapshotRef(1)]")
        );
        assert_eq!(graph.render(3), None);
    }

    #[test]
    fn keeps_ambiguous_unboxed_bits() {
        assert_eq!(unboxed_literal(42, None), "42");
        assert_eq!(unboxed_literal(2.5f64.to_bits(), None), "2.5");
        // A declared type settles what the bits mean.
        assert_eq!(
            unboxed_literal((0.1f64 / 3.0).to_bits(), Some("double")),
            "0.03333333333333333"
        );
        assert_eq!(
            unboxed_literal(2.5f64.to_bits(), Some("int")),
            "4612811918334230528"
        );
        assert_eq!(
            unboxed_literal(0x1234_5678_9abc_def0, None),
            "aot.unboxedBits(0x123456789abcdef0)"
        );
    }
}
