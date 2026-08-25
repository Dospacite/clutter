use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostic::{ClutterError, Result};
use crate::model::{
    Abi, RecoveredDeclaration, RecoveredDeclarationKind, RecoveredFunction, RecoveredFunctionKind,
    RecoveredNameSource, RecoveredType, Scope,
};
use crate::snapshot::CodeImage;

use super::debug_symbols::DebugSymbols;

pub fn recover_declarations(
    debug: &DebugSymbols,
    scope: Scope,
    application_package: Option<&str>,
) -> Vec<RecoveredDeclaration> {
    debug
        .declarations
        .iter()
        .filter(|declaration| {
            include_library(
                declaration.library_uri.as_deref(),
                scope,
                application_package,
            )
        })
        .map(|declaration| {
            let (owner, name, function_kind) = split_debug_name(&declaration.name);
            RecoveredDeclaration {
                snapshot_reference: -1,
                vm_object_id: None,
                kind: RecoveredDeclarationKind::Function,
                name,
                snapshot_name: None,
                owner,
                library_uri: declaration.library_uri.clone(),
                source_location: Some(declaration.source_location.clone()),
                function_kind,
                signature: None,
                vm_evidence: None,
                class_metadata: None,
                field_metadata: None,
                has_code: declaration.has_code,
            }
        })
        .collect()
}

pub fn recover_linked_snapshot_declarations(
    debug: &DebugSymbols,
    snapshot_functions: &[RecoveredFunction],
    snapshot_declarations: Vec<RecoveredDeclaration>,
    scope: Scope,
    application_package: Option<&str>,
) -> Vec<RecoveredDeclaration> {
    let debug_by_address =
        debug
            .functions
            .iter()
            .fold(BTreeMap::<u64, Vec<_>>::new(), |mut functions, function| {
                functions
                    .entry(function.address)
                    .or_default()
                    .push(function);
                functions
            });
    let mut library_candidates = BTreeMap::<String, BTreeSet<String>>::new();
    let mut class_candidates = BTreeMap::<String, BTreeSet<String>>::new();
    let mut scoped_class_candidates = BTreeMap::<String, BTreeSet<String>>::new();
    for snapshot in snapshot_functions {
        let Some(address) = parse_address(&snapshot.address) else {
            continue;
        };
        let Some(debug_functions) = debug_by_address.get(&address) else {
            continue;
        };
        for debug_function in debug_functions {
            let is_in_scope = include_library(
                debug_function.library_uri.as_deref(),
                scope,
                application_package,
            );
            if is_in_scope
                && let (Some(raw_library), Some(original_library)) = (
                    snapshot.library_uri.as_ref(),
                    debug_function.library_uri.as_ref(),
                )
            {
                library_candidates
                    .entry(raw_library.clone())
                    .or_default()
                    .insert(original_library.clone());
            }
            let raw_owner = snapshot.owner.as_deref().filter(|owner| {
                !owner.is_empty()
                    && !matches!(*owner, "::" | "top_level")
                    && Some(*owner) != snapshot.library_uri.as_deref()
            });
            let original_owner = split_debug_name(&debug_function.name).0;
            if let (Some(raw_owner), Some(original_owner)) = (raw_owner, original_owner) {
                let candidates = if is_in_scope {
                    &mut scoped_class_candidates
                } else {
                    &mut class_candidates
                };
                candidates
                    .entry(raw_owner.to_owned())
                    .or_default()
                    .insert(original_owner);
            }
        }
    }
    // A shared code range can have aliases from multiple libraries. Prefer
    // candidates from the requested output scope for those raw owners, while
    // retaining global exact-address candidates for SDK types used by app
    // signatures and class graphs.
    class_candidates.extend(scoped_class_candidates);
    let library_names = unambiguous_mappings(library_candidates);
    add_residual_class_candidates(
        debug,
        &snapshot_declarations,
        &library_names,
        &mut class_candidates,
    );
    add_mixin_application_candidates(
        debug,
        &snapshot_declarations,
        &library_names,
        &mut class_candidates,
    );
    let class_names = unambiguous_mappings(class_candidates);

    snapshot_declarations
        .into_iter()
        .filter_map(|mut declaration| {
            let raw_library = declaration.library_uri.as_deref()?;
            let library_uri = library_names.get(raw_library).cloned().or_else(|| {
                (scope == Scope::All
                    || raw_library.starts_with("package:")
                    || raw_library.starts_with("dart:"))
                .then(|| raw_library.to_owned())
            })?;
            if !include_library(Some(&library_uri), scope, application_package) {
                return None;
            }
            declaration.library_uri = Some(library_uri.clone());
            if declaration.kind == RecoveredDeclarationKind::Class {
                restore_declaration_name(&mut declaration, &class_names);
            }
            if let Some(owner) = &mut declaration.owner {
                if let Some(restored) = class_names.get(owner).or_else(|| library_names.get(owner))
                {
                    *owner = restored.clone();
                }
            }
            if let Some(metadata) = &mut declaration.class_metadata {
                for parameter in &mut metadata.type_parameters {
                    if let Some(bound) = &mut parameter.bound {
                        restore_type_names(bound, &class_names);
                    }
                }
                if let Some(super_type) = &mut metadata.super_type {
                    restore_type_names(super_type, &class_names);
                }
                for interface in &mut metadata.interfaces {
                    restore_type_names(interface, &class_names);
                }
            }
            if let Some(metadata) = &mut declaration.field_metadata
                && let Some(declared_type) = &mut metadata.declared_type
            {
                restore_type_names(declared_type, &class_names);
            }
            Some(declaration)
        })
        .collect()
}

fn add_residual_class_candidates(
    debug: &DebugSymbols,
    declarations: &[RecoveredDeclaration],
    library_names: &BTreeMap<String, String>,
    candidates: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let exact_names = unambiguous_mappings(candidates.clone());
    let mut raw_by_library = BTreeMap::<String, BTreeSet<String>>::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.kind == RecoveredDeclarationKind::Class)
    {
        let Some(library) = restored_library(declaration.library_uri.as_deref(), library_names)
        else {
            continue;
        };
        raw_by_library
            .entry(library)
            .or_default()
            .insert(declaration.name.clone());
    }

    let mut debug_by_library = BTreeMap::<String, BTreeSet<String>>::new();
    for declaration in &debug.declarations {
        let Some(library) = declaration.library_uri.clone() else {
            continue;
        };
        let Some(owner) = split_debug_name(&declaration.name)
            .0
            .filter(|owner| !matches!(owner.as_str(), "::" | "top_level"))
        else {
            continue;
        };
        debug_by_library.entry(library).or_default().insert(owner);
    }

    for (library, mut raw_names) in raw_by_library {
        let Some(mut debug_names) = debug_by_library.remove(&library) else {
            continue;
        };
        for (raw, restored) in &exact_names {
            raw_names.remove(raw);
            debug_names.remove(restored);
        }
        let identities = raw_names
            .intersection(&debug_names)
            .cloned()
            .collect::<Vec<_>>();
        for identity in identities {
            raw_names.remove(&identity);
            debug_names.remove(&identity);
        }
        if let ([raw], [restored]) = (
            raw_names.iter().collect::<Vec<_>>().as_slice(),
            debug_names.iter().collect::<Vec<_>>().as_slice(),
        ) {
            candidates
                .entry((*raw).clone())
                .or_default()
                .insert((*restored).clone());
        }
    }
}

fn add_mixin_application_candidates(
    debug: &DebugSymbols,
    declarations: &[RecoveredDeclaration],
    library_names: &BTreeMap<String, String>,
    candidates: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let mut source_mixins = BTreeMap::<String, BTreeSet<String>>::new();
    for declaration in &debug.declarations {
        let Some(library) = declaration.library_uri.clone() else {
            continue;
        };
        let Some(class_name) = declaration.name.strip_prefix("new ") else {
            continue;
        };
        let class_name = class_name.split('.').next().unwrap_or(class_name);
        let Some(mixin) = class_name
            .rsplit('&')
            .next()
            .filter(|_| class_name.contains('&'))
        else {
            continue;
        };
        if looks_like_class_name(mixin) {
            source_mixins
                .entry(library)
                .or_default()
                .insert(mixin.to_owned());
        }
    }

    let mut raw_mixins = BTreeMap::<String, BTreeSet<String>>::new();
    for declaration in declarations {
        let Some(metadata) = declaration
            .class_metadata
            .as_ref()
            .filter(|metadata| metadata.is_transformed_mixin_application)
        else {
            continue;
        };
        let Some(library) = restored_library(declaration.library_uri.as_deref(), library_names)
        else {
            continue;
        };
        let Some(raw_mixin) = metadata
            .interfaces
            .last()
            .map(|value| type_root(&value.display_name))
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        raw_mixins
            .entry(library)
            .or_default()
            .insert(raw_mixin.to_owned());
    }

    for (library, raw) in raw_mixins {
        let Some(source) = source_mixins.get(&library) else {
            continue;
        };
        if let ([raw], [restored]) = (
            raw.iter().collect::<Vec<_>>().as_slice(),
            source.iter().collect::<Vec<_>>().as_slice(),
        ) {
            candidates
                .entry((*raw).clone())
                .or_default()
                .insert((*restored).clone());
        }
    }
}

fn restored_library(library: Option<&str>, names: &BTreeMap<String, String>) -> Option<String> {
    let library = library?;
    names.get(library).cloned().or_else(|| {
        (library.starts_with("package:") || library.starts_with("dart:"))
            .then(|| library.to_owned())
    })
}

fn type_root(value: &str) -> &str {
    value.split(['<', '?']).next().unwrap_or(value).trim()
}

fn unambiguous_mappings(
    candidates: BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, String> {
    candidates
        .into_iter()
        .filter_map(|(raw, candidates)| {
            let mut candidates = candidates.into_iter();
            let restored = candidates.next()?;
            candidates.next().is_none().then_some((raw, restored))
        })
        .collect()
}

fn restore_declaration_name(
    declaration: &mut RecoveredDeclaration,
    names: &BTreeMap<String, String>,
) {
    let Some(restored) = names.get(&declaration.name) else {
        return;
    };
    if declaration.snapshot_name.is_none() {
        declaration.snapshot_name = Some(declaration.name.clone());
    }
    declaration.name = restored.clone();
}

fn restore_type_names(value: &mut RecoveredType, names: &BTreeMap<String, String>) {
    let mut mappings = names.iter().collect::<Vec<_>>();
    mappings.sort_by_key(|(raw, _)| std::cmp::Reverse(raw.len()));
    for (raw, restored) in mappings {
        value.display_name = replace_identifier(&value.display_name, raw, restored);
    }
}

fn replace_identifier(value: &str, raw: &str, restored: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(position) = remaining.find(raw) {
        let before = remaining[..position].chars().next_back();
        let after = remaining[position + raw.len()..].chars().next();
        let boundary = |character: Option<char>| {
            character.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
        };
        output.push_str(&remaining[..position]);
        if boundary(before) && boundary(after) {
            output.push_str(restored);
        } else {
            output.push_str(raw);
        }
        remaining = &remaining[position + raw.len()..];
    }
    output.push_str(remaining);
    output
}

pub fn recover(
    debug: &DebugSymbols,
    snapshot_functions: Vec<RecoveredFunction>,
    code: &CodeImage,
    abi: Abi,
    scope: Scope,
    application_package: Option<&str>,
) -> Result<Vec<RecoveredFunction>> {
    let image_start = code
        .image_virtual_address
        .checked_add(code.code_offset)
        .ok_or_else(|| {
            ClutterError::Analysis("isolate instruction image address overflow".to_owned())
        })?;
    let image_end = image_start
        .checked_add(code.bytes.len() as u64)
        .ok_or_else(|| {
            ClutterError::Analysis("isolate instruction image range overflow".to_owned())
        })?;

    let debug_functions = debug
        .functions
        .iter()
        .filter(|function| {
            function.address >= image_start
                && function.address < image_end
                && include_library(function.library_uri.as_deref(), scope, application_package)
        })
        .collect::<Vec<_>>();
    let all_symbols = debug
        .functions
        .iter()
        .filter(|function| function.address >= image_start && function.address < image_end)
        .map(|function| {
            (
                function.address,
                super::disassembly::Symbol::new(
                    function.name.clone(),
                    function.library_uri.clone(),
                    application_package,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut snapshot_by_address = BTreeMap::<u64, Vec<RecoveredFunction>>::new();
    for function in snapshot_functions {
        if let Some(address) = parse_address(&function.address) {
            snapshot_by_address
                .entry(address)
                .or_default()
                .push(function);
        }
    }

    let disassembler = super::disassembly::Disassembler::new(abi)?;
    let mut recovered = Vec::with_capacity(debug_functions.len());
    for debug_function in debug_functions {
        let Some(relative) = debug_function
            .address
            .checked_sub(image_start)
            .and_then(|value| usize::try_from(value).ok())
        else {
            continue;
        };
        let requested_size = usize::try_from(debug_function.size).unwrap_or(usize::MAX);
        let end = relative
            .saturating_add(requested_size)
            .min(code.bytes.len());
        let Some(bytes) = code.bytes.get(relative..end) else {
            continue;
        };

        let snapshot_match = snapshot_by_address
            .get_mut(&debug_function.address)
            .and_then(|matches| best_snapshot_match(matches, &debug_function.name));
        let (owner, name, inferred_kind) = split_debug_name(&debug_function.name);
        let obfuscated_name = snapshot_match.as_ref().and_then(|snapshot| {
            if let Some(obfuscated) = snapshot.obfuscated_name.clone() {
                return Some(obfuscated);
            }
            let value = display_name(snapshot);
            (snapshot.name_source != RecoveredNameSource::Synthetic && value != debug_function.name)
                .then_some(value)
        });
        let parameter_count = snapshot_match
            .as_ref()
            .and_then(|function| function.parameter_count);
        let mut disassembly = disassembler.analyze(
            debug_function.address,
            bytes,
            &all_symbols,
            parameter_count,
            None,
            None,
        )?;
        if let Some(snapshot) = &snapshot_match {
            retain_snapshot_analysis_hints(&mut disassembly, snapshot);
        }
        let debug_end = debug_function.address.saturating_add(bytes.len() as u64);
        let inlined_functions = debug
            .inlined_functions
            .iter()
            .filter(|inline| {
                parse_address(&inline.address)
                    .is_some_and(|address| (debug_function.address..debug_end).contains(&address))
            })
            .cloned()
            .collect();
        recovered.push(RecoveredFunction {
            code_reference: snapshot_match
                .as_ref()
                .map_or(-1, |function| function.code_reference),
            code_alias_references: snapshot_match
                .as_ref()
                .map_or_else(Vec::new, |function| function.code_alias_references.clone()),
            name,
            name_source: RecoveredNameSource::SplitDebugInfo,
            snapshot_name: snapshot_match
                .as_ref()
                .and_then(|function| function.snapshot_name.clone()),
            obfuscated_name,
            owner,
            library_uri: debug_function.library_uri.clone(),
            source_location: debug_function.source_location.clone(),
            inlined_functions,
            inline_regions: snapshot_match
                .as_ref()
                .map_or_else(Vec::new, |function| function.inline_regions.clone()),
            kind: snapshot_match
                .as_ref()
                .and_then(|function| function.kind)
                .or(inferred_kind),
            is_static: snapshot_match
                .as_ref()
                .and_then(|function| function.is_static),
            signature: snapshot_match
                .as_ref()
                .and_then(|function| function.signature.clone()),
            signature_source: snapshot_match
                .as_ref()
                .and_then(|function| function.signature_source),
            parameter_count: snapshot_match
                .as_ref()
                .and_then(|function| function.parameter_count),
            lexical_parent: snapshot_match
                .as_ref()
                .and_then(|function| function.lexical_parent.clone()),
            vm_evidence: snapshot_match
                .as_ref()
                .and_then(|function| function.vm_evidence.clone()),
            address: format!("0x{:x}", debug_function.address),
            size: bytes.len() as u64,
            code_metadata: snapshot_match
                .as_ref()
                .and_then(|function| function.code_metadata.clone()),
            machine_code: disassembly.evidence,
            instructions: disassembly.instructions,
            control_flow: disassembly.control_flow,
            semantic_statements: disassembly.semantic_statements,
            statements: disassembly.statements,
        });
    }
    recovered.sort_by_key(|function| parse_address(&function.address).unwrap_or(u64::MAX));
    Ok(recovered)
}

fn retain_snapshot_analysis_hints(
    disassembly: &mut super::disassembly::Disassembly,
    snapshot: &RecoveredFunction,
) {
    let indirect_hints = snapshot
        .statements
        .iter()
        .filter_map(|statement| match statement {
            crate::model::PseudoStatement::ObjectPoolCall { address, .. }
            | crate::model::PseudoStatement::RecoveredIndirectCall { address, .. }
            | crate::model::PseudoStatement::DispatchTableCall { address, .. } => {
                Some((address.clone(), statement.clone()))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for statement in &mut disassembly.statements {
        let crate::model::PseudoStatement::IndirectCall { address, .. } = statement else {
            continue;
        };
        if let Some(hint) = indirect_hints.get(address) {
            *statement = hint.clone();
        }
    }

    let pool_annotations = snapshot
        .instructions
        .iter()
        .filter(|instruction| instruction.object_pool_index.is_some())
        .map(|instruction| {
            (
                instruction.address.as_str(),
                (
                    instruction.object_pool_index,
                    instruction.object_pool_value.as_ref(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for instruction in &mut disassembly.instructions {
        if let Some((index, value)) = pool_annotations.get(instruction.address.as_str()) {
            instruction.object_pool_index = *index;
            instruction.object_pool_value = value.cloned();
        }
    }

    let existing_semantic_addresses = disassembly
        .semantic_statements
        .iter()
        .map(|statement| semantic_address(statement).to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    let instruction_addresses = disassembly
        .instructions
        .iter()
        .map(|instruction| instruction.address.clone())
        .collect::<std::collections::BTreeSet<_>>();
    disassembly.semantic_statements.extend(
        snapshot
            .semantic_statements
            .iter()
            .filter(|statement| {
                matches!(
                    statement,
                    crate::model::SemanticStatement::ResolvedCall { .. }
                ) && instruction_addresses.contains(semantic_address(statement))
                    && !existing_semantic_addresses.contains(semantic_address(statement))
            })
            .cloned(),
    );
    disassembly
        .semantic_statements
        .sort_by(|left, right| semantic_address(left).cmp(semantic_address(right)));
    disassembly.evidence.object_pool_loads = disassembly
        .instructions
        .iter()
        .filter(|instruction| instruction.object_pool_index.is_some())
        .count();
    disassembly.evidence.dispatch_table_calls = disassembly
        .statements
        .iter()
        .filter(|statement| {
            matches!(
                statement,
                crate::model::PseudoStatement::DispatchTableCall { .. }
            )
        })
        .count();
    disassembly.evidence.resolved_dispatch_table_calls = 0;
    disassembly.evidence.semantic_statements = disassembly.semantic_statements.len();
}

fn semantic_address(statement: &crate::model::SemanticStatement) -> &str {
    match statement {
        crate::model::SemanticStatement::Return { address, .. }
        | crate::model::SemanticStatement::ResolvedCall { address, .. }
        | crate::model::SemanticStatement::FieldRead { address, .. }
        | crate::model::SemanticStatement::FieldWrite { address, .. }
        | crate::model::SemanticStatement::Condition { address, .. }
        | crate::model::SemanticStatement::StringInterpolation { address, .. } => address,
    }
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

fn parse_address(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

fn best_snapshot_match(
    matches: &mut Vec<RecoveredFunction>,
    debug_name: &str,
) -> Option<RecoveredFunction> {
    if matches.is_empty() {
        return None;
    }
    let wanted_kind = infer_kind(debug_name);
    let index = matches
        .iter()
        .position(|function| function.kind == wanted_kind)
        .unwrap_or(0);
    Some(matches.remove(index))
}

fn split_debug_name(value: &str) -> (Option<String>, String, Option<RecoveredFunctionKind>) {
    if let Some(class_name) = value.strip_prefix("new ") {
        if let Some((owner, constructor)) = class_name.split_once('.') {
            return (
                Some(owner.to_owned()),
                constructor.to_owned(),
                Some(RecoveredFunctionKind::Constructor),
            );
        }
        return (
            Some(class_name.to_owned()),
            "constructor".to_owned(),
            Some(RecoveredFunctionKind::Constructor),
        );
    }
    if let Some((owner, name)) = value.split_once('.')
        && looks_like_class_name(owner)
    {
        return (Some(owner.to_owned()), name.to_owned(), infer_kind(value));
    }
    (Some("::".to_owned()), value.to_owned(), infer_kind(value))
}

fn looks_like_class_name(value: &str) -> bool {
    value
        .trim_start_matches('_')
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
}

fn infer_kind(value: &str) -> Option<RecoveredFunctionKind> {
    if value.starts_with("new ") {
        Some(RecoveredFunctionKind::Constructor)
    } else if value.contains("<anonymous closure>") {
        Some(RecoveredFunctionKind::Closure)
    } else if value.contains("|get#") {
        Some(RecoveredFunctionKind::Getter)
    } else if value.contains("|set#")
        || (value.ends_with('=') && !value.ends_with("==") && !value.ends_with("[]="))
    {
        Some(RecoveredFunctionKind::Setter)
    } else {
        Some(RecoveredFunctionKind::Regular)
    }
}

fn display_name(function: &RecoveredFunction) -> String {
    match function.owner.as_deref() {
        Some(owner) if !matches!(owner, "::" | "top_level") => {
            format!("{owner}.{}", function.name)
        }
        _ => function.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use crate::analysis::debug_symbols::{DebugDeclaration, DebugSymbols};
    use crate::model::{
        RecoveredClassMetadata, RecoveredDeclaration, RecoveredDeclarationKind,
        RecoveredFunctionKind, RecoveredSourceLocation, RecoveredType, Scope,
    };

    use super::{
        recover_linked_snapshot_declarations, replace_identifier, split_debug_name,
        unambiguous_mappings,
    };

    #[test]
    fn separates_class_members_from_top_level_nested_closures() {
        assert_eq!(
            split_debug_name("_ScreenState.build.<anonymous closure>"),
            (
                Some("_ScreenState".to_owned()),
                "build.<anonymous closure>".to_owned(),
                Some(RecoveredFunctionKind::Closure),
            )
        );
        assert_eq!(
            split_debug_name("pipeline.capture"),
            (
                Some("::".to_owned()),
                "pipeline.capture".to_owned(),
                Some(RecoveredFunctionKind::Regular),
            )
        );
        assert_eq!(
            split_debug_name("new Vector"),
            (
                Some("Vector".to_owned()),
                "constructor".to_owned(),
                Some(RecoveredFunctionKind::Constructor),
            )
        );
        assert_eq!(
            split_debug_name("new Vector.parse"),
            (
                Some("Vector".to_owned()),
                "parse".to_owned(),
                Some(RecoveredFunctionKind::Constructor),
            )
        );
    }

    #[test]
    fn restores_only_unambiguous_identifier_tokens() {
        let mappings = unambiguous_mappings(BTreeMap::from([
            ("qy".to_owned(), BTreeSet::from(["EdgeVector".to_owned()])),
            (
                "x".to_owned(),
                BTreeSet::from(["First".to_owned(), "Second".to_owned()]),
            ),
        ]));
        assert_eq!(mappings.get("qy").map(String::as_str), Some("EdgeVector"));
        assert!(!mappings.contains_key("x"));
        assert_eq!(
            replace_identifier("List<qy?> and qyValue", "qy", "EdgeVector"),
            "List<EdgeVector?> and qyValue"
        );
    }

    #[test]
    fn links_a_single_remaining_class_without_guessing_among_ambiguities() {
        let debug = debug_symbols(vec![debug_declaration("Screen.build")]);
        let declarations = vec![class_declaration("ab", false, None)];

        let recovered = recover_linked_snapshot_declarations(
            &debug,
            &[],
            declarations,
            Scope::App,
            Some("app"),
        );

        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].name, "Screen");
        assert_eq!(recovered[0].snapshot_name.as_deref(), Some("ab"));
    }

    #[test]
    fn recovers_a_mixin_name_from_the_synthetic_dwarf_application() {
        let debug = debug_symbols(vec![debug_declaration(
            "new _MixinApplication1&Object&AuditTrail",
        )]);
        let declarations = vec![
            class_declaration("zz", false, None),
            class_declaration("xy", true, Some("zz")),
        ];

        let recovered = recover_linked_snapshot_declarations(
            &debug,
            &[],
            declarations,
            Scope::App,
            Some("app"),
        );

        assert!(recovered.iter().any(|value| value.name == "AuditTrail"));
        let application = recovered.iter().find(|value| value.name == "xy").unwrap();
        assert_eq!(
            application.class_metadata.as_ref().unwrap().interfaces[0].display_name,
            "AuditTrail"
        );
    }

    fn debug_symbols(declarations: Vec<DebugDeclaration>) -> DebugSymbols {
        DebugSymbols {
            functions: Vec::new(),
            declarations,
            inlined_functions: Vec::new(),
            application_package: Some("app".to_owned()),
            build_id: String::new(),
            path: PathBuf::new(),
        }
    }

    fn debug_declaration(name: &str) -> DebugDeclaration {
        DebugDeclaration {
            name: name.to_owned(),
            library_uri: Some("package:app/main.dart".to_owned()),
            source_location: RecoveredSourceLocation {
                path: "lib/main.dart".to_owned(),
                line: Some(1),
                column: Some(1),
                end_line: None,
                end_column: None,
            },
            has_code: false,
        }
    }

    fn class_declaration(
        name: &str,
        is_transformed_mixin_application: bool,
        mixin: Option<&str>,
    ) -> RecoveredDeclaration {
        RecoveredDeclaration {
            snapshot_reference: 1,
            vm_object_id: None,
            kind: RecoveredDeclarationKind::Class,
            name: name.to_owned(),
            snapshot_name: None,
            owner: Some("package:app/main.dart".to_owned()),
            library_uri: Some("package:app/main.dart".to_owned()),
            source_location: None,
            function_kind: None,
            signature: None,
            vm_evidence: None,
            class_metadata: Some(RecoveredClassMetadata {
                class_id: 1,
                type_parameters: Vec::new(),
                super_type: None,
                interfaces: mixin
                    .map(|mixin| RecoveredType {
                        snapshot_reference: 2,
                        display_name: mixin.to_owned(),
                        library_uri: Some("package:app/main.dart".to_owned()),
                    })
                    .into_iter()
                    .collect(),
                is_abstract: is_transformed_mixin_application,
                is_enum: false,
                is_sealed: false,
                is_mixin_class: false,
                is_base: false,
                is_interface: false,
                is_final: false,
                is_transformed_mixin_application,
                instance_size: None,
                type_arguments_field_offset: None,
                instance_slots: Vec::new(),
            }),
            field_metadata: None,
            has_code: true,
        }
    }
}
