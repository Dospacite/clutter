mod debug_recovery;
mod debug_symbols;
pub(crate) mod disassembly;
mod obfuscation;
mod strings;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::LazyLock;

use object::{Object, ObjectSection, ObjectSymbol, SymbolKind};
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::model::{
    RecoveredClassMetadata, RecoveredDeclaration, RecoveredFieldMetadata, RecoveredLibrary,
    RecoveredProgram, RecoveredSignatureSource, RecoveredString, RecoveredStringSource, Scope,
    SnapshotInfo, Warning,
};

pub use debug_recovery::recover as recover_debug_functions;
pub use debug_recovery::recover_declarations as recover_debug_declarations;
pub use debug_recovery::recover_linked_snapshot_declarations;
pub use debug_symbols::load as load_debug_symbols;
pub use obfuscation::{LoadedObfuscationMap, load as load_obfuscation_map};

pub fn recover(libapp: &[u8], snapshot: &SnapshotInfo, scope: Scope) -> RecoveredProgram {
    let scanned = strings::scan(libapp);
    let mut library_uris = BTreeSet::new();
    let mut identifiers = BTreeSet::new();
    let mut recovered_strings = Vec::new();

    for item in scanned {
        if let Some(uri) = strings::library_uri(&item.value) {
            library_uris.insert(uri);
        }
        if strings::is_identifier(&item.value) {
            identifiers.insert(item.value.clone());
        }
        if strings::is_human_string(&item.value) {
            recovered_strings.push(RecoveredString {
                value: item.value,
                source: RecoveredStringSource::ElfScan,
                file_offset: Some(item.offset as u64),
                snapshot_reference: None,
                transform: None,
                confidence: None,
            });
        }
    }

    let application_package = choose_application_package(&library_uris);
    let libraries = library_uris
        .into_iter()
        .filter_map(|uri| {
            let package = package_name(&uri).map(str::to_owned);
            let is_application = package.as_deref() == application_package.as_deref();
            let include = match scope {
                Scope::App => is_application,
                Scope::Packages => package
                    .as_deref()
                    .is_some_and(|package| package != "flutter"),
                Scope::All => true,
            };
            include.then(|| RecoveredLibrary {
                output_path: library_output_path(&uri, application_package.as_deref()),
                uri,
                package,
                is_application,
                vm_object_id: None,
                imports: Vec::new(),
                referenced_libraries: Vec::new(),
            })
        })
        .collect();

    recovered_strings.sort_by(|left, right| {
        left.value
            .cmp(&right.value)
            .then(left.file_offset.cmp(&right.file_offset))
    });
    recovered_strings.dedup_by(|left, right| left.value == right.value);
    recovered_strings.truncate(20_000);

    let mut warnings = Vec::new();
    if application_package.is_none() {
        warnings.push(Warning {
            code: "W_APP_PACKAGE_UNKNOWN".to_owned(),
            message: "No corroborated non-Flutter application package URI was recovered. Exact application ownership is unavailable; Clutter may broaden a narrow scope when snapshot names show obfuscation."
                .to_owned(),
        });
    }
    if !crate::snapshot::is_supported(snapshot) {
        warnings.push(Warning {
            code: "W_SNAPSHOT_PROFILE_UNKNOWN".to_owned(),
            message: "Snapshot metadata is readable, but its object layout is not an exact supported profile."
                .to_owned(),
        });
    }

    RecoveredProgram {
        application_package,
        root_library_uri: None,
        split_debug_info: None,
        obfuscation_map: None,
        vm_oracle: None,
        libraries,
        declarations: Vec::new(),
        identifiers: identifiers.into_iter().take(20_000).collect(),
        strings: recovered_strings,
        functions: Vec::new(),
        snapshot_evidence: None,
        dispatch_table: None,
        cross_abi: None,
        cross_abi_consensus: None,
        body_graph_report: None,
        signature_solutions: None,
        deferred_units: Vec::new(),
        warnings,
        declaration_evidence: Vec::new(),
    }
}

pub fn attach_snapshot_strings(program: &mut RecoveredProgram, strings: Vec<RecoveredString>) {
    program.strings.extend(strings);
    program.strings.sort_by(|left, right| {
        left.value
            .cmp(&right.value)
            .then(left.source.cmp(&right.source))
            .then(left.file_offset.cmp(&right.file_offset))
    });
    program
        .strings
        .dedup_by(|left, right| left.value == right.value);
    program.strings.truncate(50_000);
}

pub fn inspect_deferred_unit(
    path: &str,
    abi: crate::model::Abi,
    bytes: &[u8],
) -> crate::diagnostic::Result<crate::model::DeferredUnitEvidence> {
    let file = object::File::parse(bytes)?;
    let mut snapshot_symbols = file
        .symbols()
        .filter_map(|symbol| symbol.name().ok())
        .filter(|name| name.contains("Snapshot") || name.contains("LoadingUnit"))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    snapshot_symbols.sort();
    snapshot_symbols.dedup();
    let instruction_section_bytes = file
        .sections()
        .filter_map(|section| {
            let name = section.name().ok()?;
            (name.contains("text") || name.contains("SnapshotInstructions"))
                .then_some(section.size())
        })
        .sum();
    let text_symbols = file
        .symbols()
        .filter(|symbol| {
            symbol.is_definition() && symbol.kind() == SymbolKind::Text && symbol.address() != 0
        })
        .count();
    let build_id = file.build_id()?.map(hex::encode);
    Ok(crate::model::DeferredUnitEvidence {
        path: path.to_owned(),
        abi,
        size: bytes.len(),
        sha256: hex::encode(Sha256::digest(bytes)),
        build_id,
        text_symbols,
        snapshot_symbols,
        instruction_section_bytes,
    })
}

pub fn attach_functions(
    program: &mut RecoveredProgram,
    functions: Vec<crate::model::RecoveredFunction>,
    scope: Scope,
) {
    program.functions = functions
        .into_iter()
        .filter(|function| match scope {
            Scope::All => true,
            Scope::Packages => function
                .library_uri
                .as_deref()
                .is_none_or(|uri| !uri.starts_with("package:flutter/")),
            Scope::App => {
                let Some(package) = program.application_package.as_deref() else {
                    return false;
                };
                function
                    .library_uri
                    .as_deref()
                    .is_some_and(|uri| uri.starts_with(&format!("package:{package}/")))
            }
        })
        .collect();
    restore_function_signature_type_names(&mut program.functions, &program.declarations);
    propagate_matching_signatures(&mut program.functions);
    mark_shared_code_aliases(&mut program.functions);
    reconcile_libraries(program, scope);
}

pub(crate) fn reconcile_libraries(program: &mut RecoveredProgram, scope: Scope) {
    let mut known: BTreeSet<_> = program
        .libraries
        .iter()
        .map(|library| library.uri.clone())
        .collect();
    let recovered_uris: BTreeSet<_> = program
        .functions
        .iter()
        .filter_map(|function| function.library_uri.clone())
        .chain(
            program
                .declarations
                .iter()
                .filter_map(|declaration| declaration.library_uri.clone()),
        )
        .collect();
    for uri in recovered_uris {
        if known.insert(uri.clone()) {
            let package = package_name(&uri).map(str::to_owned);
            program.libraries.push(RecoveredLibrary {
                output_path: library_output_path(&uri, program.application_package.as_deref()),
                is_application: package.as_deref() == program.application_package.as_deref(),
                package,
                uri,
                vm_object_id: None,
                imports: Vec::new(),
                referenced_libraries: Vec::new(),
            });
        }
    }
    if scope == Scope::All
        && program
            .functions
            .iter()
            .any(|function| function.library_uri.is_none())
    {
        program.libraries.push(RecoveredLibrary {
            uri: "clutter:unattributed".to_owned(),
            package: None,
            output_path: PathBuf::from("recovered/unattributed.dart"),
            is_application: false,
            vm_object_id: None,
            imports: Vec::new(),
            referenced_libraries: Vec::new(),
        });
    }
    program
        .libraries
        .sort_by(|left, right| left.output_path.cmp(&right.output_path));
}

/// Derives a conservative import graph from resolved direct-call evidence:
/// when a function in library A calls a named function attributed to library
/// B, A plausibly imports B (directly or through a re-export). This fills the
/// gap when no VM oracle ran; the oracle's own authoritative import lists are
/// merged on top elsewhere.
pub(crate) fn derive_import_graph(program: &mut RecoveredProgram) {
    let mut references: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let record = |from: &Option<String>,
                  to: &Option<String>,
                  references: &mut BTreeMap<String, BTreeSet<String>>| {
        if let (Some(from), Some(to)) = (from.clone(), to.clone())
            && from != to
            && !to.is_empty()
        {
            references.entry(from).or_default().insert(to);
        }
    };
    for function in &program.functions {
        for statement in &function.statements {
            match statement {
                crate::model::PseudoStatement::DirectCall {
                    target_library_uri, ..
                }
                | crate::model::PseudoStatement::RecoveredIndirectCall {
                    target_library_uri, ..
                } => {
                    record(&function.library_uri, target_library_uri, &mut references);
                }
                _ => {}
            }
        }
    }
    for library in &mut program.libraries {
        let derived = references.get(&library.uri).cloned().unwrap_or_default();
        if derived.is_empty() {
            continue;
        }
        for uri in derived {
            if !library.imports.contains(&uri) {
                library.imports.push(uri.clone());
            }
            if !library.referenced_libraries.contains(&uri) {
                library.referenced_libraries.push(uri);
            }
        }
        library.imports.sort();
        library.referenced_libraries.sort();
    }
}

fn restore_function_signature_type_names(
    functions: &mut [crate::model::RecoveredFunction],
    declarations: &[RecoveredDeclaration],
) {
    let names = declarations
        .iter()
        .filter(|declaration| declaration.kind == crate::model::RecoveredDeclarationKind::Class)
        .filter_map(|declaration| {
            let snapshot_name = declaration.snapshot_name.as_deref()?;
            let raw = readable_snapshot_name(snapshot_name);
            (raw != declaration.name).then_some((raw, declaration.name.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    if names.is_empty() {
        return;
    }
    for function in functions {
        let Some(resolved) = function
            .signature
            .as_mut()
            .and_then(|signature| signature.resolved.as_mut())
        else {
            continue;
        };
        if let Some(return_type) = &mut resolved.return_type {
            restore_type_display_name(return_type, &names);
        }
        for parameter in &mut resolved.parameters {
            if let Some(declared_type) = &mut parameter.declared_type {
                restore_type_display_name(declared_type, &names);
            }
        }
        for parameter in &mut resolved.type_parameters {
            if let Some(bound) = &mut parameter.bound {
                restore_type_display_name(bound, &names);
            }
        }
    }
}

fn restore_type_display_name(
    recovered_type: &mut crate::model::RecoveredType,
    names: &BTreeMap<String, String>,
) {
    let mut mappings = names.iter().collect::<Vec<_>>();
    mappings.sort_by_key(|(raw, _)| std::cmp::Reverse(raw.len()));
    for (raw, restored) in mappings {
        recovered_type.display_name =
            replace_identifier_token(&recovered_type.display_name, raw, restored);
    }
}

fn replace_identifier_token(value: &str, raw: &str, restored: &str) -> String {
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

pub fn attach_declarations(
    program: &mut RecoveredProgram,
    declarations: Vec<RecoveredDeclaration>,
    scope: Scope,
) {
    program
        .declarations
        .extend(declarations.into_iter().filter(|declaration| {
            include_uri(
                declaration.library_uri.as_deref(),
                scope,
                program.application_package.as_deref(),
            )
        }));
    program.declarations.sort_by(|left, right| {
        left.library_uri
            .cmp(&right.library_uri)
            .then(left.kind.label().cmp(right.kind.label()))
            .then(left.owner.cmp(&right.owner))
            .then(left.name.cmp(&right.name))
            .then(left.snapshot_reference.cmp(&right.snapshot_reference))
    });
    let mut merged = Vec::<RecoveredDeclaration>::with_capacity(program.declarations.len());
    for declaration in std::mem::take(&mut program.declarations) {
        if let Some(index) = merged
            .iter()
            .rposition(|previous| declarations_represent_same_object(previous, &declaration))
        {
            merge_declaration(&mut merged[index], declaration);
            continue;
        }
        merged.push(declaration);
    }
    program.declarations = merged;
}

fn declarations_represent_same_object(
    previous: &RecoveredDeclaration,
    incoming: &RecoveredDeclaration,
) -> bool {
    if previous.kind != incoming.kind
        || previous.library_uri != incoming.library_uri
        || previous.owner != incoming.owner
        || previous.name != incoming.name
    {
        return false;
    }
    if previous.kind != crate::model::RecoveredDeclarationKind::Function {
        return true;
    }
    if previous.function_kind != incoming.function_kind {
        return false;
    }
    if previous.vm_object_id.is_some() && previous.vm_object_id == incoming.vm_object_id {
        return true;
    }
    if previous.vm_object_id.is_none()
        && incoming.vm_object_id.is_none()
        && previous.snapshot_reference == incoming.snapshot_reference
    {
        return true;
    }

    // A snapshot decoder reference and a VM analyzer object ID live in
    // different numbering domains. Merge them one-to-one only when their
    // independently decoded signatures agree. Distinct anonymous closures
    // with the same spelling must otherwise remain distinct declarations.
    let crosses_evidence_sources =
        previous.vm_object_id.is_some() != incoming.vm_object_id.is_some();
    let previous_already_paired =
        previous.vm_object_id.is_some() && previous.snapshot_reference >= 0;
    crosses_evidence_sources
        && !previous_already_paired
        && function_signatures_compatible(
            previous.signature.as_ref(),
            incoming.signature.as_ref(),
            previous.name.as_str(),
        )
        && source_locations_compatible(
            previous.source_location.as_ref(),
            incoming.source_location.as_ref(),
        )
}

fn source_locations_compatible(
    left: Option<&crate::model::RecoveredSourceLocation>,
    right: Option<&crate::model::RecoveredSourceLocation>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.path == right.path && left.line == right.line && left.column == right.column
        }
        _ => true,
    }
}

fn function_signatures_compatible(
    left: Option<&crate::model::RecoveredSignature>,
    right: Option<&crate::model::RecoveredSignature>,
    name: &str,
) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return !matches!(name, "<anonymous closure>" | "anonymous closure");
    };
    if left.fixed_parameter_count != right.fixed_parameter_count
        || left.optional_parameter_count != right.optional_parameter_count
        || left.optional_parameters_are_named != right.optional_parameters_are_named
    {
        return false;
    }
    let fingerprint = |signature: &crate::model::RecoveredSignature| {
        signature.resolved.as_ref().map(|resolved| {
            let return_type = resolved
                .return_type
                .as_ref()
                .map(|value| canonical_declaration_type(&value.display_name));
            let parameters = resolved
                .parameters
                .iter()
                .map(|parameter| {
                    parameter
                        .declared_type
                        .as_ref()
                        .map(|value| canonical_declaration_type(&value.display_name))
                })
                .collect::<Vec<_>>();
            (return_type, parameters)
        })
    };
    match (fingerprint(left), fingerprint(right)) {
        (Some(left), Some(right)) => left == right,
        _ => !matches!(name, "<anonymous closure>" | "anonymous closure"),
    }
}

fn canonical_declaration_type(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .trim_end_matches('?')
        .to_owned()
}

fn merge_declaration(previous: &mut RecoveredDeclaration, declaration: RecoveredDeclaration) {
    previous.has_code |= declaration.has_code;
    if previous.source_location.is_none() {
        previous.source_location = declaration.source_location;
    }
    if previous.signature.is_none() {
        previous.signature = declaration.signature;
    }
    if previous.vm_object_id.is_none() {
        previous.vm_object_id = declaration.vm_object_id;
    }
    if previous.vm_evidence.is_none() {
        previous.vm_evidence = declaration.vm_evidence;
    }
    if previous.function_kind.is_none() {
        previous.function_kind = declaration.function_kind;
    }
    merge_class_metadata(&mut previous.class_metadata, declaration.class_metadata);
    merge_field_metadata(&mut previous.field_metadata, declaration.field_metadata);
    if previous.snapshot_reference < 0 && declaration.snapshot_reference >= 0 {
        previous.snapshot_reference = declaration.snapshot_reference;
        previous.snapshot_name = declaration.snapshot_name;
    }
}

fn merge_class_metadata(
    existing: &mut Option<RecoveredClassMetadata>,
    incoming: Option<RecoveredClassMetadata>,
) {
    let Some(incoming) = incoming else {
        return;
    };
    let Some(existing) = existing else {
        *existing = Some(incoming);
        return;
    };
    if existing.type_parameters.is_empty() {
        existing.type_parameters = incoming.type_parameters;
    }
    if existing.super_type.is_none() {
        existing.super_type = incoming.super_type;
    }
    if existing.interfaces.is_empty() {
        existing.interfaces = incoming.interfaces;
    }
    existing.is_abstract |= incoming.is_abstract;
    existing.is_enum |= incoming.is_enum;
    existing.is_sealed |= incoming.is_sealed;
    existing.is_mixin_class |= incoming.is_mixin_class;
    existing.is_base |= incoming.is_base;
    existing.is_interface |= incoming.is_interface;
    existing.is_final |= incoming.is_final;
    existing.is_transformed_mixin_application |= incoming.is_transformed_mixin_application;
    existing.instance_size = existing.instance_size.or(incoming.instance_size);
    existing.type_arguments_field_offset = existing
        .type_arguments_field_offset
        .or(incoming.type_arguments_field_offset);
    if existing.instance_slots.is_empty() {
        existing.instance_slots = incoming.instance_slots;
    }
}

fn merge_field_metadata(
    existing: &mut Option<RecoveredFieldMetadata>,
    incoming: Option<RecoveredFieldMetadata>,
) {
    let Some(incoming) = incoming else {
        return;
    };
    let Some(existing) = existing else {
        *existing = Some(incoming);
        return;
    };
    if existing.declared_type.is_none() {
        existing.declared_type = incoming.declared_type;
    }
    if existing.initializer_reference < 0 {
        existing.initializer_reference = incoming.initializer_reference;
    }
    existing.offset_or_field_id_reference = existing
        .offset_or_field_id_reference
        .or(incoming.offset_or_field_id_reference);
    existing.is_static |= incoming.is_static;
    existing.is_final |= incoming.is_final;
    existing.is_const |= incoming.is_const;
    existing.is_late |= incoming.is_late;
    existing.has_initializer |= incoming.has_initializer;
    existing.has_nontrivial_initializer |= incoming.has_nontrivial_initializer;
    existing.instance_field_offset = existing
        .instance_field_offset
        .or(incoming.instance_field_offset);
    existing.static_field_offset = existing
        .static_field_offset
        .or(incoming.static_field_offset);
    existing.static_value_object_id = existing
        .static_value_object_id
        .or(incoming.static_value_object_id);
    existing.is_reference = existing.is_reference.or(incoming.is_reference);
    if existing.unboxed_type.is_none() {
        existing.unboxed_type = incoming.unboxed_type;
    }
}

pub fn compare_cross_abi(
    selected_abi: crate::model::Abi,
    selected: &[crate::model::RecoveredFunction],
    alternatives: Vec<(crate::model::Abi, Vec<crate::model::RecoveredFunction>)>,
) -> crate::model::CrossAbiReport {
    type FunctionKey = (Option<String>, Option<String>, String, Option<String>);
    let key = |function: &crate::model::RecoveredFunction| -> FunctionKey {
        (
            function.library_uri.clone(),
            function.owner.clone(),
            function.name.clone(),
            function.kind.map(|kind| kind.label().to_owned()),
        )
    };
    let selected_map = selected
        .iter()
        .map(|function| (key(function), function))
        .collect::<BTreeMap<_, _>>();
    let alternative_maps = alternatives
        .iter()
        .map(|(abi, functions)| {
            (
                *abi,
                functions
                    .iter()
                    .map(|function| (key(function), function))
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .collect::<Vec<_>>();
    let mut report = crate::model::CrossAbiReport {
        selected_abi: Some(selected_abi),
        compared_abis: alternatives.iter().map(|(abi, _)| *abi).collect(),
        ..crate::model::CrossAbiReport::default()
    };
    report.function_counts.insert(selected_abi, selected.len());
    for (abi, functions) in &alternatives {
        report.function_counts.insert(*abi, functions.len());
    }
    for (function_key, selected_function) in selected_map {
        let selected_fingerprint = control_flow_fingerprint(selected_function);
        let mut present_in = vec![selected_abi];
        let mut fingerprints = BTreeMap::from([(selected_abi, selected_fingerprint.clone())]);
        for (abi, functions) in &alternative_maps {
            if let Some(function) = functions.get(&function_key) {
                present_in.push(*abi);
                fingerprints.insert(*abi, control_flow_fingerprint(function));
            }
        }
        if present_in.len() == 1 {
            report.selected_only_functions += 1;
            continue;
        }
        report.matched_functions += 1;
        if fingerprints
            .values()
            .any(|fingerprint| fingerprint != &selected_fingerprint)
        {
            report
                .disagreements
                .push(crate::model::CrossAbiDisagreement {
                    library_uri: function_key.0,
                    owner: function_key.1,
                    name: function_key.2,
                    present_in,
                    fingerprints,
                });
        }
    }
    report.disagreements.sort_by(|left, right| {
        left.library_uri
            .cmp(&right.library_uri)
            .then(left.owner.cmp(&right.owner))
            .then(left.name.cmp(&right.name))
    });
    report
}

fn control_flow_fingerprint(
    function: &crate::model::RecoveredFunction,
) -> crate::model::ControlFlowFingerprint {
    let mut resolved_target_names = function
        .statements
        .iter()
        .filter_map(|statement| match statement {
            crate::model::PseudoStatement::DirectCall {
                target: Some(target),
                ..
            } => Some(target.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    resolved_target_names.sort();
    resolved_target_names.dedup();
    crate::model::ControlFlowFingerprint {
        conditional_branches: function.machine_code.conditional_branches,
        returns: function.machine_code.returns,
        direct_calls: function.machine_code.direct_calls,
        indirect_calls: function.machine_code.indirect_calls,
        resolved_target_names,
    }
}

pub(crate) fn readable_snapshot_name(value: &str) -> String {
    if value == "::" {
        return String::new();
    }
    static PRIVATE_KEY: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"@[0-9]+").expect("static private-key regular expression"));
    PRIVATE_KEY.replace_all(value, "").into_owned()
}

/// Re-lifts every recovered function with full semantic evidence:
/// parameter names from resolved signatures, VM-verified field layouts for
/// every surviving class (including out-of-scope Flutter/Dart SDK classes),
/// constructor result classes, and call-target symbols. This is what turns
/// raw machine code into named field reads/writes, string interpolations,
/// and receiver-aware calls.
pub fn enrich_semantics(
    program: &mut RecoveredProgram,
    abi: crate::model::Abi,
    extra_declarations: &[RecoveredDeclaration],
    extra_functions: &[crate::model::RecoveredFunction],
) {
    use crate::analysis::disassembly::{build_function_symbols, semantic_parameter_hints};

    let application_package = program.application_package.clone();
    let mut all_functions = extra_functions.to_vec();
    all_functions.extend(program.functions.iter().cloned());
    let (mut symbols, target_library_candidates) =
        build_function_symbols(&all_functions, application_package.as_deref());
    // The initial snapshot lift names calls through every code range,
    // including out-of-scope Flutter/Dart SDK targets. Preserve those names
    // when re-lifting with enriched layouts.
    for function in program.functions.iter() {
        for statement in function.statements.iter() {
            let crate::model::PseudoStatement::DirectCall {
                target_address,
                target: Some(target),
                target_library_uri,
                ..
            } = statement
            else {
                continue;
            };
            let Some(address) = parse_hex_address(target_address) else {
                continue;
            };
            let symbol = disassembly::Symbol::new(
                target.clone(),
                target_library_uri.clone(),
                application_package.as_deref(),
            )
            .with_code_identity(
                address,
                0,
                crate::model::DirectCallResolution::ExactEntry,
            );
            match symbols.get(&address) {
                Some(existing) if existing.semantic_name => {}
                _ => {
                    symbols.insert(address, symbol);
                }
            }
        }
    }
    let target_libraries = target_library_candidates
        .into_iter()
        .filter_map(|(target, libraries)| {
            (libraries.len() == 1).then(|| (target, libraries.into_iter().next().flatten()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut layout_declarations = extra_declarations.to_vec();
    layout_declarations.extend(program.declarations.iter().cloned());
    let layouts = disassembly::RecoveredFieldLayout::from_declarations(abi, &layout_declarations);

    // Per-class allocation stubs surface as ranges named after their Class
    // but carry no function kind. Backfill their result class from class
    // declarations so field stores through freshly allocated objects resolve
    // real field names instead of raw slot offsets.
    let mut class_lookup: BTreeMap<String, (String, Option<String>)> = BTreeMap::new();
    for declaration in layout_declarations.iter() {
        if declaration.kind != crate::model::RecoveredDeclarationKind::Class {
            continue;
        }
        let name = readable_snapshot_name(&declaration.name);
        if let Some(entry) = class_lookup.get(&name) {
            // Ambiguous across libraries: drop the mapping.
            if entry.1 != declaration.library_uri {
                class_lookup.remove(&name);
            }
            continue;
        }
        class_lookup.insert(name.clone(), (name, declaration.library_uri.clone()));
    }
    for symbol in symbols.values_mut() {
        if symbol.result_class.is_some() || !symbol.semantic_name {
            continue;
        }
        let leaf = symbol.label.rsplit('.').next().unwrap_or(&symbol.label);
        if let Some((class, library_uri)) = class_lookup.get(leaf) {
            symbol.result_class = Some(class.clone());
            if symbol.library_uri.is_none() {
                symbol.library_uri = library_uri.clone();
            }
        }
    }

    for function in &mut program.functions {
        let parameter_hints = semantic_parameter_hints(function);
        let owner = function.owner.as_deref().map(readable_snapshot_name);
        let receiver_class = owner
            .as_deref()
            .map(|owner| (owner, function.library_uri.as_deref()));
        function.semantic_statements = disassembly::relift_semantics(
            function,
            abi,
            &parameter_hints,
            Some(&layouts),
            receiver_class,
            &symbols,
        );
        promote_recovered_indirect_calls(
            function,
            &target_libraries,
            application_package.as_deref(),
        );
        function.machine_code.semantic_statements = function.semantic_statements.len();
    }
}

pub(crate) fn promote_recovered_indirect_calls(
    function: &mut crate::model::RecoveredFunction,
    target_libraries: &BTreeMap<String, Option<String>>,
    application_package: Option<&str>,
) {
    let recovered_indirect_targets = function
        .semantic_statements
        .iter()
        .filter_map(|statement| match statement {
            crate::model::SemanticStatement::ResolvedCall {
                target, address, ..
            } => Some((address.clone(), target.clone())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for statement in &mut function.statements {
        let crate::model::PseudoStatement::IndirectCall {
            address,
            expression,
        } = statement
        else {
            continue;
        };
        let Some(target) = recovered_indirect_targets.get(address).cloned() else {
            continue;
        };
        let address = address.clone();
        let expression = expression.clone();
        let target_library_uri = target_libraries.get(&target).cloned().flatten();
        let target_scope = crate::analysis::disassembly::call_target_scope(
            &target,
            target_library_uri.as_deref(),
            application_package,
        );
        *statement = crate::model::PseudoStatement::RecoveredIndirectCall {
            address,
            expression,
            target,
            target_library_uri,
            target_scope,
        };
    }
}

fn propagate_matching_signatures(functions: &mut [crate::model::RecoveredFunction]) {
    let signatures = functions
        .iter()
        .filter_map(|function| {
            let signature = function.signature.clone()?;
            Some((
                (
                    function.library_uri.clone(),
                    function.owner.clone(),
                    function.name.clone(),
                ),
                (signature, function.parameter_count),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    for function in functions {
        if function.signature.is_some() {
            continue;
        }
        let key = (
            function.library_uri.clone(),
            function.owner.clone(),
            function.name.clone(),
        );
        if let Some((signature, parameter_count)) = signatures.get(&key) {
            function.signature = Some(signature.clone());
            function.signature_source = Some(RecoveredSignatureSource::RelatedFunction);
            function.parameter_count = *parameter_count;
        }
    }
}

fn mark_shared_code_aliases(functions: &mut [crate::model::RecoveredFunction]) {
    let mut groups = BTreeMap::<(String, u64), Vec<(usize, i32)>>::new();
    for (index, function) in functions.iter().enumerate() {
        groups
            .entry((function.address.clone(), function.size))
            .or_default()
            .push((index, function.code_reference));
    }
    for entries in groups.values().filter(|entries| entries.len() > 1) {
        for (index, _) in entries {
            functions[*index].code_alias_references = entries
                .iter()
                .filter_map(|(other_index, reference)| {
                    (*other_index != *index && *reference >= 0).then_some(*reference)
                })
                .collect();
        }
    }
}

#[derive(Clone)]
struct CallTargetIdentity {
    code_address: u64,
    size: u64,
    label: Option<String>,
    library_uri: Option<String>,
    entry_offset: u64,
    resolution: crate::model::DirectCallResolution,
}

pub(crate) fn relink_calls(program: &mut RecoveredProgram) {
    let application_package = program.application_package.clone();
    let mut grouped = BTreeMap::<u64, Vec<&crate::model::RecoveredFunction>>::new();
    for function in &program.functions {
        if let Some(address) = parse_hex_address(&function.address) {
            grouped.entry(address).or_default().push(function);
        }
    }
    let mut exact = BTreeMap::<u64, CallTargetIdentity>::new();
    let mut ranges = Vec::new();
    for (address, functions) in grouped {
        let size = functions
            .iter()
            .map(|function| function.size)
            .max()
            .unwrap_or_default();
        let semantic_candidates = functions
            .iter()
            .copied()
            .filter(|function| semantic_function_name(function).is_some())
            .filter_map(|function| {
                Some((
                    semantic_function_name(function)?,
                    function.library_uri.clone(),
                    function_name_priority(function.name_source),
                ))
            })
            .fold(
                BTreeMap::<String, (Option<String>, u8)>::new(),
                |mut candidates, (label, library_uri, priority)| {
                    let entry = candidates
                        .entry(label)
                        .or_insert((library_uri.clone(), priority));
                    if priority > entry.1 {
                        *entry = (library_uri, priority);
                    }
                    candidates
                },
            );
        // A physical code body can represent several logically distinct Dart
        // functions after AOT deduplication. Address identity remains exact,
        // but a semantic target name is safe only when every surviving alias
        // agrees on the same qualified name.
        let (label, library_uri) = if semantic_candidates.len() == 1 {
            let (label, (library_uri, _)) = semantic_candidates.into_iter().next().unwrap();
            (Some(label), library_uri)
        } else {
            (None, None)
        };
        let identity = CallTargetIdentity {
            code_address: address,
            size,
            label,
            library_uri,
            entry_offset: 0,
            resolution: crate::model::DirectCallResolution::ExactEntry,
        };
        exact.insert(address, identity.clone());
        ranges.push(identity.clone());

        let unchecked_offsets = functions
            .iter()
            .filter_map(|function| {
                function
                    .code_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.unchecked_entry_offset)
            })
            .filter(|offset| *offset > 0 && *offset < size)
            .collect::<BTreeSet<_>>();
        for offset in unchecked_offsets {
            let mut unchecked = identity.clone();
            unchecked.entry_offset = offset;
            unchecked.resolution = crate::model::DirectCallResolution::UncheckedEntry;
            exact.insert(address.saturating_add(offset), unchecked);
        }
    }
    ranges.sort_by_key(|identity| identity.code_address);

    for function in &mut program.functions {
        let mut resolved = 0usize;
        for statement in &mut function.statements {
            let crate::model::PseudoStatement::DirectCall {
                target_address,
                target_code_address,
                target_entry_offset,
                target_resolution,
                target,
                target_library_uri,
                target_scope,
                ..
            } = statement
            else {
                continue;
            };
            let Some(address) = parse_hex_address(target_address) else {
                continue;
            };
            let identity = exact
                .get(&address)
                .cloned()
                .or_else(|| containing_code_range(&ranges, address));
            let Some(identity) = identity else {
                continue;
            };
            resolved += 1;
            *target_code_address = Some(format!("0x{:x}", identity.code_address));
            *target_entry_offset = Some(address.saturating_sub(identity.code_address));
            *target_resolution = Some(if exact.contains_key(&address) {
                identity.resolution
            } else {
                crate::model::DirectCallResolution::RangeInterior
            });
            // An interior address proves physical code ownership but not a
            // callable Dart entry point. Promote semantic identity only for
            // exact checked/unchecked entries.
            if exact.contains_key(&address) {
                if let Some(label) = identity.label {
                    *target = Some(label.clone());
                    *target_library_uri = identity.library_uri.clone();
                    *target_scope = crate::analysis::disassembly::call_target_scope(
                        &label,
                        identity.library_uri.as_deref(),
                        application_package.as_deref(),
                    );
                } else {
                    *target = None;
                    *target_library_uri = None;
                    *target_scope = crate::model::CallTargetScope::Unknown;
                }
            }
        }
        function.machine_code.code_resolved_direct_calls = resolved;

        let direct_targets = function
            .statements
            .iter()
            .filter_map(|statement| match statement {
                crate::model::PseudoStatement::DirectCall {
                    address, target, ..
                } => Some((address.clone(), target.clone())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        function.semantic_statements.retain_mut(|statement| {
            let crate::model::SemanticStatement::ResolvedCall {
                target, address, ..
            } = statement
            else {
                return true;
            };
            match direct_targets.get(address) {
                Some(Some(relinked)) => {
                    *target = relinked.clone();
                    true
                }
                Some(None) => false,
                None => true,
            }
        });
        function.machine_code.semantic_statements = function.semantic_statements.len();
    }

    if let Some(dispatch) = &mut program.dispatch_table {
        for run in &mut dispatch.runs {
            let Some(address) = run.target_address.as_deref().and_then(parse_hex_address) else {
                continue;
            };
            let Some(identity) = exact.get(&address) else {
                continue;
            };
            if let Some(label) = &identity.label {
                run.target = Some(label.clone());
                run.target_library_uri = identity.library_uri.clone();
            } else {
                run.target = None;
                run.target_library_uri = None;
            }
        }
    }
}

fn containing_code_range(
    ranges: &[CallTargetIdentity],
    address: u64,
) -> Option<CallTargetIdentity> {
    let index = ranges.partition_point(|range| range.code_address <= address);
    let range = ranges.get(index.saturating_sub(1))?;
    (address < range.code_address.saturating_add(range.size)).then(|| {
        let mut interior = range.clone();
        interior.entry_offset = address.saturating_sub(range.code_address);
        interior.resolution = crate::model::DirectCallResolution::RangeInterior;
        interior.label = None;
        interior.library_uri = None;
        interior
    })
}

fn semantic_function_name(function: &crate::model::RecoveredFunction) -> Option<String> {
    if function.name_source == crate::model::RecoveredNameSource::Synthetic
        || function.name.starts_with("sub_")
        || matches!(function.name.as_str(), "" | "unknownFunction")
        || function.vm_evidence.as_ref().is_some_and(|evidence| {
            matches!(
                evidence.kind.as_deref(),
                Some("AotCodeBoundary" | "SharedAotCodeBoundary")
            )
        })
    {
        return None;
    }
    Some(match function.owner.as_deref() {
        Some(owner) if !matches!(owner, "::" | "top_level") => {
            format!("{owner}.{}", function.name)
        }
        _ => function.name.clone(),
    })
}

fn function_name_priority(source: crate::model::RecoveredNameSource) -> u8 {
    match source {
        crate::model::RecoveredNameSource::SplitDebugInfo => 5,
        crate::model::RecoveredNameSource::ObfuscationMap => 4,
        crate::model::RecoveredNameSource::Snapshot => 3,
        crate::model::RecoveredNameSource::DartVmOracle => 2,
        // Guesses rank below every evidence-backed name; LLM-assisted names
        // additionally lose to plain synthetic labels because they look
        // authoritative while having no provenance.
        crate::model::RecoveredNameSource::Synthetic => 1,
        crate::model::RecoveredNameSource::LlmAssisted => 0,
    }
}

fn parse_hex_address(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

fn include_uri(uri: Option<&str>, scope: Scope, application_package: Option<&str>) -> bool {
    match scope {
        Scope::All => true,
        Scope::Packages => uri
            .is_some_and(|uri| uri.starts_with("package:") && !uri.starts_with("package:flutter/")),
        Scope::App => application_package.is_some_and(|package| {
            uri.is_some_and(|uri| uri.starts_with(&format!("package:{package}/")))
        }),
    }
}

pub(crate) fn choose_application_package(libraries: &BTreeSet<String>) -> Option<String> {
    let mut candidates = BTreeMap::<String, (usize, bool)>::new();
    for uri in libraries {
        let Some(package) = package_name(uri) else {
            continue;
        };
        if matches!(package, "flutter" | "flutter_test") {
            continue;
        }
        let is_main = uri == &format!("package:{package}/main.dart");
        let candidate = candidates.entry(package.to_owned()).or_default();
        candidate.0 += 1;
        candidate.1 |= is_main;
    }
    let eligible = candidates
        .into_iter()
        // One isolated package URI is commonly a surviving dependency string,
        // not proof of application ownership. A main library or corroborating
        // URIs are required before narrowing `--scope app`.
        .filter(|(_, (count, has_main))| *has_main || *count >= 2)
        .collect::<Vec<_>>();
    let best_evidence = eligible
        .iter()
        .map(|(_, (count, has_main))| (*has_main, *count))
        .max()?;
    let mut best = eligible
        .into_iter()
        .filter(|(_, (count, has_main))| (*has_main, *count) == best_evidence);
    let (name, _) = best.next()?;
    // Equal evidence is ambiguity, not a reason to pick whichever package
    // name happens to sort first and then discard the other package at app
    // scope.
    best.next().is_none().then_some(name)
}

fn package_name(uri: &str) -> Option<&str> {
    uri.strip_prefix("package:")?.split('/').next()
}

pub(crate) fn library_output_path(uri: &str, application_package: Option<&str>) -> PathBuf {
    if let Some(rest) = uri.strip_prefix("package:")
        && let Some((package, path)) = rest.split_once('/')
    {
        if Some(package) != application_package {
            return PathBuf::from("packages")
                .join(package)
                .join(sanitize_relative(path));
        }
        return sanitize_relative(path);
    }
    if let Some(path) = uri.strip_prefix("dart:") {
        return PathBuf::from("dart").join(sanitize_relative(path));
    }
    let digest = Sha256::digest(uri.as_bytes());
    PathBuf::from(format!("recovered_{}.dart", &hex::encode(digest)[..12]))
}

fn sanitize_relative(value: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for segment in value.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            continue;
        }
        let safe: String = segment
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.') {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        if !safe.is_empty() {
            path.push(safe);
        }
    }
    if path.extension().is_none() {
        path.set_extension("dart");
    }
    path
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::model::{
        Abi, CallTargetScope, MachineCodeEvidence, PseudoStatement, RecoveredDeclaration,
        RecoveredDeclarationKind, RecoveredFunction, RecoveredFunctionKind, RecoveredNameSource,
        RecoveredParameter, RecoveredProgram, RecoveredSignature, RecoveredSignatureDetails,
        RecoveredType,
    };

    use super::{
        choose_application_package, compare_cross_abi, declarations_represent_same_object,
        library_output_path, relink_calls,
    };

    fn recovered_function(
        name: &str,
        conditional_branches: usize,
        target: &str,
    ) -> RecoveredFunction {
        RecoveredFunction {
            code_reference: 1,
            code_alias_references: Vec::new(),
            name: name.to_owned(),
            name_source: RecoveredNameSource::Snapshot,
            snapshot_name: Some(name.to_owned()),
            obfuscated_name: None,
            owner: Some("Example".to_owned()),
            library_uri: Some("package:app/example.dart".to_owned()),
            source_location: None,
            inlined_functions: Vec::new(),
            kind: Some(RecoveredFunctionKind::Regular),
            is_static: Some(true),
            signature: None,
            signature_source: None,
            parameter_count: None,
            lexical_parent: None,
            vm_evidence: None,
            address: "0x1000".to_owned(),
            size: 8,
            code_metadata: None,
            machine_code: MachineCodeEvidence {
                conditional_branches,
                returns: 1,
                direct_calls: 1,
                ..MachineCodeEvidence::default()
            },
            instructions: Vec::new(),
            control_flow: Vec::new(),
            semantic_statements: Vec::new(),
            statements: vec![PseudoStatement::DirectCall {
                address: "0x1000".to_owned(),
                target_address: "0x2000".to_owned(),
                target_code_address: None,
                target_entry_offset: None,
                target_resolution: None,
                target: Some(target.to_owned()),
                target_library_uri: None,
                target_scope: CallTargetScope::Unknown,
            }],
        }
    }

    fn closure_declaration(
        snapshot_reference: i32,
        vm_object_id: Option<u64>,
        parameter_type: &str,
    ) -> RecoveredDeclaration {
        RecoveredDeclaration {
            snapshot_reference,
            vm_object_id,
            kind: RecoveredDeclarationKind::Function,
            name: "<anonymous closure>".to_owned(),
            snapshot_name: None,
            owner: Some("Example".to_owned()),
            library_uri: Some("package:app/example.dart".to_owned()),
            source_location: None,
            function_kind: Some(RecoveredFunctionKind::Closure),
            signature: Some(RecoveredSignature {
                fixed_parameter_count: 1,
                optional_parameter_count: 0,
                optional_parameters_are_named: false,
                implicit_parameter_count: 1,
                type_parameters_reference: None,
                result_type_reference: None,
                parameter_types_reference: None,
                named_parameter_names_reference: None,
                flags: 0,
                packed_type_parameter_counts: 0,
                resolved: Some(RecoveredSignatureDetails {
                    return_type: Some(RecoveredType {
                        snapshot_reference: 1,
                        display_name: "bool".to_owned(),
                        library_uri: Some("dart:core".to_owned()),
                    }),
                    parameters: vec![RecoveredParameter {
                        position: 0,
                        name: None,
                        declared_type: Some(RecoveredType {
                            snapshot_reference: 2,
                            display_name: parameter_type.to_owned(),
                            library_uri: Some("package:app/model.dart".to_owned()),
                        }),
                        is_named: false,
                        is_required: false,
                    }],
                    type_parameters: Vec::new(),
                }),
            }),
            vm_evidence: None,
            class_metadata: None,
            field_metadata: None,
            has_code: true,
        }
    }

    #[test]
    fn chooses_package_with_main_library() {
        let libraries = BTreeSet::from([
            "package:collection/src/list.dart".to_owned(),
            "package:my_app/main.dart".to_owned(),
            "package:my_app/home.dart".to_owned(),
        ]);
        assert_eq!(
            choose_application_package(&libraries).as_deref(),
            Some("my_app")
        );
    }

    #[test]
    fn does_not_mistake_one_surviving_dependency_for_the_application() {
        let libraries = BTreeSet::from([
            "package:ffi/src/allocation.dart".to_owned(),
            "package:flutter/widgets.dart".to_owned(),
        ]);
        assert_eq!(choose_application_package(&libraries), None);
    }

    #[test]
    fn leaves_equally_supported_packages_ambiguous() {
        let libraries = BTreeSet::from([
            "package:first/main.dart".to_owned(),
            "package:first/model.dart".to_owned(),
            "package:second/main.dart".to_owned(),
            "package:second/model.dart".to_owned(),
        ]);
        assert_eq!(choose_application_package(&libraries), None);
    }

    #[test]
    fn maps_package_uri_to_source_path() {
        assert_eq!(
            library_output_path("package:my_app/features/home.dart", Some("my_app")),
            std::path::PathBuf::from("features/home.dart")
        );
        assert_eq!(
            library_output_path("package:dependency/src/value.dart", Some("my_app")),
            std::path::PathBuf::from("packages/dependency/src/value.dart")
        );
        assert_eq!(
            library_output_path("dart:core/duration.dart", Some("my_app")),
            std::path::PathBuf::from("dart/core/duration.dart")
        );
    }

    #[test]
    fn preserves_distinct_same_named_closures_and_pairs_cross_source_evidence() {
        let first = closure_declaration(10, None, "Cookie");
        let second = closure_declaration(11, None, "Cookie");
        let matching_vm = closure_declaration(i32::MIN + 20, Some(20), "Cookie");
        let different_vm = closure_declaration(i32::MIN + 21, Some(21), "Header");

        assert!(!declarations_represent_same_object(&first, &second));
        assert!(declarations_represent_same_object(&first, &matching_vm));
        assert!(!declarations_represent_same_object(&first, &different_vm));
    }

    #[test]
    fn cross_abi_comparison_uses_logical_identity_and_semantic_fingerprint() {
        let selected = vec![
            recovered_function("same", 1, "Target.call"),
            recovered_function("different", 0, "Old.call"),
            recovered_function("selectedOnly", 0, "Only.call"),
        ];
        let alternative = vec![
            recovered_function("same", 1, "Target.call"),
            recovered_function("different", 2, "New.call"),
        ];
        let report = compare_cross_abi(Abi::Arm64V8a, &selected, vec![(Abi::X86_64, alternative)]);

        assert_eq!(report.matched_functions, 2);
        assert_eq!(report.selected_only_functions, 1);
        assert_eq!(report.disagreements.len(), 1);
        assert_eq!(report.disagreements[0].name, "different");
        assert_eq!(
            report.disagreements[0].present_in,
            vec![Abi::Arm64V8a, Abi::X86_64]
        );
    }

    #[test]
    fn relinking_keeps_shared_code_identity_without_inventing_one_alias_name() {
        let caller = recovered_function("caller", 0, "stale.alias");
        let mut first = recovered_function("firstAlias", 0, "unused");
        first.address = "0x2000".to_owned();
        first.statements.clear();
        let mut second = recovered_function("secondAlias", 0, "unused");
        second.address = "0x2000".to_owned();
        second.statements.clear();
        let mut program = RecoveredProgram {
            application_package: Some("app".to_owned()),
            functions: vec![caller, first, second],
            ..RecoveredProgram::default()
        };

        relink_calls(&mut program);

        let PseudoStatement::DirectCall {
            target,
            target_code_address,
            ..
        } = &program.functions[0].statements[0]
        else {
            panic!("expected direct call");
        };
        assert_eq!(target, &None);
        assert_eq!(target_code_address.as_deref(), Some("0x2000"));
        assert_eq!(
            program.functions[0].machine_code.code_resolved_direct_calls,
            1
        );
    }
}
