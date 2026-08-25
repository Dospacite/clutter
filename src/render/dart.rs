use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::model::{
    EvidenceConfidence, RecoveredClassMetadata, RecoveredDeclaration, RecoveredDeclarationKind,
    RecoveredFieldMetadata, RecoveredFunction, RecoveredFunctionKind, RecoveredLibrary,
    RecoveredProgram, RecoveredType, RecoveredTypeParameter, SemanticStatement,
};

pub(super) struct RenderIndex {
    functions_by_library: BTreeMap<String, Vec<usize>>,
    declarations_by_library: BTreeMap<String, Vec<usize>>,
    shared_code_primary: BTreeMap<(String, u64), usize>,
    /// Callee evidence for call-site rendering, keyed by qualified readable
    /// name (`Owner.member`) and, for top-level members, bare name.
    callees: BTreeMap<String, CalleeInfo>,
}

/// What a recovered declaration proves about a callable at a call site.
#[derive(Clone, Debug, Default)]
pub(crate) struct CalleeInfo {
    pub implicit_parameter_count: Option<usize>,
    pub fixed_parameter_count: Option<usize>,
    /// Named optional parameters in declaration order with required flags.
    pub optional_named: Vec<(String, bool)>,
    pub optional_positional: usize,
    pub kind: Option<RecoveredFunctionKind>,
}

/// Named-optional parameter list for a callee. Resolved signature parameters
/// win; when they carry no names (common for obfuscated snapshots whose
/// descriptor strings were dropped), the bound VM oracle evidence supplies
/// them — implicit receiver/ctx parameters are excluded to keep positions
/// aligned with the snapshot signature.
pub(crate) fn callee_optional_named(
    signature: Option<&crate::model::RecoveredSignature>,
    vm_evidence: Option<&crate::model::VmFunctionEvidence>,
) -> Vec<(String, bool)> {
    let Some(resolved) = signature.and_then(|signature| signature.resolved.as_ref()) else {
        return Vec::new();
    };
    let fixed = signature.map_or(0, |signature| signature.fixed_parameter_count);
    let mut vm_names = vm_evidence.map(|evidence| {
        let mut names: BTreeMap<usize, String> = BTreeMap::new();
        for parameter in &evidence.parameters {
            if !parameter.is_implicit && let Some(name) = parameter.name.as_deref() {
                names.insert(parameter.position.saturating_sub(1), name.to_owned());
            }
        }
        names
    });
    resolved
        .parameters
        .iter()
        .skip(fixed)
        .filter(|parameter| parameter.is_named)
        .map(|parameter| {
            (
                parameter
                    .name
                    .clone()
                    .or_else(|| {
                        vm_names
                            .as_mut()
                            .and_then(|names| names.remove(&parameter.position))
                    })
                    .unwrap_or_default(),
                parameter.is_required,
            )
        })
        .collect()
}

pub(crate) fn source_visible_function(function: &RecoveredFunction) -> bool {
    let vm_kind = function
        .vm_evidence
        .as_ref()
        .and_then(|evidence| evidence.kind.as_deref());
    if matches!(vm_kind, Some("VmStubCode" | "AotCodeBoundary")) {
        return false;
    }
    if function.name_source == crate::model::RecoveredNameSource::Synthetic
        && (function.library_uri.is_none()
            || matches!(vm_kind, Some("SharedAotCodeBoundary" | "AotCodeBoundary")))
    {
        return false;
    }
    true
}

impl RenderIndex {
    pub(super) fn new(program: &RecoveredProgram) -> Self {
        let mut functions_by_library = BTreeMap::<String, Vec<usize>>::new();
        let mut shared_code_primary = BTreeMap::new();
        for (index, function) in program.functions.iter().enumerate() {
            functions_by_library
                .entry(
                    function
                        .library_uri
                        .clone()
                        .unwrap_or_else(|| "clutter:unattributed".to_owned()),
                )
                .or_default()
                .push(index);
            shared_code_primary
                .entry((function.address.clone(), function.size))
                .or_insert(index);
        }
        let mut declarations_by_library = BTreeMap::<String, Vec<usize>>::new();
        for (index, declaration) in program.declarations.iter().enumerate() {
            if let Some(library_uri) = &declaration.library_uri {
                declarations_by_library
                    .entry(library_uri.clone())
                    .or_default()
                    .push(index);
            }
        }
        let mut callees = BTreeMap::new();
        fn observe(
            callees: &mut BTreeMap<String, CalleeInfo>,
            owner: Option<&str>,
            name: &str,
            mut info: CalleeInfo,
            kind: Option<RecoveredFunctionKind>,
        ) {
            if info.kind.is_none() {
                info.kind = kind;
            }
            let readable = clean_symbol(name);
            match owner {
                Some(owner) if !matches!(owner, "::" | "top_level") => {
                    let qualified = format!("{}.{readable}", clean_symbol(owner));
                    callees.entry(qualified).or_insert(info);
                }
                _ => {
                    callees.entry(readable).or_insert(info);
                }
            }
        }
        for declaration in program
            .declarations
            .iter()
            .chain(program.declaration_evidence.iter())
        {
            if declaration.kind != RecoveredDeclarationKind::Function {
                continue;
            }
            let signature = declaration.signature.as_ref();
            let resolved = signature.and_then(|signature| signature.resolved.as_ref());
            let mut info = CalleeInfo {
                implicit_parameter_count: signature
                    .map(|signature| signature.implicit_parameter_count),
                fixed_parameter_count: signature.map(|signature| signature.fixed_parameter_count),
                optional_named: resolved
                    .map(|resolved| {
                        resolved
                            .parameters
                            .iter()
                            .skip(signature.map_or(0, |signature| signature.fixed_parameter_count))
                            .filter(|parameter| parameter.is_named)
                            .map(|parameter| {
                                (
                                    parameter.name.clone().unwrap_or_default(),
                                    parameter.is_required,
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                optional_positional: signature.map_or(0, |signature| {
                    if signature.optional_parameters_are_named {
                        0
                    } else {
                        signature.optional_parameter_count
                    }
                }),
                kind: declaration.function_kind,
            };
            // A declaration with a resolved named list wins over duplicates.
            if !info.optional_named.is_empty() {
                let owner_key = declaration.owner.as_deref().map(clean_symbol);
                let readable = clean_symbol(&declaration.name);
                let key = owner_key
                    .map_or_else(|| readable.clone(), |owner| format!("{owner}.{readable}"));
                callees.insert(key, std::mem::take(&mut info));
            } else {
                observe(
                    &mut callees,
                    declaration.owner.as_deref(),
                    &declaration.name,
                    std::mem::take(&mut info),
                    declaration.function_kind,
                );
            }
        }
        for function in program.functions.iter() {
            let signature = function.signature.as_ref();
            let info = CalleeInfo {
                implicit_parameter_count: signature
                    .map(|signature| signature.implicit_parameter_count),
                fixed_parameter_count: signature.map(|signature| signature.fixed_parameter_count),
                optional_named: callee_optional_named(
                    signature,
                    function.vm_evidence.as_ref(),
                ),
                optional_positional: signature.map_or(0, |signature| {
                    if signature.optional_parameters_are_named {
                        0
                    } else {
                        signature.optional_parameter_count
                    }
                }),
                kind: function.kind,
            };
            observe(
                &mut callees,
                function.owner.as_deref(),
                &function.name,
                info,
                function.kind,
            );
        }
        Self {
            functions_by_library,
            declarations_by_library,
            shared_code_primary,
            callees,
        }
    }

    pub(super) fn callee(&self, target: &str) -> Option<&CalleeInfo> {
        let normalized = normalized_call_target(target)?;
        if let Some(info) = self.callees.get(&normalized) {
            return Some(info);
        }
        // Try the last two segments (`Owner.member`) and the bare member.
        let segments: Vec<&str> = normalized.split('.').collect();
        if segments.len() >= 2 {
            let pair = format!(
                "{}.{}",
                segments[segments.len() - 2],
                segments[segments.len() - 1]
            );
            if let Some(info) = self.callees.get(&pair) {
                return Some(info);
            }
        }
        self.callees.get(*segments.last()?)
    }
}

pub(super) fn render_library(
    library: &RecoveredLibrary,
    program: &RecoveredProgram,
    index: &RenderIndex,
) -> String {
    let mut output = String::new();
    writeln!(output, "// GENERATED AOT PSEUDOCODE — NOT ORIGINAL SOURCE.").unwrap();
    writeln!(
        output,
        "// ignore_for_file: unused_element, unused_import, non_constant_identifier_names"
    )
    .unwrap();
    writeln!(output, "// Recovered from: {}", safe_comment(&library.uri)).unwrap();
    writeln!(output).unwrap();
    writeln!(
        output,
        "import '{}' as aot;",
        relative_support_import(library)
    )
    .unwrap();
    writeln!(output).unwrap();

    writeln!(
        output,
        "const String recoveredSourceUri = {};",
        dart_string(&library.uri)
    )
    .unwrap();

    if library.is_application && library.output_path.ends_with("main.dart") {
        writeln!(output).unwrap();
        writeln!(output, "dynamic recoveredEntry(List<dynamic> args) {{").unwrap();
        writeln!(
            output,
            "  return aot.unresolvedRegion(recoveredSourceUri, args);"
        )
        .unwrap();
        writeln!(output, "}}").unwrap();
    }

    let declarations: Vec<_> = index
        .declarations_by_library
        .get(&library.uri)
        .into_iter()
        .flatten()
        .filter_map(|declaration_index| program.declarations.get(*declaration_index))
        .collect();
    let all_functions: Vec<_> = index
        .functions_by_library
        .get(&library.uri)
        .into_iter()
        .flatten()
        .filter_map(|function_index| program.functions.get(*function_index))
        .collect();
    let hidden_function_count = all_functions
        .iter()
        .filter(|function| !source_visible_function(function))
        .count();
    let functions = all_functions
        .into_iter()
        .filter(|function| source_visible_function(function))
        .collect::<Vec<_>>();
    if hidden_function_count > 0 {
        writeln!(output).unwrap();
        writeln!(
            output,
            "// {hidden_function_count} runtime stub or ownerless AOT code body/bodies are preserved in reports/functions.json and reports/assembly.s."
        )
        .unwrap();
    }
    let declaration_only_functions = declarations
        .iter()
        .copied()
        .filter(|declaration| {
            declaration.kind == RecoveredDeclarationKind::Function
                && !declaration.has_code
                && !functions
                    .iter()
                    .any(|function| declaration_matches_function(declaration, function))
        })
        .collect::<Vec<_>>();
    let initialized_fields = functions
        .iter()
        .filter_map(|function| {
            function
                .name
                .strip_prefix("init:")
                .map(|field| (function.owner.as_deref().unwrap_or(""), clean_symbol(field)))
        })
        .collect::<BTreeSet<_>>();
    let top_level_fields = declarations
        .iter()
        .filter(|declaration| {
            declaration.kind == RecoveredDeclarationKind::Field
                && declaration
                    .owner
                    .as_deref()
                    .is_none_or(|owner| matches!(owner, "::" | "top_level"))
                && !initialized_fields.contains(&("", clean_symbol(&declaration.name)))
        })
        .collect::<Vec<_>>();
    if !top_level_fields.is_empty() {
        writeln!(output).unwrap();
        writeln!(output, "// Recovered top-level fields.").unwrap();
        for field in top_level_fields {
            render_field_declaration(&mut output, field, "", false);
        }
    }
    let mut top_level = Vec::new();
    let mut classes = BTreeMap::<String, Vec<&RecoveredFunction>>::new();
    for function in functions {
        if let Some(owner) = recovered_class_owner(library, function) {
            classes.entry(owner).or_default().push(function);
        } else {
            top_level.push(function);
        }
    }
    let mut class_declarations = BTreeMap::<String, &RecoveredDeclaration>::new();
    for declaration in declarations.iter().filter(|declaration| {
        declaration.kind == RecoveredDeclarationKind::Class
            && !matches!(declaration.name.as_str(), "::" | "top_level")
    }) {
        let name = clean_symbol(&declaration.name);
        classes.entry(name.clone()).or_default();
        class_declarations.insert(name, declaration);
    }
    for declaration in &declaration_only_functions {
        if let Some(owner) = declaration
            .owner
            .as_deref()
            .filter(|owner| !matches!(*owner, "::" | "top_level"))
            .map(clean_symbol)
        {
            classes.entry(owner).or_default();
        }
    }
    for declaration in declaration_only_functions.iter().filter(|declaration| {
        declaration
            .owner
            .as_deref()
            .is_none_or(|owner| matches!(owner, "::" | "top_level"))
    }) {
        render_function_declaration_stub(&mut output, program, declaration, "", false);
    }
    render_member_group(&mut output, program, index, &top_level, false, "");
    for (owner, functions) in classes {
        writeln!(output).unwrap();
        let declaration = class_declarations.get(&owner).copied();
        if declaration
            .and_then(|declaration| declaration.class_metadata.as_ref())
            .is_some_and(|metadata| metadata.is_enum)
        {
            writeln!(
                output,
                "/// Snapshot class flags identify this declaration as an enum; values may be tree-shaken."
            )
            .unwrap();
        }
        if declaration
            .and_then(|declaration| declaration.class_metadata.as_ref())
            .is_some_and(|metadata| metadata.is_transformed_mixin_application)
        {
            writeln!(
                output,
                "/// Synthetic mixin-application class retained by the AOT compiler."
            )
            .unwrap();
        }
        writeln!(
            output,
            "{} {{",
            class_declaration_header(&owner, declaration, &class_declarations)
        )
        .unwrap();
        let fields = declarations
            .iter()
            .filter(|declaration| {
                declaration.kind == RecoveredDeclarationKind::Field
                    && declaration
                        .owner
                        .as_deref()
                        .is_some_and(|field_owner| clean_symbol(field_owner) == owner)
                    && !initialized_fields.contains(&(
                        declaration.owner.as_deref().unwrap_or(""),
                        clean_symbol(&declaration.name),
                    ))
            })
            .collect::<Vec<_>>();
        let known_field_offsets = fields
            .iter()
            .filter_map(|field| {
                field
                    .field_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.instance_field_offset)
            })
            .collect::<BTreeSet<_>>();
        for field in fields {
            render_field_declaration(&mut output, field, "  ", true);
        }
        render_unknown_instance_slots(&mut output, declaration, &known_field_offsets, "  ");
        for declaration in declaration_only_functions.iter().filter(|declaration| {
            declaration
                .owner
                .as_deref()
                .is_some_and(|declaration_owner| clean_symbol(declaration_owner) == owner)
        }) {
            render_function_declaration_stub(&mut output, program, declaration, "  ", true);
        }
        render_member_group(&mut output, program, index, &functions, true, "  ");
        writeln!(output, "}}").unwrap();
    }
    output
}

fn recovered_class_owner(
    library: &RecoveredLibrary,
    function: &RecoveredFunction,
) -> Option<String> {
    let owner = function.owner.as_deref().unwrap_or("top_level");
    if matches!(owner, "::" | "top_level") {
        return None;
    }
    if owner == library.uri || owner.starts_with("package:") || owner.starts_with("dart:") {
        // Snapshot top-level classes are represented by their library owner.
        // Treating each such function name as a class invents thousands of
        // false constructors and invalid Dart containers.
        return None;
    }
    // Some snapshot owner names carry selector suffixes (`Class|get#name`);
    // the class lives before the separator.
    let owner = owner.split('|').next().unwrap_or(owner);
    Some(clean_symbol(owner))
}

fn is_closure_function(function: &RecoveredFunction) -> bool {
    function.kind == Some(RecoveredFunctionKind::Closure) || function.name == "<anonymous closure>"
}

/// Picks the enclosing member for a closure: authoritative VM lexical-parent
/// links first, then source-line containment (innermost span wins).
fn closure_parent_index(members: &[&RecoveredFunction], closure_index: usize) -> Option<usize> {
    let closure = members[closure_index];
    if let Some(parent_name) = closure
        .vm_evidence
        .as_ref()
        .and_then(|evidence| evidence.parent_function_name.as_deref())
    {
        let parent_clean = clean_symbol(parent_name);
        let bare = parent_clean.rsplit('.').next().unwrap_or(&parent_clean);
        for (index, candidate) in members.iter().enumerate() {
            if index == closure_index || is_closure_function(candidate) {
                continue;
            }
            let owner = clean_symbol(candidate.owner.as_deref().unwrap_or(""));
            let qualified = format!("{owner}.{}", clean_symbol(&candidate.name));
            if qualified == parent_clean || clean_symbol(&candidate.name) == bare {
                return Some(index);
            }
        }
    }
    let line = closure.source_location.as_ref()?.line?;
    let mut best: Option<(usize, u64, u64)> = None;
    for (index, candidate) in members.iter().enumerate() {
        if index == closure_index || is_closure_function(candidate) {
            continue;
        }
        let Some(location) = candidate.source_location.as_ref() else {
            continue;
        };
        let start = location.line.unwrap_or(0);
        let end = location.end_line.or(location.line).unwrap_or(start);
        if start <= line && line <= end {
            best = match best {
                Some((_, best_start, best_end)) if best_end - best_start <= end - start => best,
                _ => Some((index, start, end)),
            };
        }
    }
    best.map(|(index, _, _)| index)
}

/// Renders one grouping of functions (a class's members or a library's
/// top-level functions). Closures with a provable parent render as local
/// functions nested inside that parent; unproven closures stay as members.
fn render_member_group(
    output: &mut String,
    program: &RecoveredProgram,
    index: &RenderIndex,
    members: &[&RecoveredFunction],
    in_class: bool,
    indent: &str,
) {
    let mut nested_by_parent: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut direct: Vec<usize> = Vec::new();
    for (position, member) in members.iter().enumerate() {
        if is_closure_function(member) {
            let parent = closure_parent_index(members, position);
            if std::env::var("CLUTTER_DEBUG_STRUCTURE").is_ok() {
                eprintln!(
                    "CLOSURE {} line={:?} -> {:?}",
                    member.name,
                    member
                        .source_location
                        .as_ref()
                        .map(|l| (l.line, l.end_line)),
                    parent.map(|p| members[p].name.clone())
                );
            }
            if let Some(parent) = parent
                && parent != position
            {
                nested_by_parent.entry(parent).or_default().push(position);
                continue;
            }
        }
        direct.push(position);
    }
    for closures in nested_by_parent.values_mut() {
        closures.sort_by_key(|closure_index| {
            members[*closure_index]
                .source_location
                .as_ref()
                .and_then(|location| location.line)
                .unwrap_or(u64::MAX)
        });
    }

    let direct_functions: Vec<&RecoveredFunction> =
        direct.iter().map(|position| members[*position]).collect();
    let collisions = function_name_collisions(&direct_functions, in_class);

    let mut rendered_parents: BTreeSet<usize> = BTreeSet::new();
    for position in &direct {
        let function = members[*position];
        let collision_count = collisions
            .get(&rendered_function_symbol_root(function, in_class))
            .copied()
            .unwrap_or(1);
        let nested: Vec<(&RecoveredFunction, usize)> = nested_by_parent
            .get(position)
            .map(|closures| {
                let mut used = BTreeMap::<String, usize>::new();
                closures
                    .iter()
                    .map(|closure| {
                        let member = members[*closure];
                        let root = rendered_function_symbol_root(member, in_class);
                        let occurrence = used.entry(root.clone()).or_default();
                        *occurrence += 1;
                        (member, *occurrence)
                    })
                    .collect()
            })
            .unwrap_or_default();
        render_function(
            output,
            program,
            index,
            function,
            indent,
            in_class,
            collision_count,
            &nested,
            false,
            1,
        );
        rendered_parents.insert(*position);
    }
}

fn class_declaration_header(
    owner: &str,
    declaration: Option<&RecoveredDeclaration>,
    declarations: &BTreeMap<String, &RecoveredDeclaration>,
) -> String {
    let Some(metadata) = declaration.and_then(|declaration| declaration.class_metadata.as_ref())
    else {
        // A split-debug declaration can survive without its corresponding
        // snapshot Class object. Use Dart's neutral class spelling instead of
        // inventing an `abstract` modifier that DWARF does not prove.
        return format!("class {}", dart_identifier(owner));
    };
    let keyword = if metadata.is_enum {
        // Enum values are held as canonical instances and can be removed by
        // precompilation. Keep a class-shaped container rather than inventing
        // an enum value that did not survive.
        "abstract class"
    } else if inferred_mixin_declaration(owner, metadata, declarations) {
        "mixin"
    } else if metadata.is_mixin_class {
        "mixin class"
    } else if metadata.is_sealed {
        "sealed class"
    } else if metadata.is_final && metadata.is_abstract {
        "abstract final class"
    } else if metadata.is_final {
        "final class"
    } else if metadata.is_interface && metadata.is_abstract {
        "abstract interface class"
    } else if metadata.is_interface {
        "interface class"
    } else if metadata.is_base && metadata.is_abstract {
        "abstract base class"
    } else if metadata.is_base {
        "base class"
    } else if metadata.is_abstract {
        "abstract class"
    } else {
        "class"
    };
    let mut output = format!(
        "{keyword} {}{}",
        dart_identifier(owner),
        render_type_parameters(&metadata.type_parameters),
    );
    let (super_type, mixins) = source_facing_super_type(metadata, declarations);
    let is_plain_mixin = keyword == "mixin";
    if let Some(super_type) = super_type.filter(|value| !is_object_type(value)) {
        let relation = if is_plain_mixin { " on " } else { " extends " };
        output.push_str(relation);
        output.push_str(&rendered_type(super_type));
    }
    if !mixins.is_empty() && !is_plain_mixin {
        output.push_str(" with ");
        output.push_str(
            &mixins
                .iter()
                .map(|value| rendered_type(value))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    let mut interfaces = metadata.interfaces.as_slice();
    if metadata.is_transformed_mixin_application && !interfaces.is_empty() {
        interfaces = &interfaces[..interfaces.len() - 1];
    }
    if !interfaces.is_empty() {
        output.push_str(" implements ");
        output.push_str(
            &interfaces
                .iter()
                .map(rendered_type)
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    output
}

fn inferred_mixin_declaration(
    owner: &str,
    metadata: &RecoveredClassMetadata,
    declarations: &BTreeMap<String, &RecoveredDeclaration>,
) -> bool {
    metadata.is_mixin_class && metadata.is_abstract
        || !metadata.is_mixin_class
            && declarations.values().any(|declaration| {
                declaration
                    .class_metadata
                    .as_ref()
                    .filter(|candidate| candidate.is_transformed_mixin_application)
                    .and_then(|candidate| candidate.interfaces.last())
                    .is_some_and(|mixin| type_root(&mixin.display_name) == owner)
            })
}

fn source_facing_super_type<'a>(
    metadata: &'a RecoveredClassMetadata,
    declarations: &'a BTreeMap<String, &RecoveredDeclaration>,
) -> (Option<&'a RecoveredType>, Vec<&'a RecoveredType>) {
    let mut super_type = metadata.super_type.as_ref();
    let mut mixins = Vec::new();
    if metadata.is_transformed_mixin_application {
        if let Some(mixin) = metadata.interfaces.last() {
            mixins.push(mixin);
        }
        return (super_type, mixins);
    }
    let Some(recovered_super) = super_type else {
        return (None, mixins);
    };
    let root = type_root(&recovered_super.display_name);
    let Some(synthetic) = declarations.get(root) else {
        return (super_type, mixins);
    };
    let Some(synthetic_metadata) = synthetic.class_metadata.as_ref() else {
        return (super_type, mixins);
    };
    if synthetic_metadata.is_transformed_mixin_application {
        super_type = synthetic_metadata.super_type.as_ref();
        if let Some(mixin) = synthetic_metadata.interfaces.last() {
            mixins.push(mixin);
        }
    }
    (super_type, mixins)
}

fn type_root(value: &str) -> &str {
    value.split(['<', '?']).next().unwrap_or(value).trim()
}

fn is_object_type(value: &RecoveredType) -> bool {
    matches!(type_root(&value.display_name), "Object" | "dynamic")
}

fn render_field_declaration(
    output: &mut String,
    declaration: &RecoveredDeclaration,
    indent: &str,
    in_class: bool,
) {
    let metadata = declaration.field_metadata.as_ref();
    if metadata.is_some_and(|metadata| metadata.is_const) {
        writeln!(
            output,
            "{indent}/// Dart VM marked this field const; the constant initializer did not survive as source."
        )
        .unwrap();
    }
    let modifiers = metadata
        .map(|metadata| field_modifiers(metadata, in_class))
        .unwrap_or_default();
    let prefix = if modifiers.is_empty() {
        String::new()
    } else {
        format!("{modifiers} ")
    };
    let field_type = metadata
        .and_then(|metadata| metadata.declared_type.as_ref())
        .map_or_else(|| "dynamic".to_owned(), rendered_type);
    let slot_note = metadata
        .and_then(|metadata| {
            metadata
                .instance_field_offset
                .map(|offset| format!(" // AOT instance slot +0x{offset:x}"))
                .or_else(|| {
                    metadata
                        .static_field_offset
                        .map(|offset| format!(" // AOT static slot +0x{offset:x}"))
                })
        })
        .unwrap_or_default();
    writeln!(
        output,
        "{indent}{prefix}{field_type} {};{slot_note}",
        dart_identifier(&clean_symbol(&declaration.name)),
    )
    .unwrap();
}

fn render_unknown_instance_slots(
    output: &mut String,
    declaration: Option<&RecoveredDeclaration>,
    known_offsets: &BTreeSet<i64>,
    indent: &str,
) {
    let slots = declaration
        .and_then(|declaration| declaration.class_metadata.as_ref())
        .map(|metadata| metadata.instance_slots.as_slice())
        .unwrap_or_default()
        .iter()
        .filter(|slot| {
            slot.slot_type != "type_arguments_field" && !known_offsets.contains(&slot.offset)
        })
        .collect::<Vec<_>>();
    if slots.is_empty() {
        return;
    }
    writeln!(
        output,
        "{indent}/// AOT instance slots whose original Field declaration was tree-shaken."
    )
    .unwrap();
    for slot in slots {
        let name = slot.field_name.clone().unwrap_or_else(|| {
            format!("_slot_{:x}", u64::try_from(slot.offset).unwrap_or_default())
        });
        let storage = if slot.is_reference { "dynamic" } else { "num" };
        writeln!(
            output,
            "{indent}{storage} {}; // AOT slot +0x{:x}; {}",
            dart_identifier(&clean_symbol(&name)),
            slot.offset,
            safe_comment(&slot.slot_type),
        )
        .unwrap();
    }
}

fn field_modifiers(metadata: &RecoveredFieldMetadata, in_class: bool) -> String {
    let mut modifiers = Vec::new();
    if in_class && metadata.is_static {
        modifiers.push("static");
    }
    if metadata.is_const {
        // `const` without its original compile-time initializer is not valid
        // Dart. Preserve the flag in the evidence/comment and render the
        // closest declaration-shaped placeholder.
        modifiers.push("final");
    } else {
        if metadata.is_late {
            modifiers.push("late");
        }
        if metadata.is_final {
            modifiers.push("final");
        }
    }
    modifiers.join(" ")
}

fn declaration_matches_function(
    declaration: &RecoveredDeclaration,
    function: &RecoveredFunction,
) -> bool {
    let owner = |value: Option<&str>| {
        value
            .filter(|owner| !matches!(*owner, "::" | "top_level"))
            .map(clean_symbol)
    };
    declaration.library_uri == function.library_uri
        && owner(declaration.owner.as_deref()) == owner(function.owner.as_deref())
        && (clean_symbol(&declaration.name) == clean_symbol(&function.name)
            || readable_function_name(&declaration.name)
                == rendered_function_symbol_root(function, declaration.owner.is_some()))
        && (declaration.function_kind.is_none()
            || function.kind.is_none()
            || declaration.function_kind == function.kind)
}

/// Finds recovered functions whose code source map records this declaration
/// as an inlined callee. When the AOT compiler folded a small callee into
/// its caller, the standalone Code disappears but the caller survives —
/// that host is where the callee's statements actually live (probe EC-1).
fn inline_host_names(
    program: &RecoveredProgram,
    declaration: &RecoveredDeclaration,
) -> Vec<String> {
    let mut hosts = Vec::new();
    for function in &program.functions {
        let matches_callee = function.inlined_functions.iter().any(|callee| {
            callee.name == declaration.name
                && (callee.library_uri.is_none()
                    || callee.library_uri == declaration.library_uri)
        });
        if matches_callee {
            hosts.push(qualified_name(function));
            if hosts.len() >= 3 {
                break;
            }
        }
    }
    hosts
}

fn render_function_declaration_stub(
    output: &mut String,
    program: &RecoveredProgram,
    declaration: &RecoveredDeclaration,
    indent: &str,
    in_class: bool,
) {
    let inline_hosts = inline_host_names(program, declaration);
    let signature = declaration.signature.as_ref();
    let (parameters, _) = signature
        .map(rendered_signature_parameters)
        .unwrap_or_else(|| ("(List<dynamic> args)".to_owned(), None));
    let mut return_type = signature
        .and_then(|signature| signature.resolved.as_ref())
        .and_then(|resolved| resolved.return_type.as_ref())
        .map_or_else(|| "dynamic".to_owned(), rendered_type);
    let type_parameters = signature
        .and_then(|signature| signature.resolved.as_ref())
        .map(|resolved| render_type_parameters(&resolved.type_parameters))
        .unwrap_or_default();
    let vm = declaration.vm_evidence.as_ref();
    let constructor = in_class
        && (declaration.function_kind == Some(RecoveredFunctionKind::Constructor)
            || declaration
                .owner
                .as_deref()
                .is_some_and(|owner| clean_symbol(owner) == clean_symbol(&declaration.name)));
    let static_prefix =
        if constructor || in_class && vm.is_some_and(|evidence| evidence.is_static == Some(true)) {
            "static "
        } else {
            ""
        };
    let async_modifier = if vm.is_some_and(|evidence| evidence.is_async == Some(true)) {
        " async"
    } else {
        ""
    };
    let object_id = declaration
        .vm_object_id
        .map_or_else(|| "unknown".to_owned(), |id| id.to_string());
    let symbol = if constructor {
        return_type = declaration
            .owner
            .as_deref()
            .map(clean_symbol)
            .map(|name| dart_identifier(&name))
            .unwrap_or_else(|| "dynamic".to_owned());
        format!("create_{object_id}")
    } else {
        readable_function_name(&declaration.name)
    };
    writeln!(
        output,
        "{indent}/// Dart VM retained declaration object {object_id}; no distinct executable body survived."
    )
    .unwrap();
    if !inline_hosts.is_empty() {
        // The optimizer folded this body into a surviving caller; point at
        // it instead of pretending no code exists (probe EC-1).
        let references = inline_hosts
            .iter()
            .map(|host| format!("`{}`", safe_comment(host)))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            output,
            "{indent}/// Body inlined into {references}: its statements are recovered inside that body's output."
        )
        .unwrap();
        writeln!(
            output,
            "{indent}{static_prefix}{return_type} {}{type_parameters}{parameters}{async_modifier} {{",
            symbol,
        )
        .unwrap();
        writeln!(
            output,
            "{indent}  // No standalone machine body survived; see the host function above."
        )
        .unwrap();
        writeln!(output, "{indent}}}").unwrap();
    } else {
        writeln!(
            output,
            "{indent}{static_prefix}{return_type} {}{type_parameters}{parameters}{async_modifier} => throw UnsupportedError('AOT body unavailable');",
            symbol,
        )
        .unwrap();
    }
}

fn matching_field_declaration<'a>(
    program: &'a RecoveredProgram,
    function: &RecoveredFunction,
    field_name: &str,
) -> Option<&'a RecoveredDeclaration> {
    program.declarations.iter().find(|declaration| {
        declaration.kind == RecoveredDeclarationKind::Field
            && declaration.library_uri == function.library_uri
            && declaration.owner == function.owner
            && clean_symbol(&declaration.name) == field_name
    })
}

fn rendered_return_type(function: &RecoveredFunction, in_class: bool) -> String {
    if constructor_like(function, in_class) {
        return function
            .owner
            .as_deref()
            .map(clean_symbol)
            .filter(|value| !value.is_empty() && !value.starts_with("package:"))
            .map(|value| dart_identifier(&value))
            .unwrap_or_else(|| "dynamic".to_owned());
    }
    let resolved = function
        .signature
        .as_ref()
        .and_then(|signature| signature.resolved.as_ref())
        .and_then(|resolved| resolved.return_type.as_ref())
        .map_or_else(|| "dynamic".to_owned(), rendered_type);
    if resolved == "dynamic"
        && matches!(
            clean_symbol(&function.name).as_str(),
            "toString" | "hashCode"
        )
    {
        // Overriding these Object members with an unresolved return type is
        // invalid Dart; the VM-proven member identity fixes the type.
        return if clean_symbol(&function.name) == "toString" {
            "String".to_owned()
        } else {
            "int".to_owned()
        };
    }
    resolved
}

fn rendered_type(value: &RecoveredType) -> String {
    let normalized = normalize_function_type_syntax(&value.display_name);
    let mut output = String::with_capacity(normalized.len());
    let characters = normalized.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < characters.len() {
        let character = characters[index];
        if character.is_ascii_alphabetic() || matches!(character, '_' | '$') {
            let start = index;
            index += 1;
            while index < characters.len()
                && (characters[index].is_ascii_alphanumeric()
                    || matches!(characters[index], '_' | '$'))
            {
                index += 1;
            }
            let token = characters[start..index].iter().collect::<String>();
            if matches!(
                token.as_str(),
                "dynamic" | "void" | "Never" | "Null" | "Function" | "required" | "extends"
            ) {
                output.push_str(&token);
            } else {
                output.push_str(&dart_identifier(&token));
            }
            continue;
        }
        if character.is_ascii_digit()
            || matches!(
                character,
                '<' | '>' | ',' | '?' | '(' | ')' | '{' | '}' | '[' | ']' | '.' | ' ' | ':'
            )
        {
            output.push(character);
        } else {
            output.push('_');
        }
        index += 1;
    }
    if output.is_empty() {
        "dynamic".to_owned()
    } else {
        output
    }
}

fn normalize_function_type_syntax(value: &str) -> String {
    let mut output = value.to_owned();
    for _ in 0..64 {
        let Some(arrow) = output.rfind("=>") else {
            break;
        };
        let Some(close) = output[..arrow]
            .char_indices()
            .rev()
            .find_map(|(index, character)| {
                (!character.is_whitespace()).then_some((index, character))
            })
            .and_then(|(index, character)| (character == ')').then_some(index))
        else {
            return "Function".to_owned();
        };
        let mut depth = 0usize;
        let mut open = None;
        for (index, character) in output[..=close].char_indices().rev() {
            match character {
                ')' => depth += 1,
                '(' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        open = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(open) = open else {
            return "Function".to_owned();
        };
        let mut result_start = arrow + 2;
        while output[result_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            result_start += output[result_start..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
        }
        let mut angle = 0usize;
        let mut paren = 0usize;
        let mut square = 0usize;
        let mut brace = 0usize;
        let mut result_end = output.len();
        for (relative, character) in output[result_start..].char_indices() {
            let at_top = angle == 0 && paren == 0 && square == 0 && brace == 0;
            if at_top && matches!(character, ',' | '>' | ')' | ']' | '}') {
                result_end = result_start + relative;
                break;
            }
            match character {
                '<' => angle += 1,
                '>' => angle = angle.saturating_sub(1),
                '(' => paren += 1,
                ')' => paren = paren.saturating_sub(1),
                '[' => square += 1,
                ']' => square = square.saturating_sub(1),
                '{' => brace += 1,
                '}' => brace = brace.saturating_sub(1),
                _ => {}
            }
        }
        let parameters = output[open + 1..close].trim();
        let result = output[result_start..result_end].trim();
        if result.is_empty() {
            return "Function".to_owned();
        }
        output.replace_range(
            open..result_end,
            &format!("{result} Function({parameters})"),
        );
    }
    if output.contains("=>") {
        "Function".to_owned()
    } else {
        strip_grouped_nullable_function_types(output)
    }
}

fn strip_grouped_nullable_function_types(mut value: String) -> String {
    while let Some((close, _)) = value.match_indices(")?").find(|(close, _)| {
        let mut depth = 0usize;
        for (open, character) in value[..=*close].char_indices().rev() {
            match character {
                ')' => depth += 1,
                '(' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return value[open + 1..*close].contains("Function");
                    }
                }
                _ => {}
            }
        }
        false
    }) {
        let mut depth = 0usize;
        let mut open = None;
        for (index, character) in value[..=close].char_indices().rev() {
            match character {
                ')' => depth += 1,
                '(' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        open = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(open) = open else {
            break;
        };
        value.remove(close);
        value.remove(open);
    }
    value
}

fn render_type_parameters(parameters: &[RecoveredTypeParameter]) -> String {
    if parameters.is_empty() {
        return String::new();
    }
    format!(
        "<{}>",
        parameters
            .iter()
            .map(|parameter| parameter.bound.as_ref().map_or_else(
                || dart_identifier(&parameter.name),
                |bound| format!(
                    "{} extends {}",
                    dart_identifier(&parameter.name),
                    rendered_type(bound)
                ),
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[allow(clippy::too_many_arguments)]
fn render_function(
    output: &mut String,
    program: &RecoveredProgram,
    index: &RenderIndex,
    function: &RecoveredFunction,
    indent: &str,
    in_class: bool,
    collision_count: usize,
    nested_closures: &[(&RecoveredFunction, usize)],
    nested: bool,
    occurrence: usize,
) {
    let owner = function.owner.as_deref().unwrap_or("top_level");
    let symbol_root = rendered_function_symbol_root(function, in_class);
    let unique_suffix = if collision_count > 1 {
        format!("_{}", function.code_reference.unsigned_abs())
    } else {
        String::new()
    };
    let symbol = if nested && occurrence > 1 {
        dart_identifier(&format!("{symbol_root}_{occurrence}"))
    } else if nested {
        dart_identifier(&symbol_root)
    } else {
        dart_identifier(&format!("{symbol_root}{unique_suffix}"))
    };

    writeln!(output).unwrap();
    let display_name = if matches!(owner, "::" | "top_level") {
        clean_symbol(&function.name)
    } else {
        format!("{}.{}", clean_symbol(owner), clean_symbol(&function.name))
    };
    if nested {
        let source = function
            .library_uri
            .as_deref()
            .or_else(|| {
                function
                    .source_location
                    .as_ref()
                    .map(|location| location.path.as_str())
            })
            .unwrap_or("unknown origin");
        let line_note = function
            .source_location
            .as_ref()
            .and_then(|location| location.line)
            .map(|line| format!(" near line {line}"))
            .unwrap_or_default();
        writeln!(
            output,
            "{indent}/// Closure recovered from {}{line_note}.",
            safe_comment(source),
        )
        .unwrap();
    } else if let Some(location) = &function.source_location {
        let source = function.library_uri.as_deref().unwrap_or(&location.path);
        let line = location
            .line
            .map(|line| format!(" near line {line}"))
            .unwrap_or_default();
        writeln!(
            output,
            "{indent}/// Partially reconstructed from {}{line}.",
            safe_comment(source),
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "{indent}/// Partially reconstructed `{}`.",
            safe_comment(&display_name)
        )
        .unwrap();
    }
    let shared_primary = shared_code_primary(program, index, function);
    if let Some(primary) = shared_primary {
        writeln!(
            output,
            "{indent}/// Shares optimized code with `{}`.",
            safe_comment(&qualified_name(primary)),
        )
        .unwrap();
    }
    // The code source map records which callees the optimizer folded into
    // this body; their source statements exist only inside this range.
    if !function.inlined_functions.is_empty() {
        writeln!(
            output,
            "{indent}/// Inlined by the optimizer (statements live inside this body):"
        )
        .unwrap();
        for callee in function.inlined_functions.iter().take(8) {
            let line = callee
                .source_location
                .as_ref()
                .and_then(|location| location.line)
                .map_or_else(String::new, |line| format!(" near line {line}"));
            writeln!(
                output,
                "{indent}///   {}{}",
                safe_comment(&clean_symbol(callee.name.trim_start_matches("get:"))),
                safe_comment(&line),
            )
            .unwrap();
        }
        if function.inlined_functions.len() > 8 {
            writeln!(
                output,
                "{indent}///   … {} more in reports/functions.json.",
                function.inlined_functions.len() - 8
            )
            .unwrap();
        }
    }
    if let Some(vm) = &function.vm_evidence {
        let offset = vm.code_offset.unwrap_or_default();
        let size = vm.code_size.unwrap_or_default();
        match vm.kind.as_deref() {
            Some("AotCodeBoundary") => {
                writeln!(
                    output,
                    "{indent}/// Dart VM verified AOT code range +0x{offset:x} ({size} bytes); semantic owner/name metadata was removed."
                )
                .unwrap();
            }
            Some("SharedAotCodeBoundary") => {
                writeln!(
                    output,
                    "{indent}/// Dart VM verified shared AOT code range +0x{offset:x} ({size} bytes); logical owner is ambiguous."
                )
                .unwrap();
            }
            Some("VmStubCode") => {
                writeln!(
                    output,
                    "{indent}/// Dart VM identified runtime stub `{}` at +0x{offset:x} ({size} bytes).",
                    safe_comment(&vm.name),
                )
                .unwrap();
            }
            Some("DroppedFunctionCode") => {
                let owner = vm.owner.as_deref().unwrap_or("unknown class");
                writeln!(
                    output,
                    "{indent}/// Dart VM recovered dropped function ownership by `{}` at +0x{offset:x} ({size} bytes).",
                    safe_comment(owner),
                )
                .unwrap();
            }
            _ => {
                let signature = vm
                    .user_visible_signature
                    .as_deref()
                    .or(vm.signature.as_deref())
                    .unwrap_or("signature unavailable");
                writeln!(
                    output,
                    "{indent}/// Dart VM verified: {} (code +0x{offset:x}, {size} bytes).",
                    safe_comment(signature),
                )
                .unwrap();
            }
        }
        if let Some(parent) = vm.parent_function_name.as_deref() {
            let qualifier = if vm
                .logical_match_candidate_count
                .is_some_and(|count| count > 1)
            {
                "candidate "
            } else {
                ""
            };
            writeln!(
                output,
                "{indent}/// Closure {qualifier}parent: `{}` (VM object {}).",
                safe_comment(parent),
                vm.parent_function_object_id.unwrap_or_default(),
            )
            .unwrap();
        }
        if vm
            .logical_match_candidate_count
            .is_some_and(|count| count > 1)
        {
            let candidates = vm.logical_match_candidate_count.unwrap_or_default();
            let alternatives = if vm.alternative_parent_functions.is_empty() {
                String::new()
            } else {
                format!(
                    " Alternate lexical parent(s): {}.",
                    vm.alternative_parent_functions
                        .iter()
                        .take(5)
                        .map(|parent| format!("`{}`", safe_comment(parent)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            writeln!(
                output,
                "{indent}/// Shared-code match is ambiguous across {candidates} equally scored logical function(s).{alternatives}"
            )
            .unwrap();
        }
    }

    if let Some((field_name, field_type, expression, is_static)) =
        readable_field_initializer(function)
    {
        let declaration = matching_field_declaration(program, function, &field_name);
        let metadata = declaration.and_then(|declaration| declaration.field_metadata.as_ref());
        let modifier = metadata.map_or_else(
            || {
                if in_class && is_static {
                    "static final".to_owned()
                } else if in_class {
                    "late final".to_owned()
                } else {
                    "final".to_owned()
                }
            },
            |metadata| field_modifiers(metadata, in_class),
        );
        let field_type = metadata
            .and_then(|metadata| metadata.declared_type.as_ref())
            .map_or(field_type, rendered_type);
        writeln!(
            output,
            "{indent}{modifier} {field_type} {} = {expression};",
            dart_identifier(&field_name),
        )
        .unwrap();
        return;
    }

    // An implicit closure's only real behavior is "forward to its parent
    // with the captured receiver" — it is how `obj.method` tear-offs and
    // `main` references compile. Anything the lifter recovers from that few-
    // byte stub is noise, and rendering it as a sibling body invents wrong
    // code (probe EC-6: `e19Ackermann_tearOff` reconstructing garbage calls).
    // Emit a one-liner instead; the `_tearOff` suffix keeps names unique.
    if function.kind == Some(RecoveredFunctionKind::ImplicitClosure) {
        if let Some(parent) = function.lexical_parent.as_deref() {
            let parent_root = readable_function_name(parent.rsplit('.').next().unwrap_or(parent));
            writeln!(
                output,
                "{indent}final {symbol} = {}; // implicit closure forwarding to `{}`",
                dart_identifier(&parent_root),
                safe_comment(parent),
            )
            .unwrap();
            return;
        }
    }

    let (parameters, arguments) = rendered_parameters(function);
    let type_parameters = function
        .signature
        .as_ref()
        .and_then(|signature| signature.resolved.as_ref())
        .map(|resolved| render_type_parameters(&resolved.type_parameters))
        .unwrap_or_default();
    let return_type = rendered_return_type(function, in_class);
    let static_prefix = if is_static_member(function, in_class) {
        "static "
    } else {
        ""
    };
    let async_style = detected_async_style(function);
    // Generator modifiers stay comments: recovered bodies render `return`,
    // not `yield`, and `sync*`/`async*` would make that invalid Dart.
    if let Some(style @ (AsyncStyle::AsyncStar | AsyncStyle::SyncStar)) = async_style {
        let label = if style == AsyncStyle::AsyncStar {
            "async*"
        } else {
            "sync*"
        };
        writeln!(
            output,
            "{indent}/// Snapshot evidence identifies this member as a `{label}` generator; yield structure was not reconstructed."
        )
        .unwrap();
    }
    let async_modifier = match async_style {
        Some(AsyncStyle::AsyncStar) => "",
        Some(AsyncStyle::SyncStar) => "",
        Some(AsyncStyle::Async) => {
            if function
                .vm_evidence
                .as_ref()
                .and_then(|evidence| evidence.is_async)
                .unwrap_or(false)
            {
                " async"
            } else if return_type.starts_with("Future") || return_type == "dynamic" {
                // Static stub-name or wrapper evidence; only claim `async`
                // when the return type is compatible.
                " async"
            } else {
                ""
            }
        }
        None => "",
    };
    let is_accessor = matches!(
        function.kind,
        Some(RecoveredFunctionKind::Getter)
            | Some(RecoveredFunctionKind::Setter)
            | Some(RecoveredFunctionKind::ImplicitGetter)
            | Some(RecoveredFunctionKind::ImplicitSetter)
    ) || function.name.starts_with("get:")
        || function.name.starts_with("set:");
    // VM selectors store operators under their symbols (`==`, `%`, `-`);
    // rendering that through `readable_function_name` invents identifiers
    // like `operator_equals`, which is not Dart. Members whose name IS an
    // operator symbol declare as real `operator` members instead
    // (probe EC-8 / E15). When several Code objects collide on one name
    // (specializations), identity is ambiguous and the suffixed neutral
    // spelling stays, so members never render duplicate declarations.
    let source_operator = if collision_count > 1 {
        None
    } else {
        source_operator_syntax(&function.name)
    };
    let accessor_name: &str = if is_accessor {
        function
            .name
            .strip_prefix("get:")
            .or_else(|| function.name.strip_prefix("set:"))
            .unwrap_or(&function.name)
    } else {
        ""
    };
    let mut parameters_line = parameters;
    if let Some(operator_symbol) = &source_operator {
        writeln!(
            output,
            "{indent}{static_prefix}{return_type} operator {operator_symbol}{parameters_line}{async_modifier} {{"
        )
        .unwrap();
    } else if function.kind == Some(RecoveredFunctionKind::Setter) || function.name.starts_with("set:") {
        writeln!(
            output,
            "{indent}{static_prefix}set {accessor_name}{parameters_line}{async_modifier} {{"
        )
        .unwrap();
    } else if function.kind == Some(RecoveredFunctionKind::Getter)
        || function.name.starts_with("get:")
        || matches!(
            function.kind,
            Some(RecoveredFunctionKind::RecordFieldGetter)
        )
    {
        writeln!(
            output,
            "{indent}{static_prefix}{return_type} get {accessor_name}{async_modifier} {{"
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "{indent}{static_prefix}{return_type} {symbol}{type_parameters}{parameters_line}{async_modifier} {{"
        )
        .unwrap();
    }
    let _ = &mut parameters_line;
    let body_indent = format!("{indent}  ");
    for (closure, occurrence) in nested_closures {
        render_function(
            output,
            program,
            index,
            closure,
            &body_indent,
            in_class,
            1,
            &[],
            true,
            *occurrence,
        );
    }
    // Render the body into a scratch buffer so the synthetic incoming-argument
    // list is only emitted when some statement actually references it.
    let mut body_buffer = String::new();
    {
        let output = &mut body_buffer;
        let mut initial_aliases = BTreeMap::new();
        if constructor_like(function, in_class) {
            writeln!(
                output,
                "{body_indent}final recoveredInstance = aot.unresolvedValue({});",
                dart_string(&format!("allocated {}", clean_symbol(owner))),
            )
            .unwrap();
            initial_aliases.insert("this".to_owned(), "recoveredInstance".to_owned());
        }
        render_readable_literals(output, function, &body_indent);

        // Structure the body into Dart-like control flow.
        let entry = parse_hex(&function.address).unwrap_or_default();
        // Exception-handler rows are authoritative VM evidence: their
        // `handler_pc_offset` names blocks the VM dispatches into on throw.
        // Those blocks must never seed or extend a source loop (EC-2), and
        // any statements stranded in them render under an explicit banner.
        let base = entry;
        let handler_blocks: BTreeSet<u64> = function
            .code_metadata
            .as_ref()
            .map(|metadata| {
                metadata
                    .exception_handlers
                    .iter()
                    .filter(|handler| !handler.is_generated)
                    .map(|handler| base + u64::from(handler.handler_pc_offset))
                    .collect()
            })
            .unwrap_or_default();
        let structured = super::structure::structure_body(
            entry,
            &function.control_flow,
            &function.semantic_statements,
            &handler_blocks,
        );
        // Protected ranges come from pc-descriptor try_index rows joined with
        // the handler table. Only ranges whose handler block actually decoded
        // into statements become renderable try/catch pairs.
        let mut try_regions = Vec::new();
        if let Some(metadata) = function.code_metadata.as_ref() {
            for region in metadata.try_regions() {
                let handler = base + u64::from(region.handler_pc_offset);
                if std::env::var("CLUTTER_DEBUG_TRY").is_ok() {
                    eprintln!(
                        "TRY region idx={} pc 0x{:x}..0x{:x} handler 0x{:x} in_handler_set={}",
                        region.try_index,
                        base + u64::from(region.start_pc_offset),
                        base + u64::from(region.end_pc_offset),
                        handler,
                        handler_blocks.contains(&handler)
                    );
                }
                if !handler_blocks.contains(&handler) {
                    continue;
                }
                try_regions.push(TryRegionView {
                    start: base + u64::from(region.start_pc_offset),
                    end: base + u64::from(region.end_pc_offset),
                    handler,
                    needs_stack_trace: region.needs_stack_trace,
                    has_catch_all: region.has_catch_all,
                });
            }
        }
        let mut emitter = BodyEmitter {
            index,
            aliases: initial_aliases,
            counters: BTreeMap::new(),
            returns_seen: 0,
            claimed_pairs: BTreeSet::new(),
            awaits_rendered: 0,
            // An `async`/`async*` body whose await stubs are unnamed compiles
            // to a state-machine dispatch cycle the structurer sees as a
            // bottom-tested loop with no provable predicate; rendering that
            // as `while (true)` fabricates source control flow that was never
            // written. See `emit_node`'s `While` arm.
            is_async_machine: matches!(
                async_style,
                Some(AsyncStyle::Async | AsyncStyle::AsyncStar)
            ),
            try_regions,
            open_try: None,
            structure_depth: 0,
        };
        if std::env::var("CLUTTER_DEBUG_STRUCTURE").is_ok() {
            eprintln!(
                "STRUCTURE {display_name}: {:?}
unstructured={} branches={} loops={}",
                structured.root,
                structured.unstructured_count,
                structured.structured_branches,
                structured.structured_loops
            );
        }
        emitter.emit_node(output, function, &structured.root, &body_indent);
        // A protected range whose handler never decoded still needs a closed
        // clause; keep the guard explicit instead of emitting invalid Dart.
        if let Some(region_index) = emitter.open_try.take() {
            let region = emitter.try_regions[region_index];
            if region.has_catch_all || !region.needs_stack_trace {
                writeln!(output, "{body_indent}}} catch (e) {{").unwrap();
            } else {
                writeln!(output, "{body_indent}}} catch (e, stackTrace) {{").unwrap();
            }
            writeln!(
                output,
                "{body_indent}  aot.unresolvedRegion('catch body not recovered', <dynamic>[]);"
            )
            .unwrap();
            writeln!(output, "{body_indent}}}").unwrap();
        }
        // Statements stranded in machine regions the structurer could not reach
        // (async state dispatch, table jumps) still surface, in address order.
        let missed: Vec<usize> = function
            .semantic_statements
            .iter()
            .enumerate()
            .filter(|(statement_index, statement)| {
                !structured.claimed[*statement_index]
                    && match statement {
                        SemanticStatement::ResolvedCall { target, .. } => {
                            is_await_boundary(target)
                                || (!is_vm_runtime_helper(target) && meaningful_call_target(target))
                        }
                        SemanticStatement::FieldWrite { .. } => true,
                        _ => false,
                    }
            })
            .map(|(index, _)| index)
            .collect();
        let missed_meaningful = !missed.is_empty();
        if !missed.is_empty() {
            writeln!(
                output,
                "{body_indent}// Statements recovered from unreached machine regions:"
            )
            .unwrap();
            for statement_index in &missed {
                emitter.emit_statement(
                    output,
                    function,
                    &function.semantic_statements[*statement_index],
                    &body_indent,
                );
            }
        }
        if !nested
            && matches!(async_style, Some(AsyncStyle::Async))
            && emitter.awaits_rendered == 0
            && function
                .vm_evidence
                .as_ref()
                .and_then(|evidence| evidence.is_async)
                .is_none()
        {
            writeln!(
            output,
            "{body_indent}// Body compiled to an async state machine; await boundaries are unnamed in this snapshot."
        )
        .unwrap();
        }
        if !nested {
            render_dynamic_dispatch_evidence(output, function, &body_indent);
        }
        if nested {
        } else if structured.structured_branches + structured.structured_loops == 0 {
            render_control_flow_summary(output, function, &body_indent);
        } else {
            writeln!(
            output,
            "{body_indent}// {} branch region(s) and {} loop(s) reconstructed; exact machine structure remains in reports/functions.json.",
            structured.structured_branches,
            structured.structured_loops,
        )
        .unwrap();
        }
        let has_return = contains_return(&structured.root, &function.semantic_statements);
        if !has_return || missed_meaningful {
            writeln!(
                output,
                "{body_indent}return aot.unresolvedRegion({}, <dynamic>[]);",
                dart_string(&format!("Remaining behavior of {display_name}"))
            )
            .unwrap();
        }
    } // end scratch-buffer scope
    // Only surface the synthetic argument bundle when the recovered body
    // references it; otherwise it is noise relative to the original source.
    let has_args = contains_word(&body_buffer, "args");
    if has_args && let Some(arguments) = &arguments {
        writeln!(output, "{body_indent}final args = <dynamic>[{arguments}];").unwrap();
        writeln!(output).unwrap();
    }
    // Final tier-safe pass: rewrite free machine identifiers so the emitted
    // function body only ever names declared locals, parameters, or the
    // explicit `aot.` unresolved helpers.
    {
        let parameter_names = arguments.as_deref().unwrap_or_default();
        let bound = collect_declared_identifiers(&body_buffer, parameter_names);
        let sanitized = sanitize_free_machine_identifiers(&body_buffer, &bound);
        write!(output, "{sanitized}").unwrap();
    }

    writeln!(output, "{indent}}}").unwrap();
}

fn parse_hex(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

/// Final per-function rendering pass: rewrite free machine identifiers into
/// explicit unresolved forms so every emitted file parses as Dart.
///
/// The lifter mints `local{:x}` names for stack slots read before being
/// written and the register printer emits bare `xN`; most flow through
/// `render_expression`, but paths that splice raw text (interpolation parts,
/// promoted loop predicates, evidence comments) can leak them. Anything not
/// provably declared renders as `aot.unresolvedValue`/`unresolvedRegister`,
/// and bare `snapshotRef(`/`snapshotInstance(` helpers gain their missing
/// `aot.` qualifier (probe EC-5).
fn sanitize_free_machine_identifiers(body: &str, bound: &BTreeSet<String>) -> String {
    const KEEP_WORDS: &[&str] = &[
        "this", "null", "true", "false", "super",
    ];
    let mut output = String::with_capacity(body.len());
    let mut characters = body.char_indices().peekable();
    // Lexer state so comment and string contents pass through untouched,
    // except `${...}` interpolation holes which hold real expressions.
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string: Option<char> = None;
    while let Some((index, character)) = characters.next() {
        if in_line_comment {
            output.push(character);
            if character == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_block_comment {
            output.push(character);
            if character == '*'
                && characters.peek().is_some_and(|(_, next)| *next == '/')
            {
                output.push('/');
                characters.next();
                in_block_comment = false;
            }
            continue;
        }
        if let Some(quote) = in_string {
            output.push(character);
            match character {
                '\\' => {
                    if let Some((_, escaped)) = characters.next() {
                        output.push(escaped);
                    }
                }
                '$' => {
                    // Interpolation hole: the embedded expression is real
                    // Dart, so its free machine identifiers must be
                    // rewritten too (this is where `local70` leaked).
                    if characters.peek().is_some_and(|(_, next)| *next == '{') {
                        output.push('{');
                        characters.next();
                        let mut hole = String::new();
                        let mut depth = 1usize;
                        for (_, hole_character) in characters.by_ref() {
                            match hole_character {
                                '{' => {
                                    depth += 1;
                                    hole.push(hole_character);
                                }
                                '}' => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                    hole.push(hole_character);
                                }
                                _ => hole.push(hole_character),
                            }
                        }
                        let sanitized_hole =
                            sanitize_free_machine_identifiers(&hole, bound);
                        output.push_str(&sanitized_hole);
                        output.push('}');
                    }
                }
                closing if closing == quote => in_string = None,
                _ => {}
            }
            continue;
        }
        match character {
            '/' if characters.peek().is_some_and(|(_, next)| *next == '/') => {
                in_line_comment = true;
                output.push(character);
            }
            '/' if characters.peek().is_some_and(|(_, next)| *next == '*') => {
                in_block_comment = true;
                output.push(character);
            }
            '\'' | '"' => {
                in_string = Some(character);
                output.push(character);
            }
            character if character.is_ascii_alphabetic() || character == '_' || character == '$'
            => {
                let start = index;
                let mut end = index + character.len_utf8();
                while let Some((_, next)) = characters.peek() {
                    if next.is_ascii_alphanumeric() || *next == '_' || *next == '$' {
                        end += next.len_utf8();
                        characters.next();
                    } else {
                        break;
                    }
                }
                let token = &body[start..end];
                let preceded_by_dot = body[..start]
                    .chars()
                    .rev()
                    .find(|character| !character.is_whitespace())
                    == Some('.');
                if preceded_by_dot
                    || KEEP_WORDS.contains(&token)
                    || bound.contains(token)
                {
                    output.push_str(token);
                    continue;
                }
                // Machine stack-slot temporaries: `local` + lowercase hex.
                if let Some(slot) = token.strip_prefix("local")
                    && !slot.is_empty()
                    && slot.chars().all(|character| character.is_ascii_hexdigit())
                {
                    output.push_str(&format!(
                        "aot.unresolvedValue({})",
                        dart_string(&format!("slot 0x{slot}"))
                    ));
                    continue;
                }
                // Integer registers: ARM64 (`xN`, with `wN` canonicalized to
                // `x` upstream) and ARM32 (`rN`). VFP registers did not leak
                // in probe outputs; extend here if a future snapshot shows
                // them.
                if let Some(digits) = token
                    .strip_prefix('x')
                    .or_else(|| token.strip_prefix('r'))
                    .filter(|rest| !rest.is_empty())
                    && token.len() <= 4
                    && digits.chars().all(|character| character.is_ascii_digit())
                {
                    output.push_str(&format!("aot.unresolvedRegister('{token}')"));
                    continue;
                }
                // Pool-label helpers that skipped the `aot.` qualifier logic.
                if token == "snapshotRef" || token == "snapshotInstance" {
                    output.push_str("aot.");
                    output.push_str(token);
                    continue;
                }
                output.push_str(token);
            }
            character => output.push(character),
        }
    }
    output
}

/// Collects identifiers the sanitizer may treat as bound: every local the
/// renderer itself declared plus the function's incoming parameters.
fn collect_declared_identifiers(body: &str, parameters: &str) -> BTreeSet<String> {
    let mut bound = BTreeSet::new();
    for name in parameters.split(',') {
        let name = name.trim();
        if valid_dart_identifier(name) {
            bound.insert(name.to_owned());
        }
    }
    bound.insert("args".to_owned());
    bound.insert("recoveredInstance".to_owned());
    // Walk forward through `final ` declarations. The cursor always advances
    // by at least the keyword length, so scanning cannot stall on bodies
    // with many locals.
    let mut rest = body;
    while let Some(position) = rest.find("final ") {
        rest = &rest[position + "final ".len()..];
        let declaration = rest.trim_start();
        let mut end = 0usize;
        for character in declaration.chars() {
            if character.is_ascii_alphanumeric()
                || character == '_'
                || character == '$'
            {
                end += character.len_utf8();
            } else {
                break;
            }
        }
        if end > 0 && end <= declaration.len() {
            bound.insert(declaration[..end].to_owned());
        }
    }
    bound
}

fn contains_return(
    node: &super::structure::StructureNode,
    statements: &[SemanticStatement],
) -> bool {
    use super::structure::StructureNode;
    match node {
        StructureNode::Return(_) => true,
        StructureNode::Linear(indices) => indices
            .last()
            .and_then(|last| statements.get(*last))
            .is_some_and(|statement| statement.is_return()),
        StructureNode::If {
            then_body,
            else_body,
            ..
        } => {
            contains_return(then_body, statements)
                || else_body
                    .as_ref()
                    .is_some_and(|body| contains_return(body, statements))
        }
        StructureNode::While { body, .. } => contains_return(body, statements),
        StructureNode::UnresolvedPredicate(_) | StructureNode::CatchHandler(_) => false,
        StructureNode::Block(children) => children
            .iter()
            .any(|child| contains_return(child, statements)),
    }
}

/// True when every rendered path through `node` ends in a proven machine
/// return, so any following sibling in a block is unreachable code.
/// Word-boundary containment test used to decide whether the synthetic
/// incoming-argument local is referenced anywhere in a rendered body.
fn contains_word(buffer: &str, word: &str) -> bool {
    let boundary = |character: Option<char>| {
        character.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
    };
    let mut rest = buffer;
    while let Some(position) = rest.find(word) {
        if boundary(rest[..position].chars().next_back())
            && boundary(rest[position + word.len()..].chars().next())
        {
            return true;
        }
        rest = &rest[position + word.len()..];
    }
    false
}

fn node_ends_in_terminal_return(
    node: &super::structure::StructureNode,
    function: &crate::model::RecoveredFunction,
) -> bool {
    use super::structure::StructureNode;
    let statements = &function.semantic_statements;
    match node {
        StructureNode::Return(_) => true,
        StructureNode::Linear(indices) => indices
            .last()
            .and_then(|last| statements.get(*last))
            .is_some_and(|statement| statement.is_return()),
        StructureNode::If {
            then_body,
            else_body,
            ..
        } => {
            node_ends_in_terminal_return(then_body, function)
                && else_body
                    .as_ref()
                    .is_some_and(|body| node_ends_in_terminal_return(body, function))
        }
        // A loop may run zero iterations, so control flow can fall through.
        StructureNode::While { .. } => false,
        StructureNode::UnresolvedPredicate(_) | StructureNode::CatchHandler(_) => false,
        StructureNode::Block(children) => children
            .iter()
            .next_back()
            .is_some_and(|child| node_ends_in_terminal_return(child, function)),
    }
}

/// One protected try range in body coordinates (absolute addresses), with the
/// catch handler entry the VM dispatches into.
#[derive(Clone, Copy)]
struct TryRegionView {
    start: u64,
    end: u64,
    /// Handler entry address; kept for cross-checking against the structurer's
    /// handler-block set (regions whose handler produced no statements are
    /// filtered out before rendering).
    #[allow(dead_code)]
    handler: u64,
    needs_stack_trace: bool,
    has_catch_all: bool,
}

struct BodyEmitter<'a> {
    index: &'a RenderIndex,
    aliases: BTreeMap<String, String>,
    counters: BTreeMap<String, usize>,
    returns_seen: usize,
    claimed_pairs: BTreeSet<usize>,
    awaits_rendered: usize,
    /// Body belongs to an `async`/`async*` member: unpredicated loops are
    /// async-machine dispatch cycles, not source `while` loops.
    is_async_machine: bool,
    /// Protected ranges recovered from pc-descriptor try_index rows; empty
    /// when the body compiled without try blocks.
    try_regions: Vec<TryRegionView>,
    /// Index into `try_regions` of the currently open `try {`, if any. It
    /// closes at its handler (`CatchHandler`) or, unreachable-guard, at body end.
    open_try: Option<usize>,
    /// Structural nesting depth while emitting; try brackets are opened only
    /// at depth 0 so a `catch` can never close across an `if`/`while` brace
    /// boundary and produce invalid Dart.
    structure_depth: usize,
}

/// First and last statement address of any renderable node, for try-range
/// membership checks. Returns `None` for nodes carrying no statements.
fn node_statement_span(
    function: &RecoveredFunction,
    node: &super::structure::StructureNode,
) -> Option<(u64, u64)> {
    use super::structure::StructureNode;
    let parse = |index: usize| -> Option<u64> {
        Some(parse_hex(function.semantic_statements.get(index)?.address()).unwrap_or_default())
    };
    match node {
        StructureNode::Linear(indices) => {
            let first = indices.first()?;
            let last = indices.last()?;
            Some((parse(*first)?, parse(*last)?))
        }
        StructureNode::Return(index) => {
            let address = parse(*index)?;
            Some((address, address))
        }
        StructureNode::If { then_body, else_body, .. } => {
            let mut spans = vec![node_statement_span(function, then_body)];
            if let Some(else_body) = else_body {
                spans.push(node_statement_span(function, else_body));
            }
            let mut spans = spans.into_iter().flatten().collect::<Vec<_>>();
            if spans.is_empty() {
                return None;
            }
            spans.sort();
            Some((spans[0].0, spans[spans.len() - 1].1))
        }
        StructureNode::While { body, .. } => node_statement_span(function, body),
        StructureNode::Block(children) => {
            let spans = children
                .iter()
                .filter_map(|child| node_statement_span(function, child))
                .collect::<Vec<_>>();
            let first = spans.iter().map(|span| span.0).min()?;
            let last = spans.iter().map(|span| span.1).max()?;
            Some((first, last))
        }
        StructureNode::UnresolvedPredicate(_) | StructureNode::CatchHandler(_) => None,
    }
}

fn sanitize_semantic_key(target: &str) -> String {
    target
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

impl<'a> BodyEmitter<'a> {
    /// Wraps a root-level child whose whole statement span lies inside an
    /// unopened protected range. Bracketing at this level is safe: the
    /// matching `CatchHandler` is always a sibling at the same depth, so the
    /// `catch` clause can never close across an `if`/`while` brace boundary.
    fn try_wrap_root_child(
        &mut self,
        output: &mut String,
        function: &RecoveredFunction,
        node: &super::structure::StructureNode,
        indent: &str,
    ) -> bool {
        let Some((first, _last)) = node_statement_span(function, node) else {
            return false;
        };
        let Some(region_index) = self.try_regions.iter().position(|region| {
            self.open_try.is_none() && region.start <= first && first < region.end
        }) else {
            return false;
        };
        self.open_try = Some(region_index);
        let region = self.try_regions[region_index];
        writeln!(
            output,
            "{indent}try {{ // protected range 0x{:x}..0x{:x}",
            region.start,
            region.end
        )
        .unwrap();
        self.emit_node(output, function, node, indent);
        false
    }

    fn emit_node(
        &mut self,
        output: &mut String,
        function: &RecoveredFunction,
        node: &super::structure::StructureNode,
        indent: &str,
    ) {
        use super::structure::StructureNode;
        match node {
            StructureNode::Block(children) => {
                self.structure_depth += 1;
                for child in children {
                    // Root-level children (depth becomes 1 inside) are the one
                    // place a try bracket is guaranteed to pair with its
                    // handler sibling without crossing structured braces.
                    if self.structure_depth == 1
                        && self.try_wrap_root_child(output, function, child, indent)
                    {
                        continue;
                    }
                    self.emit_node(output, function, child, indent);
                    // A child whose every path ends in a machine return makes
                    // its following siblings unreachable; rendering them as
                    // reachable Dart statements would be misleading.
                    if node_ends_in_terminal_return(child, function) {
                        break;
                    }
                }
                self.structure_depth -= 1;
            }
            StructureNode::Linear(statements) => {
                let mut position = 0usize;
                while position < statements.len() {
                    let index = statements[position];
                    let statement = &function.semantic_statements[index];
                    // AOT lowers `Class(...)` into an allocator call followed
                    // by the real constructor consuming the new instance.
                    // Render the pair as one constructor invocation.
                    if let SemanticStatement::ResolvedCall {
                        target: allocator_target,
                        arguments: allocator_arguments,
                        ..
                    } = statement
                        && position + 1 < statements.len()
                    {
                        let SemanticStatement::ResolvedCall {
                            target: ctor_target,
                            arguments: ctor_arguments,
                            ..
                        } = &function.semantic_statements[statements[position + 1]]
                        else {
                            self.emit_statement(output, function, statement, indent);
                            position += 1;
                            continue;
                        };
                        let allocator_class = allocation_class_name(allocator_target)
                            .or_else(|| constructor_class_name(allocator_target));
                        let ctor_class = constructor_class_name(ctor_target);
                        let consumes_allocation = ctor_arguments.first().is_some_and(|first| {
                            let expected =
                                format!("{}_result", sanitize_semantic_key(allocator_target));
                            first == &expected
                                || first
                                    == &format!(
                                        "{}_result",
                                        crate::analysis::readable_snapshot_name(allocator_target)
                                            .chars()
                                            .map(|character| {
                                                if character.is_ascii_alphanumeric()
                                                    || character == '_'
                                                {
                                                    character
                                                } else {
                                                    '_'
                                                }
                                            })
                                            .collect::<String>()
                                    )
                        });
                        if let (Some(allocator_class), Some(ctor_class)) =
                            (allocator_class, ctor_class)
                            && allocator_class == ctor_class
                            && consumes_allocation
                        {
                            self.emit_constructor_pair(
                                output,
                                ctor_target,
                                ctor_arguments,
                                allocator_target,
                                allocator_arguments,
                                indent,
                            );
                            self.claimed_pairs.insert(statements[position + 1]);
                            position += 2;
                            continue;
                        }
                    }
                    self.emit_statement(output, function, statement, indent);
                    position += 1;
                    // A machine return ends this straight-line region; any
                    // lifted statements after it came from unreachable code
                    // and must not render as reachable Dart.
                    if matches!(statement, SemanticStatement::Return { .. }) {
                        break;
                    }
                }
            }
            StructureNode::Return(statement) => {
                self.returns_seen += 1;
                let expression = match &function.semantic_statements[*statement] {
                    SemanticStatement::Return { expression, .. } => expression.clone(),
                    _ => String::new(),
                };
                let rendered = self.render_expression(&expression);
                if rendered == "null" || rendered.is_empty() {
                    writeln!(output, "{indent}return;").unwrap();
                } else {
                    writeln!(output, "{indent}return {rendered};").unwrap();
                }
            }
            StructureNode::UnresolvedPredicate(condition) => {
                // The structurer proved only that a branch existed here; both
                // arms carried no renderable statement. Keep the evidence as
                // a comment instead of an executable `if`.
                writeln!(
                    output,
                    "{indent}// unresolved predicate: {}{}",
                    self.clean_condition(condition),
                    confidence_note(EvidenceConfidence::Low),
                )
                .unwrap();
            }
            StructureNode::CatchHandler(body) => {
                match self.open_try.take() {
                    Some(region_index) => {
                        let region = &self.try_regions[region_index];
                        let catch_head = if region.has_catch_all || !region.needs_stack_trace {
                            format!("{indent}}} catch (e) {{")
                        } else {
                            format!("{indent}}} catch (e, stackTrace) {{")
                        };
                        writeln!(output, "{catch_head}").unwrap();
                    }
                    None => {
                        // The handler decoded without a renderable protected
                        // range: keep the explicit banner rather than invent
                        // a try block.
                        writeln!(
                            output,
                            "{indent}// catch handler (type unresolved): the VM dispatches here on throw;"
                        )
                        .unwrap();
                        writeln!(
                            output,
                            "{indent}// try/catch structure is not reconstructed in this snapshot."
                        )
                        .unwrap();
                    }
                }
                self.emit_node(output, function, body, indent);
                if self.open_try.is_none() {
                    // Close the catch clause opened above. A nested try opened
                    // *inside* the handler stays open and closes at its own
                    // handler or at function end.
                    writeln!(output, "{indent}}}" ).unwrap();
                }
            }
            StructureNode::While {
                condition,
                confidence,
                body,
            } => {
                // A promoted exit-test predicate names values that the body
                // defines (for example the `moveNext()` call result). Emit
                // the body into a scratch buffer first so alias substitution
                // sees those names, then write the header.
                let mut scratch = String::new();
                self.emit_node(&mut scratch, function, body, &format!("{indent}  "));
                match condition {
                    Some(condition) => {
                        let condition = self.clean_condition(condition);
                        writeln!(output, "{indent}while ({condition}) {{").unwrap();
                    }
                    None if self.is_async_machine => {
                        // The unpredicated cycle is the async state-machine
                        // dispatch, not a source `while`. Rendering the body
                        // statements without a fabricated loop header keeps
                        // the recovered fragments and drops the invention;
                        // await structure is documented separately when the
                        // stubs stay unnamed.
                        writeln!(
                            output,
                            "{indent}// Async state-machine dispatch region (await boundaries unnamed in this snapshot):"
                        )
                        .unwrap();
                    }
                    None => {
                        writeln!(
                            output,
                            "{indent}// Loop shape recovered without a provable predicate{}.",
                            confidence_note(*confidence)
                        )
                        .unwrap();
                        writeln!(output, "{indent}while (true) {{").unwrap();
                    }
                }
                write!(output, "{scratch}").unwrap();
                if condition.is_some() || !self.is_async_machine {
                    writeln!(output, "{indent}}}").unwrap();
                }
            }
            StructureNode::If {
                condition,
                confidence,
                then_body,
                else_body,
            } => {
                let condition = self.clean_condition(condition);
                let note = confidence_note(*confidence);
                let note = if note.is_empty() {
                    String::new()
                } else {
                    format!(" {note}")
                };
                writeln!(output, "{indent}if ({condition}) {{{note}").unwrap();
                self.emit_node(output, function, then_body, &format!("{indent}  "));
                if let Some(else_body) = else_body {
                    match else_body.as_ref() {
                        StructureNode::Linear(statements) if statements.is_empty() => {}
                        _ => {
                            writeln!(output, "{indent}}} else {{").unwrap();
                            self.emit_node(output, function, else_body, &format!("{indent}  "));
                        }
                    }
                }
                writeln!(output, "{indent}}}").unwrap();
            }
        }
    }

    /// Renders raw lifted predicates as Dart-like conditions.
    fn clean_condition(&self, condition: &str) -> String {
        let mut rendered = condition.replace("(1 << 0)", "1");
        rendered = self.substitute_aliases(&rendered);
        // Tagged-pointer smi checks stay explicit but readable.
        if let Some(rest) = rendered.strip_prefix("(this.") {
            let _ = rest;
        }
        rendered
    }

    fn substitute_aliases(&self, text: &str) -> String {
        let mut replacements = self.aliases.iter().collect::<Vec<_>>();
        replacements.sort_by_key(|(raw, _)| std::cmp::Reverse(raw.len()));
        let mut rendered = text.to_owned();
        for (raw, alias) in replacements {
            rendered = replace_identifier_like(&rendered, raw, alias);
        }
        rendered
    }

    fn render_expression(&mut self, expression: &str) -> String {
        if let Some(alias) = self.aliases.get(expression) {
            return alias.clone();
        }
        let substituted = self.substitute_aliases(expression);
        let compact = substituted.replace(' ', "_");
        if (compact.starts_with("sub_")
            || compact.starts_with("_iso_stub_")
            || compact.starts_with("stub_"))
            && substituted.ends_with("_result")
        {
            return format!("aot.unresolvedValue({})", dart_string("shared-code result"));
        }
        render_readable_expression(&substituted, &self.aliases)
    }

    fn emit_statement(
        &mut self,
        output: &mut String,
        function: &RecoveredFunction,
        statement: &SemanticStatement,
        indent: &str,
    ) {
        match statement {
            SemanticStatement::ResolvedCall {
                target, arguments, ..
            } => {
                // Async state machines surface each `await` as an
                // `AwaitStub` call carrying the suspended future.
                if is_await_boundary(target) {
                    let awaited = arguments.first().map(|value| self.render_expression(value));
                    let known_local = awaited.as_deref().is_some_and(|value| {
                        value != "null"
                            && !value.is_empty()
                            && self.aliases.values().any(|alias| alias == value)
                    });
                    match (known_local, awaited.as_deref()) {
                        (true, Some(value)) => {
                            writeln!(output, "{indent}await {value};").unwrap();
                        }
                        _ => writeln!(
                            output,
                            "{indent}await aot.unresolvedValue('awaited future');"
                        )
                        .unwrap(),
                    }
                    self.awaits_rendered += 1;
                    return;
                }
                if target.ends_with("InitAsyncStub") || is_vm_runtime_helper(target) {
                    return;
                }
                if !meaningful_call_target(target) {
                    return;
                }
                let callee = self.index.callee(target);
                let expression = self.call_expression(target, arguments, callee, function);
                let stem =
                    variable_stem_for(target, callee, constructor_class_name(target).is_some());
                let variable = next_variable_name(&stem, &mut self.counters);
                if expression.ends_with(';') {
                    writeln!(output, "{indent}{expression}").unwrap();
                } else if indent.len() + variable.len() + expression.len() + 10 <= 120 {
                    writeln!(output, "{indent}final {variable} = {expression};").unwrap();
                } else {
                    writeln!(output, "{indent}final {variable} =").unwrap();
                    writeln!(output, "{indent}  {expression};").unwrap();
                }
                self.aliases
                    .insert(format!("{}_result", sanitize_key(target)), variable.clone());
                // Also alias by the raw key used in lifted expressions.
                self.aliases.insert(sanitize_key(target), variable);
            }
            SemanticStatement::FieldWrite {
                receiver,
                field,
                value,
                offset,
                ..
            } => {
                let receiver = self.render_expression(receiver);
                let value = self.render_expression(value);
                writeln!(
                    output,
                    "{indent}{receiver}.{} = {value}; // AOT field store +0x{offset:x}",
                    dart_identifier(&clean_symbol(field)),
                )
                .unwrap();
            }
            // Field reads flow through register expressions; interpolations
            // surface through their consumers.
            SemanticStatement::FieldRead { .. } | SemanticStatement::StringInterpolation { .. } => {
            }
            SemanticStatement::Return { expression, .. } => {
                self.returns_seen += 1;
                let rendered = self.render_expression(expression);
                if rendered == "null" || rendered.is_empty() {
                    writeln!(output, "{indent}return;").unwrap();
                } else {
                    writeln!(output, "{indent}return {rendered};").unwrap();
                }
            }
            SemanticStatement::Condition { .. } => {}
        }
    }

    /// Applies the named-argument join for optional named parameters. AOT
    /// callers fill every named slot up to the last provided one with null
    /// dummies; leading/trailing dummies are dropped and the remaining
    /// values zip onto the declared names in order.
    fn named_arguments(&self, callee: Option<&CalleeInfo>, visible: Vec<String>) -> Vec<String> {
        let optional_named = callee
            .map(|callee| callee.optional_named.clone())
            .unwrap_or_default();
        let optional_positional = callee
            .map(|callee| callee.optional_positional)
            .unwrap_or_default();
        let fixed = callee
            .and_then(|callee| callee.fixed_parameter_count)
            .map(|fixed| fixed.min(visible.len()))
            .unwrap_or(visible.len());
        let mut positional: Vec<String> = visible[..fixed].to_vec();
        let rest: Vec<String> = visible[visible.len().min(fixed)..].to_vec();
        let mut pairs: Vec<String> = Vec::new();
        let known_named = !optional_named.is_empty() && optional_positional == 0;
        let known_positional_optional = optional_positional > 0;
        if known_named || known_positional_optional {
            // AOT callers fill every optional slot up to the last provided
            // one; leading/trailing null dummies are omitted arguments.
            let mut values = rest;
            while values.first().map(|value| value == "null").unwrap_or(false) && values.len() > 1 {
                values.remove(0);
            }
            while values.last().map(|value| value == "null").unwrap_or(false) {
                values.pop();
            }
            for (index, value) in values.iter().enumerate() {
                if known_named {
                    match optional_named.get(index) {
                        Some((name, _)) if !name.is_empty() => {
                            pairs.push(format!("{name}: {value}"));
                        }
                        _ => pairs.push(value.clone()),
                    }
                } else {
                    pairs.push(value.clone());
                }
            }
            // Keep calls arity-honest: unrecovered arguments become explicit
            // placeholders instead of silently short calls.
            if known_named {
                let covered = values.len();
                for name in optional_named.iter().skip(covered) {
                    let (name, required) = name;
                    if *required && !name.is_empty() {
                        pairs.push(format!("{name}: aot.unresolvedValue('argument')"));
                    }
                }
            }
        } else if let Some(fixed_count) = callee.and_then(|callee| callee.fixed_parameter_count) {
            while positional.len() < fixed_count {
                positional.push("aot.unresolvedValue('argument')".to_owned());
            }
            positional.extend(rest);
            return positional;
        } else {
            pairs.extend(rest);
        }
        positional.extend(pairs);
        positional
    }

    /// Renders an allocator + constructor call pair as one Dart constructor.
    fn emit_constructor_pair(
        &mut self,
        output: &mut String,
        ctor_target: &str,
        ctor_arguments: &[String],
        allocator_target: &str,
        allocator_arguments: &[String],
        indent: &str,
    ) {
        let class = constructor_class_name(ctor_target)
            .or_else(|| allocation_class_name(allocator_target))
            .unwrap_or_else(|| "Unknown".to_owned());
        let suffix = constructor_suffix(ctor_target);
        // Drop the leading new-instance argument, plus any argument the
        // allocator call itself already received (instantiator type
        // arguments and class metadata are VM plumbing, never source-level
        // constructor arguments).
        let allocator_args: std::collections::BTreeSet<&String> =
            allocator_arguments.iter().collect();
        let visible: Vec<String> = ctor_arguments
            .iter()
            .enumerate()
            .filter(|(index, argument)| *index != 0 && !allocator_args.contains(argument))
            .map(|(_, argument)| self.render_expression(argument))
            .collect();
        let callee = self.index.callee(ctor_target);
        let call_arguments = self.named_arguments(callee, visible);
        let name = dart_identifier(&clean_symbol(&class));
        let name = if suffix.is_empty() || suffix == class {
            name
        } else {
            format!("{name}.{}", dart_identifier(&clean_symbol(&suffix)))
        };
        let variable = next_variable_name(
            &lower_camel_identifier(&dart_identifier(&clean_symbol(&class))),
            &mut self.counters,
        );
        let expression = format!("{name}({})", call_arguments.join(", "));
        writeln!(output, "{indent}final {variable} = {expression};").unwrap();
        for key in [
            format!("{}_result", sanitize_semantic_key(allocator_target)),
            format!("{}_result", sanitize_key(allocator_target)),
            format!("{}_result", sanitize_semantic_key(ctor_target)),
            format!("{}_result", sanitize_key(ctor_target)),
        ] {
            self.aliases.insert(key, variable.clone());
        }
    }

    /// Renders a call with receiver/property syntax and named arguments when
    /// the callee's declaration proves them.
    fn call_expression(
        &mut self,
        target: &str,
        arguments: &[String],
        callee: Option<&CalleeInfo>,
        function: &RecoveredFunction,
    ) -> String {
        let rendered_arguments: Vec<String> = arguments
            .iter()
            .map(|argument| self.render_expression(argument))
            .collect();
        let implicit = callee
            .and_then(|callee| callee.implicit_parameter_count)
            .unwrap_or(0);
        let kind = callee.and_then(|callee| callee.kind);
        let normalized = normalized_call_target(target).unwrap_or_else(|| target.to_owned());

        // Getter / setter property syntax.
        let member = normalized.rsplit('.').next().unwrap_or(&normalized);
        const RECEIVER_PLACEHOLDER: &str = "aot.unresolvedValue('receiver')";
        if matches!(kind, Some(RecoveredFunctionKind::Getter))
            || matches!(kind, Some(RecoveredFunctionKind::ImplicitGetter))
            || member.strip_prefix("get:").is_some()
        {
            let property = clean_symbol(member.trim_start_matches("get:"));
            if rendered_arguments.first().map(String::as_str) == Some("this") {
                return property;
            }
            let receiver = rendered_arguments
                .first()
                .cloned()
                .unwrap_or_else(|| RECEIVER_PLACEHOLDER.to_owned());
            return format!("{receiver}.{property}");
        }
        if matches!(kind, Some(RecoveredFunctionKind::Setter)) || member.starts_with("set:") {
            let property = clean_symbol(member.trim_start_matches("set:"));
            if rendered_arguments.first().map(String::as_str) == Some("this") {
                let value = rendered_arguments
                    .get(1)
                    .cloned()
                    .unwrap_or_else(|| "aot.unresolvedValue('setter value')".to_owned());
                return format!("{property} = {value}");
            }
            let receiver = rendered_arguments
                .first()
                .cloned()
                .unwrap_or_else(|| RECEIVER_PLACEHOLDER.to_owned());
            let value = rendered_arguments
                .get(1)
                .cloned()
                .unwrap_or_else(|| "aot.unresolvedValue('setter value')".to_owned());
            return format!("{receiver}.{property} = {value}");
        }

        // Split the receiver for instance members. An own-`this` receiver is
        // implicit in Dart source and must not render as an expression.
        let (receiver_part, visible) = if implicit > 0 && !rendered_arguments.is_empty() {
            let receiver_is_this = rendered_arguments[0] == "this";
            (
                (!receiver_is_this).then(|| rendered_arguments[0].clone()),
                rendered_arguments[1..].to_vec(),
            )
        } else {
            (None, rendered_arguments.clone())
        };

        // Operator invocations (`EdgeVector.+`, `.==`, `.<`, ...) render as
        // infix expressions when both operands survived.
        if let Some(operator) = operator_member_name(member)
            && let (Some(receiver), Some(operand)) = (receiver_part.clone(), visible.first())
        {
            return format!("{receiver} {operator} {operand}");
        }

        // Named-argument join from the callee signature.
        let call_arguments = self.named_arguments(callee, visible);

        // VM container allocators are not source-level constructors: a
        // growable-list allocation is what `<T>[]` lowers to.
        if let Some(literal) = internal_allocator_literal(&normalized) {
            return literal;
        }

        // An own-`this` receiver on an otherwise-unresolved callee still
        // renders as an implicit-this member call instead of passing `this`
        // as ordinary data.
        if rendered_arguments.first().map(String::as_str) == Some("this")
            && valid_dart_identifier(member)
        {
            let rest = rendered_arguments[1..].join(", ");
            return format!("{member}({rest})");
        }

        // Constructor syntax (`Class.named(...)`), instance methods, statics.
        if let Some(class) = constructor_class_name(target) {
            let suffix = constructor_suffix(target);
            let name = if suffix.is_empty() {
                class.clone()
            } else {
                format!(
                    "{}.{}",
                    dart_identifier(&clean_symbol(&class)),
                    dart_identifier(&clean_symbol(&suffix))
                )
            };
            let name = if suffix.is_empty() {
                dart_identifier(&clean_symbol(&class))
            } else {
                name
            };
            return format!("{name}({})", call_arguments.join(", "));
        }
        let direct = if is_internal_runtime_target(&normalized) {
            None
        } else {
            readable_direct_call_target(target, function.owner.as_deref())
        };
        let direct_known = direct.is_some();
        let internal = is_internal_runtime_target(&normalized);
        let target_name = direct.unwrap_or_else(|| friendly_invoke_target(target));
        if !internal && direct_known && receiver_part.is_none() && !target_name.contains('.') {
            return format!("{target_name}({})", call_arguments.join(", "));
        }
        if !internal && let Some(receiver) = receiver_part.clone() {
            let method = target_name.rsplit('.').next().unwrap_or(&target_name);
            if valid_dart_identifier(method) || method.contains(':') {
                let method = method.trim_start_matches("get:");
                if call_arguments.is_empty()
                    && (method.starts_with("get:")
                        || normalized
                            .rsplit('.')
                            .next()
                            .unwrap_or("")
                            .starts_with("get:"))
                {
                    return format!(
                        "{receiver}.{}",
                        clean_symbol(method.trim_start_matches("get:"))
                    );
                }
                return format!("{receiver}.{method}({})", call_arguments.join(", "));
            }
        }
        if !internal && target_name.contains('.') && receiver_part.is_none() && direct_known {
            return format!("{target_name}({})", call_arguments.join(", "));
        }
        if internal && receiver_part.is_none() {
            // VM-internal helper: keep it visible but clearly non-source.
            let short = friendly_invoke_target(target);
            return format!(
                "aot.invoke({}, <dynamic>[{}])",
                dart_string(&short),
                call_arguments.join(", ")
            );
        }
        format!(
            "aot.invoke({}, <dynamic>[{}])",
            dart_string(&target_name),
            call_arguments.join(", ")
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AsyncStyle {
    Async,
    AsyncStar,
    SyncStar,
}

/// Recovers `async`/`async*`/`sync*` from authoritative VM evidence first,
/// then from the AOT async-machine stubs (`InitAsyncStub`, `AwaitStub`,
/// wrapper helpers) and named async-runtime collaborators that split-debug
/// symbols or call targets restore.
fn detected_async_style(function: &RecoveredFunction) -> Option<AsyncStyle> {
    if let Some(evidence) = function.vm_evidence.as_ref() {
        if evidence.is_async_generator == Some(true) {
            return Some(AsyncStyle::AsyncStar);
        }
        if evidence.is_sync_generator == Some(true) {
            return Some(AsyncStyle::SyncStar);
        }
        if evidence.is_async == Some(true) {
            return Some(AsyncStyle::Async);
        }
    }
    let mut targets = function
        .semantic_statements
        .iter()
        .filter_map(|statement| match statement {
            SemanticStatement::ResolvedCall { target, .. } => Some(target.as_str()),
            _ => None,
        })
        .chain(
            function
                .statements
                .iter()
                .filter_map(|statement| match statement {
                    crate::model::PseudoStatement::DirectCall {
                        target: Some(target),
                        ..
                    } => Some(target.as_str()),
                    _ => None,
                }),
        );
    let mut style: Option<AsyncStyle> = None;
    for target in targets.by_ref() {
        let compact = target.replace([' ', '_'], "");
        if compact.contains("asyncStar") || compact.contains("AsyncStar") {
            return Some(AsyncStyle::AsyncStar);
        }
        if compact.contains("syncStar") || compact.contains("SyncStar") {
            style = style.or(Some(AsyncStyle::SyncStar));
        }
        if compact.contains("AwaitStub")
            || compact.contains("InitAsyncStub")
            || compact.contains("completeOnAsyncReturn")
            || compact.contains("AsyncThenWrapper")
            || compact.contains("asyncThenWrapper")
        {
            style = Some(AsyncStyle::Async);
        }
        // Named async-runtime collaborators: awaiting a `Future`/`Stream`
        // through the runtime classes proves the body participates in the
        // async machine even when every stub itself stayed unnamed. The
        // `await` keyword itself stays unrendered (never invented), but the
        // structurer must stop treating the dispatch cycle as `while (true)`.
        // `_asyncComplete`/`_Future.immediate` are how a fully-AOT snapshot's
        // await/completed-future lowering survives when `AwaitStub` itself is
        // inlined away (probe EC-3 / E10).
        if compact.contains("StreamIterator.")
            || compact.contains("AsyncStarStreamController")
            || compact.contains("_Future.await")
            || compact.contains("_FutureListener")
            || compact.contains("Future.delayed")
            || compact.contains("_asyncComplete")
            || compact.contains("_Future.immediate")
            || compact.contains(":await_jump_var")
        {
            style = Some(AsyncStyle::Async);
        }
    }
    // Generator fallback for bodies whose collaborators all stayed unnamed:
    // both `sync*` and async machines record suspension points as
    // non-negative `yield_index` rows in the pc descriptors (probe EC-3:
    // e11 `yield*` at indices 12/36/44), so a descriptor hit proves *some*
    // machine, but only the return type separates the flavors. Claim
    // `sync*` solely when the recovered return type is Iterable-shaped;
    // Future/Stream shapes stay unclaimed here because misreading a plain
    // counted loop in such a body as dispatch would erase real control flow.
    // VM evidence above already proves real generators where it survives.
    if style.is_none()
        && function
            .code_metadata
            .as_ref()
            .is_some_and(|metadata| {
                metadata
                    .pc_descriptors
                    .iter()
                    .any(|descriptor| descriptor.yield_index >= 0)
            })
        && function
            .signature
            .as_ref()
            .and_then(|signature| signature.resolved.as_ref())
            .and_then(|resolved| resolved.return_type.as_ref())
            .map(|return_type| {
                let root = return_type.display_name.split('<').next().unwrap_or("");
                matches!(root.trim(), "Iterable" | "Iterator" | "_Iterable")
            })
            .unwrap_or(false)
    {
        style = Some(AsyncStyle::SyncStar);
    }
    style
}

fn is_await_boundary(target: &str) -> bool {
    let compact = target.replace([' ', '_'], "");
    compact.contains("AwaitStub") || compact.contains("awaitStub")
}

/// Rewrites `snapshotInstance(Class)` pool labels into readable const
/// constructor expressions, preserving a trailing `.member` chain.
fn prettify_snapshot_instance(expression: &str) -> Option<String> {
    let rest = expression.strip_prefix("snapshotInstance(")?;
    let close = rest.find(')')?;
    let class_name = &rest[..close];
    let tail = rest[close + 1..].trim_start();
    if !class_name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '$')
    {
        return None;
    }
    Some(format!(
        "const {}(){tail}",
        dart_identifier(&clean_symbol(class_name))
    ))
}

fn confidence_note(confidence: EvidenceConfidence) -> &'static str {
    match confidence {
        EvidenceConfidence::High => "",
        EvidenceConfidence::Medium => " /* medium-confidence predicate */",
        EvidenceConfidence::Low => " /* low-confidence predicate */",
    }
}

/// Replaces occurrences that look like whole identifiers, not substrings.
fn replace_identifier_like(text: &str, pattern: &str, replacement: &str) -> String {
    if pattern.is_empty() {
        return text.to_owned();
    }
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;
    while let Some(position) = remaining.find(pattern) {
        let before = remaining[..position].chars().next_back();
        let after = remaining[position + pattern.len()..].chars().next();
        let boundary = |character: Option<char>| {
            character.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
        };
        if boundary(before) && boundary(after) {
            output.push_str(&remaining[..position]);
            output.push_str(replacement);
        } else {
            output.push_str(&remaining[..position + pattern.len()]);
        }
        remaining = &remaining[position + pattern.len()..];
    }
    output.push_str(remaining);
    output
}

fn constructor_suffix(target: &str) -> String {
    let target = target
        .split_once(".dart.")
        .map_or(target, |(_, suffix)| suffix);
    let parts: Vec<&str> = target.trim_end_matches('.').split('.').collect();
    // Owner.Class.named → named ; Class.Class → none.
    if parts.len() >= 2 && parts[parts.len() - 1] != parts[parts.len() - 2] {
        return clean_symbol(parts[parts.len() - 1]).to_owned();
    }
    String::new()
}

fn variable_stem_for(target: &str, callee: Option<&CalleeInfo>, constructor: bool) -> String {
    if constructor
        || matches!(
            callee.and_then(|callee| callee.kind),
            Some(RecoveredFunctionKind::Constructor)
        )
    {
        return lower_camel_identifier(&dart_identifier(
            constructor_class_name(target)
                .or_else(|| normalized_call_target(target))
                .unwrap_or_else(|| "value".to_owned())
                .rsplit('.')
                .next()
                .unwrap_or("value")
                .trim_start_matches('_'),
        ));
    }
    variable_stem(target, false)
}

fn sanitize_key(target: &str) -> String {
    target
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn render_readable_literals(output: &mut String, function: &RecoveredFunction, indent: &str) {
    const MAX_LITERALS: usize = 40;
    let mut by_line = BTreeMap::<Option<i64>, BTreeSet<String>>::new();
    let mut seen = BTreeSet::new();
    for instruction in &function.instructions {
        let Some(label) = instruction.object_pool_value.as_deref() else {
            continue;
        };
        for (value, nested) in strings_from_pool_label(label) {
            if !seen.insert(value.clone())
                || snapshot_private_name(&value)
                || (nested && !readable_nested_string(&value))
            {
                continue;
            }
            by_line
                .entry(nearest_source_line(function, &instruction.address))
                .or_default()
                .insert(value);
        }
    }
    let total = by_line.values().map(BTreeSet::len).sum::<usize>();
    if total == 0 {
        return;
    }
    writeln!(output, "{indent}// Recovered source literals:").unwrap();
    let mut rendered = 0usize;
    'lines: for (line, values) in by_line {
        for value in values {
            if rendered == MAX_LITERALS {
                break 'lines;
            }
            let location = line.map_or_else(String::new, |line| format!("line {line}: "));
            writeln!(output, "{indent}//   {location}{}", dart_string(&value)).unwrap();
            rendered += 1;
        }
    }
    if total > rendered {
        writeln!(
            output,
            "{indent}//   … {} additional literal(s) are retained in ir/program.json.",
            total - rendered
        )
        .unwrap();
    }
}

fn strings_from_pool_label(label: &str) -> Vec<(String, bool)> {
    if label.starts_with('"') {
        return decode_debug_string(label)
            .map(|value| vec![(value, false)])
            .unwrap_or_default();
    }
    let Some((_, nested)) = label.split_once(" nestedStrings[") else {
        return Vec::new();
    };
    let Some(json) = nested.strip_suffix(']') else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<String>>(&format!("[{json}]"))
        .unwrap_or_default()
        .into_iter()
        .map(|value| (value, true))
        .collect()
}

fn decode_debug_string(value: &str) -> Option<String> {
    let value = value.strip_suffix('…').unwrap_or(value);
    serde_json::from_str(value).ok()
}

fn readable_nested_string(value: &str) -> bool {
    const IMPLEMENTATION_LABELS: &[&str] = &[
        "MaterialIcons",
        "disabled",
        "down",
        "elevated",
        "enabled",
        "filled",
        "hardEdge",
        "horizontal",
        "material",
        "none",
        "padding",
        "rectangle",
        "sRGB",
        "solid",
        "standard",
        "style",
        "text",
        "tight",
        "vertical",
    ];
    if IMPLEMENTATION_LABELS.contains(&value) {
        return false;
    }
    if snapshot_private_name(value) {
        return false;
    }
    let identifier_only = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_');
    !identifier_only
        || value
            .chars()
            .next()
            .is_some_and(|character| character.is_uppercase())
}

fn snapshot_private_name(value: &str) -> bool {
    value.rsplit_once('@').is_some_and(|(name, suffix)| {
        name.starts_with('_')
            && !suffix.is_empty()
            && suffix.chars().all(|character| character.is_ascii_digit())
    })
}

#[cfg_attr(not(test), allow(dead_code))]
fn render_readable_calls(
    output: &mut String,
    function: &RecoveredFunction,
    indent: &str,
    initial_aliases: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    const MAX_CALLS: usize = 40;
    let all_calls = function
        .semantic_statements
        .iter()
        .filter_map(|statement| match statement {
            SemanticStatement::ResolvedCall {
                target, arguments, ..
            } if meaningful_call_target(target) => Some((target.as_str(), arguments.as_slice())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let calls = all_calls
        .iter()
        .copied()
        .filter(|(target, _)| !is_vm_runtime_helper(target))
        .collect::<Vec<_>>();
    let runtime_helpers = all_calls
        .iter()
        .filter(|(target, _)| is_vm_runtime_helper(target))
        .map(|(target, _)| runtime_helper_display_name(target))
        .fold(BTreeMap::<String, usize>::new(), |mut counts, target| {
            *counts.entry(target).or_default() += 1;
            counts
        });
    let mut aliases = initial_aliases.clone();
    let mut variable_counts = BTreeMap::<String, usize>::new();
    let mut rendered = 0usize;
    let mut index = 0usize;
    while index < calls.len() && rendered < MAX_CALLS {
        let (target, arguments) = calls[index];
        if let Some((next_target, next_arguments)) = calls.get(index + 1).copied()
            && let Some(constructor_class) = constructor_class_name(next_target)
            && allocation_class_name(target)
                .or_else(|| constructor_class_name(target))
                .as_deref()
                == Some(constructor_class.as_str())
            && next_arguments.contains(&semantic_result_key(target))
        {
            let allocation_result = semantic_result_key(target);
            let constructor_arguments = next_arguments
                .iter()
                .filter(|argument| argument.as_str() != allocation_result)
                .cloned()
                .collect::<Vec<_>>();
            let variable =
                next_variable_name(&variable_stem(next_target, true), &mut variable_counts);
            write_call_assignment(
                output,
                indent,
                &variable,
                next_target,
                &constructor_arguments,
                &aliases,
                function.owner.as_deref(),
            );
            aliases.insert(allocation_result, variable.clone());
            aliases.insert(semantic_result_key(next_target), variable);
            rendered += 1;
            index += 2;
            continue;
        }

        let variable = next_variable_name(
            &variable_stem(target, constructor_class_name(target).is_some()),
            &mut variable_counts,
        );
        write_call_assignment(
            output,
            indent,
            &variable,
            target,
            arguments,
            &aliases,
            function.owner.as_deref(),
        );
        aliases.insert(semantic_result_key(target), variable);
        rendered += 1;
        index += 1;
    }
    if calls.len() > rendered {
        writeln!(
            output,
            "{indent}// {} additional recovered call(s) are available in reports/functions.json.",
            calls.len() - rendered
        )
        .unwrap();
    }
    if !runtime_helpers.is_empty() {
        let total = runtime_helpers.values().sum::<usize>();
        let summary = runtime_helpers
            .iter()
            .take(8)
            .map(|(name, count)| {
                if *count == 1 {
                    name.clone()
                } else {
                    format!("{name} ×{count}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let omitted_kinds = runtime_helpers.len().saturating_sub(8);
        let suffix = if omitted_kinds == 0 {
            String::new()
        } else {
            format!(", plus {omitted_kinds} other kind(s)")
        };
        writeln!(
            output,
            "{indent}// VM plumbing omitted from readable flow ({total} call site(s)): {summary}{suffix}."
        )
        .unwrap();
        writeln!(
            output,
            "{indent}// Exact helper calls remain in reports/functions.json and reports/assembly.s."
        )
        .unwrap();
    }
    aliases
}

fn render_dynamic_dispatch_evidence(
    output: &mut String,
    function: &RecoveredFunction,
    indent: &str,
) {
    const MAX_SELECTORS: usize = 8;
    #[derive(Default)]
    struct SelectorEvidence {
        sites: usize,
        candidate_count: usize,
        examples: BTreeSet<String>,
    }

    let mut selectors = BTreeMap::<String, SelectorEvidence>::new();
    let mut unresolved_indirect = 0usize;
    for statement in &function.statements {
        match statement {
            crate::model::PseudoStatement::DispatchTableCall {
                selector_name,
                candidate_targets,
                candidate_count,
                ..
            } => {
                let selector = selector_name
                    .clone()
                    .unwrap_or_else(|| "<unknown selector>".to_owned());
                let evidence = selectors.entry(selector).or_default();
                evidence.sites += 1;
                evidence.candidate_count = evidence.candidate_count.max(*candidate_count);
                evidence.examples.extend(
                    candidate_targets
                        .iter()
                        .filter(|target| meaningful_call_target(target))
                        .take(4)
                        .cloned(),
                );
            }
            crate::model::PseudoStatement::IndirectCall { .. } => unresolved_indirect += 1,
            _ => {}
        }
    }
    if selectors.is_empty() && unresolved_indirect == 0 {
        return;
    }
    writeln!(output, "{indent}// Dynamic-call evidence:").unwrap();
    for (selector, evidence) in selectors.iter().take(MAX_SELECTORS) {
        let examples = evidence
            .examples
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        let candidates = if evidence.candidate_count == 0 {
            "candidate set unresolved".to_owned()
        } else if examples.is_empty() {
            format!("{} candidate implementation(s)", evidence.candidate_count)
        } else {
            format!(
                "{} candidate implementation(s), e.g. {}",
                evidence.candidate_count,
                examples.join(", ")
            )
        };
        writeln!(
            output,
            "{indent}//   .{}(...) at {} site(s): {candidates}.",
            safe_comment(selector),
            evidence.sites,
        )
        .unwrap();
    }
    if selectors.len() > MAX_SELECTORS {
        writeln!(
            output,
            "{indent}//   … {} additional selector(s) are retained in reports/call_graph.json.",
            selectors.len() - MAX_SELECTORS,
        )
        .unwrap();
    }
    if unresolved_indirect > 0 {
        writeln!(
            output,
            "{indent}//   {unresolved_indirect} register-indirect call site(s) remain unresolved."
        )
        .unwrap();
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn render_control_flow_summary(output: &mut String, function: &RecoveredFunction, indent: &str) {
    let back_edges = function
        .control_flow
        .iter()
        .filter(|edge| {
            let from = u64::from_str_radix(edge.from.trim_start_matches("0x"), 16).ok();
            let to = u64::from_str_radix(edge.to.trim_start_matches("0x"), 16).ok();
            from.zip(to).is_some_and(|(from, to)| to <= from)
        })
        .count();
    let handlers = function
        .code_metadata
        .as_ref()
        .map_or(0, |metadata| metadata.exception_handlers.len());
    if function.machine_code.conditional_branches == 0 && back_edges == 0 && handlers == 0 {
        return;
    }
    writeln!(
        output,
        "{indent}// Control-flow evidence: {} conditional branch(es), {back_edges} loop back-edge(s), {handlers} exception handler(s).",
        function.machine_code.conditional_branches,
    )
    .unwrap();
    writeln!(
        output,
        "{indent}// Remaining block structure is preserved in reports/functions.json and reports/assembly.s."
    )
    .unwrap();
}

fn meaningful_call_target(target: &str) -> bool {
    !target.starts_with("sub_")
        && !target.contains("StackOverflow")
        && !matches!(target, "AllocateObject" | "AllocateArray")
}

fn is_vm_runtime_helper(target: &str) -> bool {
    let compact = target.replace(' ', "_");
    compact.starts_with("_iso_stub_")
        || compact.starts_with("_vm_stub_")
        || compact.starts_with("stub_")
        || target.starts_with("stub ")
}

fn runtime_helper_display_name(target: &str) -> String {
    target
        .trim_start_matches("_iso_stub_")
        .trim_start_matches("_vm_stub_")
        .trim_start_matches("stub_")
        .trim_end_matches("SharedWithoutFPURegsStub")
        .trim_end_matches("SharedWithFPURegsStub")
        .trim_end_matches("Stub")
        .to_owned()
}

#[cfg_attr(not(test), allow(dead_code))]
fn semantic_result_key(target: &str) -> String {
    format!(
        "{}_result",
        target
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '_' {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>()
    )
}

fn allocation_class_name(target: &str) -> Option<String> {
    let target = normalized_call_target(target)?;
    (!target.contains('.')).then_some(target)
}

fn constructor_name(target: &str) -> Option<String> {
    let target = target.trim_end_matches('.');
    let mut parts = target.rsplit('.');
    let last = parts.next()?;
    let prior = parts.next()?;
    (last == prior && valid_dart_identifier(last)).then(|| last.to_owned())
}

fn normalized_call_target(target: &str) -> Option<String> {
    // Split-debug symbol names spell allocators as `new Class[.named]`.
    let target = target
        .strip_prefix("new ")
        .unwrap_or(target)
        .split_once(".dart.")
        .map_or(
            target.strip_prefix("new ").unwrap_or(target),
            |(_, suffix)| suffix,
        )
        .trim_end_matches('.');
    if let Some(rest) = target.strip_prefix("new ") {
        return Some(rest.to_owned());
    }
    if let Some(constructor) = constructor_name(target) {
        return Some(constructor);
    }
    let normalized = target
        .strip_prefix("_StringBase.")
        .map_or(target.to_owned(), |method| format!("String.{method}"));
    let mut parts = normalized.split('.').collect::<Vec<_>>();
    if parts.len() >= 3 {
        let repeated_owner = parts.len() - 3;
        if parts[repeated_owner] == parts[repeated_owner + 1] {
            parts.remove(repeated_owner + 1);
        }
    }
    let normalized = parts.join(".");
    normalized
        .split('.')
        .all(valid_dart_identifier)
        .then_some(normalized)
}

fn constructor_class_name(target: &str) -> Option<String> {
    let stripped = target.strip_prefix("new ").map(str::to_owned);
    if let Some(stripped) = stripped.as_deref() {
        let head = stripped.split('.').next()?;
        return valid_dart_identifier(head).then(|| head.to_owned());
    }
    if let Some(constructor) = constructor_name(target) {
        return Some(constructor);
    }
    let target = target
        .split_once(".dart.")
        .map_or(target, |(_, suffix)| suffix)
        .trim_end_matches('.');
    let parts = target.split('.').collect::<Vec<_>>();
    (parts.len() >= 3
        && parts[parts.len() - 3] == parts[parts.len() - 2]
        && valid_dart_identifier(parts[parts.len() - 2]))
    .then(|| parts[parts.len() - 2].to_owned())
}

fn valid_dart_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn readable_call_expression(
    target: &str,
    arguments: &[String],
    aliases: &BTreeMap<String, String>,
    owner: Option<&str>,
) -> String {
    let rendered_arguments = render_call_arguments(arguments, aliases);
    if let Some(target) = readable_direct_call_target(target, owner) {
        return format!("{target}({rendered_arguments})");
    }
    format!(
        "aot.invoke({}, <dynamic>[{}])",
        dart_string(&friendly_invoke_target(target)),
        rendered_arguments
    )
}

fn readable_direct_call_target(target: &str, owner: Option<&str>) -> Option<String> {
    let original_target = target;
    if let Some(mut target) = normalized_call_target(target) {
        if let Some(owner) = owner.map(clean_symbol)
            && let Some(local) = target.strip_prefix(&format!("{owner}."))
        {
            target = local.to_owned();
        }
        let direct = constructor_class_name(original_target).is_some()
            || target == "Uri.parse"
            || !target.contains('.')
            || target.ends_with(".of");
        if direct && !is_internal_runtime_target(&target) {
            return Some(target);
        }
    }
    None
}

fn write_call_assignment(
    output: &mut String,
    indent: &str,
    variable: &str,
    target: &str,
    arguments: &[String],
    aliases: &BTreeMap<String, String>,
    owner: Option<&str>,
) {
    const MAX_COMPACT_WIDTH: usize = 120;
    let expression = readable_call_expression(target, arguments, aliases, owner);
    if indent.len() + variable.len() + expression.len() + 10 <= MAX_COMPACT_WIDTH {
        writeln!(output, "{indent}final {variable} = {expression};").unwrap();
        return;
    }

    let rendered_arguments = arguments
        .iter()
        .map(|argument| render_readable_expression(argument, aliases))
        .collect::<Vec<_>>();
    if let Some(target) = readable_direct_call_target(target, owner) {
        writeln!(output, "{indent}final {variable} = {target}(").unwrap();
        for argument in rendered_arguments {
            writeln!(output, "{indent}  {argument},").unwrap();
        }
        writeln!(output, "{indent});").unwrap();
    } else {
        writeln!(output, "{indent}final {variable} = aot.invoke(").unwrap();
        writeln!(
            output,
            "{indent}  {},",
            dart_string(&friendly_invoke_target(target))
        )
        .unwrap();
        writeln!(output, "{indent}  <dynamic>[").unwrap();
        for argument in rendered_arguments {
            writeln!(output, "{indent}    {argument},").unwrap();
        }
        writeln!(output, "{indent}  ],").unwrap();
        writeln!(output, "{indent});").unwrap();
    }
}

fn is_internal_runtime_target(target: &str) -> bool {
    target.starts_with("String.")
        || target.starts_with("_OneByteString.")
        || target.starts_with("ListBase.")
        || target.starts_with("_GrowableList.")
        || target.starts_with("_Compact")
        || target.starts_with("dart:")
}

fn friendly_invoke_target(target: &str) -> String {
    let normalized = normalized_call_target(target).unwrap_or_else(|| target.to_owned());
    let target = normalized.as_str();
    if let Some(method) = target
        .strip_prefix("_StringBase.")
        .or_else(|| target.strip_prefix("_OneByteString."))
    {
        return format!("String.{method}");
    }
    if let Some(method) = target
        .strip_prefix("ListBase.")
        .or_else(|| target.strip_prefix("_GrowableList."))
    {
        return format!("List.{method}");
    }
    if target.contains("_LinkedHashMapMixin._getValueOrData") {
        return "Map.lookup".to_owned();
    }
    target.to_owned()
}

fn render_call_arguments(arguments: &[String], aliases: &BTreeMap<String, String>) -> String {
    arguments
        .iter()
        .map(|argument| render_readable_expression(argument, aliases))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_readable_expression(expression: &str, aliases: &BTreeMap<String, String>) -> String {
    if let Some(alias) = aliases.get(expression) {
        return alias.clone();
    }
    let expression = expression
        .split_once(" nestedStrings[")
        .map_or(expression, |(root, _)| root);
    if expression.starts_with('"')
        && let Some(value) = decode_debug_string(expression)
    {
        return dart_string(&value);
    }
    // Canonical snapshot instances render as const constructors and keep
    // any trailing member access (`snapshotInstance(Product).name`).
    if let Some(prettified) = prettify_snapshot_instance(expression) {
        return prettified;
    }
    // Recovered Dart literals produced by Clutter itself stay verbatim.
    if expression.len() >= 2 && expression.starts_with('\'') && expression.ends_with('\'') {
        return expression.to_owned();
    }
    // Numeric literals are already valid Dart expression atoms.
    if expression.parse::<i64>().is_ok() || expression.parse::<f64>().is_ok() {
        return expression.to_owned();
    }
    if matches!(expression, "true" | "false" | "null")
        || expression
            .strip_prefix("arg")
            .is_some_and(|index| index.chars().all(|character| character.is_ascii_digit()))
        || valid_dart_identifier(expression)
    {
        return if matches!(expression, "true" | "false" | "null" | "this") {
            expression.to_owned()
        } else {
            dart_identifier(expression)
        };
    }
    if expression.contains('.')
        && expression
            .split('.')
            .all(|segment| valid_dart_identifier(&clean_symbol(segment)))
    {
        return expression
            .split('.')
            .map(|segment| {
                if segment == "this" {
                    "this".to_owned()
                } else {
                    dart_identifier(&clean_symbol(segment))
                }
            })
            .collect::<Vec<_>>()
            .join(".");
    }
    if expression.starts_with("snapshotRef(") && expression.ends_with(')') {
        return format!("aot.{expression}");
    }
    // Arithmetic and comparison expressions rebuilt by the lifter over named
    // registers, literals, and field reads are already valid Dart; render
    // them directly instead of hiding them behind an unresolved boundary.
    if looks_like_recovered_expression(expression) {
        return expression.to_owned();
    }
    format!("aot.unresolvedValue({})", dart_string(expression))
}

/// True when an expression is composed purely of Dart-safe tokens the lifter
/// produced: identifiers, numeric literals, operators, parentheses, member
/// access, and interpolation placeholders it generated itself.
fn looks_like_recovered_expression(expression: &str) -> bool {
    if expression.is_empty()
        || !(expression.contains(['+', '-', '*', '/', '%', '^', '&', '|', '<', '>', '!'])
            || expression.contains("<<")
            || expression.contains(">>"))
    {
        return false;
    }
    // Interpolation placeholders stay explicit.
    if expression.contains("aot.") || expression.contains("pool[") || expression.contains("sub_") {
        return false;
    }
    let mut identifier = String::new();
    for character in expression.chars() {
        if character.is_alphanumeric() || character == '_' || character == '$' {
            identifier.push(character);
            continue;
        }
        if !identifier.is_empty() {
            if character == '(' && !valid_dart_identifier(&identifier) {
                return false;
            }
            identifier.clear();
        }
        if !(character.is_whitespace()
            || matches!(
                character,
                '+' | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '^'
                    | '&'
                    | '|'
                    | '<'
                    | '>'
                    | '!'
                    | '='
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '.'
                    | ','
                    | '?'
                    | ':'
                    | '~'
            ))
        {
            return false;
        }
    }
    if !identifier.is_empty() && !valid_dart_identifier(&identifier) {
        return false;
    }
    true
}

fn readable_call_type(target: &str) -> String {
    if target == "Uri.parse" {
        return "Uri".to_owned();
    }
    constructor_class_name(target)
        .or_else(|| allocation_class_name(target))
        .filter(|value| valid_dart_identifier(value))
        .unwrap_or_else(|| "dynamic".to_owned())
}

/// VM collection allocators lower from literal constructions; render them
/// back as literals instead of invented constructor calls.
fn internal_allocator_literal(normalized_target: &str) -> Option<String> {
    let leaf = normalized_target.rsplit('.').next()?.trim_end_matches('.');
    let literal = match leaf {
        "_GrowableList" | "_ImmutableList" | "_Array" | "List" => "<dynamic>[]",
        "_Map" | "Map" | "_ConstMap" => "<dynamic, dynamic>{}",
        "_Set" | "Set" | "_ConstSet" => "<dynamic>{}",
        _ => return None,
    };
    // Only bare allocations (no real named members) are literal lowers.
    let segments = normalized_target.split('.').count();
    if normalized_target.starts_with("dart:") || segments <= 3 {
        Some(literal.to_owned())
    } else {
        None
    }
}

/// Maps a call member that is a Dart operator symbol to its infix spelling.
fn operator_member_name(member: &str) -> Option<&'static str> {
    match clean_symbol(member).as_str() {
        "+" => Some("+"),
        "-" => Some("-"),
        "*" => Some("*"),
        "/" => Some("/"),
        "~/" => Some("~/"),
        "%" => Some("%"),
        "|" => Some("|"),
        "&" => Some("&"),
        "^" => Some("^"),
        "<<" => Some("<<"),
        ">>" => Some(">>"),
        "==" => Some("=="),
        "<" => Some("<"),
        ">" => Some(">"),
        "<=" => Some("<="),
        ">=" => Some(">="),
        _ => None,
    }
}

fn readable_field_initializer(
    function: &RecoveredFunction,
) -> Option<(String, String, String, bool)> {
    let field_name = clean_symbol(function.name.strip_prefix("init:")?);
    let calls = function
        .semantic_statements
        .iter()
        .filter_map(|statement| match statement {
            SemanticStatement::ResolvedCall {
                target, arguments, ..
            } if meaningful_call_target(target) => Some((target.as_str(), arguments.as_slice())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let returned = function
        .semantic_statements
        .iter()
        .find_map(|statement| match statement {
            SemanticStatement::Return { expression, .. } => Some(expression.as_str()),
            _ => None,
        });
    let aliases = BTreeMap::new();

    let selected = calls
        .windows(2)
        .find_map(|pair| {
            let (allocation, _) = pair[0];
            let (constructor, arguments) = pair[1];
            let class = constructor_name(constructor)?;
            (allocation_class_name(allocation).as_deref() == Some(class.as_str())).then_some((
                constructor,
                arguments
                    .iter()
                    .filter(|argument| argument.as_str() != semantic_result_key(allocation))
                    .cloned()
                    .collect::<Vec<_>>(),
            ))
        })
        .or_else(|| {
            returned.and_then(|returned| {
                calls
                    .iter()
                    .find(|(target, _)| semantic_result_key(target) == returned)
                    .map(|(target, arguments)| (*target, arguments.to_vec()))
            })
        })
        .or_else(|| {
            let [(target, arguments)] = calls.as_slice() else {
                return None;
            };
            Some((*target, arguments.to_vec()))
        });

    let (field_type, expression) = selected.map_or_else(
        || {
            (
                "dynamic".to_owned(),
                format!(
                    "aot.unresolvedValue({})",
                    dart_string(&format!("Initializer for {field_name}"))
                ),
            )
        },
        |(target, arguments)| {
            (
                readable_call_type(target),
                readable_call_expression(target, &arguments, &aliases, function.owner.as_deref()),
            )
        },
    );
    let is_static = function
        .signature
        .as_ref()
        .is_some_and(|signature| signature.implicit_parameter_count == 0);
    Some((field_name, field_type, expression, is_static))
}

fn variable_stem(target: &str, constructor: bool) -> String {
    if target == "Uri.parse" {
        return "uri".to_owned();
    }
    let normalized = friendly_invoke_target(target);
    let constructor_class = constructor_class_name(target);
    let name = if normalized.ends_with(".of") {
        normalized
            .split('.')
            .next()
            .unwrap_or("value")
            .trim_start_matches('_')
    } else if constructor {
        constructor_class.as_deref().unwrap_or_else(|| {
            normalized
                .rsplit('.')
                .next()
                .unwrap_or("value")
                .trim_start_matches('_')
        })
    } else {
        match normalized
            .rsplit('.')
            .next()
            .unwrap_or("value")
            .trim_start_matches('_')
        {
            "+" => "combined",
            "-" => "difference",
            "*" => "product",
            "/" | "~/" => "quotient",
            "==" => "equals",
            "[]" => "item",
            "[]=" => "updatedItem",
            name => name,
        }
    };
    let name = lower_camel_identifier(&dart_identifier(name));
    if constructor {
        name
    } else {
        format!("{name}Result")
    }
}

fn lower_camel_identifier(value: &str) -> String {
    let mut output = value.to_owned();
    if let Some((index, character)) = output
        .char_indices()
        .find(|(_, character)| character.is_ascii_alphabetic())
    {
        let end = index + character.len_utf8();
        output[index..end].make_ascii_lowercase();
        while output.starts_with('_') {
            output.remove(0);
        }
    }
    if output.is_empty() {
        "value".to_owned()
    } else {
        output
    }
}

fn next_variable_name(stem: &str, counts: &mut BTreeMap<String, usize>) -> String {
    let count = counts.entry(stem.to_owned()).or_default();
    *count += 1;
    if *count == 1 {
        stem.to_owned()
    } else {
        format!("{stem}{count}")
    }
}

fn nearest_source_line(function: &RecoveredFunction, instruction_address: &str) -> Option<i64> {
    let function_address =
        u64::from_str_radix(function.address.trim_start_matches("0x"), 16).ok()?;
    let instruction_address =
        u64::from_str_radix(instruction_address.trim_start_matches("0x"), 16).ok()?;
    let pc_offset = u32::try_from(instruction_address.checked_sub(function_address)?).ok()?;
    function
        .code_metadata
        .as_ref()?
        .code_source_map
        .iter()
        .filter(|entry| entry.inline_depth == 0 && entry.pc_offset <= pc_offset)
        .filter_map(|entry| entry.source_line.map(|line| (entry.pc_offset, line)))
        .max_by_key(|(offset, _)| *offset)
        .map(|(_, line)| line)
}

fn shared_code_primary<'a>(
    program: &'a RecoveredProgram,
    index: &RenderIndex,
    function: &RecoveredFunction,
) -> Option<&'a RecoveredFunction> {
    let primary_index = index
        .shared_code_primary
        .get(&(function.address.clone(), function.size))
        .copied()?;
    let primary = program.functions.get(primary_index)?;
    (!std::ptr::eq(primary, function)).then_some(primary)
}

fn rendered_function_symbol_root(function: &RecoveredFunction, in_class: bool) -> String {
    let owner = function.owner.as_deref().unwrap_or("top_level");
    let root = if constructor_like(function, in_class) {
        "create".to_owned()
    } else if function.kind == Some(RecoveredFunctionKind::Closure)
        || function.name == "<anonymous closure>"
    {
        function
            .source_location
            .as_ref()
            .and_then(|location| location.line)
            .map_or_else(
                || {
                    function
                        .vm_evidence
                        .as_ref()
                        .and_then(|evidence| evidence.parent_function_name.as_deref())
                        .and_then(|parent| parent.rsplit('.').next())
                        .map(|parent| {
                            let multiple_candidates = function
                                .vm_evidence
                                .as_ref()
                                .and_then(|evidence| evidence.logical_match_candidate_count)
                                .is_some_and(|count| count > 1);
                            let candidate = if multiple_candidates { "Candidate" } else { "" };
                            format!(
                                "closure{candidate}In{}",
                                upper_camel_fragment(&readable_function_name(parent)),
                            )
                        })
                        .unwrap_or_else(|| "closure".to_owned())
                },
                |line| format!("closureAtLine{line}"),
            )
    } else if let Some(suffix) = function_kind_symbol_suffix(function.kind) {
        format!("{}_{}", readable_function_name(&function.name), suffix)
    } else if let Some(field) = function.name.strip_prefix("init:") {
        format!("initialize_{}", clean_symbol(field))
    } else {
        readable_function_name(&function.name)
    };
    if in_class || matches!(owner, "::" | "top_level") {
        root
    } else {
        format!("{}_{}", clean_symbol(owner), root)
    }
}

fn upper_camel_fragment(value: &str) -> String {
    let mut output = value.trim_start_matches('_').to_owned();
    if let Some(index) = output
        .char_indices()
        .find_map(|(index, character)| character.is_ascii_alphabetic().then_some(index))
    {
        let end = index + output[index..].chars().next().map_or(0, char::len_utf8);
        output[index..end].make_ascii_uppercase();
    }
    output
}

fn function_kind_symbol_suffix(kind: Option<RecoveredFunctionKind>) -> Option<&'static str> {
    match kind? {
        RecoveredFunctionKind::ImplicitClosure => Some("tearOff"),
        RecoveredFunctionKind::DynamicInvocationForwarder => Some("dynamicForwarder"),
        RecoveredFunctionKind::MethodExtractor => Some("methodExtractor"),
        RecoveredFunctionKind::NoSuchMethodDispatcher => Some("noSuchMethodDispatcher"),
        RecoveredFunctionKind::InvokeFieldDispatcher => Some("invokeFieldDispatcher"),
        RecoveredFunctionKind::FfiTrampoline => Some("ffiTrampoline"),
        _ => None,
    }
}

fn is_static_member(function: &RecoveredFunction, in_class: bool) -> bool {
    if !in_class {
        return false;
    }
    if constructor_like(function, in_class) {
        return true;
    }
    // Closures are rendered as ordinary members so captured-`this` property
    // accesses stay valid Dart; the closure identity stays in the name/doc.
    if function.kind == Some(RecoveredFunctionKind::Closure)
        || function.name == "<anonymous closure>"
    {
        return false;
    }
    if let Some(is_static) = function.is_static {
        return is_static;
    }
    if let Some(is_static) = function
        .vm_evidence
        .as_ref()
        .and_then(|evidence| evidence.is_static)
    {
        return is_static;
    }
    function
        .signature
        .as_ref()
        .is_some_and(|signature| signature.implicit_parameter_count == 0)
}

fn constructor_like(function: &RecoveredFunction, in_class: bool) -> bool {
    in_class
        && (function.kind == Some(RecoveredFunctionKind::Constructor)
            || function
                .owner
                .as_deref()
                .is_some_and(|owner| clean_symbol(owner) == clean_symbol(&function.name)))
}

fn function_name_collisions(
    functions: &[&RecoveredFunction],
    in_class: bool,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for function in functions {
        *counts
            .entry(rendered_function_symbol_root(function, in_class))
            .or_default() += 1;
    }
    counts
}

#[cfg_attr(not(test), allow(dead_code))]
fn qualified_name(function: &RecoveredFunction) -> String {
    match function.owner.as_deref() {
        Some(owner) if !matches!(owner, "::" | "top_level") => {
            format!("{owner}.{}", function.name)
        }
        _ => function.name.clone(),
    }
}

fn rendered_parameters(function: &RecoveredFunction) -> (String, Option<String>) {
    let Some(signature) = &function.signature else {
        match clean_symbol(&function.name).as_str() {
            "build" => {
                return ("(dynamic context)".to_owned(), Some("context".to_owned()));
            }
            "createState" | "dispose" | "initState" | "toString" => {
                return ("()".to_owned(), Some(String::new()));
            }
            _ => {}
        }
        return ("(List<dynamic> args)".to_owned(), None);
    };
    rendered_signature_parameters(signature)
}

fn rendered_signature_parameters(
    signature: &crate::model::RecoveredSignature,
) -> (String, Option<String>) {
    if let Some(resolved) = &signature.resolved {
        let mut fixed = Vec::new();
        let mut optional = Vec::new();
        let mut arguments = Vec::new();
        for parameter in &resolved.parameters {
            let fallback_name = if parameter.is_named {
                format!(
                    "namedArg{}",
                    parameter
                        .position
                        .saturating_sub(signature.fixed_parameter_count)
                )
            } else {
                format!("arg{}", parameter.position)
            };
            let name = dart_identifier(parameter.name.as_deref().unwrap_or(&fallback_name));
            let type_name = parameter
                .declared_type
                .as_ref()
                .map_or_else(|| "dynamic".to_owned(), rendered_type);
            let required = if parameter.is_named && parameter.is_required {
                "required "
            } else {
                ""
            };
            let default_note = if parameter.position >= signature.fixed_parameter_count
                && !parameter.is_required
            {
                " /* default unavailable */"
            } else {
                ""
            };
            let type_name = if !parameter.is_named
                || parameter.is_required
                || default_note.is_empty()
                || type_name.ends_with('?')
                || matches!(type_name.as_str(), "dynamic" | "void" | "Never")
            {
                type_name
            } else {
                format!("{type_name}?")
            };
            let declaration = format!("{required}{type_name} {name}{default_note}");
            if parameter.position < signature.fixed_parameter_count {
                fixed.push(declaration);
            } else {
                optional.push(declaration);
            }
            arguments.push(name);
        }
        if !optional.is_empty() {
            fixed.push(if signature.optional_parameters_are_named {
                format!("{{{}}}", optional.join(", "))
            } else {
                format!("[{}]", optional.join(", "))
            });
        }
        return (
            format!("({})", fixed.join(", ")),
            Some(arguments.join(", ")),
        );
    }
    let mut declarations = Vec::new();
    let mut arguments = Vec::new();
    for index in 0..signature.fixed_parameter_count {
        let name = format!("arg{index}");
        declarations.push(format!("dynamic {name}"));
        arguments.push(name);
    }
    if signature.optional_parameter_count > 0 {
        let mut optional = Vec::new();
        for index in 0..signature.optional_parameter_count {
            let name = if signature.optional_parameters_are_named {
                format!("namedArg{index}")
            } else {
                format!("optionalArg{index}")
            };
            optional.push(format!("dynamic {name} /* default unavailable */"));
            arguments.push(name);
        }
        let block = optional.join(", ");
        declarations.push(if signature.optional_parameters_are_named {
            format!("{{{block}}}")
        } else {
            format!("[{block}]")
        });
    }
    (
        format!("({})", declarations.join(", ")),
        Some(arguments.join(", ")),
    )
}

fn readable_function_name(value: &str) -> String {
    if let Some(operator) = value.strip_prefix("dyn:") {
        return dart_identifier(&format!(
            "dynamic_{}",
            readable_operator_name(operator).unwrap_or(operator)
        ));
    }
    if let Some(getter) = value.strip_prefix("get:") {
        return dart_identifier(&format!("get_{}", clean_symbol(getter)));
    }
    if let Some(setter) = value.strip_prefix("set:") {
        return dart_identifier(&format!("set_{}", clean_symbol(setter)));
    }
    let value = value.replace("|get#", "_get_").replace("|set#", "_set_");
    let value = match readable_operator_name(&value) {
        Some(operator) => operator,
        None => match value.as_str() {
            "<anonymous closure>" => "anonymous_closure",
            other => other,
        },
    };
    dart_identifier(&clean_symbol(value))
}

fn readable_operator_name(value: &str) -> Option<&'static str> {
    match value {
        "+" => Some("operator_add"),
        "-" => Some("operator_subtract"),
        "*" => Some("operator_multiply"),
        "/" => Some("operator_divide"),
        "~/" => Some("operator_integer_divide"),
        "==" => Some("operator_equals"),
        "[]" => Some("operator_index"),
        "[]=" => Some("operator_index_assign"),
        _ => None,
    }
}

/// Dart `operator` syntax for a member whose VM selector name is an operator
/// symbol (`==`, `%`, `-`, `[]=`). Returns the source-facing spelling; the
/// only rename is the VM's `unary-` selector, which is Dart's unary `-`.
fn source_operator_syntax(name: &str) -> Option<&'static str> {
    const OPERATOR_SYMBOLS: &[&str] = &[
        "+", "-", "*", "/", "~/", "%", "|", "&", "^", "<<", ">>", ">>>", "==", "<", ">",
        "<=", ">=", "[]", "[]=", "~",
    ];
    if name == "unary-" {
        return Some("-");
    }
    OPERATOR_SYMBOLS.iter().copied().find(|symbol| *symbol == name)
}

pub fn render_support() -> String {
    r#"// GENERATED SUPPORT FOR CONSERVATIVE AOT PSEUDOCODE.

Never _unresolved(String kind, Object? evidence) =>
    throw UnsupportedError('Unresolved AOT operation: $kind ($evidence)');

dynamic unresolvedRegion(String sourceUri, List<dynamic> args) =>
    _unresolved('region', <Object?>[sourceUri, args]);

dynamic unresolvedValue(String description) =>
    _unresolved('value', description);

dynamic unresolvedRegister(String register) =>
    _unresolved('register', register);

dynamic invoke(String target, List<dynamic> arguments) =>
    _unresolved('call', <Object?>[target, arguments]);

dynamic unknownOperation(
  String address,
  String bytes,
  List<dynamic> inputs,
) =>
    _unresolved('instruction', <Object?>[address, bytes, inputs]);

dynamic snapshotRef(int reference) =>
    _unresolved('snapshot-object', reference);

dynamic nativePoolEntry(int index) =>
    _unresolved('native-pool-entry', index);

dynamic resetPoolEntry(int index) =>
    _unresolved('reset-pool-entry', index);
"#
    .to_owned()
}

fn dart_identifier(value: &str) -> String {
    let mut output: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    if output.is_empty() || output.starts_with(|character: char| character.is_ascii_digit()) {
        output.insert(0, '_');
    }
    if DART_RESERVED_WORDS.contains(&output.as_str()) {
        output.insert(0, '_');
    }
    output
}

fn clean_symbol(value: &str) -> String {
    value
        .split('@')
        .next()
        .unwrap_or(value)
        .trim_matches(|character: char| character == '<' || character == '>')
        .to_owned()
}

fn relative_support_import(library: &RecoveredLibrary) -> String {
    let levels = library.output_path.components().count().max(1);
    format!("{}support/aot_intrinsics.dart", "../".repeat(levels))
}

fn safe_comment(value: &str) -> String {
    value.replace(['\n', '\r'], " ").replace("*/", "* /")
}

const DART_RESERVED_WORDS: &[&str] = &[
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "base",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "for",
    "Function",
    "get",
    "hide",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "late",
    "library",
    "mixin",
    "new",
    "null",
    "of",
    "on",
    "operator",
    "part",
    "required",
    "rethrow",
    "return",
    "sealed",
    "set",
    "show",
    "static",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "var",
    "void",
    "when",
    "while",
    "with",
    "yield",
];

fn dart_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('\'');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\'' => output.push_str("\\'"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '$' => output.push_str("\\$"),
            value if value.is_control() => {
                write!(output, "\\u{{{:x}}}", value as u32).unwrap();
            }
            value => output.push(value),
        }
    }
    output.push('\'');
    output
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use super::{collect_declared_identifiers, sanitize_free_machine_identifiers};
    use crate::model::{
        EvidenceConfidence, MachineCodeEvidence, MachineInstruction, PseudoStatement,
        RecoveredFunction, RecoveredFunctionKind, RecoveredLibrary, RecoveredNameSource,
        RecoveredParameter, RecoveredProgram, RecoveredSignature, RecoveredSignatureDetails,
        RecoveredSignatureSource, RecoveredType, RecoveredTypeParameter, SemanticStatement,
    };

    use super::{
        AsyncStyle, RenderIndex, closure_parent_index, dart_string, detected_async_style,
        friendly_invoke_target, normalize_function_type_syntax, prettify_snapshot_instance,
        readable_function_name, readable_nested_string, relative_support_import,
        render_dynamic_dispatch_evidence, render_library, render_readable_calls,
        render_readable_expression, render_support, render_type_parameters,
        rendered_function_symbol_root, rendered_parameters, rendered_return_type, variable_stem,
        callee_optional_named,
    };

    #[test]
    fn escapes_dart_strings() {
        assert_eq!(dart_string("a'b$"), "'a\\'b\\$'");
    }

    #[test]
    fn falls_back_to_vm_descriptor_names_for_unnamed_parameters() {
        let signature = RecoveredSignature {
            fixed_parameter_count: 0,
            optional_parameter_count: 2,
            optional_parameters_are_named: true,
            implicit_parameter_count: 0,
            type_parameters_reference: None,
            result_type_reference: None,
            parameter_types_reference: None,
            named_parameter_names_reference: None,
            flags: 0,
            packed_type_parameter_counts: 0,
            resolved: Some(RecoveredSignatureDetails {
                return_type: None,
                type_parameters: Vec::new(),
                parameters: vec![
                    RecoveredParameter {
                        position: 0,
                        name: None,
                        declared_type: None,
                        is_named: true,
                        is_required: false,
                    },
                    RecoveredParameter {
                        position: 1,
                        name: None,
                        declared_type: None,
                        is_named: true,
                        is_required: true,
                    },
                ],
            }),
        };
        let evidence = crate::model::VmFunctionEvidence {
            parameters: vec![
                crate::model::VmParameterEvidence {
                    position: 0,
                    name: Some("receiver".to_owned()),
                    declared_type: None,
                    is_implicit: true,
                    is_named: false,
                    is_required: false,
                },
                crate::model::VmParameterEvidence {
                    position: 1,
                    name: Some("width".to_owned()),
                    declared_type: None,
                    is_implicit: false,
                    is_named: true,
                    is_required: false,
                },
                crate::model::VmParameterEvidence {
                    position: 2,
                    name: Some("height".to_owned()),
                    declared_type: None,
                    is_implicit: false,
                    is_named: true,
                    is_required: true,
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            callee_optional_named(Some(&signature), Some(&evidence)),
            vec![("width".to_owned(), false), ("height".to_owned(), true)]
        );
        // Without VM evidence the unnamed parameters stay empty strings.
        assert_eq!(
            callee_optional_named(Some(&signature), None),
            vec![(String::new(), false), (String::new(), true)]
        );
    }

    fn sample_statement_call(target: &str) -> SemanticStatement {
        SemanticStatement::ResolvedCall {
            target: target.to_owned(),
            arguments: Vec::new(),
            confidence: EvidenceConfidence::Medium,
            address: "0x1000".to_owned(),
        }
    }

    #[test]
    fn detects_async_style_from_vm_evidence_and_stubs() {
        let mut function = sample_function();
        function.vm_evidence = None;
        assert_eq!(detected_async_style(&function), None);

        function.semantic_statements = vec![
            sample_statement_call("stub _iso_stub_InitAsyncStub"),
            sample_statement_call("stub _iso_stub_AwaitStub"),
        ];
        assert_eq!(detected_async_style(&function), Some(AsyncStyle::Async));

        function.semantic_statements = vec![sample_statement_call("_syncStarThenWrapperHelper")];
        assert_eq!(detected_async_style(&function), Some(AsyncStyle::SyncStar));

        function.vm_evidence = Some(crate::model::VmFunctionEvidence {
            is_async_generator: Some(true),
            ..Default::default()
        });
        function.semantic_statements = Vec::new();
        assert_eq!(detected_async_style(&function), Some(AsyncStyle::AsyncStar));
    }

    #[test]
    fn prettifies_snapshot_instance_labels_with_member_tails() {
        assert_eq!(
            prettify_snapshot_instance("snapshotInstance(Product)").as_deref(),
            Some("const Product()")
        );
        assert_eq!(
            prettify_snapshot_instance("snapshotInstance(_CatalogPageState).itemCount").as_deref(),
            Some("const _CatalogPageState().itemCount")
        );
        assert_eq!(prettify_snapshot_instance("snapshotRef(12)"), None);
    }

    #[test]
    fn sanitizes_free_machine_identifiers_only() {
        let bound = BTreeSet::from(["arg0".to_owned(), "list".to_owned()]);
        assert_eq!(
            sanitize_free_machine_identifiers(
                "if (x0 != x16) {\n  return 'ok:${local70}';\n}\nreturn snapshotRef(458);",
                &bound,
            ),
            "if (aot.unresolvedRegister('x0') != aot.unresolvedRegister('x16')) {\n  \
             return 'ok:${aot.unresolvedValue('slot 0x70')}';\n}\nreturn aot.snapshotRef(458);"
        );
        // Bound identifiers, members after a dot, and keywords stay intact.
        assert_eq!(
            sanitize_free_machine_identifiers(
                "return list.length; // keep local70 mention in comments",
                &bound,
            ),
            "return list.length; // keep local70 mention in comments"
        );
    }

    #[test]
    fn collects_declared_locals_without_stalling_on_many_finals() {
        let body = "
final firstResult = f();
final secondResult = g(firstResult);
while (true) {
  final thirdResult = h();
}
";
        let bound = collect_declared_identifiers(body, "arg0, arg1");
        assert!(bound.contains("firstResult"));
        assert!(bound.contains("secondResult"));
        assert!(bound.contains("thirdResult"));
        assert!(bound.contains("arg0"));
        assert!(bound.contains("args"));
        // Regression: the previous cursor mixed relative and absolute
        // offsets and could fail to advance on such bodies.
        let repeated: String = "final a = b;\n".repeat(64);
        let _ = collect_declared_identifiers(&repeated, "");
    }

    #[test]
    fn assigns_closures_to_parents_by_source_line_containment() {
        let mut parent = sample_function();
        parent.name = "build".to_owned();
        parent.source_location = Some(crate::model::RecoveredSourceLocation {
            path: "package:app/main.dart".to_owned(),
            line: Some(70),
            column: None,
            end_line: Some(120),
            end_column: None,
        });
        let mut outside = sample_function();
        outside.name = "other".to_owned();
        outside.source_location = Some(crate::model::RecoveredSourceLocation {
            path: "package:app/main.dart".to_owned(),
            line: Some(200),
            column: None,
            end_line: Some(210),
            end_column: None,
        });
        let mut closure = sample_function();
        closure.name = "<anonymous closure>".to_owned();
        closure.kind = Some(RecoveredFunctionKind::Closure);
        closure.source_location = Some(crate::model::RecoveredSourceLocation {
            path: "package:app/main.dart".to_owned(),
            line: Some(100),
            column: None,
            end_line: None,
            end_column: None,
        });

        let members = [&outside, &parent, &closure];
        assert_eq!(closure_parent_index(&members, 2), Some(1));
    }

    #[test]
    fn converts_vm_arrow_types_to_dart_function_types() {
        assert_eq!(
            normalize_function_type_syntax(
                "Pointer<NativeFunction<(Pointer<Void>, Uint8) => Bool>>"
            ),
            "Pointer<NativeFunction<Bool Function(Pointer<Void>, Uint8)>>"
        );
        assert_eq!(
            normalize_function_type_syntax("(() => String)?"),
            "String Function()?"
        );
    }

    #[test]
    fn computes_nested_support_import() {
        let library = RecoveredLibrary {
            uri: "package:app/features/home.dart".to_owned(),
            package: Some("app".to_owned()),
            output_path: PathBuf::from("features/home.dart"),
            is_application: true,
            vm_object_id: None,
            imports: Vec::new(),
            referenced_libraries: Vec::new(),
        };
        assert_eq!(
            relative_support_import(&library),
            "../../support/aot_intrinsics.dart"
        );
    }

    #[test]
    fn qualifies_snapshot_evidence_intrinsics_in_generated_dart() {
        assert_eq!(
            render_readable_expression("snapshotRef(42)", &BTreeMap::new()),
            "aot.snapshotRef(42)"
        );
        assert_eq!(
            render_readable_expression(
                "snapshotRef(42) nestedStrings[\"value\"]",
                &BTreeMap::new()
            ),
            "aot.snapshotRef(42)"
        );
        assert_eq!(render_readable_expression("arg0", &BTreeMap::new()), "arg0");
        assert!(render_support().contains("dynamic snapshotRef(int reference)"));
    }

    #[test]
    fn presents_runtime_calls_with_dart_facing_names() {
        assert_eq!(
            friendly_invoke_target("_GrowableList._GrowableList.of"),
            "List.of"
        );
        assert_eq!(
            variable_stem("ScaffoldMessenger.of", false),
            "scaffoldMessengerResult"
        );
        assert_eq!(
            variable_stem("FilledButton.FilledButton.icon", true),
            "filledButton"
        );
        assert_eq!(variable_stem("_StringBase.+", false), "combinedResult");
        assert!(!readable_nested_string("_apiKeySubscription@737003944"));
    }

    #[test]
    fn separates_vm_plumbing_and_dynamic_dispatch_from_readable_calls() {
        let mut function = sample_function();
        function.semantic_statements = vec![
            SemanticStatement::ResolvedCall {
                target: "_iso_stub_AllocateClosureStub".to_owned(),
                arguments: Vec::new(),
                confidence: EvidenceConfidence::Medium,
                address: "0x1000".to_owned(),
            },
            SemanticStatement::ResolvedCall {
                target: "Widget.build".to_owned(),
                arguments: vec!["arg0".to_owned()],
                confidence: EvidenceConfidence::Medium,
                address: "0x1004".to_owned(),
            },
        ];
        function.statements = vec![PseudoStatement::DispatchTableCall {
            address: "0x1008".to_owned(),
            expression: "dispatch[42 + class_id]".to_owned(),
            selector_offset: 42,
            selector_name: Some("render".to_owned()),
            candidate_targets: vec!["Alpha.render".to_owned(), "Beta.render".to_owned()],
            candidate_count: 2,
            raw_slot_target_count: 2,
        }];

        let mut output = String::new();
        render_readable_calls(&mut output, &function, "  ", &BTreeMap::new());
        render_dynamic_dispatch_evidence(&mut output, &function, "  ");

        assert!(output.contains("Widget.build"));
        assert!(!output.contains("_iso_stub_AllocateClosureStub"));
        assert!(output.contains("VM plumbing omitted"));
        assert!(output.contains(".render(...)"));
        assert!(output.contains("Alpha.render"));
    }

    #[test]
    fn constructor_pseudocode_names_the_implicit_instance_receiver() {
        let library = RecoveredLibrary {
            uri: "package:app/vector.dart".to_owned(),
            package: Some("app".to_owned()),
            output_path: PathBuf::from("vector.dart"),
            is_application: true,
            vm_object_id: None,
            imports: Vec::new(),
            referenced_libraries: Vec::new(),
        };
        let mut function = sample_function();
        function.name = "_".to_owned();
        function.kind = Some(RecoveredFunctionKind::Constructor);
        function.semantic_statements = vec![SemanticStatement::FieldWrite {
            receiver: "this".to_owned(),
            field: "_slot_8".to_owned(),
            offset: 8,
            value: "arg0".to_owned(),
            confidence: EvidenceConfidence::High,
            address: "0x1000".to_owned(),
        }];
        let program = RecoveredProgram {
            libraries: vec![library.clone()],
            functions: vec![function],
            ..RecoveredProgram::default()
        };
        let index = RenderIndex::new(&program);
        let output = render_library(&library, &program, &index);

        assert!(output.contains("final recoveredInstance ="));
        assert!(output.contains("recoveredInstance._slot_8 = arg0;"));
        assert!(!output.contains("this._slot_8 ="));
    }

    #[test]
    fn renders_operator_and_optional_signature_evidence() {
        let function = sample_function();

        assert_eq!(readable_function_name(&function.name), "operator_add");
        assert_eq!(readable_function_name("dyn:+"), "dynamic_operator_add");
        assert_eq!(
            rendered_parameters(&function),
            (
                "(dynamic arg0, [dynamic optionalArg0 /* default unavailable */, dynamic optionalArg1 /* default unavailable */])".to_owned(),
                Some("arg0, optionalArg0, optionalArg1".to_owned())
            )
        );
    }

    #[test]
    fn renders_resolved_types_generic_bounds_and_required_names() {
        let mut function = sample_function();
        function.name = "convert".to_owned();
        function.signature = Some(RecoveredSignature {
            fixed_parameter_count: 0,
            optional_parameter_count: 2,
            optional_parameters_are_named: true,
            implicit_parameter_count: 0,
            type_parameters_reference: Some(40),
            result_type_reference: Some(41),
            parameter_types_reference: Some(42),
            named_parameter_names_reference: Some(43),
            flags: 1,
            packed_type_parameter_counts: 0,
            resolved: Some(RecoveredSignatureDetails {
                return_type: Some(recovered_type(41, "Future<List<String>>")),
                parameters: vec![
                    RecoveredParameter {
                        position: 0,
                        name: Some("label".to_owned()),
                        declared_type: Some(recovered_type(44, "String")),
                        is_named: true,
                        is_required: true,
                    },
                    RecoveredParameter {
                        position: 1,
                        name: Some("count".to_owned()),
                        declared_type: Some(recovered_type(45, "int")),
                        is_named: true,
                        is_required: false,
                    },
                ],
                type_parameters: vec![RecoveredTypeParameter {
                    name: "T".to_owned(),
                    bound: Some(recovered_type(46, "Object")),
                }],
            }),
        });

        assert_eq!(
            rendered_parameters(&function),
            (
                "({required String label, int? count /* default unavailable */})".to_owned(),
                Some("label, count".to_owned())
            )
        );
        assert_eq!(
            rendered_return_type(&function, false),
            "Future<List<String>>"
        );
        let generics = function
            .signature
            .as_ref()
            .and_then(|signature| signature.resolved.as_ref())
            .map(|resolved| render_type_parameters(&resolved.type_parameters))
            .unwrap();
        assert_eq!(generics, "<T extends Object>");
    }

    #[test]
    fn disambiguates_rendered_names_with_precomputed_container_counts() {
        let library = RecoveredLibrary {
            uri: "package:app/vector.dart".to_owned(),
            package: Some("app".to_owned()),
            output_path: PathBuf::from("vector.dart"),
            is_application: true,
            vm_object_id: None,
            imports: Vec::new(),
            referenced_libraries: Vec::new(),
        };
        let first = sample_function();
        let mut second = first.clone();
        second.code_reference = 2;
        second.address = "0x2000".to_owned();
        let program = RecoveredProgram {
            libraries: vec![library.clone()],
            functions: vec![first, second],
            ..RecoveredProgram::default()
        };
        let index = RenderIndex::new(&program);
        let output = render_library(&library, &program, &index);
        assert!(output.contains("operator_add_1("));
        assert!(output.contains("operator_add_2("));
    }

    #[test]
    fn distinguishes_vm_generated_forwarders_from_source_functions() {
        let mut function = sample_function();
        function.name = "main".to_owned();
        function.owner = None;
        function.kind = Some(RecoveredFunctionKind::ImplicitClosure);
        assert_eq!(
            rendered_function_symbol_root(&function, false),
            "main_tearOff"
        );

        function.kind = Some(RecoveredFunctionKind::DynamicInvocationForwarder);
        assert_eq!(
            rendered_function_symbol_root(&function, false),
            "main_dynamicForwarder"
        );
    }

    #[test]
    fn renders_function_scoped_string_pool_evidence() {
        let library = RecoveredLibrary {
            uri: "package:app/vector.dart".to_owned(),
            package: Some("app".to_owned()),
            output_path: PathBuf::from("vector.dart"),
            is_application: true,
            vm_object_id: None,
            imports: Vec::new(),
            referenced_libraries: Vec::new(),
        };
        let mut function = sample_function();
        function.instructions = vec![MachineInstruction {
            address: "0x1000".to_owned(),
            bytes: "00000000".to_owned(),
            mnemonic: "ldr".to_owned(),
            operands: "x1, [x27, #0x20]".to_owned(),
            object_pool_index: Some(2),
            object_pool_value: Some("\"recovered value\"".to_owned()),
        }];
        let program = RecoveredProgram {
            libraries: vec![library.clone()],
            functions: vec![function],
            ..RecoveredProgram::default()
        };
        let index = RenderIndex::new(&program);
        let output = render_library(&library, &program, &index);

        assert!(output.contains("Recovered source literals"));
        assert!(output.contains("'recovered value'"));
        assert!(!output.contains("pool[2]"));
    }

    #[test]
    fn renders_simple_field_initializers_as_dart_fields() {
        let library = RecoveredLibrary {
            uri: "package:app/settings.dart".to_owned(),
            package: Some("app".to_owned()),
            output_path: PathBuf::from("settings.dart"),
            is_application: true,
            vm_object_id: None,
            imports: Vec::new(),
            referenced_libraries: Vec::new(),
        };
        let mut function = sample_function();
        function.name = "init:_settingsUri".to_owned();
        function.owner = Some("_SettingsState".to_owned());
        function.library_uri = Some(library.uri.clone());
        function.kind = Some(RecoveredFunctionKind::FieldInitializer);
        function.signature = Some(RecoveredSignature {
            fixed_parameter_count: 0,
            optional_parameter_count: 0,
            optional_parameters_are_named: false,
            implicit_parameter_count: 0,
            type_parameters_reference: None,
            result_type_reference: None,
            parameter_types_reference: None,
            named_parameter_names_reference: None,
            flags: 0,
            packed_type_parameter_counts: 0,
            resolved: None,
        });
        function.semantic_statements = vec![
            SemanticStatement::ResolvedCall {
                target: "Uri.parse".to_owned(),
                arguments: vec!["\"https://example.test/settings\"".to_owned()],
                confidence: EvidenceConfidence::Medium,
                address: "0x1000".to_owned(),
            },
            SemanticStatement::Return {
                expression: "Uri_parse_result".to_owned(),
                confidence: EvidenceConfidence::Low,
                address: "0x1004".to_owned(),
            },
        ];
        let program = RecoveredProgram {
            libraries: vec![library.clone()],
            functions: vec![function],
            ..RecoveredProgram::default()
        };
        let index = RenderIndex::new(&program);
        let output = render_library(&library, &program, &index);

        assert!(output.contains(
            "static final Uri _settingsUri = Uri.parse('https://example.test/settings');"
        ));
        assert!(!output.contains("init__settingsUri"));
        assert!(!output.contains("0x1000"));
    }

    fn sample_function() -> RecoveredFunction {
        RecoveredFunction {
            code_reference: 1,
            code_alias_references: Vec::new(),
            name: "+".to_owned(),
            name_source: RecoveredNameSource::Snapshot,
            snapshot_name: None,
            obfuscated_name: None,
            owner: Some("Vector".to_owned()),
            library_uri: Some("package:app/vector.dart".to_owned()),
            source_location: None,
            inlined_functions: Vec::new(),
            kind: Some(RecoveredFunctionKind::Regular),
            is_static: None,
            signature: Some(RecoveredSignature {
                fixed_parameter_count: 1,
                optional_parameter_count: 2,
                optional_parameters_are_named: false,
                implicit_parameter_count: 1,
                type_parameters_reference: None,
                result_type_reference: None,
                parameter_types_reference: None,
                named_parameter_names_reference: None,
                flags: 0,
                packed_type_parameter_counts: 0,
                resolved: None,
            }),
            signature_source: Some(RecoveredSignatureSource::SnapshotFunction),
            parameter_count: Some(3),
            lexical_parent: None,
            vm_evidence: None,
            address: "0x1000".to_owned(),
            size: 4,
            code_metadata: None,
            machine_code: MachineCodeEvidence::default(),
            instructions: Vec::new(),
            control_flow: Vec::new(),
            semantic_statements: Vec::new(),
            statements: Vec::new(),
        }
    }

    fn recovered_type(snapshot_reference: i32, display_name: &str) -> RecoveredType {
        RecoveredType {
            snapshot_reference,
            display_name: display_name.to_owned(),
            library_uri: Some("dart:core".to_owned()),
        }
    }
}
