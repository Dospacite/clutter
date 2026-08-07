use std::collections::BTreeMap;
use std::ops::Range;

pub fn decode_one_byte_string(raw: &[u8]) -> String {
    // Dart's OneByteString stores Latin-1 code units, not UTF-8 bytes.
    raw.iter().copied().map(char::from).collect()
}

#[derive(Clone, Debug)]
pub struct ClusterHeader {
    pub num_base_objects: i64,
    pub num_objects: i64,
    pub num_clusters: i64,
    pub instruction_table_length: i64,
    pub instruction_table_data_offset: i64,
}

#[derive(Clone, Debug)]
pub struct Cluster {
    pub cid: i32,
    pub canonical: bool,
    pub count: usize,
    pub start_ref: i32,
    pub next_field_words: i32,
    pub main_count: usize,
    pub lengths: Vec<usize>,
    pub predefined_cids: Vec<i32>,
    pub discarded: Vec<bool>,
    /// Scalar payloads serialized during allocation rather than object fill.
    ///
    /// Dart serializes Smi/Mint values in `WriteAlloc`, so dropping these
    /// values loses integer constants and metadata flags referenced by the
    /// rest of the snapshot graph.
    pub allocation_values: Vec<i64>,
}

impl Cluster {
    pub fn new(cid: i32, canonical: bool, start_ref: i32) -> Self {
        Self {
            cid,
            canonical,
            count: 0,
            start_ref,
            next_field_words: 0,
            main_count: 0,
            lengths: Vec::new(),
            predefined_cids: Vec::new(),
            discarded: Vec::new(),
            allocation_values: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NamedObject {
    pub cid: i32,
    pub name_ref: i32,
    pub owner_ref: i32,
    pub signature_ref: i32,
    pub function_kind_tag: Option<u32>,
    pub instruction_index: Option<usize>,
    pub source_uri_ref: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct FunctionType {
    pub fixed: usize,
    pub optional: usize,
    pub optional_are_named: bool,
    pub implicit: usize,
    pub type_parameters_ref: Option<i32>,
    pub result_type_ref: Option<i32>,
    pub parameter_types_ref: Option<i32>,
    pub named_parameter_names_ref: Option<i32>,
    pub flags: u8,
    pub packed_type_parameter_counts: u32,
}

#[derive(Clone, Debug)]
pub struct Code {
    pub ref_id: i32,
    pub owner_ref: i32,
    pub instruction_index: Option<usize>,
    pub payload_info: Option<i64>,
    pub unchecked_entry_offset: Option<u64>,
    pub has_monomorphic_entrypoint: bool,
    pub exception_handlers_ref: Option<i32>,
    pub pc_descriptors_ref: Option<i32>,
    pub catch_entry_ref: Option<i32>,
    pub inlined_functions_ref: Option<i32>,
    pub code_source_map_ref: Option<i32>,
}

#[derive(Clone, Debug)]
pub enum PoolValue {
    Reference(i32),
    Immediate(i64),
    Native,
    Empty,
}

#[derive(Clone, Debug)]
pub struct ObjectPool {
    pub reference: i32,
    pub entries: Vec<PoolValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotObjectKind {
    Standard,
    String,
    Integer,
    Double,
    Code,
    ObjectPool,
    Array,
    WeakArray,
    TypedData,
    ExceptionHandlers,
    Context,
    TypeArguments,
    MetadataBytes,
    Instance,
    Record,
    ContextScope,
    Class,
}

#[derive(Clone, Debug)]
pub enum SnapshotScalar {
    Unsigned(i64),
    Tagged32(u32),
    Tagged64(i64),
    Uint16(u16),
    Int16(i16),
    Byte(u8),
    Reference(i32),
}

#[derive(Clone, Debug)]
pub struct SnapshotObject {
    pub reference: i32,
    pub cid: i32,
    pub canonical: bool,
    pub kind: SnapshotObjectKind,
    pub reference_range: Range<usize>,
    pub scalar_range: Range<usize>,
    pub byte_range: Range<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct SnapshotObjectPayload {
    pub references: Vec<i32>,
    pub scalars: Vec<SnapshotScalar>,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ExceptionHandler {
    pub handler_pc_offset: u32,
    pub outer_try_index: i16,
    pub needs_stack_trace: bool,
    pub has_catch_all: bool,
    pub is_generated: bool,
}

#[derive(Clone, Debug)]
pub struct ExceptionHandlers {
    pub reference: i32,
    pub has_async_handler: bool,
    pub handled_types_ref: i32,
    pub entries: Vec<ExceptionHandler>,
}

#[derive(Clone, Debug)]
pub struct ParseResult {
    pub header: ClusterHeader,
    pub clusters: Vec<Cluster>,
    pub fill_start: usize,
    pub strings: BTreeMap<i32, String>,
    pub named: BTreeMap<i32, NamedObject>,
    pub library_uris: BTreeMap<i32, i32>,
    pub function_types: BTreeMap<i32, FunctionType>,
    pub codes: Vec<Code>,
    /// Flattened for backwards compatibility. `object_pools` preserves pool boundaries.
    pub pool: Vec<PoolValue>,
    pub object_pools: Vec<ObjectPool>,
    /// Lossless semantic graph of all cluster objects parsed by this profile.
    pub objects: Vec<SnapshotObject>,
    pub object_references: Vec<i32>,
    pub object_scalars: Vec<SnapshotScalar>,
    pub object_bytes: Vec<u8>,
    /// Sorted `(target, source)` pairs form a compact reverse-reference index.
    pub back_references: Vec<(i32, i32)>,
    pub exception_handlers: BTreeMap<i32, ExceptionHandlers>,
    pub instance_bitmaps: BTreeMap<i32, u64>,
    /// Snapshot Code indices from the VM's compressed class dispatch table.
    /// `None` is a null dispatch entry; positive values use GetCodeIndex encoding.
    pub dispatch_table_code_indices: Vec<Option<usize>>,
}

impl ParseResult {
    pub fn new(header: ClusterHeader) -> Self {
        Self {
            header,
            clusters: Vec::new(),
            fill_start: 0,
            strings: BTreeMap::new(),
            named: BTreeMap::new(),
            library_uris: BTreeMap::new(),
            function_types: BTreeMap::new(),
            codes: Vec::new(),
            pool: Vec::new(),
            object_pools: Vec::new(),
            objects: Vec::new(),
            object_references: Vec::new(),
            object_scalars: Vec::new(),
            object_bytes: Vec::new(),
            back_references: Vec::new(),
            exception_handlers: BTreeMap::new(),
            instance_bitmaps: BTreeMap::new(),
            dispatch_table_code_indices: Vec::new(),
        }
    }

    pub fn insert_object(
        &mut self,
        reference: i32,
        cid: i32,
        canonical: bool,
        kind: SnapshotObjectKind,
        payload: SnapshotObjectPayload,
    ) {
        let reference_start = self.object_references.len();
        self.object_references.extend(payload.references);
        let scalar_start = self.object_scalars.len();
        self.object_scalars.extend(payload.scalars);
        let byte_start = self.object_bytes.len();
        self.object_bytes.extend(payload.bytes);
        self.objects.push(SnapshotObject {
            reference,
            cid,
            canonical,
            kind,
            reference_range: reference_start..self.object_references.len(),
            scalar_range: scalar_start..self.object_scalars.len(),
            byte_range: byte_start..self.object_bytes.len(),
        });
    }

    pub fn rebuild_back_references(&mut self) {
        let mut back_references = Vec::with_capacity(self.object_references.len());
        for object in &self.objects {
            for target in &self.object_references[object.reference_range.clone()] {
                if *target >= 0 {
                    back_references.push((*target, object.reference));
                }
            }
        }
        back_references.sort_unstable();
        back_references.dedup();
        self.back_references = back_references;
    }

    pub fn object(&self, reference: i32) -> Option<&SnapshotObject> {
        self.objects
            .binary_search_by_key(&reference, |object| object.reference)
            .ok()
            .and_then(|index| self.objects.get(index))
    }

    pub fn references_of(&self, object: &SnapshotObject) -> &[i32] {
        &self.object_references[object.reference_range.clone()]
    }

    pub fn scalars_of(&self, object: &SnapshotObject) -> &[SnapshotScalar] {
        &self.object_scalars[object.scalar_range.clone()]
    }

    pub fn bytes_of(&self, object: &SnapshotObject) -> &[u8] {
        &self.object_bytes[object.byte_range.clone()]
    }

    pub fn reverse_reference_target_count(&self) -> usize {
        self.back_references
            .iter()
            .map(|(target, _)| target)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::decode_one_byte_string;

    #[test]
    fn decodes_dart_one_byte_strings_as_latin1() {
        assert_eq!(decode_one_byte_string(b" headers \xb7 "), " headers · ");
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstructionEntry {
    pub pc_offset: u32,
    pub stack_map_offset: u32,
}

#[derive(Clone, Debug)]
pub struct InstructionTable {
    pub first_code: usize,
    pub entries: Vec<InstructionEntry>,
    pub canonical_stack_map_offset: Option<u32>,
    pub stack_maps: BTreeMap<u32, CompressedStackMap>,
}

#[derive(Clone, Debug)]
pub struct CompressedStackMap {
    pub offset: u32,
    pub global_table: bool,
    pub uses_global_table: bool,
    pub payload: Vec<u8>,
    pub entry_count: usize,
}
