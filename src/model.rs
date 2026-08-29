use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Abi {
    #[serde(rename = "arm64-v8a")]
    Arm64V8a,
    #[serde(rename = "armeabi-v7a")]
    ArmeabiV7a,
    #[serde(rename = "x86_64")]
    X86_64,
}

impl Abi {
    pub const ALL: [Self; 3] = [Self::Arm64V8a, Self::ArmeabiV7a, Self::X86_64];

    pub fn archive_name(self) -> &'static str {
        match self {
            Self::Arm64V8a => "arm64-v8a",
            Self::ArmeabiV7a => "armeabi-v7a",
            Self::X86_64 => "x86_64",
        }
    }
}

impl fmt::Display for Abi {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.archive_name())
    }
}

impl FromStr for Abi {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "arm64-v8a" => Ok(Self::Arm64V8a),
            "armeabi-v7a" => Ok(Self::ArmeabiV7a),
            "x86_64" => Ok(Self::X86_64),
            _ => Err(format!("unknown ABI {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactFormat {
    Apk,
    Aab,
}

impl fmt::Display for ArtifactFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Apk => formatter.write_str("APK"),
            Self::Aab => formatter.write_str("AAB"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    App,
    Packages,
    All,
}

impl FromStr for Scope {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "app" => Ok(Self::App),
            "packages" => Ok(Self::Packages),
            "all" => Ok(Self::All),
            _ => Err(format!("unknown scope {value:?}")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ArchiveInfo {
    pub path: PathBuf,
    pub format: ArtifactFormat,
    pub input_size: u64,
    pub input_sha256: String,
    pub modules: Vec<String>,
    pub available_abis: Vec<Abi>,
    pub payloads: BTreeMap<String, PayloadPaths>,
    pub deferred_payloads: Vec<DeferredPayload>,
    pub asset_count: usize,
    pub asset_uncompressed_bytes: u64,
    pub manifest_path: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PayloadPaths {
    pub module: String,
    pub abi: Abi,
    pub libapp: String,
    pub libflutter: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeferredPayload {
    pub module: String,
    pub abi: Abi,
    pub path: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AndroidMetadata {
    pub package_name: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u64>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SnapshotHeader {
    pub length: u64,
    pub kind: SnapshotKind,
    pub snapshot_hash: String,
    pub features: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum SnapshotKind {
    Full,
    FullCore,
    FullJit,
    FullAot,
    Unknown(i64),
}

#[derive(Clone, Debug, Serialize)]
pub struct SnapshotRegion {
    pub name: String,
    pub virtual_address: String,
    pub file_offset: u64,
    pub size: u64,
    pub sha256: String,
    #[serde(skip)]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SnapshotInfo {
    pub dart_version: Option<String>,
    pub profile_id: String,
    pub profile_match: ProfileMatch,
    pub vm_header: SnapshotHeader,
    pub isolate_header: SnapshotHeader,
    pub regions: Vec<SnapshotRegion>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileMatch {
    Exact,
    Compatible,
    Unknown,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RecoveredProgram {
    pub application_package: Option<String>,
    pub root_library_uri: Option<String>,
    pub split_debug_info: Option<SplitDebugInfo>,
    pub obfuscation_map: Option<ObfuscationMapInfo>,
    pub vm_oracle: Option<VmOracleEvidence>,
    pub libraries: Vec<RecoveredLibrary>,
    pub declarations: Vec<RecoveredDeclaration>,
    pub identifiers: Vec<String>,
    pub strings: Vec<RecoveredString>,
    pub functions: Vec<RecoveredFunction>,
    pub snapshot_evidence: Option<SnapshotEvidence>,
    pub dispatch_table: Option<RecoveredDispatchTable>,
    pub cross_abi: Option<CrossAbiReport>,
    /// Physical-body / logical-occurrence resolution summary: shared bodies,
    /// extent conflicts, and unbound oracle occurrences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_graph_report: Option<crate::evidence::body::BodyGraphReport>,
    /// ABI-neutral consensus over aligned occurrences; facts promoted to
    /// `CrossAbiCorroborated` live here, disputes are retained verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_abi_consensus: Option<crate::evidence::consensus::ConsensusReport>,
    /// Solved parameter-shape outcomes per logical function, tiered by the
    /// strength of their supporting evidence. Erased signatures stay Unknown.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::evidence::signature_solver::serialize_solutions"
    )]
    pub signature_solutions: Option<crate::evidence::signature_solver::SignatureSolutions>,
    pub deferred_units: Vec<DeferredUnitEvidence>,
    pub warnings: Vec<Warning>,
    /// Declaration evidence from every scope, used for call-site signature
    /// joins (named arguments, receiver detection) even when --scope limits
    /// which libraries are rendered. Not serialized.
    #[serde(skip)]
    pub declaration_evidence: Vec<RecoveredDeclaration>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VmOracleEvidence {
    pub source: PathBuf,
    pub source_size: u64,
    pub source_sha256: String,
    pub dart_version: Option<String>,
    pub dart_commit: Option<String>,
    pub snapshot_hash: String,
    pub analyzer_version: u64,
    pub target_arch: Option<String>,
    pub word_size: u64,
    pub compressed_word_size: u64,
    pub root_library_object_id: Option<u64>,
    pub root_library_uri: Option<String>,
    pub object_count: usize,
    pub object_kinds: BTreeMap<String, usize>,
    pub library_count: usize,
    pub class_count: usize,
    pub field_count: usize,
    pub function_count: usize,
    pub functions_with_code: usize,
    pub functions_linked_via_code_owner: usize,
    pub code_object_count: usize,
    pub stub_code_count: usize,
    pub unattributed_code_count: usize,
    pub type_count: usize,
    pub closure_parent_links: usize,
    pub fields_with_offsets: usize,
    pub class_instance_slots: usize,
    pub object_pool_references: usize,
    /// Schema 5: total entries in the AOT global object pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_object_pool_length: Option<u64>,
    /// Schema 5: dispatch-table origin element (architecture dependent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatch_table_origin_element: Option<u64>,
    /// Schema 5: number of static-call targets reachable from the pool.
    #[serde(default)]
    pub static_call_targets: usize,
    /// Schema 5: populated [start, end] class-id runs from the live table.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_id_ranges: Vec<(u64, u64)>,
    pub library_import_edges: usize,
    pub library_reference_edges: usize,
    pub enriched_object_pool_entries: usize,
    pub matched_functions: usize,
    pub matched_code_offsets: usize,
    pub strongly_matched_functions: usize,
    pub unmatched_recovered_functions: usize,
    /// Dispatch-table candidate labels rewritten from synthetic `sub_<addr>`
    /// to oracle Function identities.
    #[serde(default)]
    pub relabeled_dispatch_candidates: usize,
    /// Synthetic `sub_<addr>` call statements whose target resolved to a VM
    /// runtime stub and were rewritten to the stub's identity (CheckNull
    /// slow paths and friends are compiler-inserted, never source calls).
    #[serde(default)]
    pub relabeled_stub_call_sites: usize,
    /// Dispatch-table call sites whose selector was proven because every
    /// distinct implementation resolved through the oracle to one member name.
    #[serde(default)]
    pub oracle_proven_dispatch_selectors: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct VmFunctionEvidence {
    pub object_id: u64,
    pub code_object_id: Option<u64>,
    pub analyzer_code_object_id: Option<u64>,
    pub current_code_object_id: Option<u64>,
    pub code_link_source: Option<String>,
    pub code_offset: Option<u64>,
    pub code_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_match_score: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_match_candidate_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternative_parent_functions: Vec<String>,
    pub name: String,
    pub raw_name: Option<String>,
    pub user_visible_name: Option<String>,
    pub owner: Option<String>,
    pub library_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_function_object_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_function_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_function_owner: Option<String>,
    pub signature: Option<String>,
    pub user_visible_signature: Option<String>,
    pub result_type: Option<VmTypeEvidence>,
    pub parameters: Vec<VmParameterEvidence>,
    pub type_parameters: Vec<VmTypeParameterEvidence>,
    pub owner_type_parameters: Vec<String>,
    pub kind: Option<String>,
    pub fixed_parameter_count: Option<usize>,
    pub optional_parameter_count: Option<usize>,
    pub implicit_parameter_count: Option<usize>,
    pub optional_parameters_are_named: Option<bool>,
    pub is_static: Option<bool>,
    pub is_async: Option<bool>,
    pub is_sync_generator: Option<bool>,
    pub is_async_generator: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VmTypeEvidence {
    pub object_id: u64,
    pub type_kind: Option<String>,
    pub display_name: String,
    pub library_uri: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VmParameterEvidence {
    pub position: usize,
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub declared_type: Option<VmTypeEvidence>,
    pub is_implicit: bool,
    pub is_named: bool,
    pub is_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VmTypeParameterEvidence {
    pub name: String,
    pub bound: Option<VmTypeEvidence>,
    pub default_type: Option<VmTypeEvidence>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredDispatchTable {
    pub abi: Abi,
    pub origin_element: usize,
    pub entry_count: usize,
    pub non_null_entries: usize,
    pub unique_code_indices: usize,
    pub runs: Vec<RecoveredDispatchRun>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredDispatchRun {
    pub start_index: usize,
    pub length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_slot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_library_uri: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeferredUnitEvidence {
    pub path: String,
    pub abi: Abi,
    pub size: usize,
    pub sha256: String,
    pub build_id: Option<String>,
    pub text_symbols: usize,
    pub snapshot_symbols: Vec<String>,
    pub instruction_section_bytes: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CrossAbiReport {
    pub selected_abi: Option<Abi>,
    pub compared_abis: Vec<Abi>,
    pub function_counts: BTreeMap<Abi, usize>,
    pub matched_functions: usize,
    pub selected_only_functions: usize,
    pub disagreements: Vec<CrossAbiDisagreement>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CrossAbiDisagreement {
    pub library_uri: Option<String>,
    pub owner: Option<String>,
    pub name: String,
    pub present_in: Vec<Abi>,
    pub fingerprints: BTreeMap<Abi, ControlFlowFingerprint>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlFlowFingerprint {
    pub conditional_branches: usize,
    pub returns: usize,
    pub direct_calls: usize,
    pub indirect_calls: usize,
    pub resolved_target_names: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SnapshotEvidence {
    pub vm_objects: usize,
    pub isolate_objects: usize,
    pub reference_edges: usize,
    pub reverse_reference_targets: usize,
    pub object_pools: usize,
    pub object_pool_entries: usize,
    pub metadata_payloads: usize,
    pub metadata_bytes: usize,
    pub canonical_objects: usize,
    pub scalar_fields: usize,
    pub instance_layout_bitmaps: usize,
    pub object_kinds: BTreeMap<String, usize>,
    pub cluster_cids: BTreeMap<i32, usize>,
    pub code_objects: usize,
    pub exception_handler_tables: usize,
    pub exception_handlers: usize,
    pub instruction_table_entries: usize,
    pub instruction_entries_with_stack_maps: usize,
    pub compressed_stack_maps: usize,
    pub stack_map_entries: usize,
    pub canonical_stack_map_entries: usize,
    pub dispatch_table_entries: usize,
    pub dispatch_table_code_entries: usize,
    pub dispatch_table_unique_code_indices: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct SplitDebugInfo {
    pub path: PathBuf,
    pub build_id: String,
    pub text_symbol_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ObfuscationMapInfo {
    pub path: PathBuf,
    pub identifier_pairs: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredLibrary {
    pub uri: String,
    pub package: Option<String>,
    pub output_path: PathBuf,
    pub is_application: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_object_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referenced_libraries: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredString {
    pub value: String,
    pub source: RecoveredStringSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<EvidenceConfidence>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveredStringSource {
    ElfScan,
    SnapshotObject,
    SnapshotTypedData,
    SnapshotTransduction,
}

impl RecoveredStringSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::ElfScan => "elf_scan",
            Self::SnapshotObject => "snapshot_object",
            Self::SnapshotTypedData => "snapshot_typed_data",
            Self::SnapshotTransduction => "snapshot_transduction",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveredDeclarationKind {
    Class,
    Field,
    Function,
}

impl RecoveredDeclarationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Class => "class",
            Self::Field => "field",
            Self::Function => "function",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredDeclaration {
    pub snapshot_reference: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_object_id: Option<u64>,
    pub kind: RecoveredDeclarationKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_name: Option<String>,
    pub owner: Option<String>,
    pub library_uri: Option<String>,
    pub source_location: Option<RecoveredSourceLocation>,
    pub function_kind: Option<RecoveredFunctionKind>,
    pub signature: Option<RecoveredSignature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_evidence: Option<VmFunctionEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_metadata: Option<RecoveredClassMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_metadata: Option<RecoveredFieldMetadata>,
    pub has_code: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredClassMetadata {
    pub class_id: i32,
    pub type_parameters: Vec<RecoveredTypeParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_type: Option<RecoveredType>,
    pub interfaces: Vec<RecoveredType>,
    pub is_abstract: bool,
    pub is_enum: bool,
    /// Enum constant names recovered from the snapshot's const-instance
    /// graph (`_Enum._name` Strings). Empty when every value was
    /// tree-shaken; names are object-identity evidence, ordinals follow the
    /// instance `index` field when its layout is proven.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    pub is_sealed: bool,
    pub is_mixin_class: bool,
    pub is_base: bool,
    pub is_interface: bool,
    pub is_final: bool,
    pub is_transformed_mixin_application: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_arguments_field_offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_slots: Vec<RecoveredInstanceSlot>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredInstanceSlot {
    pub offset: i64,
    pub is_reference: bool,
    pub slot_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_object_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredFieldMetadata {
    pub type_reference: i32,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub declared_type: Option<RecoveredType>,
    pub initializer_reference: i32,
    pub offset_or_field_id_reference: Option<i32>,
    pub is_static: bool,
    pub is_final: bool,
    pub is_const: bool,
    pub is_late: bool,
    pub has_initializer: bool,
    pub has_nontrivial_initializer: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_field_offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_field_offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_value_object_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reference: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unboxed_type: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredFunction {
    pub code_reference: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_alias_references: Vec<i32>,
    pub name: String,
    pub name_source: RecoveredNameSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_name: Option<String>,
    pub obfuscated_name: Option<String>,
    pub owner: Option<String>,
    pub library_uri: Option<String>,
    pub source_location: Option<RecoveredSourceLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inlined_functions: Vec<RecoveredInlineFunction>,
    /// Pc ranges the optimizer folded from other functions into this body,
    /// paired with each inlinee's identity. Statements inside a region belong
    /// to the inlinee, not to this host (probe EC-1).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inline_regions: Vec<RecoveredInlineRegion>,
    pub kind: Option<RecoveredFunctionKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_static: Option<bool>,
    /// Name of the lexically enclosing member for closures, recovered from
    /// the snapshot's `ClosureData.parent_function` edge without debug info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical_parent: Option<String>,
    pub signature: Option<RecoveredSignature>,
    pub signature_source: Option<RecoveredSignatureSource>,
    pub parameter_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_evidence: Option<VmFunctionEvidence>,
    pub address: String,
    pub size: u64,
    pub code_metadata: Option<RecoveredCodeMetadata>,
    pub machine_code: MachineCodeEvidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instructions: Vec<MachineInstruction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_flow: Vec<ControlFlowEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_statements: Vec<SemanticStatement>,
    /// Source-line bands from the body's retained CodeSourceMap, keyed by
    /// statement address. A band is the source line the compiler recorded for
    /// a pc range; statements sharing a band are lowering fragments of one
    /// source statement. Empty when the snapshot shipped no CSM payloads
    /// (`--split-debug-info` builds, arm32).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub source_bands: BTreeMap<String, i64>,
    pub statements: Vec<PseudoStatement>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredCodeMetadata {
    pub stack_map_offset: u32,
    pub stack_map_payload_bytes: usize,
    pub stack_map_entries: usize,
    pub stack_map_uses_global_table: bool,
    pub stack_map_is_global_table: bool,
    pub payload_info: Option<i64>,
    pub unchecked_entry_offset: Option<u64>,
    pub has_monomorphic_entrypoint: bool,
    pub catch_entry_reference: Option<i32>,
    pub inlined_functions_reference: Option<i32>,
    pub pc_descriptors_reference: Option<i32>,
    pub pc_descriptors: Vec<RecoveredPcDescriptor>,
    pub code_source_map_reference: Option<i32>,
    pub code_source_map: Vec<CodeSourceMapEntry>,
    pub exception_handlers_reference: Option<i32>,
    pub handled_types_reference: Option<i32>,
    /// Per-handler-row proven guard types (`on X catch`), resolved from the
    /// snapshot's `ExceptionHandlers.handled_types_data` array-of-arrays.
    /// Empty inner lists mean the row catches everything.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handled_types: Vec<Vec<String>>,
    pub has_async_exception_handler: bool,
    pub exception_handlers: Vec<RecoveredExceptionHandler>,
    /// Protected pc ranges derived from pc-descriptor `try_index` rows joined
    /// with their handler-table rows; empty when the body has no try blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub try_regions: Vec<RecoveredTryRegion>,
}

impl RecoveredCodeMetadata {
    /// Joins each real (non-generated) handler row with the pc-descriptor
    /// rows carrying its try index. The descriptor rows are the only surviving
    /// record of *which instructions* the try block protected — the handler
    /// table itself stores just the catch entry.
    pub fn try_regions(&self) -> Vec<RecoveredTryRegion> {
        let mut regions = Vec::new();
        for handler in &self.exception_handlers {
            if handler.is_generated {
                continue;
            }
            let inside: Vec<u32> = self
                .pc_descriptors
                .iter()
                .filter(|row| row.try_index == handler.try_index as i32)
                .map(|row| row.pc_offset)
                .collect();
            let Some(start) = inside.iter().min() else {
                continue;
            };
            // The range ends where the next descriptor outside the try begins;
            // using the max in-range offset plus one instruction slot keeps the
            // bracket honest without decoding instruction lengths here.
            let end = inside.iter().max().expect("non-empty").saturating_add(1);
            regions.push(RecoveredTryRegion {
                try_index: handler.try_index,
                start_pc_offset: *start,
                end_pc_offset: end,
                handler_pc_offset: handler.handler_pc_offset,
                needs_stack_trace: handler.needs_stack_trace,
                has_catch_all: handler.has_catch_all,
            });
        }
        regions.sort_by_key(|region| region.start_pc_offset);
        regions
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredPcDescriptor {
    pub pc_offset: u32,
    pub kind: String,
    pub try_index: i32,
    pub yield_index: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CodeSourceMapEntry {
    pub pc_offset: u32,
    pub operation: CodeSourceMapOperation,
    pub argument: i32,
    pub inline_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_reference: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeSourceMapOperation {
    ChangePosition,
    AdvancePc,
    PushFunction,
    PopFunction,
    NullCheck,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveredExceptionHandler {
    /// This row's own try index (its position in the handler table); the
    /// `try_index` recorded in pc-descriptor rows refers to it.
    #[serde(default)]
    pub try_index: usize,
    pub handler_pc_offset: u32,
    pub outer_try_index: i16,
    pub needs_stack_trace: bool,
    pub has_catch_all: bool,
    pub is_generated: bool,
}

/// One protected region: pc-descriptor rows with `try_index == index` span
/// `[start_pc, end_pc)` inside this body, dispatched by the VM into the
/// handler at `handler_pc_offset`.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct RecoveredTryRegion {
    pub try_index: usize,
    pub start_pc_offset: u32,
    pub end_pc_offset: u32,
    pub handler_pc_offset: u32,
    pub needs_stack_trace: bool,
    pub has_catch_all: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveredNameSource {
    Snapshot,
    DartVmOracle,
    SplitDebugInfo,
    ObfuscationMap,
    Synthetic,
    /// Assigned by an LLM or any other heuristic guesser. Such names are
    /// speculative by definition: they must be rendered with an explicit
    /// caveat and can never be promoted into proven evidence. Constructed by
    /// external naming pipelines, not by Clutter's own parsers.
    #[allow(dead_code)]
    LlmAssisted,
}

impl RecoveredNameSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Snapshot => "snapshot",
            Self::DartVmOracle => "dart_vm_oracle",
            Self::SplitDebugInfo => "split_debug_info",
            Self::ObfuscationMap => "obfuscation_map",
            Self::Synthetic => "synthetic",
            Self::LlmAssisted => "llm_assisted",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredSourceLocation {
    pub path: String,
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredInlineFunction {
    pub name: String,
    pub library_uri: Option<String>,
    pub source_location: Option<RecoveredSourceLocation>,
    pub call_location: Option<RecoveredSourceLocation>,
    pub address: String,
    pub size: u64,
}

/// A pc range inside a host body that the optimizer folded from another
/// function (`push_function` .. `pop_function` in the code source map). The
/// statements in this range are the inlinee's source logic, not the host's —
/// attributing them lets a vanished callee's body survive inside the host's
/// output (probe EC-1).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredInlineRegion {
    /// Snapshot object reference of the inlined Function (-1 when unresolved).
    pub function_reference: i32,
    pub name: String,
    pub library_uri: Option<String>,
    /// Inclusive start / exclusive end pc offsets relative to the code entry.
    pub start_pc_offset: u32,
    pub end_pc_offset: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveredFunctionKind {
    Regular,
    Closure,
    ImplicitClosure,
    Getter,
    Setter,
    Constructor,
    ImplicitGetter,
    ImplicitSetter,
    ImplicitStaticGetter,
    FieldInitializer,
    MethodExtractor,
    NoSuchMethodDispatcher,
    InvokeFieldDispatcher,
    Irregexp,
    DynamicInvocationForwarder,
    FfiTrampoline,
    RecordFieldGetter,
}

impl RecoveredFunctionKind {
    pub fn from_raw_tag(tag: u32) -> Option<Self> {
        Some(match tag & 0x1f {
            0 => Self::Regular,
            1 => Self::Closure,
            2 => Self::ImplicitClosure,
            3 => Self::Getter,
            4 => Self::Setter,
            5 => Self::Constructor,
            6 => Self::ImplicitGetter,
            7 => Self::ImplicitSetter,
            8 => Self::ImplicitStaticGetter,
            9 => Self::FieldInitializer,
            10 => Self::MethodExtractor,
            11 => Self::NoSuchMethodDispatcher,
            12 => Self::InvokeFieldDispatcher,
            13 => Self::Irregexp,
            14 => Self::DynamicInvocationForwarder,
            15 => Self::FfiTrampoline,
            16 => Self::RecordFieldGetter,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Closure => "closure",
            Self::ImplicitClosure => "implicit_closure",
            Self::Getter => "getter",
            Self::Setter => "setter",
            Self::Constructor => "constructor",
            Self::ImplicitGetter => "implicit_getter",
            Self::ImplicitSetter => "implicit_setter",
            Self::ImplicitStaticGetter => "implicit_static_getter",
            Self::FieldInitializer => "field_initializer",
            Self::MethodExtractor => "method_extractor",
            Self::NoSuchMethodDispatcher => "no_such_method_dispatcher",
            Self::InvokeFieldDispatcher => "invoke_field_dispatcher",
            Self::Irregexp => "irregexp",
            Self::DynamicInvocationForwarder => "dynamic_invocation_forwarder",
            Self::FfiTrampoline => "ffi_trampoline",
            Self::RecordFieldGetter => "record_field_getter",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredSignature {
    pub fixed_parameter_count: usize,
    pub optional_parameter_count: usize,
    pub optional_parameters_are_named: bool,
    pub implicit_parameter_count: usize,
    pub type_parameters_reference: Option<i32>,
    pub result_type_reference: Option<i32>,
    pub parameter_types_reference: Option<i32>,
    pub named_parameter_names_reference: Option<i32>,
    pub flags: u8,
    pub packed_type_parameter_counts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<RecoveredSignatureDetails>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredSignatureDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<RecoveredType>,
    pub parameters: Vec<RecoveredParameter>,
    pub type_parameters: Vec<RecoveredTypeParameter>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredParameter {
    pub position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub declared_type: Option<RecoveredType>,
    pub is_named: bool,
    pub is_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredTypeParameter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound: Option<RecoveredType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveredType {
    pub snapshot_reference: i32,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_uri: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveredSignatureSource {
    SnapshotFunction,
    DartVmOracle,
    RelatedFunction,
}

impl RecoveredSignatureSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::SnapshotFunction => "snapshot_function",
            Self::DartVmOracle => "dart_vm_oracle",
            Self::RelatedFunction => "related_function",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct MachineCodeEvidence {
    pub instruction_bytes: usize,
    pub decoded_bytes: usize,
    pub decoded_instructions: usize,
    /// Basic-block starts discovered from direct branch targets and terminators.
    pub basic_block_starts: usize,
    pub control_flow_edges: usize,
    pub reachable_basic_blocks: usize,
    pub conditional_branches: usize,
    pub unconditional_branches: usize,
    pub returns: usize,
    pub direct_calls: usize,
    pub code_resolved_direct_calls: usize,
    pub indirect_calls: usize,
    pub unknown_bytes: usize,
    pub object_pool_loads: usize,
    pub dispatch_table_calls: usize,
    pub resolved_dispatch_table_calls: usize,
    pub semantic_statements: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct MachineInstruction {
    pub address: String,
    pub bytes: String,
    pub mnemonic: String,
    pub operands: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_pool_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_pool_value: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlFlowEdge {
    pub from: String,
    pub to: String,
    pub kind: ControlFlowEdgeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFlowEdgeKind {
    Fallthrough,
    Branch,
    ConditionalTrue,
    ConditionalFalse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceConfidence {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SemanticStatement {
    Return {
        expression: String,
        confidence: EvidenceConfidence,
        address: String,
    },
    ResolvedCall {
        target: String,
        arguments: Vec<String>,
        confidence: EvidenceConfidence,
        address: String,
    },
    FieldRead {
        receiver: String,
        field: String,
        offset: i64,
        expression: String,
        confidence: EvidenceConfidence,
        address: String,
    },
    FieldWrite {
        receiver: String,
        field: String,
        offset: i64,
        value: String,
        confidence: EvidenceConfidence,
        address: String,
    },
    Condition {
        expression: String,
        true_target: Option<String>,
        false_target: Option<String>,
        confidence: EvidenceConfidence,
        address: String,
    },
    /// A string literal rebuilt from the AOT allocation + `_interpolate`
    /// lowering. `parts` currently holds one fully rendered literal.
    StringInterpolation {
        parts: Vec<String>,
        confidence: EvidenceConfidence,
        address: String,
    },
}

impl SemanticStatement {
    pub fn address(&self) -> &str {
        match self {
            Self::Return { address, .. }
            | Self::ResolvedCall { address, .. }
            | Self::FieldRead { address, .. }
            | Self::FieldWrite { address, .. }
            | Self::Condition { address, .. }
            | Self::StringInterpolation { address, .. } => address,
        }
    }

    pub fn is_return(&self) -> bool {
        matches!(self, Self::Return { .. })
    }

    #[allow(dead_code)]
    pub fn confidence(&self) -> EvidenceConfidence {
        match self {
            Self::Return { confidence, .. }
            | Self::ResolvedCall { confidence, .. }
            | Self::FieldRead { confidence, .. }
            | Self::FieldWrite { confidence, .. }
            | Self::Condition { confidence, .. }
            | Self::StringInterpolation { confidence, .. } => *confidence,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallTargetScope {
    Application,
    Package,
    FlutterSdk,
    DartSdk,
    Runtime,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectCallResolution {
    ExactEntry,
    UncheckedEntry,
    RangeInterior,
}

impl DirectCallResolution {
    pub fn label(self) -> &'static str {
        match self {
            Self::ExactEntry => "exact_entry",
            Self::UncheckedEntry => "unchecked_entry",
            Self::RangeInterior => "range_interior",
        }
    }
}

impl CallTargetScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Package => "package",
            Self::FlutterSdk => "flutter_sdk",
            Self::DartSdk => "dart_sdk",
            Self::Runtime => "runtime",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PseudoStatement {
    Comment {
        text: String,
    },
    DirectCall {
        address: String,
        target_address: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_code_address: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_entry_offset: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_resolution: Option<DirectCallResolution>,
        target: Option<String>,
        target_library_uri: Option<String>,
        target_scope: CallTargetScope,
    },
    IndirectCall {
        address: String,
        expression: String,
    },
    RecoveredIndirectCall {
        address: String,
        expression: String,
        target: String,
        target_library_uri: Option<String>,
        target_scope: CallTargetScope,
    },
    ObjectPoolCall {
        address: String,
        expression: String,
        pool_index: usize,
        target: String,
        target_scope: CallTargetScope,
    },
    DispatchTableCall {
        address: String,
        expression: String,
        selector_offset: usize,
        selector_name: Option<String>,
        candidate_targets: Vec<String>,
        candidate_count: usize,
        raw_slot_target_count: usize,
    },
    Branch {
        address: String,
        target_address: Option<String>,
        conditional: bool,
    },
    MachineReturn {
        address: String,
    },
    UnknownOperation {
        address: String,
        bytes: String,
    },
    ReturnUnknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct Warning {
    pub code: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{Abi, RecoveredFunctionKind};

    #[test]
    fn decodes_function_kind_from_low_tag_bits() {
        assert_eq!(
            RecoveredFunctionKind::from_raw_tag(5),
            Some(RecoveredFunctionKind::Constructor)
        );
        assert_eq!(
            RecoveredFunctionKind::from_raw_tag(0xffff_ffe1),
            Some(RecoveredFunctionKind::Closure)
        );
        assert_eq!(RecoveredFunctionKind::from_raw_tag(17), None);
    }

    #[test]
    fn serializes_android_abi_names_verbatim() {
        assert_eq!(serde_json::to_string(&Abi::X86_64).unwrap(), "\"x86_64\"");
    }
}
