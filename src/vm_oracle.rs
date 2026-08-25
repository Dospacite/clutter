use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::evidence::subject::ArtifactSubject;
use crate::model::{
    Abi, PseudoStatement, RecoveredClassMetadata, RecoveredDeclaration, RecoveredDeclarationKind,
    RecoveredFieldMetadata, RecoveredFunction, RecoveredFunctionKind, RecoveredInstanceSlot,
    RecoveredLibrary, RecoveredNameSource, RecoveredParameter, RecoveredProgram,
    RecoveredSignature, RecoveredSignatureDetails, RecoveredSignatureSource, RecoveredType,
    RecoveredTypeParameter, Scope, SnapshotInfo, VmFunctionEvidence, VmOracleEvidence,
    VmParameterEvidence, VmTypeEvidence, VmTypeParameterEvidence, Warning,
};

const MAX_ORACLE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) struct LoadedVmOracle {
    pub(crate) evidence: VmOracleEvidence,
    functions: Vec<VmFunctionEvidence>,
    declarations: Vec<RecoveredDeclaration>,
    libraries: Vec<RecoveredLibrary>,
    object_pool_labels: BTreeMap<usize, String>,
    /// Schema 5 dispatch rows: selector index -> owner function name. Exact
    /// receiver-CID evidence for recovered indirect calls.
    dispatch_selectors: BTreeMap<usize, String>,
}

impl LoadedVmOracle {
    /// Function evidence for the body/occurrence graph. Includes every
    /// retained entry: matched functions, code-owner links, stubs, and
    /// unattributed code boundaries.
    pub(crate) fn functions(&self) -> &[VmFunctionEvidence] {
        &self.functions
    }
}

struct AnalyzerDocument {
    objects: Vec<AnalyzerObject>,
    metadata: AnalyzerMetadata,
    static_calls: Option<AnalyzerStaticCallsSection>,
    dispatch_metadata: Option<AnalyzerDispatchMetadataSection>,
    class_ranges: Option<AnalyzerClassRangesSection>,
}

#[derive(Deserialize)]
struct AnalyzerEnvelope {
    #[serde(default)]
    objects: Vec<serde_json::Value>,
    metadata: AnalyzerMetadata,
    /// Schema 5 emits payload-wide semantic evidence as top-level keys after
    /// the objects array; older schemas omit them entirely.
    #[serde(default)]
    static_calls: Option<AnalyzerStaticCallsSection>,
    #[serde(default)]
    dispatch_metadata: Option<AnalyzerDispatchMetadataSection>,
    #[serde(default)]
    class_ranges: Option<AnalyzerClassRangesSection>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AnalyzerStaticCallsSection {
    #[serde(default)]
    targets: Vec<AnalyzerStaticCall>,
    #[serde(default)]
    entry_count: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AnalyzerStaticCall {
    #[serde(default)]
    pool_index: u64,
    // Identity fields beyond pool_index/owner_name are retained as part of the
    // row schema; later accuracy phases join them against Code/Function ids.
    #[serde(default)]
    #[allow(dead_code)]
    target_offset: i64,
    #[serde(default)]
    #[allow(dead_code)]
    size: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    owner_id: Option<u64>,
    #[serde(default)]
    owner_name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    owner_is_static: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    owner_parameter_count: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AnalyzerDispatchMetadataSection {
    #[serde(default)]
    #[allow(dead_code)] // reported alongside dispatch selectors in a later phase
    code_entry_count: Option<u64>,
    #[serde(default)]
    targets: Vec<AnalyzerDispatchTarget>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AnalyzerDispatchTarget {
    #[serde(default)]
    selector_index: u64,
    #[serde(default)]
    #[allow(dead_code)]
    target_offset: i64,
    #[serde(default)]
    #[allow(dead_code)]
    size: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    owner_id: Option<u64>,
    #[serde(default)]
    owner_name: Option<String>,
}

/// The analyzer emits each populated CID run as two consecutive integers
/// (`[start, end, start, end, …]`); older patch revisions may instead emit
/// `[start, end]` pairs. Accept both shapes so oracle documents from either
/// analyzer build parse identically.
fn deserialize_cid_runs<'de, D>(deserializer: D) -> std::result::Result<Vec<(u64, u64)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Patch revisions differ on the run encoding: current builds emit two
    // consecutive integers per run (`[start, end, …]`); older drafts emitted a
    // `[start, end]` pair per run. Accept either shape.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Runs {
        Paired(Vec<(u64, u64)>),
        Flat(Vec<u64>),
    }
    match Runs::deserialize(deserializer)? {
        Runs::Paired(runs) => Ok(runs),
        Runs::Flat(flat) => {
            let mut runs = Vec::with_capacity(flat.len() / 2);
            let mut values = flat.into_iter();
            while let (Some(start), Some(end)) = (values.next(), values.next()) {
                runs.push((start, end));
            }
            Ok(runs)
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AnalyzerClassRangesSection {
    #[serde(default)]
    #[allow(dead_code)] // consumed by the class-range attribution phase
    num_cids: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)] // consumed by the class-range attribution phase
    num_top_level_cids: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_cid_runs")]
    populated_runs: Vec<(u64, u64)>,
}

#[derive(Default, Deserialize)]
struct AnalyzerMetadata {
    #[serde(default)]
    dart_version: Option<String>,
    #[serde(default)]
    dart_commit: Option<String>,
    #[serde(default)]
    snapshot_hash: String,
    #[serde(default)]
    root_library: Option<u64>,
    #[serde(default)]
    word_size: u64,
    #[serde(default)]
    compressed_word_size: u64,
    #[serde(default)]
    analyzer_version: u64,
    /// Schema 5: length of the AOT global object pool.
    #[serde(default)]
    global_object_pool_length: Option<u64>,
    /// Schema 5: dispatch table origin element for the analyzer's target arch.
    #[serde(default)]
    dispatch_table_origin_element: Option<u64>,
}

#[derive(Default, Deserialize)]
struct AnalyzerObject {
    #[serde(default)]
    id: Option<u64>,
    #[serde(rename = "type", default)]
    object_type: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    raw_name: Option<String>,
    #[serde(default)]
    user_visible_name: Option<String>,
    #[serde(default)]
    scrubbed_name: Option<String>,
    #[serde(default)]
    type_kind: Option<String>,
    #[serde(default)]
    signature: Option<String>,
    #[serde(default)]
    user_visible_signature: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    library: Option<u64>,
    #[serde(default)]
    imports: Vec<u64>,
    #[serde(default)]
    class_id: Option<i32>,
    #[serde(rename = "class", default)]
    class_object: Option<u64>,
    #[serde(default)]
    instance_size: Option<u64>,
    #[serde(default)]
    type_arguments_field_offset: Option<i64>,
    #[serde(default)]
    instance_slots: Vec<AnalyzerInstanceSlot>,
    #[serde(default)]
    super_type: Option<u64>,
    #[serde(default)]
    interfaces: Vec<u64>,
    #[serde(default)]
    is_top_level: Option<bool>,
    #[serde(default)]
    is_abstract: Option<bool>,
    #[serde(default)]
    is_enum: Option<bool>,
    #[serde(default)]
    is_sealed: Option<bool>,
    #[serde(default)]
    is_mixin_class: Option<bool>,
    #[serde(default)]
    is_base: Option<bool>,
    #[serde(default)]
    is_interface: Option<bool>,
    #[serde(default)]
    is_final: Option<bool>,
    #[serde(default)]
    is_transformed_mixin_application: Option<bool>,
    #[serde(default)]
    owner_class: Option<u64>,
    #[serde(default)]
    type_class: Option<u64>,
    #[serde(default)]
    result_type: Option<u64>,
    #[serde(default)]
    parameters: Vec<AnalyzerParameter>,
    #[serde(default)]
    type_parameters: Vec<AnalyzerTypeParameter>,
    #[serde(default)]
    code: Option<u64>,
    #[serde(default)]
    current_code: Option<u64>,
    #[serde(default)]
    code_link_source: Option<String>,
    #[serde(default)]
    parent_function: Option<u64>,
    #[serde(default)]
    owner: Option<u64>,
    #[serde(default)]
    offset: Option<i64>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    section: Option<String>,
    #[serde(default)]
    is_stub: Option<bool>,
    /// Schema 5: raw unboxed-field bitmap per class (bit per word slot).
    /// Serialized through `int64_t`, so masks with bit 63 set arrive negative
    /// and are reinterpreted losslessly on load. Parsed for forward
    /// compatibility; consumed by field layout resolution later.
    #[serde(default)]
    #[allow(dead_code)]
    unboxed_field_bitmap: Option<i64>,
    /// Schema 5: populated CID range start/end for the class.
    #[serde(default)]
    #[allow(dead_code)] // superseded by the top-level `class_ranges` section
    cid_range_start: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    cid_range_end: Option<i64>,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    initializer_function: Option<u64>,
    #[serde(default)]
    instance_field_offset: Option<i64>,
    #[serde(default)]
    static_field_offset: Option<i64>,
    #[serde(default)]
    instance: Option<u64>,
    #[serde(default)]
    is_reference: Option<bool>,
    #[serde(default)]
    unboxed_type: Option<String>,
    #[serde(default)]
    references: Vec<u64>,
    #[serde(default)]
    elements: Vec<u64>,
    #[serde(default)]
    fixed_parameter_count: Option<usize>,
    #[serde(default)]
    optional_parameter_count: Option<usize>,
    #[serde(default)]
    implicit_parameter_count: Option<usize>,
    #[serde(default)]
    optional_parameters_are_named: Option<bool>,
    #[serde(default)]
    is_static: Option<bool>,
    #[serde(default)]
    is_async: Option<bool>,
    #[serde(default)]
    is_sync_generator: Option<bool>,
    #[serde(default)]
    is_async_generator: Option<bool>,
}

#[derive(Clone, Default, Deserialize)]
struct AnalyzerInstanceSlot {
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    is_reference: bool,
    #[serde(default)]
    slot_type: String,
    #[serde(default)]
    field: Option<u64>,
}

#[derive(Default, Deserialize)]
struct AnalyzerParameter {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type", default)]
    declared_type: Option<u64>,
    #[serde(default)]
    is_implicit: bool,
    #[serde(default)]
    is_named: bool,
    #[serde(default)]
    is_required: bool,
}

#[derive(Default, Deserialize)]
struct AnalyzerTypeParameter {
    #[serde(default)]
    name: String,
    #[serde(default)]
    bound: Option<u64>,
    #[serde(rename = "default", default)]
    default_type: Option<u64>,
}

/// Overlays schema-5 static-call rows onto the object-pool labels. Rows carry
/// the pool index plus the target Code's owner identity, so entries whose
/// graph walk dead-ended still get the owner's name. Existing labels (from
/// deeper graph evidence) always win.
fn overlay_static_call_labels(labels: &mut BTreeMap<usize, String>, rows: &[AnalyzerStaticCall]) {
    for row in rows {
        let Ok(index) = usize::try_from(row.pool_index) else {
            continue;
        };
        if labels.contains_key(&index) {
            continue;
        }
        if let Some(name) = row
            .owner_name
            .as_deref()
            .filter(|name| !name.is_empty())
        {
            labels.insert(index, name.to_owned());
        }
    }
}

/// Applies oracle-proven dispatch selector names to recovered indirect calls.
/// Returns how many statements gained a proven selector name. A selector the
/// oracle names is exact evidence: it overrides heuristic synthetic labels and
/// clears candidate lists derived from them.
pub(crate) fn apply_dispatch_selector_evidence(
    functions: &mut [RecoveredFunction],
    selectors: &BTreeMap<usize, String>,
) -> usize {
    let mut resolved = 0usize;
    let looks_synthetic = |label: &str| {
        label.is_empty()
            || label.starts_with("sub_")
            || label.starts_with("0x")
            || label.starts_with("dispatch[")
    };
    for function in functions {
        for statement in &mut function.statements {
            let PseudoStatement::DispatchTableCall {
                selector_offset,
                selector_name,
                candidate_targets,
                ..
            } = statement
            else {
                continue;
            };
            let Some(name) = selectors.get(selector_offset) else {
                continue;
            };
            let improves = selector_name.as_deref().is_none_or(looks_synthetic);
            if improves && !name.is_empty() {
                *selector_name = Some(name.clone());
                candidate_targets.retain(|target| !looks_synthetic(target));
                resolved += 1;
            }
        }
    }
    resolved
}

/// Labels object-pool entries by walking each entry's referenced object graph
/// and rendering the first meaningful identity (function name, field, class,
/// type, library, string payload).
fn recover_object_pool_labels(
    objects: &[AnalyzerObject],
    classes: &HashMap<u64, (Option<String>, Option<u64>)>,
    libraries: &HashMap<u64, String>,
) -> BTreeMap<usize, String> {
    let by_id = objects
        .iter()
        .filter_map(|object| Some((object.id?, object)))
        .collect::<HashMap<_, _>>();
    let Some(pool) = objects
        .iter()
        .find(|object| object.object_type.as_deref() == Some("ObjectPool"))
    else {
        return BTreeMap::new();
    };
    let mut labels = BTreeMap::new();
    for triplet in pool.references.chunks_exact(3) {
        let Ok(index) = usize::try_from(triplet[0]) else {
            continue;
        };
        let mut visiting = HashSet::new();
        if let Some(label) =
            vm_object_label(triplet[2], &by_id, classes, libraries, 0, &mut visiting)
        {
            labels.insert(index, label);
        }
    }
    labels
}

fn vm_object_label(
    object_id: u64,
    objects: &HashMap<u64, &AnalyzerObject>,
    classes: &HashMap<u64, (Option<String>, Option<u64>)>,
    libraries: &HashMap<u64, String>,
    depth: usize,
    visiting: &mut HashSet<u64>,
) -> Option<String> {
    if depth >= 3 || !visiting.insert(object_id) {
        return None;
    }
    let object = objects.get(&object_id)?;
    let label = match object.object_type.as_deref()? {
        "String" => object
            .value
            .as_ref()
            .and_then(|value| serde_json::to_string(value).ok()),
        "Function" => object
            .user_visible_name
            .clone()
            .or_else(|| object.name.clone())
            .or_else(|| object.raw_name.clone()),
        "Code" => object.owner.and_then(|owner| {
            vm_object_label(owner, objects, classes, libraries, depth + 1, visiting)
        }),
        "Field" => {
            let name = object.user_visible_name.as_ref().or(object.name.as_ref())?;
            let owner = object
                .owner_class
                .and_then(|id| classes.get(&id))
                .and_then(|(name, _)| name.as_ref());
            Some(match owner {
                Some(owner) if !matches!(owner.as_str(), "::" | "top_level") => {
                    format!("snapshotField({owner}.{name})")
                }
                _ => format!("snapshotField({name})"),
            })
        }
        "Class" => object
            .user_visible_name
            .as_ref()
            .or(object.scrubbed_name.as_ref())
            .or(object.name.as_ref())
            .map(|name| format!("snapshotClass({name})")),
        "Type" => object
            .user_visible_name
            .as_ref()
            .or(object.scrubbed_name.as_ref())
            .or(object.name.as_ref())
            .map(|name| format!("snapshotType({name})")),
        "Library" => object
            .url
            .as_ref()
            .map(|uri| format!("snapshotLibrary({uri})")),
        "Instance" | "Array" => {
            let class_name = object
                .class_object
                .or(object.type_class)
                .or(object.owner_class)
                .and_then(|id| classes.get(&id))
                .and_then(|(name, _)| name.clone())
                .unwrap_or_else(|| object.object_type.clone().unwrap_or_default());
            let references = if object.elements.is_empty() {
                &object.references
            } else {
                &object.elements
            };
            let mut strings = references
                .iter()
                .filter_map(|reference| {
                    let nested = objects.get(reference)?;
                    (nested.object_type.as_deref() == Some("String"))
                        .then(|| nested.value.clone())
                        .flatten()
                })
                .take(4)
                .collect::<Vec<_>>();
            strings.sort();
            strings.dedup();
            if strings.is_empty() {
                Some(format!("snapshotInstance({class_name})"))
            } else {
                Some(format!(
                    "snapshotInstance({class_name}) nestedStrings[{}]",
                    strings
                        .iter()
                        .filter_map(|value| serde_json::to_string(value).ok())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        }
        "Null" => Some("null".to_owned()),
        _ => None,
    };
    visiting.remove(&object_id);
    let _ = libraries;
    label
}

pub(crate) fn load(
    path: &Path,
    snapshot: &SnapshotInfo,
    abi: Abi,
    subject: &ArtifactSubject,
) -> Result<LoadedVmOracle> {
    let oracle = load_unbound(path, snapshot, abi)?;
    crate::evidence::oracle::verify_binding(path, subject, &oracle.evidence)?;
    Ok(oracle)
}

pub(crate) fn load_unbound(
    path: &Path,
    snapshot: &SnapshotInfo,
    abi: Abi,
) -> Result<LoadedVmOracle> {
    let metadata = fs::metadata(path).at(path)?;
    if metadata.len() > MAX_ORACLE_BYTES {
        return Err(ClutterError::InvalidArtifact(format!(
            "Dart VM oracle JSON is {} bytes, exceeding the {} byte limit",
            metadata.len(),
            MAX_ORACLE_BYTES
        )));
    }
    let bytes = fs::read(path).at(path)?;
    let source_sha256 = hex::encode(Sha256::digest(&bytes));
    let envelope: AnalyzerEnvelope = serde_json::from_slice(&bytes)?;
    let objects = envelope
        .objects
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<Vec<AnalyzerObject>, _>>()?;
    let document = AnalyzerDocument {
        objects,
        metadata: envelope.metadata,
        static_calls: envelope.static_calls,
        dispatch_metadata: envelope.dispatch_metadata,
        class_ranges: envelope.class_ranges,
    };

    validate_metadata(&document.metadata, snapshot, abi)?;

    let mut object_kinds = BTreeMap::new();
    for object in &document.objects {
        *object_kinds
            .entry(
                object
                    .object_type
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_owned()),
            )
            .or_default() += 1;
    }

    let libraries = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Library"))
        .filter_map(|object| Some((object.id?, object.url.clone()?)))
        .collect::<HashMap<_, _>>();
    let mut recovered_libraries = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Library"))
        .filter_map(|object| {
            let object_id = object.id?;
            let uri = object.url.clone()?;
            let mut imports = object
                .imports
                .iter()
                .filter_map(|id| libraries.get(id).cloned())
                .collect::<Vec<_>>();
            imports.sort();
            imports.dedup();
            Some(RecoveredLibrary {
                output_path: crate::analysis::library_output_path(&uri, None),
                package: root_package(&uri).map(str::to_owned),
                is_application: false,
                uri,
                vm_object_id: Some(object_id),
                imports,
                referenced_libraries: Vec::new(),
            })
        })
        .collect::<Vec<_>>();
    let classes = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Class"))
        .filter_map(|object| {
            Some((
                object.id?,
                (
                    object
                        .user_visible_name
                        .clone()
                        .or_else(|| object.scrubbed_name.clone())
                        .or_else(|| object.name.clone()),
                    object.library,
                ),
            ))
        })
        .collect::<HashMap<_, _>>();
    let field_names = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Field"))
        .filter_map(|object| {
            Some((
                object.id?,
                object
                    .user_visible_name
                    .clone()
                    .or_else(|| object.name.clone())?,
            ))
        })
        .collect::<HashMap<_, _>>();
    let object_pool_labels = recover_object_pool_labels(&document.objects, &classes, &libraries);
    // Schema 5 static-call rows identify pool entries whose Code target has a
    // Function owner even when the graph walk cannot reach the owner object.
    let mut object_pool_labels = object_pool_labels;
    if let Some(static_calls) = &document.static_calls {
        overlay_static_call_labels(&mut object_pool_labels, &static_calls.targets);
    }
    let class_type_parameters = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Class"))
        .filter_map(|object| {
            Some((
                object.id?,
                object
                    .type_parameters
                    .iter()
                    .map(|parameter| parameter.name.clone())
                    .collect::<Vec<_>>(),
            ))
        })
        .collect::<HashMap<_, _>>();
    let types = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Type"))
        .filter_map(|object| {
            let object_id = object.id?;
            let display_name = object
                .user_visible_name
                .clone()
                .or_else(|| object.scrubbed_name.clone())
                .or_else(|| object.name.clone())?;
            let library_uri = object
                .type_class
                .and_then(|id| classes.get(&id))
                .and_then(|(_, library)| *library)
                .and_then(|id| libraries.get(&id))
                .cloned();
            Some((
                object_id,
                VmTypeEvidence {
                    object_id,
                    type_kind: object.type_kind.clone(),
                    display_name,
                    library_uri,
                },
            ))
        })
        .collect::<HashMap<_, _>>();
    let codes = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Code"))
        .filter_map(|object| {
            Some((
                object.id?,
                (
                    object.offset.and_then(|offset| u64::try_from(offset).ok()),
                    object.size,
                    object.section.as_deref() == Some("_kDartIsolateSnapshotInstructions"),
                ),
            ))
        })
        .collect::<HashMap<_, _>>();
    let function_ids = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Function"))
        .filter_map(|object| object.id)
        .collect::<HashSet<_>>();
    let function_identities = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Function"))
        .filter_map(|object| {
            let id = object.id?;
            let name = object
                .user_visible_name
                .clone()
                .or_else(|| object.name.clone())
                .or_else(|| object.raw_name.clone())?;
            let owner = object
                .owner_class
                .and_then(|owner| classes.get(&owner))
                .and_then(|(name, _)| name.clone())
                .filter(|name| !matches!(name.as_str(), "::" | "top_level"));
            Some((id, (name, owner)))
        })
        .collect::<HashMap<_, _>>();
    let mut codes_by_function_owner = HashMap::<u64, Vec<u64>>::new();
    for object in document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Code"))
    {
        let (Some(code_id), Some(owner_id)) = (object.id, object.owner) else {
            continue;
        };
        if function_ids.contains(&owner_id)
            && codes.get(&code_id).is_some_and(|(_, _, isolate)| *isolate)
        {
            codes_by_function_owner
                .entry(owner_id)
                .or_default()
                .push(code_id);
        }
    }

    let mut functions_linked_via_code_owner = 0usize;
    let mut functions = document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Function"))
        .filter_map(|object| {
            let object_id = object.id?;
            let (owner, library_uri) = object
                .owner_class
                .and_then(|id| classes.get(&id))
                .map(|(name, library)| {
                    (
                        name.clone()
                            .filter(|name| !matches!(name.as_str(), "::" | "top_level")),
                        library.and_then(|id| libraries.get(&id).cloned()),
                    )
                })
                .unwrap_or_default();
            let analyzer_code_object_id = object.code;
            let direct_code_object_id = object
                .code
                .filter(|id| codes.get(id).is_some_and(|(_, _, isolate)| *isolate));
            let owner_code_object_id = codes_by_function_owner
                .get(&object_id)
                .and_then(|ids| ids.first())
                .copied();
            let code_object_id = direct_code_object_id.or(owner_code_object_id);
            let code_link_source = if object.code_link_source.as_deref() == Some("code_owner")
                || (direct_code_object_id.is_none() && owner_code_object_id.is_some())
            {
                functions_linked_via_code_owner += 1;
                Some("code.owner".to_owned())
            } else if direct_code_object_id.is_some() {
                Some("function.code".to_owned())
            } else {
                None
            };
            let (code_offset, code_size) = code_object_id
                .and_then(|id| codes.get(&id))
                .map_or((None, None), |(offset, size, _)| (*offset, *size));
            Some(VmFunctionEvidence {
                object_id,
                code_object_id,
                analyzer_code_object_id,
                current_code_object_id: object.current_code,
                code_link_source,
                code_offset,
                code_size,
                logical_match_score: None,
                logical_match_candidate_count: None,
                alternative_parent_functions: Vec::new(),
                name: object.name.clone().unwrap_or_default(),
                raw_name: object.raw_name.clone(),
                user_visible_name: object.user_visible_name.clone(),
                owner,
                library_uri,
                parent_function_object_id: object.parent_function,
                parent_function_name: object
                    .parent_function
                    .and_then(|id| function_identities.get(&id))
                    .map(|(name, _)| name.clone()),
                parent_function_owner: object
                    .parent_function
                    .and_then(|id| function_identities.get(&id))
                    .and_then(|(_, owner)| owner.clone()),
                signature: object.signature.clone(),
                user_visible_signature: object.user_visible_signature.clone(),
                result_type: object.result_type.and_then(|id| types.get(&id).cloned()),
                parameters: object
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(position, parameter)| VmParameterEvidence {
                        position,
                        name: parameter.name.clone(),
                        declared_type: parameter
                            .declared_type
                            .and_then(|id| types.get(&id).cloned()),
                        is_implicit: parameter.is_implicit,
                        is_named: parameter.is_named,
                        is_required: parameter.is_required,
                    })
                    .collect(),
                type_parameters: object
                    .type_parameters
                    .iter()
                    .map(|parameter| VmTypeParameterEvidence {
                        name: parameter.name.clone(),
                        bound: parameter.bound.and_then(|id| types.get(&id).cloned()),
                        default_type: parameter
                            .default_type
                            .and_then(|id| types.get(&id).cloned()),
                    })
                    .collect(),
                owner_type_parameters: object
                    .owner_class
                    .and_then(|id| class_type_parameters.get(&id))
                    .cloned()
                    .unwrap_or_default(),
                kind: object.kind.clone(),
                fixed_parameter_count: object.fixed_parameter_count,
                optional_parameter_count: object.optional_parameter_count,
                implicit_parameter_count: object.implicit_parameter_count,
                optional_parameters_are_named: object.optional_parameters_are_named,
                is_static: object.is_static,
                is_async: object.is_async,
                is_sync_generator: object.is_sync_generator,
                is_async_generator: object.is_async_generator,
            })
        })
        .collect::<Vec<_>>();
    let retained_function_count = functions.len();
    let retained_functions_with_code = functions
        .iter()
        .filter(|function| function.code_offset.is_some())
        .count();
    let referenced_codes = functions
        .iter()
        .filter_map(|function| function.code_object_id)
        .collect::<HashSet<_>>();

    // Full AOT deliberately drops many Function objects after compilation.
    // Their Code owner is then a class id. Dart's analyzer resolves that id
    // back to the live Class object, which restores otherwise-lost
    // library/class attribution even though the original function name is no
    // longer present. In DWARF stack-trace mode most remaining instruction
    // table entries have no Code owner at all; retain those too as exact VM
    // code-boundary evidence. VM stubs preserve useful symbolic names.
    functions.extend(
        document
            .objects
            .iter()
            .filter(|object| object.object_type.as_deref() == Some("Code"))
            .filter(|object| object.id.is_some_and(|id| !referenced_codes.contains(&id)))
            .filter(|object| {
                object.section.as_deref() == Some("_kDartIsolateSnapshotInstructions")
                    && object.offset.is_some_and(|offset| offset >= 0)
            })
            .filter_map(|object| {
                let code_object_id = object.id?;
                let (owner, library_uri) = object
                    .owner
                    .and_then(|id| classes.get(&id))
                    .map(|(name, library)| {
                        (
                            name.clone()
                                .filter(|name| !matches!(name.as_str(), "::" | "top_level")),
                            library.and_then(|id| libraries.get(&id).cloned()),
                        )
                    })
                    .unwrap_or_default();
                let is_stub = object.is_stub == Some(true);
                let is_unattributed =
                    object.owner.is_none() && object.name.as_deref() == Some("Unknown Code");
                let (name, raw_name, kind, code_link_source) = if is_stub {
                    let name = object
                        .name
                        .clone()
                        .unwrap_or_else(|| "DartVmStub".to_owned());
                    (name.clone(), Some(name), "VmStubCode", "code.stub")
                } else if owner.is_some() {
                    (
                        object
                            .name
                            .clone()
                            .unwrap_or_else(|| "unknown Dart function".to_owned()),
                        None,
                        "DroppedFunctionCode",
                        "code.owner_class",
                    )
                } else if is_unattributed {
                    (String::new(), None, "AotCodeBoundary", "instructions_table")
                } else {
                    let name = object.name.clone().unwrap_or_default();
                    (
                        name.clone(),
                        (!name.is_empty()).then_some(name),
                        "VmCode",
                        "code.object",
                    )
                };
                Some(VmFunctionEvidence {
                    object_id: code_object_id,
                    code_object_id: Some(code_object_id),
                    analyzer_code_object_id: None,
                    current_code_object_id: None,
                    code_link_source: Some(code_link_source.to_owned()),
                    code_offset: object.offset.and_then(|offset| u64::try_from(offset).ok()),
                    code_size: object.size,
                    logical_match_score: None,
                    logical_match_candidate_count: None,
                    alternative_parent_functions: Vec::new(),
                    name,
                    raw_name,
                    user_visible_name: None,
                    owner,
                    library_uri,
                    parent_function_object_id: None,
                    parent_function_name: None,
                    parent_function_owner: None,
                    signature: None,
                    user_visible_signature: None,
                    result_type: None,
                    parameters: Vec::new(),
                    type_parameters: Vec::new(),
                    owner_type_parameters: Vec::new(),
                    kind: Some(kind.to_owned()),
                    fixed_parameter_count: None,
                    optional_parameter_count: None,
                    implicit_parameter_count: None,
                    optional_parameters_are_named: None,
                    is_static: None,
                    is_async: None,
                    is_sync_generator: None,
                    is_async_generator: None,
                })
            }),
    );

    let retained_function_evidence = functions
        .iter()
        .take(retained_function_count)
        .map(|function| (function.object_id, function))
        .collect::<HashMap<_, _>>();
    let mut declarations = Vec::new();
    for object in document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Class"))
        .filter(|object| object.is_top_level != Some(true))
    {
        let (Some(object_id), Some(class_id), Some(library_id)) =
            (object.id, object.class_id, object.library)
        else {
            continue;
        };
        let Some(library_uri) = libraries.get(&library_id).cloned() else {
            continue;
        };
        let Some((Some(name), _)) = classes.get(&object_id) else {
            continue;
        };
        let parameter_names = class_type_parameters
            .get(&object_id)
            .cloned()
            .unwrap_or_default();
        let type_parameters = object
            .type_parameters
            .iter()
            .map(|parameter| RecoveredTypeParameter {
                name: parameter.name.clone(),
                bound: parameter
                    .bound
                    .and_then(|id| types.get(&id))
                    .and_then(|value| recovered_vm_type(value, &parameter_names, &[])),
            })
            .collect();
        let super_type = object
            .super_type
            .and_then(|id| types.get(&id))
            .and_then(|value| recovered_vm_type(value, &parameter_names, &[]));
        let interfaces = object
            .interfaces
            .iter()
            .filter_map(|id| types.get(id))
            .filter_map(|value| recovered_vm_type(value, &parameter_names, &[]))
            .collect();
        declarations.push(RecoveredDeclaration {
            snapshot_reference: vm_declaration_reference(object_id),
            vm_object_id: Some(object_id),
            kind: RecoveredDeclarationKind::Class,
            name: name.clone(),
            snapshot_name: object.name.clone().filter(|raw| raw != name),
            owner: Some(library_uri.clone()),
            library_uri: Some(library_uri),
            source_location: None,
            function_kind: None,
            signature: None,
            vm_evidence: None,
            class_metadata: Some(RecoveredClassMetadata {
                class_id,
                type_parameters,
                super_type,
                interfaces,
                is_abstract: object.is_abstract.unwrap_or(false),
                is_enum: object.is_enum.unwrap_or(false),
                is_sealed: object.is_sealed.unwrap_or(false),
                is_mixin_class: object.is_mixin_class.unwrap_or(false),
                is_base: object.is_base.unwrap_or(false),
                is_interface: object.is_interface.unwrap_or(false),
                is_final: object.is_final.unwrap_or(false),
                is_transformed_mixin_application: object
                    .is_transformed_mixin_application
                    .unwrap_or(false),
                instance_size: object.instance_size,
                type_arguments_field_offset: object.type_arguments_field_offset,
                instance_slots: object
                    .instance_slots
                    .iter()
                    .map(|slot| RecoveredInstanceSlot {
                        offset: slot.offset,
                        is_reference: slot.is_reference,
                        slot_type: slot.slot_type.clone(),
                        field_object_id: slot.field,
                        field_name: slot.field.and_then(|id| field_names.get(&id)).cloned(),
                    })
                    .collect(),
            }),
            field_metadata: None,
            has_code: false,
        });
    }
    for object in document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Field"))
    {
        let (Some(object_id), Some(owner_id)) = (object.id, object.owner_class) else {
            continue;
        };
        let Some((owner_name, library_id)) = classes.get(&owner_id) else {
            continue;
        };
        let Some(library_uri) = library_id.and_then(|id| libraries.get(&id)).cloned() else {
            continue;
        };
        let name = object
            .user_visible_name
            .clone()
            .or_else(|| object.name.clone())
            .unwrap_or_else(|| format!("field_{object_id}"));
        let declared_type = object
            .type_class
            .and_then(|id| types.get(&id))
            .and_then(|value| {
                recovered_vm_type(
                    value,
                    class_type_parameters
                        .get(&owner_id)
                        .map(Vec::as_slice)
                        .unwrap_or_default(),
                    &[],
                )
            });
        let has_flag = |flag: &str| object.flags.iter().any(|value| value == flag);
        declarations.push(RecoveredDeclaration {
            snapshot_reference: vm_declaration_reference(object_id),
            vm_object_id: Some(object_id),
            kind: RecoveredDeclarationKind::Field,
            name: name.clone(),
            snapshot_name: object.name.clone().filter(|raw| raw != &name),
            owner: owner_name
                .clone()
                .filter(|owner| !matches!(owner.as_str(), "::" | "top_level")),
            library_uri: Some(library_uri),
            source_location: None,
            function_kind: None,
            signature: None,
            vm_evidence: None,
            class_metadata: None,
            field_metadata: Some(RecoveredFieldMetadata {
                type_reference: object
                    .type_class
                    .and_then(|id| i32::try_from(id).ok())
                    .unwrap_or(-1),
                declared_type,
                initializer_reference: object
                    .initializer_function
                    .map(vm_declaration_reference)
                    .unwrap_or(-1),
                offset_or_field_id_reference: None,
                is_static: has_flag("static"),
                is_final: has_flag("final"),
                is_const: has_flag("const"),
                is_late: has_flag("late"),
                has_initializer: object.initializer_function.is_some(),
                has_nontrivial_initializer: object.initializer_function.is_some(),
                instance_field_offset: object.instance_field_offset,
                static_field_offset: object.static_field_offset,
                static_value_object_id: object.instance,
                is_reference: object.is_reference,
                unboxed_type: object.unboxed_type.clone(),
            }),
            has_code: false,
        });
    }
    for object in document
        .objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some("Function"))
    {
        let Some(object_id) = object.id else {
            continue;
        };
        let Some(evidence) = retained_function_evidence.get(&object_id) else {
            continue;
        };
        let Some(library_uri) = evidence.library_uri.clone() else {
            continue;
        };
        let name = evidence
            .raw_name
            .clone()
            .or_else(|| evidence.user_visible_name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("function_{object_id}"));
        let name = crate::analysis::readable_snapshot_name(&name);
        declarations.push(RecoveredDeclaration {
            snapshot_reference: vm_declaration_reference(object_id),
            vm_object_id: Some(object_id),
            kind: RecoveredDeclarationKind::Function,
            name,
            snapshot_name: None,
            owner: evidence.owner.clone(),
            library_uri: Some(library_uri),
            source_location: None,
            function_kind: evidence.kind.as_deref().and_then(vm_function_kind),
            signature: recovered_signature_from_vm(evidence),
            vm_evidence: Some((**evidence).clone()),
            class_metadata: None,
            field_metadata: None,
            has_code: evidence.code_offset.is_some(),
        });
    }

    let referenced_libraries = recover_library_references(&functions, &declarations);
    for library in &mut recovered_libraries {
        library.referenced_libraries = referenced_libraries
            .get(&library.uri)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default();
    }
    let library_reference_edges = referenced_libraries
        .values()
        .map(BTreeSet::len)
        .sum::<usize>();

    let root_library_uri = document
        .metadata
        .root_library
        .and_then(|id| libraries.get(&id).cloned());
    let target_arch = document
        .metadata
        .dart_version
        .as_deref()
        .and_then(target_arch);
    let evidence = VmOracleEvidence {
        source: path.to_path_buf(),
        source_size: metadata.len(),
        source_sha256,
        dart_version: document.metadata.dart_version,
        dart_commit: document.metadata.dart_commit,
        snapshot_hash: document.metadata.snapshot_hash,
        analyzer_version: document.metadata.analyzer_version,
        target_arch,
        word_size: document.metadata.word_size,
        compressed_word_size: document.metadata.compressed_word_size,
        root_library_object_id: document.metadata.root_library,
        root_library_uri,
        object_count: document.objects.len(),
        object_kinds,
        library_count: count_kind(&document.objects, "Library"),
        class_count: count_kind(&document.objects, "Class"),
        field_count: count_kind(&document.objects, "Field"),
        function_count: retained_function_count,
        functions_with_code: retained_functions_with_code,
        functions_linked_via_code_owner,
        code_object_count: count_kind(&document.objects, "Code"),
        stub_code_count: document
            .objects
            .iter()
            .filter(|object| {
                object.object_type.as_deref() == Some("Code")
                    && object.section.as_deref() == Some("_kDartIsolateSnapshotInstructions")
                    && object.is_stub == Some(true)
            })
            .count(),
        unattributed_code_count: document
            .objects
            .iter()
            .filter(|object| {
                object.object_type.as_deref() == Some("Code")
                    && object.section.as_deref() == Some("_kDartIsolateSnapshotInstructions")
                    && object.owner.is_none()
                    && object.name.as_deref() == Some("Unknown Code")
            })
            .count(),
        type_count: count_kind(&document.objects, "Type"),
        closure_parent_links: document
            .objects
            .iter()
            .filter(|object| {
                object.object_type.as_deref() == Some("Function")
                    && object.parent_function.is_some()
            })
            .count(),
        fields_with_offsets: document
            .objects
            .iter()
            .filter(|object| {
                object.object_type.as_deref() == Some("Field")
                    && (object.instance_field_offset.is_some()
                        || object.static_field_offset.is_some())
            })
            .count(),
        class_instance_slots: document
            .objects
            .iter()
            .filter(|object| object.object_type.as_deref() == Some("Class"))
            .map(|object| object.instance_slots.len())
            .sum(),
        object_pool_references: document
            .objects
            .iter()
            .filter(|object| object.object_type.as_deref() == Some("ObjectPool"))
            .map(|object| object.references.len() / 3)
            .sum(),
        global_object_pool_length: document.metadata.global_object_pool_length,
        dispatch_table_origin_element: document.metadata.dispatch_table_origin_element,
        // Schema 5 emits one StaticCalls pseudo-object carrying the pool's
        // code targets; older schemas simply report zero.
        static_call_targets: document.static_calls.as_ref().map_or(0, |section| {
            section.entry_count.unwrap_or(section.targets.len() as u64) as usize
        }),
        class_id_ranges: document
            .class_ranges
            .as_ref()
            .map(|ranges| ranges.populated_runs.clone())
            .unwrap_or_default(),
        library_import_edges: document
            .objects
            .iter()
            .filter(|object| object.object_type.as_deref() == Some("Library"))
            .map(|object| object.imports.len())
            .sum(),
        library_reference_edges,
        enriched_object_pool_entries: object_pool_labels.len(),
        matched_functions: 0,
        matched_code_offsets: 0,
        strongly_matched_functions: 0,
        unmatched_recovered_functions: 0,
    relabeled_dispatch_candidates: 0,
    };
    Ok(LoadedVmOracle {
        evidence,
        functions,
        declarations,
        libraries: recovered_libraries,
        object_pool_labels,
        dispatch_selectors: document
            .dispatch_metadata
            .map(|section| {
                section
                    .targets
                    .into_iter()
                    .filter_map(|target| {
                        let name = target.owner_name.filter(|name| !name.is_empty())?;
                        let index = usize::try_from(target.selector_index).ok()?;
                        Some((index, name))
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub(crate) fn apply_root(program: &mut RecoveredProgram, oracle: &LoadedVmOracle) {
    let Some(uri) = oracle.evidence.root_library_uri.clone() else {
        return;
    };
    program.root_library_uri = Some(uri.clone());
    if let Some(package) = root_package(&uri) {
        program.application_package = Some(package.to_owned());
        program
            .warnings
            .retain(|warning| warning.code != "W_APP_PACKAGE_UNKNOWN");
    }

    let mut found = false;
    for library in &mut program.libraries {
        if library.uri != uri {
            continue;
        }
        found = true;
        library.is_application = true;
        library.vm_object_id = library
            .vm_object_id
            .or(oracle.evidence.root_library_object_id);
        if let Some(vm_library) = oracle
            .libraries
            .iter()
            .find(|candidate| candidate.uri == uri)
        {
            library.imports.extend(vm_library.imports.iter().cloned());
            library.imports.sort();
            library.imports.dedup();
            library
                .referenced_libraries
                .extend(vm_library.referenced_libraries.iter().cloned());
            library.referenced_libraries.sort();
            library.referenced_libraries.dedup();
        }
        if root_package(&uri).is_none() {
            library.output_path = PathBuf::from("vm_root/main.dart");
        }
    }
    if !found {
        let package = root_package(&uri).map(str::to_owned);
        let output_path = if package.is_some() {
            PathBuf::from("main.dart")
        } else {
            PathBuf::from("vm_root/main.dart")
        };
        let vm_library = oracle
            .libraries
            .iter()
            .find(|candidate| candidate.uri == uri);
        program.libraries.push(RecoveredLibrary {
            uri,
            package,
            output_path,
            is_application: true,
            vm_object_id: oracle.evidence.root_library_object_id,
            imports: vm_library.map_or_else(Vec::new, |library| library.imports.clone()),
            referenced_libraries: vm_library
                .map_or_else(Vec::new, |library| library.referenced_libraries.clone()),
        });
    }
}

pub(crate) fn apply_declarations(
    program: &mut RecoveredProgram,
    oracle: &LoadedVmOracle,
    scope: Scope,
) {
    for library in &oracle.libraries {
        let include = match scope {
            Scope::All => true,
            Scope::Packages => {
                library.uri.starts_with("package:") && !library.uri.starts_with("package:flutter/")
            }
            Scope::App => program
                .application_package
                .as_deref()
                .is_some_and(|package| library.uri.starts_with(&format!("package:{package}/"))),
        };
        if !include && Some(&library.uri) != program.root_library_uri.as_ref() {
            continue;
        }
        if let Some(existing) = program
            .libraries
            .iter_mut()
            .find(|existing| existing.uri == library.uri)
        {
            existing.vm_object_id = existing.vm_object_id.or(library.vm_object_id);
            existing.imports.extend(library.imports.iter().cloned());
            existing.imports.sort();
            existing.imports.dedup();
            existing
                .referenced_libraries
                .extend(library.referenced_libraries.iter().cloned());
            existing.referenced_libraries.sort();
            existing.referenced_libraries.dedup();
        } else {
            let mut recovered = library.clone();
            recovered.output_path = crate::analysis::library_output_path(
                &recovered.uri,
                program.application_package.as_deref(),
            );
            recovered.is_application = Some(recovered.uri.as_str())
                == program.root_library_uri.as_deref()
                || recovered.package.as_deref() == program.application_package.as_deref();
            program.libraries.push(recovered);
        }
    }
    // Keep every oracle declaration as enrichment evidence even when
    // --scope filters what is rendered: field layouts, signatures, and
    // constructor identities sharpen call sites across all libraries.
    program
        .declaration_evidence
        .extend(oracle.declarations.clone());
    crate::analysis::attach_declarations(program, oracle.declarations.clone(), scope);
    crate::analysis::reconcile_libraries(program, scope);
}

pub(crate) fn attach(
    program: &mut RecoveredProgram,
    mut oracle: LoadedVmOracle,
    snapshot: &SnapshotInfo,
    abi: Abi,
) -> Result<()> {
    let image_base = isolate_instructions_base(snapshot)?;
    let mut by_offset = HashMap::<u64, Vec<usize>>::new();
    for (index, function) in oracle.functions.iter().enumerate() {
        if let Some(offset) = function.code_offset {
            by_offset.entry(offset).or_default().push(index);
        }
    }

    let mut recovered_by_offset = BTreeMap::<u64, Vec<usize>>::new();
    for (index, function) in program.functions.iter().enumerate() {
        let Some(address) = parse_address(&function.address) else {
            continue;
        };
        let Some(offset) = address.checked_sub(image_base) else {
            continue;
        };
        if by_offset.contains_key(&offset) {
            recovered_by_offset.entry(offset).or_default().push(index);
        }
    }

    let mut matched = 0usize;
    let mut strong = 0usize;
    let mut matched_code_offsets = HashSet::new();
    let mut root_main_matched = false;
    for (offset, recovered_indices) in recovered_by_offset {
        let Some(candidate_indices) = by_offset.get(&offset) else {
            continue;
        };
        let assignments = assign_vm_candidates(
            &program.functions,
            &recovered_indices,
            &oracle.functions,
            candidate_indices,
        );
        for assignment in assignments {
            let Some(candidate) = oracle.functions.get(assignment.candidate_index) else {
                continue;
            };
            let mut evidence = candidate.clone();
            evidence.logical_match_score = Some(assignment.score);
            evidence.logical_match_candidate_count = Some(assignment.tied_candidates.len());
            evidence.alternative_parent_functions = assignment
                .tied_candidates
                .iter()
                .filter_map(|index| oracle.functions.get(*index))
                .filter_map(|candidate| candidate.parent_function_name.clone())
                .filter(|parent| Some(parent.as_str()) != evidence.parent_function_name.as_deref())
                .collect();
            evidence.alternative_parent_functions.sort();
            evidence.alternative_parent_functions.dedup();

            // A "strong" match specifically means independent static name or
            // signature evidence corroborates the VM identity. Exact code
            // identity is tracked separately and never inflates this count.
            let is_strong = assignment.score >= 7;
            matched += 1;
            matched_code_offsets.insert(offset);
            strong += usize::from(is_strong);
            if evidence.library_uri == oracle.evidence.root_library_uri
                && evidence.raw_name.as_deref() == Some("main")
            {
                root_main_matched = true;
            }
            let function = &mut program.functions[assignment.function_index];
            if is_strong || candidate_indices.len() == 1 {
                apply_function_evidence(function, &evidence);
            } else {
                // Exact code identity is still authoritative even when a
                // shared optimized body cannot safely inherit one semantic
                // name/owner.
                function.vm_evidence = Some(code_identity_only(&evidence));
            }
        }
    }

    let root_has_main = oracle.functions.iter().any(|function| {
        function.library_uri == oracle.evidence.root_library_uri
            && function.raw_name.as_deref() == Some("main")
            && function.code_offset.is_some()
    });
    if root_has_main && !root_main_matched {
        return Err(ClutterError::Analysis(
            "the Dart VM oracle root `main` code does not match the selected ABI payload; regenerate the oracle from this exact APK/AAB and ABI"
                .to_owned(),
        ));
    }
    if !program.functions.is_empty() && matched == 0 {
        return Err(ClutterError::Analysis(
            "the Dart VM oracle contains no code offsets matching the selected payload".to_owned(),
        ));
    }

    let enriched_pool_entries =
        enrich_object_pool_evidence(&mut program.functions, &oracle.object_pool_labels);
    oracle.evidence.enriched_object_pool_entries = enriched_pool_entries;
    // Oracle dispatch rows name selectors exactly; apply them after the pool
    // enrichment so proven names override any heuristic labels.
    let resolved_dispatch_selectors = if oracle.dispatch_selectors.is_empty() {
        0
    } else {
        apply_dispatch_selector_evidence(&mut program.functions, &oracle.dispatch_selectors)
    };
    // Dispatch-table candidate labels were derived from static snapshot names
    // before the oracle attached. Under obfuscation those labels are synthetic
    // (`sub_<addr>`), but the oracle knows which Function owns each code
    // offset — relabel candidates so bounded candidate lists carry real
    // identities instead of addresses.
    let relabeled_candidates = relabel_dispatch_candidates(
        &mut program.functions,
        &oracle.functions,
        image_base,
    );
    oracle.evidence.relabeled_dispatch_candidates = relabeled_candidates;
    enrich_semantics(program, abi);

    oracle.evidence.matched_functions = matched;
    oracle.evidence.matched_code_offsets = matched_code_offsets.len();
    oracle.evidence.strongly_matched_functions = strong;
    oracle.evidence.unmatched_recovered_functions = program.functions.len().saturating_sub(matched);
    let root = oracle
        .evidence
        .root_library_uri
        .as_deref()
        .unwrap_or("unknown");
    program.warnings.push(Warning {
        code: "W_VM_ORACLE_APPLIED".to_owned(),
        message: format!(
            "A matching Dart VM snapshot analyzer identified root library `{root}` and linked {matched} recovered function entries across {} exact isolate-instruction offsets ({strong} name-corroborated, {resolved_dispatch_selectors} dispatch selectors named). The VM loaded the snapshot but did not invoke `main`.",
            matched_code_offsets.len(),
        ),
    });
    program.vm_oracle = Some(oracle.evidence);
    Ok(())
}

/// Rewrites synthetic dispatch-candidate labels (`sub_<addr>`) to the oracle
/// Function identity owning that code offset. The static analysis could only
/// label table slots from snapshot symbols, which obfuscation erases; the
/// oracle's per-offset Function graph restores real identities. Returns how
/// many candidate labels were relabeled.
fn relabel_dispatch_candidates(
    functions: &mut [RecoveredFunction],
    oracle_functions: &[VmFunctionEvidence],
    image_base: u64,
) -> usize {
    let mut label_by_offset = HashMap::<u64, String>::new();
    for evidence in oracle_functions {
        let Some(offset) = evidence.code_offset else {
            continue;
        };
        let Some(name) = evidence
            .user_visible_name
            .as_deref()
            .or(Some(evidence.name.as_str()))
            .filter(|name| !name.is_empty() && !name.starts_with("sub_"))
        else {
            continue;
        };
        let owner = evidence.owner.as_deref().unwrap_or_default();
        let qualified = if matches!(owner, "" | "::" | "top_level") {
            name.to_owned()
        } else {
            format!("{owner}.{name}")
        };
        label_by_offset.insert(offset, qualified);
    }
    if label_by_offset.is_empty() {
        return 0;
    }
    let mut relabeled = 0usize;
    for function in functions {
        for statement in &mut function.statements {
            let PseudoStatement::DispatchTableCall {
                candidate_targets, ..
            } = statement
            else {
                continue;
            };
            for target in candidate_targets.iter_mut() {
                let Some(hex) = target.strip_prefix("sub_") else {
                    continue;
                };
                let Ok(address) = u64::from_str_radix(hex, 16) else {
                    continue;
                };
                let Some(offset) = address.checked_sub(image_base) else {
                    continue;
                };
                if let Some(qualified) = label_by_offset.get(&offset) {
                    *target = qualified.clone();
                    relabeled += 1;
                }
            }
        }
    }
    relabeled
}

fn enrich_object_pool_evidence(
    functions: &mut [RecoveredFunction],
    labels: &BTreeMap<usize, String>,
) -> usize {
    let mut enriched_indices = HashSet::new();
    for function in functions {
        for instruction in &mut function.instructions {
            let Some(index) = instruction.object_pool_index else {
                continue;
            };
            let Some(label) = labels.get(&index) else {
                continue;
            };
            let replace = instruction
                .object_pool_value
                .as_deref()
                .is_none_or(|existing| {
                    existing.starts_with("snapshotRef(")
                        || existing.starts_with("nativePoolEntry(")
                        || existing.starts_with("resetPoolEntry(")
                });
            if replace {
                instruction.object_pool_value = Some(label.clone());
                enriched_indices.insert(index);
            }
        }
    }
    enriched_indices.len()
}

fn enrich_semantics(program: &mut RecoveredProgram, abi: Abi) {
    // The shared pass rebuilds call symbols and field layouts from every
    // surviving declaration (oracle-enriched declarations included) and
    // re-lifts all functions with full semantic evidence.
    crate::analysis::enrich_semantics(program, abi, &[], &[]);
}

fn recover_library_references(
    functions: &[VmFunctionEvidence],
    declarations: &[RecoveredDeclaration],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut references = BTreeMap::<String, BTreeSet<String>>::new();
    let mut add = |source: Option<&str>, target: Option<&str>| {
        let (Some(source), Some(target)) = (source, target) else {
            return;
        };
        if source != target {
            references
                .entry(source.to_owned())
                .or_default()
                .insert(target.to_owned());
        }
    };

    for function in functions {
        let source = function.library_uri.as_deref();
        add(
            source,
            function
                .result_type
                .as_ref()
                .and_then(|value| value.library_uri.as_deref()),
        );
        for parameter in &function.parameters {
            add(
                source,
                parameter
                    .declared_type
                    .as_ref()
                    .and_then(|value| value.library_uri.as_deref()),
            );
        }
        for parameter in &function.type_parameters {
            add(
                source,
                parameter
                    .bound
                    .as_ref()
                    .and_then(|value| value.library_uri.as_deref()),
            );
            add(
                source,
                parameter
                    .default_type
                    .as_ref()
                    .and_then(|value| value.library_uri.as_deref()),
            );
        }
    }
    for declaration in declarations {
        let source = declaration.library_uri.as_deref();
        if let Some(metadata) = declaration.class_metadata.as_ref() {
            add(
                source,
                metadata
                    .super_type
                    .as_ref()
                    .and_then(|value| value.library_uri.as_deref()),
            );
            for interface in &metadata.interfaces {
                add(source, interface.library_uri.as_deref());
            }
            for parameter in &metadata.type_parameters {
                add(
                    source,
                    parameter
                        .bound
                        .as_ref()
                        .and_then(|value| value.library_uri.as_deref()),
                );
            }
        }
        if let Some(metadata) = declaration.field_metadata.as_ref() {
            add(
                source,
                metadata
                    .declared_type
                    .as_ref()
                    .and_then(|value| value.library_uri.as_deref()),
            );
        }
        if let Some(resolved) = declaration
            .signature
            .as_ref()
            .and_then(|signature| signature.resolved.as_ref())
        {
            add(
                source,
                resolved
                    .return_type
                    .as_ref()
                    .and_then(|value| value.library_uri.as_deref()),
            );
            for parameter in &resolved.parameters {
                add(
                    source,
                    parameter
                        .declared_type
                        .as_ref()
                        .and_then(|value| value.library_uri.as_deref()),
                );
            }
            for parameter in &resolved.type_parameters {
                add(
                    source,
                    parameter
                        .bound
                        .as_ref()
                        .and_then(|value| value.library_uri.as_deref()),
                );
            }
        }
    }
    references
}

fn apply_function_evidence(function: &mut RecoveredFunction, evidence: &VmFunctionEvidence) {
    if function.name_source == RecoveredNameSource::Synthetic {
        let recovered = if evidence.kind.as_deref() == Some("DroppedFunctionCode") {
            Some("unknownFunction")
        } else {
            evidence
                .raw_name
                .as_deref()
                .filter(|name| !name.is_empty())
                .or_else(|| (!evidence.name.is_empty()).then_some(evidence.name.as_str()))
        };
        if let Some(name) = recovered {
            function.name = name.to_owned();
            function.name_source = RecoveredNameSource::DartVmOracle;
        }
    }
    if function.owner.is_none() {
        function.owner = evidence.owner.clone();
    }
    if function.library_uri.is_none() {
        function.library_uri = evidence.library_uri.clone();
    }
    if function.kind.is_none() {
        function.kind = evidence.kind.as_deref().and_then(vm_function_kind);
    }
    if function.signature.is_none() {
        function.signature = recovered_signature_from_vm(evidence);
    }
    if function.signature_source.is_none() && function.signature.is_some() {
        function.signature_source = Some(RecoveredSignatureSource::DartVmOracle);
    }
    if let Some(signature) = &mut function.signature {
        function.parameter_count = Some(
            signature
                .fixed_parameter_count
                .saturating_add(signature.optional_parameter_count),
        );
        merge_vm_signature_details(signature, evidence);
    }
    function.vm_evidence = Some(evidence.clone());
}

fn recovered_signature_from_vm(evidence: &VmFunctionEvidence) -> Option<RecoveredSignature> {
    let fixed = evidence.fixed_parameter_count?;
    let implicit = evidence.implicit_parameter_count.unwrap_or_default();
    let visible_fixed = fixed.saturating_sub(implicit);
    let optional = evidence.optional_parameter_count.unwrap_or_default();
    Some(RecoveredSignature {
        fixed_parameter_count: visible_fixed,
        optional_parameter_count: optional,
        optional_parameters_are_named: evidence.optional_parameters_are_named.unwrap_or(false),
        implicit_parameter_count: implicit,
        type_parameters_reference: None,
        result_type_reference: None,
        parameter_types_reference: None,
        named_parameter_names_reference: None,
        flags: 0,
        packed_type_parameter_counts: 0,
        resolved: vm_signature_details(evidence),
    })
}

fn code_identity_only(evidence: &VmFunctionEvidence) -> VmFunctionEvidence {
    VmFunctionEvidence {
        object_id: evidence.object_id,
        code_object_id: evidence.code_object_id,
        analyzer_code_object_id: evidence.analyzer_code_object_id,
        current_code_object_id: evidence.current_code_object_id,
        code_link_source: Some("shared_code_ambiguous".to_owned()),
        code_offset: evidence.code_offset,
        code_size: evidence.code_size,
        logical_match_score: evidence.logical_match_score,
        logical_match_candidate_count: evidence.logical_match_candidate_count,
        alternative_parent_functions: evidence.alternative_parent_functions.clone(),
        name: String::new(),
        raw_name: None,
        user_visible_name: None,
        owner: None,
        library_uri: None,
        parent_function_object_id: evidence.parent_function_object_id,
        parent_function_name: evidence.parent_function_name.clone(),
        parent_function_owner: evidence.parent_function_owner.clone(),
        signature: None,
        user_visible_signature: None,
        result_type: None,
        parameters: Vec::new(),
        type_parameters: Vec::new(),
        owner_type_parameters: Vec::new(),
        kind: Some("SharedAotCodeBoundary".to_owned()),
        fixed_parameter_count: None,
        optional_parameter_count: None,
        implicit_parameter_count: None,
        optional_parameters_are_named: None,
        is_static: None,
        is_async: None,
        is_sync_generator: None,
        is_async_generator: None,
    }
}

fn vm_signature_details(evidence: &VmFunctionEvidence) -> Option<RecoveredSignatureDetails> {
    let function_type_parameters = evidence
        .type_parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .collect::<Vec<_>>();
    let parameters = evidence
        .parameters
        .iter()
        .filter(|parameter| !parameter.is_implicit)
        .enumerate()
        .map(|(position, parameter)| RecoveredParameter {
            position,
            name: parameter.name.clone(),
            declared_type: parameter.declared_type.as_ref().and_then(|value| {
                recovered_vm_type(
                    value,
                    &evidence.owner_type_parameters,
                    &function_type_parameters,
                )
            }),
            is_named: parameter.is_named,
            is_required: parameter.is_required,
        })
        .collect::<Vec<_>>();
    let type_parameters = evidence
        .type_parameters
        .iter()
        .map(|parameter| RecoveredTypeParameter {
            name: parameter.name.clone(),
            bound: parameter.bound.as_ref().and_then(|value| {
                recovered_vm_type(
                    value,
                    &evidence.owner_type_parameters,
                    &function_type_parameters,
                )
            }),
        })
        .collect::<Vec<_>>();
    let return_type = evidence.result_type.as_ref().and_then(|value| {
        recovered_vm_type(
            value,
            &evidence.owner_type_parameters,
            &function_type_parameters,
        )
    });
    (!parameters.is_empty() || !type_parameters.is_empty() || return_type.is_some()).then_some(
        RecoveredSignatureDetails {
            return_type,
            parameters,
            type_parameters,
        },
    )
}

fn merge_vm_signature_details(signature: &mut RecoveredSignature, evidence: &VmFunctionEvidence) {
    let Some(vm) = vm_signature_details(evidence) else {
        return;
    };
    let Some(existing) = &mut signature.resolved else {
        signature.resolved = Some(vm);
        return;
    };
    if existing.return_type.is_none() {
        existing.return_type = vm.return_type;
    }
    if existing.type_parameters.is_empty() {
        existing.type_parameters = vm.type_parameters;
    }
    if existing.parameters.is_empty() {
        existing.parameters = vm.parameters;
        return;
    }
    for (parameter, vm_parameter) in existing.parameters.iter_mut().zip(vm.parameters) {
        if parameter.name.is_none() {
            parameter.name = vm_parameter.name;
        }
        if parameter.declared_type.is_none() {
            parameter.declared_type = vm_parameter.declared_type;
        }
        parameter.is_named |= vm_parameter.is_named;
        parameter.is_required |= vm_parameter.is_required;
    }
}

fn recovered_vm_type(
    evidence: &VmTypeEvidence,
    owner_type_parameters: &[String],
    function_type_parameters: &[String],
) -> Option<RecoveredType> {
    let snapshot_reference = i32::try_from(evidence.object_id).ok()?;
    let mut display_name = evidence.display_name.clone();
    contextualize_vm_type(&mut display_name, "X", owner_type_parameters);
    contextualize_vm_type(&mut display_name, "Y", function_type_parameters);
    (!evidence.display_name.is_empty()).then(|| RecoveredType {
        snapshot_reference,
        display_name,
        library_uri: evidence.library_uri.clone(),
    })
}

fn contextualize_vm_type(value: &mut String, prefix: &str, names: &[String]) {
    for (index, name) in names.iter().enumerate() {
        *value = replace_type_parameter_token(value, &format!("{prefix}{index}"), name);
    }
}

fn replace_type_parameter_token(value: &str, needle: &str, name: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(position) = remaining.find(needle) {
        let before = remaining[..position].chars().next_back();
        let after = remaining[position + needle.len()..].chars().next();
        let boundary = |character: Option<char>| {
            character.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
        };
        output.push_str(&remaining[..position]);
        if boundary(before) && boundary(after) {
            output.push_str(name);
        } else {
            output.push_str(needle);
        }
        remaining = &remaining[position + needle.len()..];
    }
    output.push_str(remaining);
    output
}

fn validate_metadata(metadata: &AnalyzerMetadata, snapshot: &SnapshotInfo, abi: Abi) -> Result<()> {
    if metadata.analyzer_version < 2 {
        return Err(ClutterError::Unsupported(format!(
            "Dart snapshot analyzer schema {} is unsupported; version 2 or newer is required",
            metadata.analyzer_version
        )));
    }
    if metadata.snapshot_hash != snapshot.isolate_header.snapshot_hash {
        return Err(ClutterError::Analysis(format!(
            "Dart VM oracle snapshot hash {} does not match selected payload {}",
            metadata.snapshot_hash, snapshot.isolate_header.snapshot_hash
        )));
    }
    if metadata.analyzer_version >= 4 {
        let commit = metadata.dart_commit.as_deref().ok_or_else(|| {
            ClutterError::Analysis("Dart VM oracle schema 4 is missing its Dart commit".to_owned())
        })?;
        if commit.len() < 10 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ClutterError::Analysis(format!(
                "Dart VM oracle Dart commit {commit:?} is not a hexadecimal revision"
            )));
        }
    }
    let expected_word_size = match abi {
        Abi::ArmeabiV7a => 4,
        Abi::Arm64V8a | Abi::X86_64 => 8,
    };
    if metadata.word_size != expected_word_size {
        return Err(ClutterError::Analysis(format!(
            "Dart VM oracle word size {} does not match ABI {abi}",
            metadata.word_size
        )));
    }
    if let Some(version) = metadata.dart_version.as_deref()
        && let Some(actual) = target_arch(version)
    {
        let expected = match abi {
            Abi::ArmeabiV7a => "arm",
            Abi::Arm64V8a => "arm64",
            Abi::X86_64 => "x64",
        };
        if actual != expected {
            return Err(ClutterError::Analysis(format!(
                "Dart VM oracle target architecture {actual} does not match ABI {abi}"
            )));
        }
    }
    let compressed = snapshot
        .isolate_header
        .features
        .iter()
        .any(|feature| feature == "compressed-pointers");
    let expected_compressed_word_size = if compressed { 4 } else { expected_word_size };
    if metadata.compressed_word_size != expected_compressed_word_size {
        return Err(ClutterError::Analysis(format!(
            "Dart VM oracle compressed word size {} does not match selected snapshot feature set",
            metadata.compressed_word_size
        )));
    }
    Ok(())
}

struct VmCandidateAssignment {
    function_index: usize,
    candidate_index: usize,
    score: usize,
    tied_candidates: Vec<usize>,
}

fn assign_vm_candidates(
    functions: &[RecoveredFunction],
    function_indices: &[usize],
    candidates: &[VmFunctionEvidence],
    candidate_indices: &[usize],
) -> Vec<VmCandidateAssignment> {
    let mut ranked_functions = function_indices
        .iter()
        .filter_map(|function_index| {
            let function = functions.get(*function_index)?;
            let scores = candidate_indices
                .iter()
                .filter_map(|candidate_index| {
                    let candidate = candidates.get(*candidate_index)?;
                    Some((*candidate_index, match_score(function, candidate)))
                })
                .collect::<Vec<_>>();
            let best_score = scores.iter().map(|(_, score)| *score).max()?;
            let tied_count = scores
                .iter()
                .filter(|(_, score)| *score == best_score)
                .count();
            Some((*function_index, scores, best_score, tied_count))
        })
        .collect::<Vec<_>>();
    // Assign the most discriminating recovered aliases first. This prevents a
    // generic anonymous closure from consuming the only VM candidate whose
    // parameter types corroborate a later, more specific alias.
    ranked_functions.sort_by(|left, right| {
        left.3
            .cmp(&right.3)
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut used = HashSet::new();
    let mut assignments = Vec::with_capacity(ranked_functions.len());
    for (function_index, mut scores, best_score, _) in ranked_functions {
        scores.sort_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| {
                let left_id = candidates
                    .get(left.0)
                    .map_or(u64::MAX, |candidate| candidate.object_id);
                let right_id = candidates
                    .get(right.0)
                    .map_or(u64::MAX, |candidate| candidate.object_id);
                left_id.cmp(&right_id)
            })
        });
        let candidate_index = scores
            .iter()
            .find(|(candidate_index, _)| !used.contains(candidate_index))
            .or_else(|| scores.first())
            .map(|(candidate_index, _)| *candidate_index);
        let Some(candidate_index) = candidate_index else {
            continue;
        };
        used.insert(candidate_index);
        let score = scores
            .iter()
            .find(|(index, _)| *index == candidate_index)
            .map_or(best_score, |(_, score)| *score);
        let tied_candidates = scores
            .iter()
            .filter_map(|(candidate_index, candidate_score)| {
                (*candidate_score == best_score).then_some(*candidate_index)
            })
            .collect();
        assignments.push(VmCandidateAssignment {
            function_index,
            candidate_index,
            score,
            tied_candidates,
        });
    }
    assignments
}

fn match_score(function: &RecoveredFunction, candidate: &VmFunctionEvidence) -> usize {
    let mut score = 0usize;
    let static_names = [
        Some(function.name.as_str()),
        function.snapshot_name.as_deref(),
        function.obfuscated_name.as_deref(),
    ];
    if candidate
        .raw_name
        .as_deref()
        .is_some_and(|name| static_names.contains(&Some(name)))
    {
        score += 8;
    } else if static_names
        .into_iter()
        .flatten()
        .any(|name| candidate.name == name || candidate.name.ends_with(&format!(".{name}")))
    {
        score += 7;
    }
    if candidate.library_uri.is_some() && candidate.library_uri == function.library_uri {
        score += 2;
    }
    if candidate.owner.is_some() && candidate.owner == function.owner {
        score += 1;
    }
    if candidate
        .kind
        .as_deref()
        .and_then(vm_function_kind)
        .is_some()
        && candidate.kind.as_deref().and_then(vm_function_kind) == function.kind
    {
        score += 1;
    }
    let recovered_parameters = function
        .signature
        .as_ref()
        .and_then(|signature| signature.resolved.as_ref())
        .map(|resolved| {
            resolved
                .parameters
                .iter()
                .map(|parameter| {
                    parameter
                        .declared_type
                        .as_ref()
                        .map(|value| canonical_type_name(&value.display_name))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let candidate_parameters = candidate
        .parameters
        .iter()
        .filter(|parameter| !parameter.is_implicit)
        .map(|parameter| {
            parameter
                .declared_type
                .as_ref()
                .map(|value| canonical_type_name(&value.display_name))
        })
        .collect::<Vec<_>>();
    if !recovered_parameters.is_empty() && recovered_parameters.len() == candidate_parameters.len()
    {
        score += 1;
        if recovered_parameters == candidate_parameters {
            score += 5;
        }
    }
    let recovered_return = function
        .signature
        .as_ref()
        .and_then(|signature| signature.resolved.as_ref())
        .and_then(|resolved| resolved.return_type.as_ref())
        .map(|value| canonical_type_name(&value.display_name));
    let candidate_return = candidate
        .result_type
        .as_ref()
        .map(|value| canonical_type_name(&value.display_name));
    if recovered_return.is_some() && recovered_return == candidate_return {
        score += 2;
    }
    score
}

fn canonical_type_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .trim_end_matches('?')
        .to_owned()
}

fn vm_function_kind(value: &str) -> Option<RecoveredFunctionKind> {
    Some(match value {
        "RegularFunction" => RecoveredFunctionKind::Regular,
        "ClosureFunction" => RecoveredFunctionKind::Closure,
        "ImplicitClosureFunction" => RecoveredFunctionKind::ImplicitClosure,
        "GetterFunction" => RecoveredFunctionKind::Getter,
        "SetterFunction" => RecoveredFunctionKind::Setter,
        "Constructor" => RecoveredFunctionKind::Constructor,
        "ImplicitGetter" => RecoveredFunctionKind::ImplicitGetter,
        "ImplicitSetter" => RecoveredFunctionKind::ImplicitSetter,
        "ImplicitStaticGetter" => RecoveredFunctionKind::ImplicitStaticGetter,
        "FieldInitializer" => RecoveredFunctionKind::FieldInitializer,
        "MethodExtractor" => RecoveredFunctionKind::MethodExtractor,
        "NoSuchMethodDispatcher" => RecoveredFunctionKind::NoSuchMethodDispatcher,
        "InvokeFieldDispatcher" => RecoveredFunctionKind::InvokeFieldDispatcher,
        "IrregexpFunction" => RecoveredFunctionKind::Irregexp,
        "DynamicInvocationForwarder" => RecoveredFunctionKind::DynamicInvocationForwarder,
        "FfiTrampoline" => RecoveredFunctionKind::FfiTrampoline,
        "RecordFieldGetter" => RecoveredFunctionKind::RecordFieldGetter,
        _ => return None,
    })
}

fn isolate_instructions_base(snapshot: &SnapshotInfo) -> Result<u64> {
    let region = snapshot
        .regions
        .iter()
        .find(|region| region.name == "_kDartIsolateSnapshotInstructions")
        .ok_or_else(|| {
            ClutterError::Analysis(
                "selected snapshot has no isolate instructions region".to_owned(),
            )
        })?;
    parse_address(&region.virtual_address).ok_or_else(|| {
        ClutterError::InvalidArtifact(format!(
            "invalid isolate instructions address {}",
            region.virtual_address
        ))
    })
}

fn parse_address(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

fn vm_declaration_reference(object_id: u64) -> i32 {
    i32::try_from(object_id)
        .ok()
        .and_then(|id| i32::MIN.checked_add(id))
        .unwrap_or(i32::MIN)
}

fn root_package(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix("package:")?;
    let (package, path) = rest.split_once('/')?;
    (path == "main.dart").then_some(package)
}

fn target_arch(version: &str) -> Option<String> {
    [
        ("android_arm64", "arm64"),
        ("android_x64", "x64"),
        ("android_arm", "arm"),
        ("linux_arm64", "arm64"),
        ("linux_x64", "x64"),
        ("linux_arm", "arm"),
    ]
    .into_iter()
    .find_map(|(needle, arch)| version.contains(needle).then(|| arch.to_owned()))
}

fn count_kind(objects: &[AnalyzerObject], kind: &str) -> usize {
    objects
        .iter()
        .filter(|object| object.object_type.as_deref() == Some(kind))
        .count()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        assign_vm_candidates, recover_library_references, replace_type_parameter_token,
        root_package, target_arch, vm_declaration_reference, vm_function_kind,
    };
    use crate::analysis::promote_recovered_indirect_calls;
    use crate::model::{
        CallTargetScope, EvidenceConfidence, MachineCodeEvidence, PseudoStatement,
        RecoveredFunction, RecoveredFunctionKind, RecoveredNameSource, RecoveredParameter,
        RecoveredSignature, RecoveredSignatureDetails, RecoveredSignatureSource, RecoveredType,
        SemanticStatement, VmFunctionEvidence, VmParameterEvidence, VmTypeEvidence,
    };

    fn recovered_closure(parameter_type: &str) -> RecoveredFunction {
        RecoveredFunction {
            code_reference: 1,
            code_alias_references: Vec::new(),
            name: "<anonymous closure>".to_owned(),
            name_source: RecoveredNameSource::Snapshot,
            snapshot_name: None,
            obfuscated_name: None,
            owner: Some("RequestService".to_owned()),
            library_uri: Some("package:app/request.dart".to_owned()),
            source_location: None,
            inlined_functions: Vec::new(),
            inline_regions: Vec::new(),
            kind: Some(RecoveredFunctionKind::Closure),
            is_static: None,
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
            signature_source: Some(RecoveredSignatureSource::SnapshotFunction),
            parameter_count: Some(1),
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

    fn vm_closure(object_id: u64, parameter_type: &str) -> VmFunctionEvidence {
        VmFunctionEvidence {
            object_id,
            name: "RequestService.parent.<anonymous closure>".to_owned(),
            raw_name: Some("<anonymous closure>".to_owned()),
            owner: Some("RequestService".to_owned()),
            library_uri: Some("package:app/request.dart".to_owned()),
            kind: Some("ClosureFunction".to_owned()),
            result_type: Some(VmTypeEvidence {
                object_id: 1,
                type_kind: Some("Type".to_owned()),
                display_name: "bool".to_owned(),
                library_uri: Some("dart:core".to_owned()),
            }),
            parameters: vec![
                VmParameterEvidence {
                    position: 0,
                    name: None,
                    declared_type: None,
                    is_implicit: true,
                    is_named: false,
                    is_required: false,
                },
                VmParameterEvidence {
                    position: 1,
                    name: None,
                    declared_type: Some(VmTypeEvidence {
                        object_id: object_id + 100,
                        type_kind: Some("Type".to_owned()),
                        display_name: parameter_type.to_owned(),
                        library_uri: Some("package:app/model.dart".to_owned()),
                    }),
                    is_implicit: false,
                    is_named: false,
                    is_required: false,
                },
            ],
            ..VmFunctionEvidence::default()
        }
    }

    #[test]
    fn recognizes_vm_function_kinds() {
        assert_eq!(
            vm_function_kind("ImplicitClosureFunction"),
            Some(RecoveredFunctionKind::ImplicitClosure)
        );
        assert_eq!(vm_function_kind("future-kind"), None);
    }

    #[test]
    fn parses_schema5_top_level_static_call_dispatch_and_class_rows() {
        let json = r#"{
            \"objects\": [{\"id\": 7, \"type\": \"Library\", \"url\": \"dart:core\"}],
            \"metadata\": {\"analyzer_version\": 5, \"snapshot_hash\": \"abc\"},
            \"static_calls\": {
                \"type\": \"StaticCalls\",
                \"targets\": [
                    {\"pool_index\": 12, \"target_offset\": 4919, \"size\": 208,
                     \"owner_id\": 91, \"owner_name\": \"+\",
                     \"owner_is_static\": false, \"owner_parameter_count\": 2},
                    {\"pool_index\": 15, \"target_offset\": 8192}
                ],
                \"entry_count\": 2
            },
            \"dispatch_metadata\": {
                \"type\": \"Dispatch\",
                \"code_entry_count\": 4096,
                \"targets\": [
                    {\"selector_index\": 42, \"target_offset\": 20480,
                     \"size\": 144, \"owner_id\": 55, \"owner_name\": \"get:isEmpty\"}
                ]
            },
            \"class_ranges\": {
                \"type\": \"ClassRanges\",
                \"num_cids\": 1200,
                \"num_top_level_cids\": 24,
                \"populated_runs\": [[1, 900], [902, 1199]]
            }
        }"#;
        let json = json.replace("\\\"", "\"");
        let envelope: super::AnalyzerEnvelope = serde_json::from_str(&json).expect("valid json");
        let static_calls = envelope.static_calls.expect("schema 5 static_calls");
        assert_eq!(static_calls.targets.len(), 2);
        let first = &static_calls.targets[0];
        assert_eq!(first.pool_index, 12);
        assert_eq!(first.target_offset, 4919);
        assert_eq!(first.owner_id, Some(91));
        assert_eq!(first.owner_name.as_deref(), Some("+"));
        assert_eq!(first.owner_is_static, Some(false));
        assert_eq!(first.owner_parameter_count, Some(2));

        let dispatch = envelope
            .dispatch_metadata
            .expect("schema 5 dispatch_metadata");
        assert_eq!(dispatch.code_entry_count, Some(4096));
        assert_eq!(dispatch.targets.len(), 1);
        assert_eq!(dispatch.targets[0].selector_index, 42);
        assert_eq!(dispatch.targets[0].owner_name.as_deref(), Some("get:isEmpty"));

        let ranges = envelope.class_ranges.expect("schema 5 class_ranges");
        assert_eq!(ranges.populated_runs, vec![(1, 900), (902, 1199)]);
    }

    #[test]
    fn labels_unnamed_pool_entries_from_schema5_static_call_rows() {
        let rows = vec![
            super::AnalyzerStaticCall {
                pool_index: 12,
                target_offset: 4919,
                size: None,
                owner_id: Some(91),
                owner_name: Some("+".to_owned()),
                owner_is_static: Some(false),
                owner_parameter_count: Some(2),
            },
            super::AnalyzerStaticCall {
                pool_index: 15,
                target_offset: 8192,
                size: None,
                owner_id: None,
                owner_name: None,
                owner_is_static: None,
                owner_parameter_count: None,
            },
        ];
        let mut labels = BTreeMap::new();
        labels.insert(9usize, "existingLabel".to_owned());
        super::overlay_static_call_labels(&mut labels, &rows);
        assert_eq!(labels.get(&12).map(String::as_str), Some("+"));
        // Rows without a resolvable owner never fabricate labels.
        assert!(!labels.contains_key(&15));
        // Existing labels win over the coarse static-call identity.
        assert_eq!(labels.get(&9).map(String::as_str), Some("existingLabel"));
    }

    #[test]
    fn applies_oracle_selector_names_to_dispatch_table_calls() {
        let mut function = recovered_closure("int");
        function.machine_code.dispatch_table_calls = 2;
        function.statements = vec![
            PseudoStatement::DispatchTableCall {
                address: "0x1008".to_owned(),
                expression: "dispatch[42 + class_id]".to_owned(),
                selector_offset: 42,
                selector_name: None,
                candidate_targets: Vec::new(),
                candidate_count: 0,
                raw_slot_target_count: 0,
            },
            PseudoStatement::DispatchTableCall {
                address: "0x1010".to_owned(),
                expression: "dispatch[43 + class_id]".to_owned(),
                selector_offset: 43,
                selector_name: Some("sub_1965".to_owned()),
                candidate_targets: vec!["sub_1965".to_owned()],
                candidate_count: 1,
                raw_slot_target_count: 1,
            },
            PseudoStatement::Comment {
                text: "unrelated".to_owned(),
            },
        ];
        let mut selectors = BTreeMap::new();
        selectors.insert(42usize, "get:isEmpty".to_owned());
        selectors.insert(43usize, "+".to_owned());

        let mut functions = vec![function];
        let resolved =
            super::apply_dispatch_selector_evidence(&mut functions, &selectors);
        assert_eq!(resolved, 2);
        let statements = &functions[0].statements;
        let PseudoStatement::DispatchTableCall { selector_name, .. } = &statements[0] else {
            panic!("expected dispatch call");
        };
        assert_eq!(selector_name.as_deref(), Some("get:isEmpty"));
        // An oracle-proven name overrides a heuristic synthetic one.
        let PseudoStatement::DispatchTableCall {
            selector_name,
            candidate_targets,
            ..
        } = &statements[1]
        else {
            panic!("expected dispatch call");
        };
        assert_eq!(selector_name.as_deref(), Some("+"));
        assert!(candidate_targets.iter().all(|target| target != "sub_1965"));
    }

    #[test]
    fn recognizes_root_packages_and_runtime_architectures() {
        assert_eq!(root_package("package:example/main.dart"), Some("example"));
        assert_eq!(root_package("package:example/feature.dart"), None);
        assert_eq!(
            target_arch("3.11.4 on \"android_arm64\"").as_deref(),
            Some("arm64")
        );
    }

    #[test]
    fn contextualizes_only_complete_vm_type_parameter_tokens() {
        assert_eq!(
            replace_type_parameter_token("Map<X0, Future<X01> Function(X0)>", "X0", "Value"),
            "Map<Value, Future<X01> Function(Value)>"
        );
    }

    #[test]
    fn vm_declaration_references_do_not_collide_with_snapshot_references() {
        assert!(vm_declaration_reference(1).is_negative());
        assert_ne!(vm_declaration_reference(1), vm_declaration_reference(2));
    }

    #[test]
    fn shared_code_assignment_uses_parameter_types_to_preserve_logical_aliases() {
        let functions = vec![recovered_closure("Header"), recovered_closure("Cookie")];
        let candidates = vec![vm_closure(10, "Cookie"), vm_closure(11, "Header")];
        let assignments = assign_vm_candidates(&functions, &[0, 1], &candidates, &[0, 1]);
        let assigned_ids = assignments
            .iter()
            .map(|assignment| {
                (
                    assignment.function_index,
                    candidates[assignment.candidate_index].object_id,
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_eq!(assigned_ids.get(&0), Some(&11));
        assert_eq!(assigned_ids.get(&1), Some(&10));
    }

    #[test]
    fn derives_library_dependencies_from_vm_type_edges() {
        let references = recover_library_references(&[vm_closure(10, "Cookie")], &[]);
        assert!(references["package:app/request.dart"].contains("package:app/model.dart"));
        assert!(references["package:app/request.dart"].contains("dart:core"));
    }

    #[test]
    fn promotes_vm_named_register_calls_without_inventing_a_pool_index() {
        let mut function = recovered_closure("Cookie");
        function.statements = vec![PseudoStatement::IndirectCall {
            address: "0x1000".to_owned(),
            expression: "blr x16".to_owned(),
        }];
        function.semantic_statements = vec![SemanticStatement::ResolvedCall {
            target: "CookieStore.load".to_owned(),
            arguments: Vec::new(),
            confidence: EvidenceConfidence::Medium,
            address: "0x1000".to_owned(),
        }];
        let libraries = BTreeMap::from([(
            "CookieStore.load".to_owned(),
            Some("package:app/store.dart".to_owned()),
        )]);

        promote_recovered_indirect_calls(&mut function, &libraries, Some("app"));

        assert!(matches!(
            &function.statements[0],
            PseudoStatement::RecoveredIndirectCall {
                target,
                target_scope: CallTargetScope::Application,
                ..
            } if target == "CookieStore.load"
        ));
    }
}
