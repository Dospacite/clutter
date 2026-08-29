use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

use crate::archive::Artifact;
use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::model::{
    Abi, AndroidMetadata, ArchiveInfo, PseudoStatement, RecoveredDeclaration,
    RecoveredDeclarationKind, RecoveredProgram, RecoveredString, SnapshotInfo,
};

const MANIFEST_FILE: &str = "decompilation.json";

#[derive(Serialize)]
pub struct DecompilationManifest<'a> {
    pub schema: &'static str,
    pub tool_version: &'static str,
    pub input: &'a ArchiveInfo,
    pub selected_module: &'a str,
    pub selected_abi: Abi,
    pub android: &'a AndroidMetadata,
    pub snapshot: &'a SnapshotInfo,
    pub application_package: &'a Option<String>,
    pub root_library_uri: &'a Option<String>,
    pub split_debug_info: &'a Option<crate::model::SplitDebugInfo>,
    pub obfuscation_map: &'a Option<crate::model::ObfuscationMapInfo>,
    pub vm_oracle: &'a Option<crate::model::VmOracleEvidence>,
    pub coverage: Coverage,
    pub warnings: &'a [crate::model::Warning],
}

#[derive(Serialize)]
pub struct Coverage {
    pub recovered_libraries: usize,
    pub recovered_identifiers: usize,
    pub recovered_strings: usize,
    pub recovered_string_sources: BTreeMap<&'static str, usize>,
    /// Logical Dart function entries. Multiple entries can share one machine-code range.
    pub recovered_functions: usize,
    pub unique_code_ranges: usize,
    pub shared_code_entries: usize,
    pub recovered_function_bytes: u64,
    pub recovered_declarations: usize,
    pub declarations_linked_by_vm_oracle: usize,
    pub class_declarations: usize,
    pub field_declarations: usize,
    pub function_declarations: usize,
    pub function_declarations_with_code: usize,
    pub function_declarations_without_code: usize,
    pub functions_with_signatures: usize,
    pub functions_linked_by_vm_oracle: usize,
    pub functions_with_resolved_return_types: usize,
    pub recovered_parameter_types: usize,
    pub recovered_named_parameter_names: usize,
    pub classes_with_recovered_metadata: usize,
    pub typed_field_declarations: usize,
    pub signatures_from_related_functions: usize,
    pub functions_with_source_locations: usize,
    pub dwarf_inline_ranges: usize,
    pub functions_with_internal_source_maps: usize,
    pub internal_source_map_events: usize,
    pub pc_descriptor_entries: usize,
    pub decoded_stack_map_entries: usize,
    pub recovered_exception_handlers: usize,
    pub function_name_sources: BTreeMap<&'static str, usize>,
    pub direct_call_sites: usize,
    pub code_resolved_direct_call_sites: usize,
    pub resolved_direct_call_sites: usize,
    pub unchecked_entry_direct_call_sites: usize,
    pub range_interior_direct_call_sites: usize,
    pub indirect_call_sites: usize,
    pub resolved_indirect_call_sites: usize,
    pub dispatch_table_call_sites: usize,
    pub resolved_dispatch_table_call_sites: usize,
    pub dispatch_table_entries: usize,
    pub dispatch_table_runs: usize,
    pub call_target_scopes: BTreeMap<&'static str, usize>,
    pub conditional_branches: usize,
    pub control_flow_edges: usize,
    pub reachable_basic_blocks: usize,
    pub object_pool_loads: usize,
    pub semantic_statements: usize,
    pub recovered_field_reads: usize,
    pub recovered_field_writes: usize,
    pub recovered_conditions: usize,
    pub high_confidence_returns: usize,
    pub rendered_source_functions: usize,
    pub evidence_only_functions: usize,
    pub decoded_function_bytes: usize,
    pub undecoded_function_bytes: usize,
    pub function_kinds: BTreeMap<&'static str, usize>,
    /// Evidence-tier distribution of solved signature outcomes.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub signature_tiers: BTreeMap<String, usize>,
    /// Cross-ABI consensus tier summary (corroborated / disputed / unaligned).
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub cross_abi_tiers: BTreeMap<String, usize>,
    /// Physical-body graph resolution counts, when computed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_graph: Option<crate::evidence::body::BodyGraphReport>,
    /// Total bytes in the selected AOT code image, including dependencies.
    pub instruction_bytes: usize,
}

#[derive(Serialize)]
struct SymbolIndex<'a> {
    schema: &'static str,
    identifiers: &'a [String],
    strings: &'a [RecoveredString],
}

#[derive(Serialize)]
struct DeclarationIndex<'a> {
    schema: &'static str,
    note: &'static str,
    declarations: &'a [RecoveredDeclaration],
}

#[derive(Serialize)]
struct LibraryIndex<'a> {
    schema: &'static str,
    libraries: &'a [crate::model::RecoveredLibrary],
}

#[derive(Serialize)]
struct FunctionIndex<'a> {
    schema: &'static str,
    functions: Vec<FunctionSummary<'a>>,
}

#[derive(Serialize)]
struct FunctionSummary<'a> {
    name: &'a str,
    owner: &'a Option<String>,
    library_uri: &'a Option<String>,
    address: &'a str,
    size: u64,
    code_reference: i32,
    code_alias_references: &'a [i32],
    name_source: &'static str,
    source_location: &'a Option<crate::model::RecoveredSourceLocation>,
    kind: Option<&'static str>,
    signature: &'a Option<crate::model::RecoveredSignature>,
    signature_source: Option<&'static str>,
    vm_evidence: &'a Option<crate::model::VmFunctionEvidence>,
    code_metadata: &'a Option<crate::model::RecoveredCodeMetadata>,
    machine_code: &'a crate::model::MachineCodeEvidence,
    control_flow: &'a [crate::model::ControlFlowEdge],
    semantic_statements: &'a [crate::model::SemanticStatement],
    inlined_function_count: usize,
}

#[derive(Serialize)]
struct CallGraph {
    schema: &'static str,
    edges: Vec<CallGraphEdge>,
}

#[derive(Serialize)]
struct CallGraphEdge {
    caller: String,
    caller_address: String,
    call_address: String,
    dispatch: &'static str,
    target: Option<String>,
    target_address: Option<String>,
    target_code_address: Option<String>,
    target_entry_offset: Option<u64>,
    target_resolution: Option<&'static str>,
    target_library_uri: Option<String>,
    target_scope: &'static str,
    selector_name: Option<String>,
    candidate_targets: Vec<String>,
    candidate_count: usize,
    raw_slot_target_count: usize,
}

#[derive(Serialize)]
struct ResourceIndex {
    schema: &'static str,
    files: Vec<ResourceEntry>,
}

#[derive(Serialize)]
struct ResourceEntry {
    path: String,
    size: u64,
    sha256: String,
}

pub struct WriteRequest<'a> {
    pub artifact: &'a Artifact,
    pub module: &'a str,
    pub abi: Abi,
    pub android: &'a AndroidMetadata,
    pub snapshot: &'a SnapshotInfo,
    pub program: &'a RecoveredProgram,
    pub instruction_bytes: usize,
    pub include_assets: bool,
    pub emit_ir: bool,
    pub replace: bool,
}

pub fn write(output: &Path, request: WriteRequest<'_>) -> Result<Option<PathBuf>> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).at(parent)?;
    let temporary = TempDir::new_in(parent).at(parent)?;
    let stage = temporary.path().join("result");
    fs::create_dir(&stage).at(&stage)?;

    write_generated_files(&stage, &request)?;
    let backup = prepare_destination(output, request.replace)?;
    fs::rename(&stage, output).at(output)?;
    Ok(backup)
}

fn write_generated_files(stage: &Path, request: &WriteRequest<'_>) -> Result<()> {
    let rendered = crate::render::render_libraries(request.program);
    if rendered.len() != request.program.libraries.len() {
        return Err(ClutterError::Analysis(
            "multiple recovered library URIs map to the same output path".to_owned(),
        ));
    }
    for (relative, contents) in rendered {
        write_text(&stage.join("lib").join(relative), &contents)?;
    }
    write_text(
        &stage.join("support/aot_intrinsics.dart"),
        &crate::render::render_support(),
    )?;

    let coverage = coverage(request);
    let manifest = DecompilationManifest {
        schema: "clutter.decompilation/v1",
        tool_version: env!("CARGO_PKG_VERSION"),
        input: request.artifact.info(),
        selected_module: request.module,
        selected_abi: request.abi,
        android: request.android,
        snapshot: request.snapshot,
        application_package: &request.program.application_package,
        root_library_uri: &request.program.root_library_uri,
        split_debug_info: &request.program.split_debug_info,
        obfuscation_map: &request.program.obfuscation_map,
        vm_oracle: &request.program.vm_oracle,
        coverage,
        warnings: &request.program.warnings,
    };
    write_json(&stage.join(MANIFEST_FILE), &manifest)?;
    write_json(&stage.join("metadata/android.json"), request.android)?;
    write_json(
        &stage.join("metadata/symbols.json"),
        &SymbolIndex {
            schema: "clutter.symbols/v2",
            identifiers: &request.program.identifiers,
            strings: &request.program.strings,
        },
    )?;
    write_json(&stage.join("reports/coverage.json"), &manifest.coverage)?;
    write_json(
        &stage.join("reports/libraries.json"),
        &LibraryIndex {
            schema: "clutter.libraries/v1",
            libraries: &request.program.libraries,
        },
    )?;
    write_json(
        &stage.join("reports/declarations.json"),
        &DeclarationIndex {
            schema: "clutter.declarations/v1",
            note: "A function declaration without code may have been inlined, folded, deferred, or tree-shaken; the artifact does not prove which.",
            declarations: &request.program.declarations,
        },
    )?;
    write_json(
        &stage.join("reports/functions.json"),
        &function_index(request.program),
    )?;
    write_json(
        &stage.join("reports/call_graph.json"),
        &call_graph(request.program),
    )?;
    write_assembly(&stage.join("reports/assembly.s"), request.program)?;
    if let Some(evidence) = &request.program.snapshot_evidence {
        write_json(&stage.join("metadata/snapshot_evidence.json"), evidence)?;
    }
    if let Some(evidence) = &request.program.vm_oracle {
        write_json(&stage.join("metadata/vm_oracle.json"), evidence)?;
        let raw_path = stage.join("metadata/vm_snapshot_analyzer.json");
        fs::copy(&evidence.source, &raw_path).at(&evidence.source)?;
    }
    if let Some(report) = &request.program.cross_abi {
        write_json(&stage.join("reports/cross_abi.json"), report)?;
    }
    if let Some(consensus) = &request.program.cross_abi_consensus {
        write_json(&stage.join("reports/cross_abi_consensus.json"), consensus)?;
    }
    if let Some(body_graph) = &request.program.body_graph_report {
        write_json(&stage.join("reports/body_graph.json"), body_graph)?;
    }
    if let Some(dispatch_table) = &request.program.dispatch_table {
        write_json(&stage.join("reports/dispatch_table.json"), dispatch_table)?;
    }
    if !request.program.deferred_units.is_empty() {
        write_json(
            &stage.join("metadata/deferred_units.json"),
            &request.program.deferred_units,
        )?;
    }
    write_unresolved(&stage.join("reports/unresolved.jsonl"), request.program)?;
    if request.emit_ir {
        write_json(&stage.join("ir/program.json"), request.program)?;
    }

    let resources = if request.include_assets {
        extract_resources(stage, request.artifact, request.module)?
    } else {
        Vec::new()
    };
    write_json(
        &stage.join("metadata/resources.json"),
        &ResourceIndex {
            schema: "clutter.resources/v1",
            files: resources,
        },
    )?;
    write_text(&stage.join("README.md"), &generated_readme(request))?;
    Ok(())
}

fn coverage(request: &WriteRequest<'_>) -> Coverage {
    let mut direct_call_sites = 0usize;
    let mut code_resolved_direct_call_sites = 0usize;
    let mut resolved_direct_call_sites = 0usize;
    let mut unchecked_entry_direct_call_sites = 0usize;
    let mut range_interior_direct_call_sites = 0usize;
    let mut indirect_call_sites = 0usize;
    let mut resolved_indirect_call_sites = 0usize;
    let mut dispatch_table_call_sites = 0usize;
    let mut resolved_dispatch_table_call_sites = 0usize;
    let mut function_kinds = BTreeMap::new();
    let mut function_name_sources = BTreeMap::new();
    let mut call_target_scopes = BTreeMap::new();
    let mut conditional_branches = 0usize;
    let mut decoded_function_bytes = 0usize;
    let mut undecoded_function_bytes = 0usize;
    let mut dwarf_inline_ranges = 0usize;
    let mut functions_with_internal_source_maps = 0usize;
    let mut internal_source_map_events = 0usize;
    let mut pc_descriptor_entries = 0usize;
    let mut decoded_stack_map_entries = 0usize;
    let mut recovered_exception_handlers = 0usize;
    let mut control_flow_edges = 0usize;
    let mut reachable_basic_blocks = 0usize;
    let mut object_pool_loads = 0usize;
    let mut semantic_statements = 0usize;
    let mut recovered_field_reads = 0usize;
    let mut recovered_field_writes = 0usize;
    let mut recovered_conditions = 0usize;
    let mut high_confidence_returns = 0usize;
    let mut unique_ranges = std::collections::BTreeSet::new();
    for function in &request.program.functions {
        *function_name_sources
            .entry(function.name_source.label())
            .or_default() += 1;
        if let Some(kind) = function.kind {
            *function_kinds.entry(kind.label()).or_default() += 1;
        }
        dwarf_inline_ranges += function.inlined_functions.len();
        if unique_ranges.insert((function.address.as_str(), function.size)) {
            decoded_function_bytes += function.machine_code.decoded_bytes;
            undecoded_function_bytes += function.machine_code.unknown_bytes;
            conditional_branches += function.machine_code.conditional_branches;
            control_flow_edges += function.machine_code.control_flow_edges;
            reachable_basic_blocks += function.machine_code.reachable_basic_blocks;
            object_pool_loads += function.machine_code.object_pool_loads;
            dispatch_table_call_sites += function.machine_code.dispatch_table_calls;
            resolved_dispatch_table_call_sites +=
                function.machine_code.resolved_dispatch_table_calls;
            semantic_statements += function.semantic_statements.len();
            recovered_field_reads += function
                .semantic_statements
                .iter()
                .filter(|statement| {
                    matches!(statement, crate::model::SemanticStatement::FieldRead { .. })
                })
                .count();
            recovered_field_writes += function
                .semantic_statements
                .iter()
                .filter(|statement| {
                    matches!(
                        statement,
                        crate::model::SemanticStatement::FieldWrite { .. }
                    )
                })
                .count();
            recovered_conditions += function
                .semantic_statements
                .iter()
                .filter(|statement| {
                    matches!(statement, crate::model::SemanticStatement::Condition { .. })
                })
                .count();
            high_confidence_returns += function
                .semantic_statements
                .iter()
                .filter(|statement| {
                    matches!(
                        statement,
                        crate::model::SemanticStatement::Return {
                            confidence: crate::model::EvidenceConfidence::High,
                            ..
                        }
                    )
                })
                .count();
            if let Some(metadata) = &function.code_metadata {
                functions_with_internal_source_maps +=
                    usize::from(!metadata.code_source_map.is_empty());
                internal_source_map_events += metadata.code_source_map.len();
                pc_descriptor_entries += metadata.pc_descriptors.len();
                decoded_stack_map_entries += metadata.stack_map_entries;
                recovered_exception_handlers += metadata.exception_handlers.len();
            }
            for statement in &function.statements {
                match statement {
                    PseudoStatement::DirectCall {
                        target,
                        target_code_address,
                        target_resolution,
                        target_scope,
                        ..
                    } => {
                        direct_call_sites += 1;
                        code_resolved_direct_call_sites +=
                            usize::from(target_code_address.is_some());
                        resolved_direct_call_sites += usize::from(target.is_some());
                        unchecked_entry_direct_call_sites += usize::from(
                            *target_resolution
                                == Some(crate::model::DirectCallResolution::UncheckedEntry),
                        );
                        range_interior_direct_call_sites += usize::from(
                            *target_resolution
                                == Some(crate::model::DirectCallResolution::RangeInterior),
                        );
                        *call_target_scopes.entry(target_scope.label()).or_default() += 1;
                    }
                    PseudoStatement::IndirectCall { .. } => {
                        indirect_call_sites += 1;
                        *call_target_scopes.entry("dynamic").or_default() += 1;
                    }
                    PseudoStatement::RecoveredIndirectCall { target_scope, .. } => {
                        indirect_call_sites += 1;
                        resolved_indirect_call_sites += 1;
                        *call_target_scopes.entry(target_scope.label()).or_default() += 1;
                    }
                    PseudoStatement::ObjectPoolCall { target_scope, .. } => {
                        indirect_call_sites += 1;
                        resolved_indirect_call_sites += 1;
                        *call_target_scopes.entry(target_scope.label()).or_default() += 1;
                    }
                    PseudoStatement::DispatchTableCall { .. } => {
                        indirect_call_sites += 1;
                        *call_target_scopes.entry("dynamic").or_default() += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    let class_declarations = declaration_count(
        &request.program.declarations,
        RecoveredDeclarationKind::Class,
    );
    let field_declarations = declaration_count(
        &request.program.declarations,
        RecoveredDeclarationKind::Field,
    );
    let function_declarations = declaration_count(
        &request.program.declarations,
        RecoveredDeclarationKind::Function,
    );
    let function_declarations_with_code = request
        .program
        .declarations
        .iter()
        .filter(|declaration| {
            declaration.kind == RecoveredDeclarationKind::Function && declaration.has_code
        })
        .count();
    let mut recovered_string_sources = BTreeMap::new();
    for value in &request.program.strings {
        *recovered_string_sources
            .entry(value.source.label())
            .or_default() += 1;
    }
    Coverage {
        recovered_libraries: request.program.libraries.len(),
        recovered_identifiers: request.program.identifiers.len(),
        recovered_strings: request.program.strings.len(),
        recovered_string_sources,
        recovered_functions: request.program.functions.len(),
        unique_code_ranges: unique_ranges.len(),
        shared_code_entries: request
            .program
            .functions
            .len()
            .saturating_sub(unique_ranges.len()),
        recovered_function_bytes: unique_ranges.iter().map(|(_, size)| *size).sum(),
        // Authoritative shared-body counts from the physical-body graph when
        // available; the address-based approximation remains as a fallback.
        body_graph: request.program.body_graph_report,
        recovered_declarations: request.program.declarations.len(),
        declarations_linked_by_vm_oracle: request
            .program
            .declarations
            .iter()
            .filter(|declaration| declaration.vm_object_id.is_some())
            .count(),
        class_declarations,
        field_declarations,
        function_declarations,
        function_declarations_with_code,
        function_declarations_without_code: function_declarations
            .saturating_sub(function_declarations_with_code),
        functions_with_signatures: request
            .program
            .functions
            .iter()
            .filter(|function| function.signature.is_some())
            .count(),
        functions_linked_by_vm_oracle: request
            .program
            .functions
            .iter()
            .filter(|function| function.vm_evidence.is_some())
            .count(),
        functions_with_resolved_return_types: request
            .program
            .functions
            .iter()
            .filter(|function| {
                function
                    .signature
                    .as_ref()
                    .and_then(|signature| signature.resolved.as_ref())
                    .and_then(|resolved| resolved.return_type.as_ref())
                    .is_some()
            })
            .count(),
        recovered_parameter_types: request
            .program
            .functions
            .iter()
            .filter_map(|function| function.signature.as_ref())
            .filter_map(|signature| signature.resolved.as_ref())
            .flat_map(|resolved| &resolved.parameters)
            .filter(|parameter| parameter.declared_type.is_some())
            .count(),
        recovered_named_parameter_names: request
            .program
            .functions
            .iter()
            .filter_map(|function| function.signature.as_ref())
            .filter_map(|signature| signature.resolved.as_ref())
            .flat_map(|resolved| &resolved.parameters)
            .filter(|parameter| parameter.is_named && parameter.name.is_some())
            .count(),
        classes_with_recovered_metadata: request
            .program
            .declarations
            .iter()
            .filter(|declaration| {
                declaration.kind == RecoveredDeclarationKind::Class
                    && declaration.class_metadata.is_some()
            })
            .count(),
        typed_field_declarations: request
            .program
            .declarations
            .iter()
            .filter(|declaration| {
                declaration.kind == RecoveredDeclarationKind::Field
                    && declaration
                        .field_metadata
                        .as_ref()
                        .is_some_and(|metadata| metadata.declared_type.is_some())
            })
            .count(),
        signatures_from_related_functions: request
            .program
            .functions
            .iter()
            .filter(|function| {
                function.signature_source
                    == Some(crate::model::RecoveredSignatureSource::RelatedFunction)
            })
            .count(),
        functions_with_source_locations: request
            .program
            .functions
            .iter()
            .filter(|function| function.source_location.is_some())
            .count(),
        dwarf_inline_ranges,
        functions_with_internal_source_maps,
        internal_source_map_events,
        pc_descriptor_entries,
        decoded_stack_map_entries,
        recovered_exception_handlers,
        function_name_sources,
        direct_call_sites,
        code_resolved_direct_call_sites,
        resolved_direct_call_sites,
        unchecked_entry_direct_call_sites,
        range_interior_direct_call_sites,
        indirect_call_sites,
        resolved_indirect_call_sites,
        dispatch_table_call_sites,
        resolved_dispatch_table_call_sites,
        dispatch_table_entries: request
            .program
            .dispatch_table
            .as_ref()
            .map_or(0, |table| table.entry_count),
        dispatch_table_runs: request
            .program
            .dispatch_table
            .as_ref()
            .map_or(0, |table| table.runs.len()),
        call_target_scopes,
        conditional_branches,
        control_flow_edges,
        reachable_basic_blocks,
        object_pool_loads,
        semantic_statements,
        recovered_field_reads,
        recovered_field_writes,
        recovered_conditions,
        high_confidence_returns,
        rendered_source_functions: request
            .program
            .functions
            .iter()
            .filter(|function| crate::render::source_visible_function(function))
            .count(),
        evidence_only_functions: request
            .program
            .functions
            .iter()
            .filter(|function| !crate::render::source_visible_function(function))
            .count(),
        decoded_function_bytes,
        undecoded_function_bytes,
        function_kinds,
        signature_tiers: request
            .program
            .signature_solutions
            .as_ref()
            .map(|solutions| {
                let mut tiers = BTreeMap::new();
                for solved in solutions.values() {
                    *tiers.entry(solved.outcome.label().to_owned()).or_default() += 1;
                }
                tiers
            })
            .unwrap_or_default(),
        cross_abi_tiers: request
            .program
            .cross_abi_consensus
            .as_ref()
            .map(|consensus| consensus.tier_summary())
            .unwrap_or_default(),
        instruction_bytes: request.instruction_bytes,
    }
}

fn declaration_count(
    declarations: &[RecoveredDeclaration],
    kind: RecoveredDeclarationKind,
) -> usize {
    declarations
        .iter()
        .filter(|declaration| declaration.kind == kind)
        .count()
}

fn function_index(program: &RecoveredProgram) -> FunctionIndex<'_> {
    FunctionIndex {
        schema: "clutter.functions/v1",
        functions: program
            .functions
            .iter()
            .map(|function| FunctionSummary {
                name: &function.name,
                owner: &function.owner,
                library_uri: &function.library_uri,
                address: &function.address,
                size: function.size,
                code_reference: function.code_reference,
                code_alias_references: &function.code_alias_references,
                name_source: function.name_source.label(),
                source_location: &function.source_location,
                kind: function.kind.map(|kind| kind.label()),
                signature: &function.signature,
                signature_source: function.signature_source.map(|source| source.label()),
                vm_evidence: &function.vm_evidence,
                code_metadata: &function.code_metadata,
                machine_code: &function.machine_code,
                control_flow: &function.control_flow,
                semantic_statements: &function.semantic_statements,
                inlined_function_count: function.inlined_functions.len(),
            })
            .collect(),
    }
}

fn call_graph(program: &RecoveredProgram) -> CallGraph {
    let mut edges = Vec::new();
    let mut seen_ranges = std::collections::BTreeSet::new();
    for function in &program.functions {
        if !seen_ranges.insert((function.address.as_str(), function.size)) {
            continue;
        }
        let caller = match function.owner.as_deref() {
            Some(owner) if !matches!(owner, "::" | "top_level") => {
                format!("{owner}.{}", function.name)
            }
            _ => function.name.clone(),
        };
        for statement in &function.statements {
            match statement {
                PseudoStatement::DirectCall {
                    address,
                    target_address,
                    target_code_address,
                    target_entry_offset,
                    target_resolution,
                    target,
                    target_library_uri,
                    target_scope,
                } => edges.push(CallGraphEdge {
                    caller: caller.clone(),
                    caller_address: function.address.clone(),
                    call_address: address.clone(),
                    dispatch: "direct",
                    target: target.clone(),
                    target_address: Some(target_address.clone()),
                    target_code_address: target_code_address.clone(),
                    target_entry_offset: *target_entry_offset,
                    target_resolution: target_resolution.map(|resolution| resolution.label()),
                    target_library_uri: target_library_uri.clone(),
                    target_scope: target_scope.label(),
                    selector_name: None,
                    candidate_targets: Vec::new(),
                    candidate_count: 0,
                    raw_slot_target_count: 0,
                }),
                PseudoStatement::IndirectCall {
                    address,
                    expression,
                } => edges.push(CallGraphEdge {
                    caller: caller.clone(),
                    caller_address: function.address.clone(),
                    call_address: address.clone(),
                    dispatch: "indirect",
                    target: Some(expression.clone()),
                    target_address: None,
                    target_code_address: None,
                    target_entry_offset: None,
                    target_resolution: None,
                    target_library_uri: None,
                    target_scope: "dynamic",
                    selector_name: None,
                    candidate_targets: Vec::new(),
                    candidate_count: 0,
                    raw_slot_target_count: 0,
                }),
                PseudoStatement::RecoveredIndirectCall {
                    address,
                    expression,
                    target,
                    target_library_uri,
                    target_scope,
                } => edges.push(CallGraphEdge {
                    caller: caller.clone(),
                    caller_address: function.address.clone(),
                    call_address: address.clone(),
                    dispatch: "register_indirect_recovered",
                    target: Some(target.clone()),
                    target_address: Some(expression.clone()),
                    target_code_address: None,
                    target_entry_offset: None,
                    target_resolution: None,
                    target_library_uri: target_library_uri.clone(),
                    target_scope: target_scope.label(),
                    selector_name: None,
                    candidate_targets: Vec::new(),
                    candidate_count: 0,
                    raw_slot_target_count: 0,
                }),
                PseudoStatement::ObjectPoolCall {
                    address,
                    pool_index,
                    target,
                    target_scope,
                    ..
                } => edges.push(CallGraphEdge {
                    caller: caller.clone(),
                    caller_address: function.address.clone(),
                    call_address: address.clone(),
                    dispatch: "object_pool",
                    target: Some(target.clone()),
                    target_address: Some(format!("pool[{pool_index}]")),
                    target_code_address: None,
                    target_entry_offset: None,
                    target_resolution: None,
                    target_library_uri: None,
                    target_scope: target_scope.label(),
                    selector_name: None,
                    candidate_targets: Vec::new(),
                    candidate_count: 0,
                    raw_slot_target_count: 0,
                }),
                PseudoStatement::DispatchTableCall {
                    address,
                    selector_offset,
                    selector_name,
                    candidate_targets,
                    candidate_count,
                    raw_slot_target_count,
                    ..
                } => edges.push(CallGraphEdge {
                    caller: caller.clone(),
                    caller_address: function.address.clone(),
                    call_address: address.clone(),
                    dispatch: "class_dispatch_table",
                    target: None,
                    target_address: Some(format!("dispatch[{selector_offset} + class_id]")),
                    target_code_address: None,
                    target_entry_offset: None,
                    target_resolution: None,
                    target_library_uri: None,
                    target_scope: "dynamic",
                    selector_name: selector_name.clone(),
                    candidate_targets: candidate_targets.clone(),
                    candidate_count: *candidate_count,
                    raw_slot_target_count: *raw_slot_target_count,
                }),
                _ => {}
            }
        }
    }
    CallGraph {
        schema: "clutter.call-graph/v3",
        edges,
    }
}

fn write_assembly(path: &Path, program: &RecoveredProgram) -> Result<()> {
    let mut output = String::from(
        "# Complete decoded machine-code evidence. Generated source intentionally omits this noise.\n",
    );
    let mut seen_ranges = std::collections::BTreeSet::new();
    for function in &program.functions {
        if !seen_ranges.insert((function.address.as_str(), function.size)) {
            continue;
        }
        let owner = function.owner.as_deref().unwrap_or("top_level");
        writeln!(
            output,
            "\n# {}.{} at {} ({} bytes)",
            owner, function.name, function.address, function.size
        )
        .expect("write to string");
        for instruction in &function.instructions {
            write!(
                output,
                "{}  {:<18} {:<8} {}",
                instruction.address, instruction.bytes, instruction.mnemonic, instruction.operands
            )
            .expect("write to string");
            if let (Some(index), Some(value)) = (
                instruction.object_pool_index,
                instruction.object_pool_value.as_deref(),
            ) {
                write!(output, "  # pool[{index}] = {value}").expect("write to string");
            }
            output.push('\n');
        }
        if !function.control_flow.is_empty() {
            output.push_str("# CFG:");
            for edge in &function.control_flow {
                write!(output, " {}->{}/{:?}", edge.from, edge.to, edge.kind)
                    .expect("write to string");
            }
            output.push('\n');
        }
    }
    write_text(path, &output)
}

fn extract_resources(
    stage: &Path,
    artifact: &Artifact,
    module: &str,
) -> Result<Vec<ResourceEntry>> {
    let root = stage.join("resources/flutter_assets");
    let mut resources = Vec::new();
    artifact.for_each_asset(module, |relative, reader, declared_size| {
        let relative_path = checked_relative_path(relative)?;
        let destination = root.join(&relative_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).at(parent)?;
        }
        let mut file = File::create(&destination).at(&destination)?;
        let mut digest = Sha256::new();
        let mut written = 0u64;
        let mut buffer = [0u8; 128 * 1024];
        loop {
            let count = reader.read(&mut buffer).at(&destination)?;
            if count == 0 {
                break;
            }
            file.write_all(&buffer[..count]).at(&destination)?;
            digest.update(&buffer[..count]);
            written += count as u64;
        }
        if written != declared_size {
            return Err(ClutterError::InvalidArtifact(format!(
                "asset {relative:?} declared {declared_size} bytes but produced {written}"
            )));
        }
        resources.push(ResourceEntry {
            path: relative_path.to_string_lossy().replace('\\', "/"),
            size: written,
            sha256: hex::encode(digest.finalize()),
        });
        Ok(())
    })?;
    resources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(resources)
}

fn checked_relative_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ClutterError::InvalidArtifact(format!(
            "unsafe resource path {value:?}"
        )));
    }
    Ok(path.to_path_buf())
}

fn write_unresolved(path: &Path, program: &RecoveredProgram) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).at(parent)?;
    }
    let mut file = File::create(path).at(path)?;
    for warning in &program.warnings {
        serde_json::to_writer(&mut file, warning)?;
        file.write_all(b"\n").at(path)?;
    }
    for function in &program.functions {
        for statement in &function.statements {
            if let crate::model::PseudoStatement::UnknownOperation { address, bytes } = statement {
                serde_json::to_writer(
                    &mut file,
                    &serde_json::json!({
                        "code": "W_UNKNOWN_INSTRUCTION",
                        "library_uri": function.library_uri,
                        "function": function.name,
                        "function_address": function.address,
                        "address": address,
                        "bytes": bytes,
                    }),
                )?;
                file.write_all(b"\n").at(path)?;
            }
        }
    }
    Ok(())
}

fn prepare_destination(output: &Path, replace: bool) -> Result<Option<PathBuf>> {
    if !output.exists() {
        return Ok(None);
    }
    if !replace {
        return Err(ClutterError::OutputExists(output.to_path_buf()));
    }
    if !output.join(MANIFEST_FILE).is_file() {
        return Err(ClutterError::InvalidArtifact(format!(
            "refusing to replace {output:?}: it is not a Clutter output directory"
        )));
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let name = output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("clutter-output");
    let backup = output.with_file_name(format!("{name}.backup.{timestamp}"));
    fs::rename(output, &backup).at(output)?;
    Ok(Some(backup))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).at(parent)?;
    }
    let file = File::create(path).at(path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

fn write_text(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).at(parent)?;
    }
    fs::write(path, contents).at(path)
}

fn generated_readme(request: &WriteRequest<'_>) -> String {
    let metrics = coverage(request);
    format!(
        "# Clutter AOT recovery\n\n\
         This directory contains conservative pseudocode recovered from `{}`.\n\n\
         - Dart snapshot: `{}`\n\
         - ABI: `{}`\n\
         - Application package: `{}`\n\
         - VM-resolved root library: `{}`\n\
         - Recovered libraries: {}\n\
         - Recovered identifiers: {}\n\
         - Recovered strings: {}\n\
         - Logical function entries: {} across {} unique AOT code ranges\n\
         - Source-view functions: {}; evidence-only runtime/boundary functions: {}\n\
         - Dart VM-linked function entries: {}\n\
         - Dart VM-linked declarations: {}\n\
         - Function declarations: {} with standalone code, {} without standalone code\n\
         - Resolved snapshot types: {} return types, {} parameter types, {} named parameter names\n\
         - Typed declarations: {} classes with metadata, {} fields with declared types\n\
         - Direct call sites: {} ({} code targets, {} semantic targets); indirect call sites: {}\n\n\
         Global identifiers and strings are stored in `metadata/symbols.json` \
         instead of being mixed into the generated Dart source. `reports/functions.json` \
         contains function, CFG, semantic, and VM metadata; `reports/libraries.json` \
         separates literal VM imports from inferred type-reference dependencies; `reports/declarations.json` \
         includes declaration-only evidence; `reports/call_graph.json` separates direct, \
         dynamic, and object-pool calls; and `reports/assembly.s` contains the complete \
         annotated instruction stream. Typed snapshot totals are in \
         `metadata/snapshot_evidence.json`; compressed class-dispatch runs are in \
         `reports/dispatch_table.json` when snapshot recovery succeeds. Surviving \
         FunctionType, Class, and Field graphs drive the rendered types and \
         relationships. When supplied, the complete no-main-execution VM index \
         is preserved in `metadata/vm_snapshot_analyzer.json` and its validated \
         match summary is in `metadata/vm_oracle.json`. Full AOT omits positional parameter names and optional \
         default expressions, so those remain labeled placeholders rather than \
         guesses. VM/runtime helper calls are summarized in the source view and \
         preserved exactly in the reports. Dynamic class-dispatch sites retain \
         selector families and bounded candidate sets, while field names require \
         receiver-class layout proof.\n\n\
         The `.dart` files are not original source and are not promised to build into an \
         equivalent application. A recovered return is emitted only for branch-free, fully \
         decoded machine code with one return and one high-confidence data-flow expression. \
         Release AOT compilation removes debug information, tree-shakes unreachable code, \
         folds constants, and may inline functions. A declaration without standalone code \
         does not by itself distinguish inlining from folding, deferral, or tree shaking. \
         Unknown behavior is intentionally left explicit instead of being guessed.\n",
        request.artifact.info().path.display(),
        request.snapshot.vm_header.snapshot_hash,
        request.abi,
        request
            .program
            .application_package
            .as_deref()
            .unwrap_or("unknown"),
        request
            .program
            .root_library_uri
            .as_deref()
            .unwrap_or("unavailable"),
        request.program.libraries.len(),
        request.program.identifiers.len(),
        request.program.strings.len(),
        metrics.recovered_functions,
        metrics.unique_code_ranges,
        metrics.rendered_source_functions,
        metrics.evidence_only_functions,
        metrics.functions_linked_by_vm_oracle,
        metrics.declarations_linked_by_vm_oracle,
        metrics.function_declarations_with_code,
        metrics.function_declarations_without_code,
        metrics.functions_with_resolved_return_types,
        metrics.recovered_parameter_types,
        metrics.recovered_named_parameter_names,
        metrics.classes_with_recovered_metadata,
        metrics.typed_field_declarations,
        metrics.direct_call_sites,
        metrics.code_resolved_direct_call_sites,
        metrics.resolved_direct_call_sites,
        metrics.indirect_call_sites,
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{checked_relative_path, prepare_destination};

    #[test]
    fn rejects_resource_path_traversal() {
        assert!(checked_relative_path("../escape").is_err());
        assert!(checked_relative_path("assets/icon.png").is_ok());
    }

    #[test]
    fn replace_refuses_non_clutter_directories() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("existing");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("keep.txt"), "user data").unwrap();

        assert!(prepare_destination(&output, true).is_err());
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).unwrap(),
            "user data"
        );
    }
}
