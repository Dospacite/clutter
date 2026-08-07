use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::model::{
    RecoveredClassMetadata, RecoveredDeclaration, RecoveredDeclarationKind, RecoveredFieldMetadata,
    RecoveredFunction, RecoveredFunctionKind, RecoveredLibrary, RecoveredProgram, RecoveredType,
    RecoveredTypeParameter, SemanticStatement,
};

pub(super) struct RenderIndex {
    functions_by_library: BTreeMap<String, Vec<usize>>,
    declarations_by_library: BTreeMap<String, Vec<usize>>,
    shared_code_primary: BTreeMap<(String, u64), usize>,
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
        Self {
            functions_by_library,
            declarations_by_library,
            shared_code_primary,
        }
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
        render_function_declaration_stub(&mut output, declaration, "", false);
    }
    let top_level_collisions = function_name_collisions(&top_level, false);
    for function in top_level {
        let collision_count = top_level_collisions
            .get(&rendered_function_symbol_root(function, false))
            .copied()
            .unwrap_or(1);
        render_function(
            &mut output,
            program,
            index,
            function,
            "",
            false,
            collision_count,
        );
    }
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
            render_function_declaration_stub(&mut output, declaration, "  ", true);
        }
        let collisions = function_name_collisions(&functions, true);
        for function in functions {
            let collision_count = collisions
                .get(&rendered_function_symbol_root(function, true))
                .copied()
                .unwrap_or(1);
            render_function(
                &mut output,
                program,
                index,
                function,
                "  ",
                true,
                collision_count,
            );
        }
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
    Some(clean_symbol(owner))
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

fn render_function_declaration_stub(
    output: &mut String,
    declaration: &RecoveredDeclaration,
    indent: &str,
    in_class: bool,
) {
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
    writeln!(
        output,
        "{indent}{static_prefix}{return_type} {}{type_parameters}{parameters}{async_modifier} => throw UnsupportedError('AOT body unavailable');",
        symbol,
    )
    .unwrap();
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
    function
        .signature
        .as_ref()
        .and_then(|signature| signature.resolved.as_ref())
        .and_then(|resolved| resolved.return_type.as_ref())
        .map_or_else(|| "dynamic".to_owned(), rendered_type)
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

fn render_function(
    output: &mut String,
    program: &RecoveredProgram,
    index: &RenderIndex,
    function: &RecoveredFunction,
    indent: &str,
    in_class: bool,
    collision_count: usize,
) {
    let owner = function.owner.as_deref().unwrap_or("top_level");
    let symbol_root = rendered_function_symbol_root(function, in_class);
    let unique_suffix = if collision_count > 1 {
        format!("_{}", function.code_reference.unsigned_abs())
    } else {
        String::new()
    };
    let symbol = dart_identifier(&format!("{symbol_root}{unique_suffix}"));

    writeln!(output).unwrap();
    let display_name = if matches!(owner, "::" | "top_level") {
        clean_symbol(&function.name)
    } else {
        format!("{}.{}", clean_symbol(owner), clean_symbol(&function.name))
    };
    if let Some(location) = &function.source_location {
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
    let async_modifier = if function
        .vm_evidence
        .as_ref()
        .is_some_and(|evidence| evidence.is_async == Some(true))
    {
        " async"
    } else {
        ""
    };
    writeln!(
        output,
        "{indent}{static_prefix}{return_type} {symbol}{type_parameters}{parameters}{async_modifier} {{"
    )
    .unwrap();
    let body_indent = format!("{indent}  ");
    if let Some(arguments) = arguments {
        writeln!(output, "{body_indent}final args = <dynamic>[{arguments}];").unwrap();
    }
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
    let aliases = render_readable_calls(output, function, &body_indent, &initial_aliases);
    render_dynamic_dispatch_evidence(output, function, &body_indent);
    render_recovered_field_writes(output, function, &body_indent, &aliases);
    render_recovered_conditions(output, function, &body_indent, &aliases);
    render_control_flow_summary(output, function, &body_indent);
    if let Some((expression, _)) = trustworthy_semantic_return(function) {
        writeln!(
            output,
            "{body_indent}return {};",
            render_readable_expression(expression, &aliases),
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "{body_indent}return aot.unresolvedRegion({}, args);",
            dart_string(&format!("Remaining behavior of {display_name}"))
        )
        .unwrap();
    }
    writeln!(output, "{indent}}}").unwrap();
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

fn render_recovered_field_writes(
    output: &mut String,
    function: &RecoveredFunction,
    indent: &str,
    aliases: &BTreeMap<String, String>,
) {
    const MAX_WRITES: usize = 16;
    let mut rendered = 0usize;
    let mut seen = BTreeSet::new();
    for statement in &function.semantic_statements {
        let SemanticStatement::FieldWrite {
            receiver,
            field,
            value,
            offset,
            ..
        } = statement
        else {
            continue;
        };
        if rendered == MAX_WRITES
            || !seen.insert((receiver.as_str(), field.as_str(), value.as_str(), *offset))
        {
            continue;
        }
        let receiver = render_readable_expression(receiver, aliases);
        let value = render_readable_expression(value, aliases);
        writeln!(
            output,
            "{indent}{receiver}.{} = {value}; // recovered AOT field store +0x{offset:x}",
            dart_identifier(&clean_symbol(field)),
        )
        .unwrap();
        rendered += 1;
    }
    let total = function
        .semantic_statements
        .iter()
        .filter(|statement| matches!(statement, SemanticStatement::FieldWrite { .. }))
        .count();
    if total > rendered {
        writeln!(
            output,
            "{indent}// {} additional field store(s) are retained in reports/functions.json.",
            total - rendered,
        )
        .unwrap();
    }
}

fn render_recovered_conditions(
    output: &mut String,
    function: &RecoveredFunction,
    indent: &str,
    aliases: &BTreeMap<String, String>,
) {
    const MAX_CONDITIONS: usize = 10;
    let conditions = function
        .semantic_statements
        .iter()
        .filter_map(|statement| match statement {
            SemanticStatement::Condition {
                expression,
                true_target,
                ..
            } => Some((expression, true_target)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if conditions.is_empty() {
        return;
    }
    writeln!(output, "{indent}// Recovered branch predicates:").unwrap();
    for (expression, target) in conditions.iter().take(MAX_CONDITIONS) {
        let expression = render_condition_expression(expression, aliases);
        let target = target.as_deref().unwrap_or("unknown target");
        writeln!(
            output,
            "{indent}//   if ({}) → {}",
            safe_comment(&expression),
            safe_comment(target),
        )
        .unwrap();
    }
    if conditions.len() > MAX_CONDITIONS {
        writeln!(
            output,
            "{indent}//   … {} additional predicate(s) are retained in reports/functions.json.",
            conditions.len() - MAX_CONDITIONS,
        )
        .unwrap();
    }
}

fn render_condition_expression(expression: &str, aliases: &BTreeMap<String, String>) -> String {
    let mut rendered = expression.to_owned();
    let mut replacements = aliases.iter().collect::<Vec<_>>();
    replacements.sort_by_key(|(raw, _)| std::cmp::Reverse(raw.len()));
    for (raw, alias) in replacements {
        rendered = rendered.replace(raw, alias);
    }
    rendered
}

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
    target.starts_with("_iso_stub_")
        || target.starts_with("_vm_stub_")
        || target.starts_with("stub_")
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
    let target = target
        .split_once(".dart.")
        .map_or(target, |(_, suffix)| suffix)
        .trim_end_matches('.');
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
    if expression.parse::<i64>().is_ok()
        || expression.parse::<f64>().is_ok()
        || matches!(expression, "true" | "false" | "null")
        || expression
            .strip_prefix("arg")
            .is_some_and(|index| index.chars().all(|character| character.is_ascii_digit()))
        || valid_dart_identifier(expression)
    {
        return if expression == "this" {
            "this".to_owned()
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
    format!("aot.unresolvedValue({})", dart_string(expression))
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
                            let candidate = function
                                .vm_evidence
                                .as_ref()
                                .and_then(|evidence| evidence.logical_match_candidate_count)
                                .is_some_and(|count| count > 1)
                                .then_some("Candidate")
                                .unwrap_or("");
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
    if function.kind == Some(RecoveredFunctionKind::Closure) {
        return true;
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

fn trustworthy_semantic_return(
    function: &RecoveredFunction,
) -> Option<(&str, crate::model::EvidenceConfidence)> {
    if function.machine_code.conditional_branches != 0
        || function.machine_code.unknown_bytes != 0
        || function.machine_code.returns != 1
    {
        return None;
    }
    let mut returns = function
        .semantic_statements
        .iter()
        .filter_map(|statement| match statement {
            SemanticStatement::Return {
                expression,
                confidence,
                ..
            } if *confidence == crate::model::EvidenceConfidence::High => {
                Some((expression.as_str(), *confidence))
            }
            _ => None,
        });
    let value = returns.next()?;
    returns.next().is_none().then_some(value)
}

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

pub fn render_support() -> String {
    r#"// GENERATED SUPPORT FOR CONSERVATIVE AOT PSEUDOCODE.

Never _unresolved(String kind, Object? evidence) =>
    throw UnsupportedError('Unresolved AOT operation: $kind ($evidence)');

dynamic unresolvedRegion(String sourceUri, List<dynamic> args) =>
    _unresolved('region', <Object?>[sourceUri, args]);

dynamic unresolvedValue(String description) =>
    _unresolved('value', description);

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
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::model::{
        EvidenceConfidence, MachineCodeEvidence, MachineInstruction, PseudoStatement,
        RecoveredFunction, RecoveredFunctionKind, RecoveredLibrary, RecoveredNameSource,
        RecoveredParameter, RecoveredProgram, RecoveredSignature, RecoveredSignatureDetails,
        RecoveredSignatureSource, RecoveredType, RecoveredTypeParameter, SemanticStatement,
    };

    use super::{
        RenderIndex, dart_string, friendly_invoke_target, normalize_function_type_syntax,
        readable_function_name, readable_nested_string, relative_support_import,
        render_dynamic_dispatch_evidence, render_library, render_readable_calls,
        render_readable_expression, render_support, render_type_parameters,
        rendered_function_symbol_root, rendered_parameters, rendered_return_type, variable_stem,
    };

    #[test]
    fn escapes_dart_strings() {
        assert_eq!(dart_string("a'b$"), "'a\\'b\\$'");
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
                "({required String label, int count /* default unavailable */})".to_owned(),
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
