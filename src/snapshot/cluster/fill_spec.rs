use super::alloc::is_typed_data;
use super::cid::Cids;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FillKind {
    Refs,
    String,
    None,
    Double,
    Code,
    ObjectPool,
    Array,
    WeakArray,
    TypedData,
    ExceptionHandlers,
    Context,
    TypeArguments,
    InlineBytes,
    Instance,
    Record,
    ContextScope,
    Class,
}

#[derive(Clone, Copy, Debug)]
pub enum Scalar {
    Tagged32,
    Tagged64,
    Uint16,
    Unsigned,
    Byte,
    Reference,
}

#[derive(Clone, Debug)]
pub struct Spec {
    pub kind: FillKind,
    pub refs: usize,
    pub scalars: Vec<Scalar>,
    pub name_index: Option<usize>,
    pub owner_index: Option<usize>,
    pub signature_index: Option<usize>,
    pub function: bool,
    pub function_type: bool,
}

impl Spec {
    fn refs(count: usize) -> Self {
        Self {
            kind: FillKind::Refs,
            refs: count,
            scalars: Vec::new(),
            name_index: None,
            owner_index: None,
            signature_index: None,
            function: false,
            function_type: false,
        }
    }

    fn special(kind: FillKind) -> Self {
        Self {
            kind,
            ..Self::refs(0)
        }
    }

    fn scalars(mut self, values: &[Scalar]) -> Self {
        self.scalars.extend_from_slice(values);
        self
    }

    fn named(mut self, name: usize, owner: Option<usize>) -> Self {
        self.name_index = Some(name);
        self.owner_index = owner;
        self
    }
}

pub fn for_cid(cid: i32, cids: &Cids, compressed: bool) -> Spec {
    if cid == cids.function {
        let mut spec = Spec::refs(4)
            .scalars(&[Scalar::Unsigned, Scalar::Tagged32])
            .named(0, Some(1));
        spec.signature_index = Some(2);
        spec.function = true;
        spec
    } else if cid == cids.class {
        Spec::special(FillKind::Class)
    } else if cid == cids.patch_class {
        Spec::refs(2)
    } else if cid == cids.closure_data {
        Spec::refs(2).scalars(&[Scalar::Unsigned])
    } else if cid == cids.field {
        Spec::refs(4)
            .scalars(&[Scalar::Tagged32, Scalar::Reference])
            .named(0, Some(1))
    } else if cid == cids.script {
        Spec::refs(1).scalars(&[Scalar::Tagged32]).named(0, None)
    } else if cid == cids.library {
        Spec::refs(10)
            .scalars(&[Scalar::Tagged32, Scalar::Uint16, Scalar::Byte, Scalar::Byte])
            .named(0, None)
    } else if cid == cids.namespace {
        Spec::refs(1)
    } else if cid == cids.closure {
        Spec::refs(6)
    } else if cid == cids.unlinked_call {
        Spec::refs(2).scalars(&[Scalar::Byte]).named(0, None)
    } else if cid == cids.subtype_test_cache {
        Spec::refs(1).scalars(&[Scalar::Tagged32, Scalar::Tagged32])
    } else if cid == cids.loading_unit {
        Spec::refs(1).scalars(&[Scalar::Tagged32])
    } else if cid == cids.type_ {
        Spec::refs(3).scalars(&[Scalar::Unsigned])
    } else if cid == cids.function_type {
        let mut spec = Spec::refs(6).scalars(&[Scalar::Byte, Scalar::Tagged32, Scalar::Uint16]);
        spec.function_type = true;
        spec
    } else if cid == cids.record_type {
        Spec::refs(4).scalars(&[Scalar::Byte])
    } else if cid == cids.type_parameter {
        Spec::refs(3).scalars(&[Scalar::Tagged32, Scalar::Tagged32, Scalar::Byte])
    } else if cid == cids.growable_array {
        Spec::refs(3)
    } else if matches!(
        cid,
        value if value == cids.map
            || value == cids.const_map
            || value == cids.set
            || value == cids.const_set
    ) {
        Spec::refs(5)
    } else if cid == cids.regexp {
        Spec::refs(6).scalars(&[Scalar::Tagged32, Scalar::Tagged32, Scalar::Byte])
    } else if cid == cids.weak_property || cid == cids.weak_reference {
        Spec::refs(2)
    } else if cid == cids.library_prefix {
        Spec::refs(2)
            .scalars(&[Scalar::Uint16, Scalar::Byte])
            .named(0, None)
    } else if cid == cids.language_error {
        Spec::refs(4).scalars(&[Scalar::Tagged32, Scalar::Byte, Scalar::Byte])
    } else if cid == cids.unhandled_exception {
        Spec::refs(2)
    } else if cid == cids.ic_data {
        Spec::refs(3).scalars(&[Scalar::Tagged32])
    } else if cid == cids.megamorphic_cache {
        Spec::refs(4).scalars(&[Scalar::Tagged32])
    } else if cid == cids.single_target_cache {
        Spec::refs(1).scalars(&[Scalar::Tagged64, Scalar::Tagged64])
    } else if cid == cids.kernel_program_info {
        Spec::refs(9)
    } else if cid == cids.ffi_trampoline_data {
        Spec::refs(4).scalars(&[Scalar::Tagged32, Scalar::Byte])
    } else if cid == cids.type_parameters {
        Spec::refs(4)
    } else if cid == cids.monomorphic_smiable_call {
        Spec::refs(0).scalars(&[Scalar::Tagged64, Scalar::Tagged64])
    } else if cid == cids.typed_data_view {
        Spec::refs(3)
    } else if cid == cids.external_typed_data {
        Spec::refs(1)
    } else if cid == cids.stack_trace {
        Spec::refs(2)
    } else if cid == cids.send_port {
        Spec::refs(0).scalars(&[Scalar::Tagged64, Scalar::Tagged64])
    } else if cid == cids.capability {
        Spec::refs(0).scalars(&[Scalar::Tagged64])
    } else if cid == cids.receive_port {
        Spec::refs(1).scalars(&[Scalar::Tagged64])
    } else if cid == cids.suspend_state {
        Spec::refs(2).scalars(&[Scalar::Tagged32])
    } else if cid == cids.transferable_typed_data || cid == cids.sentinel {
        Spec::special(FillKind::None)
    } else if cid == cids.user_tag {
        Spec::refs(1).scalars(&[Scalar::Tagged64]).named(0, None)
    } else if cid == cids.future_or {
        Spec::refs(2)
    } else if cid == cids.weak_serialization_reference {
        Spec::refs(1)
    } else if matches!(
        cid,
        value if value == cids.string
            || value == cids.one_byte_string
            || value == cids.two_byte_string
    ) {
        if compressed {
            Spec::special(FillKind::String)
        } else {
            Spec::special(FillKind::None)
        }
    } else if cid == cids.mint {
        Spec::special(FillKind::None)
    } else if cid == cids.double {
        Spec::special(FillKind::Double)
    } else if cid == cids.float32x4 || cid == cids.int32x4 {
        Spec::refs(0).scalars(&[
            Scalar::Tagged32,
            Scalar::Tagged32,
            Scalar::Tagged32,
            Scalar::Tagged32,
        ])
    } else if cid == cids.float64x2 {
        Spec::refs(0).scalars(&[Scalar::Tagged64, Scalar::Tagged64])
    } else if cid == cids.code {
        Spec::special(FillKind::Code)
    } else if cid == cids.object_pool {
        Spec::special(FillKind::ObjectPool)
    } else if cid == cids.array || cid == cids.immutable_array {
        Spec::special(FillKind::Array)
    } else if cid == cids.weak_array {
        Spec::special(FillKind::WeakArray)
    } else if cid == cids.type_arguments {
        Spec::special(FillKind::TypeArguments)
    } else if cid == cids.exception_handlers {
        Spec::special(FillKind::ExceptionHandlers)
    } else if cid == cids.context {
        Spec::special(FillKind::Context)
    } else if cid == cids.context_scope {
        Spec::special(FillKind::ContextScope)
    } else if matches!(
        cid,
        value if value == cids.pc_descriptors
            || value == cids.code_source_map
            || value == cids.compressed_stack_maps
    ) {
        if compressed {
            Spec::special(FillKind::InlineBytes)
        } else {
            Spec::special(FillKind::None)
        }
    } else if is_typed_data(cid, cids) {
        Spec::special(FillKind::TypedData)
    } else if cid == cids.record {
        Spec::special(FillKind::Record)
    } else if cid == cids.instance || cid >= cids.predefined_count {
        Spec::special(FillKind::Instance)
    } else {
        Spec::refs(0)
    }
}
