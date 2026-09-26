use std::collections::{BTreeMap, BTreeSet};

use capstone::prelude::*;

use crate::analysis::calling_convention::{
    ArgumentLocation, BodyEvidence, DeclaredRepresentation, LocationProof, RegisterWindow,
    Representation, ResolvedParameters, TargetLayout,
};
use crate::diagnostic::{ClutterError, Result};
use crate::model::{
    Abi, CallTargetScope, ControlFlowEdge, ControlFlowEdgeKind, DirectCallResolution,
    EvidenceConfidence, MachineCodeEvidence, MachineInstruction, PseudoStatement,
    SemanticStatement,
};

const MAX_RENDERED_INSTRUCTIONS: usize = 80;
const MAX_SEMANTIC_WORKLIST_VISITS: usize = 4096;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub label: String,
    pub library_uri: Option<String>,
    pub scope: CallTargetScope,
    pub semantic_name: bool,
    pub code_address: Option<u64>,
    pub entry_offset: Option<u64>,
    pub resolution: Option<DirectCallResolution>,
    pub result_class: Option<String>,
    /// Library declaring `result_class`, when the declared type carries one.
    pub result_library_uri: Option<String>,
    /// `(class, library)` owning an instance member: a direct call to it
    /// proves its receiver (the first argument) is an instance of that
    /// class or a subclass that inherits the member.
    pub receiver_class: Option<(String, Option<String>)>,
    /// Where the callee expects its fixed parameters, when its body and
    /// kind pin the calling convention down. Optional parameters are always
    /// stacked by the caller in argument order.
    pub parameters: Option<std::sync::Arc<[ArgumentLocation]>>,
    /// Exact retained value-argument count when no optional parameters exist.
    /// This can bound an opaque call whose code body did not survive.
    pub value_argument_count: Option<usize>,
    /// Whether the callee returns its value in the FPU return register.
    pub returns_fpu: bool,
    /// A per-class allocation stub: its result is a fresh instance of
    /// exactly `result_class`.
    pub allocation_stub: bool,
}

impl Symbol {
    pub fn new(
        label: String,
        library_uri: Option<String>,
        application_package: Option<&str>,
    ) -> Self {
        let scope = call_target_scope(&label, library_uri.as_deref(), application_package);
        Self {
            label,
            library_uri,
            scope,
            semantic_name: true,
            code_address: None,
            entry_offset: None,
            resolution: None,
            result_class: None,
            result_library_uri: None,
            receiver_class: None,
            parameters: None,
            value_argument_count: None,
            returns_fpu: false,
            allocation_stub: false,
        }
    }

    pub fn code_boundary(address: u64) -> Self {
        Self {
            label: format!("sub_{address:x}"),
            library_uri: None,
            scope: CallTargetScope::Unknown,
            semantic_name: false,
            code_address: Some(address),
            entry_offset: Some(0),
            resolution: Some(DirectCallResolution::ExactEntry),
            result_class: None,
            result_library_uri: None,
            receiver_class: None,
            parameters: None,
            value_argument_count: None,
            returns_fpu: false,
            allocation_stub: false,
        }
    }

    pub fn with_code_identity(
        mut self,
        code_address: u64,
        entry_offset: u64,
        resolution: DirectCallResolution,
    ) -> Self {
        self.code_address = Some(code_address);
        self.entry_offset = Some(entry_offset);
        self.resolution = Some(resolution);
        self
    }

    pub fn with_result_class(mut self, class_name: Option<String>) -> Self {
        self.result_class = class_name;
        self
    }
}

pub struct DispatchTableAnalysis<'a> {
    pub origin_element: usize,
    pub targets: &'a [Option<String>],
    pub class_ids: &'a [usize],
    pub cid_to_name: Option<&'a BTreeMap<usize, String>>,
    pub name_to_cids: Option<&'a BTreeMap<String, Vec<usize>>>,
    /// Qualified `(library_uri, class_name) -> cids` for library-sensitive
    /// lookup. When an expression carries a library URI the qualified map is
    /// consulted first; on miss the name-only map remains the fallback.
    pub qualified_to_cids: Option<&'a BTreeMap<(Option<String>, String), Vec<usize>>>,
    /// Direct super-class link `child_cid -> super_cid` derived from
    /// `TypeRecovery::class_metadata` super_type. Used to walk the chain
    /// when a precise receiver is known but the selector has no direct slot
    /// for that cid (rare synthetic gaps).
    pub super_cids: Option<&'a BTreeMap<usize, usize>>,
    /// Class id owning the implementation stored in each table slot
    /// (parallel to `targets`), when its Code keeps a Function owner.
    pub target_owner_cids: &'a [Option<usize>],
    /// Static class id -> every concrete class id that is a subtype of it.
    pub subtype_cids: Option<&'a BTreeMap<usize, Vec<usize>>>,
    /// Call-target label -> `(class, library)` its function declares as
    /// its result, for labels every same-named target agrees on. Types the
    /// result of a call resolved to a label rather than a code address.
    pub label_results: Option<&'a BTreeMap<String, (String, Option<String>)>>,
}

impl DispatchTableAnalysis<'_> {
    /// Whether table slot `index`, reached with receiver `class_id`, holds an
    /// implementation the receiver can inherit. Rows are packed with holes
    /// that other selectors fill, so a slot whose owner is not `class_id` or
    /// one of its superclasses belongs to a different row. `None` when the
    /// owner or the hierarchy is unknown.
    fn slot_fits_receiver(&self, index: usize, class_id: usize) -> Option<bool> {
        let owner = (*self.target_owner_cids.get(index)?)?;
        let supers = self.super_cids?;
        let mut current = class_id;
        for _ in 0..64 {
            if current == owner {
                return Some(true);
            }
            match supers.get(&current) {
                Some(parent) => current = *parent,
                None => return Some(false),
            }
        }
        None
    }
}

/// Owned dispatch-table evidence kept for the final semantic relift.
#[derive(Clone, Debug, Default)]
pub struct DispatchTableData {
    pub origin_element: usize,
    pub targets: Vec<Option<String>>,
    pub class_ids: Vec<usize>,
    pub cid_to_name: BTreeMap<usize, String>,
    pub name_to_cids: BTreeMap<String, Vec<usize>>,
    pub qualified_to_cids: BTreeMap<(Option<String>, String), Vec<usize>>,
    pub super_cids: BTreeMap<usize, usize>,
    pub target_owner_cids: Vec<Option<usize>>,
    pub subtype_cids: BTreeMap<usize, Vec<usize>>,
}

impl DispatchTableData {
    pub fn analysis(&self) -> Option<DispatchTableAnalysis<'_>> {
        (!self.targets.is_empty() && !self.class_ids.is_empty()).then(|| DispatchTableAnalysis {
            origin_element: self.origin_element,
            targets: &self.targets,
            class_ids: &self.class_ids,
            cid_to_name: Some(&self.cid_to_name),
            name_to_cids: Some(&self.name_to_cids),
            qualified_to_cids: Some(&self.qualified_to_cids),
            super_cids: Some(&self.super_cids),
            target_owner_cids: &self.target_owner_cids,
            subtype_cids: Some(&self.subtype_cids),
            label_results: None,
        })
    }
}

pub struct Disassembly {
    pub statements: Vec<PseudoStatement>,
    pub evidence: MachineCodeEvidence,
    pub instructions: Vec<MachineInstruction>,
    pub control_flow: Vec<ControlFlowEdge>,
    pub semantic_statements: Vec<SemanticStatement>,
}

pub(crate) struct SemanticLiftOutcome {
    pub statements: Vec<SemanticStatement>,
    pub worklist_exhausted: bool,
    /// Optional positional parameter defaults the prologue materializes,
    /// keyed by parameter index (implicit parameters included).
    pub parameter_defaults: BTreeMap<usize, String>,
    /// Argument and return classes the final round observed.
    pub facts: LiftFacts,
}

pub struct Disassembler {
    capstone: Capstone,
    abi: Abi,
}

/// Decodes the two VFP instructions Dart uses for immediate double
/// comparisons when the bundled Capstone reports their bytes as skip-data.
/// The masks come directly from Dart's ARM `EmitVFPddd`, `vmovd`, and
/// `vcmpd` encodings; everything outside those exact opcode families stays
/// unknown.
fn decode_arm32_vfp_fallback(bytes: &[u8]) -> Option<(String, String)> {
    let bytes: [u8; 4] = bytes.try_into().ok()?;
    let word = u32::from_le_bytes(bytes);

    // Register-move / unary data-processing forms in the VFP A2 space:
    // `cond 1110 1011 11nn Vddd 101S ..op....` — same skeleton as the
    // three-operand arithmetic block below but with the opcode nibble
    // bits[23:20] == 1011 instead of {2,3,8}; that nibble is what separates
    // `vmov.f64 dD, dM` (two-register) from `vsub.f64 dD, d0, dM`
    // (three-register, where bits[19:16] would read as opcode 0). Families
    // verified against every residual unknown word of the Dart 3.12.2
    // obf-raw-arm32 matrix run (361 words, all matched, operands identical
    // to binutils):
    //   column B, vn=0000, op=0100  vmov.f64   (register move)
    //   column B, vn=0001, op=0100  vneg.f64
    //   column B, vn=0001, op=1100  vsqrt.f64
    //   column A, vn=0111, op=1100  vcvt.f64.f32 (single-precision source)
    // Disjoint from the `vmovd` immediate form below (bits[7:4] == 0000)
    // and from `vcmpd` (bits[19:16] == 0100 with a different opcode nibble),
    // so evaluation order versus those blocks is irrelevant.
    if word & 0x0ff0_0e00 == 0x0eb0_0a00 {
        let condition = (word >> 28) & 0xf;
        let column = (word >> 8) & 0xf;
        let operation = ((word >> 16) & 0xf, (word >> 4) & 0xf);
        let destination = ((word >> 12) & 0xf) | (((word >> 22) & 1) << 4);
        let source = (word & 0xf) | (((word >> 5) & 1) << 4);
        let single_source = ((word & 0xf) << 1) | ((word >> 5) & 1);
        let decoded = match (column, operation) {
            (0xb, (0x0, 0x4)) => Some(("vmov", "f64", format!("d{destination}, d{source}"))),
            (0xb, (0x1, 0x4)) => Some(("vneg", "f64", format!("d{destination}, d{source}"))),
            (0xb, (0x1, 0xc)) => Some(("vsqrt", "f64", format!("d{destination}, d{source}"))),
            // Single-precision source register: the five-bit S number reads
            // the field whole instead of being split around bit 4.
            (0xa, (0x7, 0xc)) => Some((
                "vcvt",
                "f64.f32",
                format!("d{destination}, s{single_source}"),
            )),
            _ => None,
        };
        if let Some((root, extension, operands)) = decoded {
            return Some((
                conditional_mnemonic(&format!("{root}.{extension}"), condition),
                operands,
            ));
        }
    }

    const VMOVD_VARIABLE_BITS: u32 = 0x004f_f00f;
    const VMOVD_BASE: u32 = 0xeeb0_0b00;
    if word & !VMOVD_VARIABLE_BITS == VMOVD_BASE {
        let destination = ((word >> 12) & 0xf) | (((word >> 22) & 1) << 4);
        let immediate = (((word >> 16) & 0xf) << 4) | (word & 0xf);
        let sign = u64::from((immediate >> 7) & 1);
        let exponent_bit = u64::from((immediate >> 6) & 1);
        let exponent_fill = if exponent_bit == 1 { 0xff } else { 0 };
        let bits = (sign << 63)
            | ((1 ^ exponent_bit) << 62)
            | (exponent_fill << 54)
            | (u64::from(immediate & 0x3f) << 48);
        let value = f64::from_bits(bits);
        let text = floating_immediate_text(&value.to_string())?;
        return Some(("vmovd".to_owned(), format!("d{destination}, #{text}")));
    }

    const VCMPD_VARIABLE_BITS: u32 = 0x0040_f02f;
    const VCMPD_BASE: u32 = 0xeeb4_0b40;
    if word & !VCMPD_VARIABLE_BITS == VCMPD_BASE {
        let left = ((word >> 12) & 0xf) | (((word >> 22) & 1) << 4);
        let right = (word & 0xf) | (((word >> 5) & 1) << 4);
        return Some(("vcmpd".to_owned(), format!("d{left}, d{right}")));
    }

    // VFP data-processing (A2) forms, pinned to Dart's own ARM assembler
    // (`EmitVFPddd` callers in assembler_arm.cc): the fixed skeleton is
    // `cond 1110 111x Vn Vd 1011 szN0M Vm`, and the operation lives in
    // bits[23:20] plus bits[7:6]. Everything outside these families stays
    // unknown.
    let condition = (word >> 28) & 0xf;
    let d = ((word >> 12) & 0xf) | (((word >> 22) & 1) << 4);
    let m = (word & 0xf) | (((word >> 5) & 1) << 4);
    if word & 0x0f00_0e10 == 0x0e00_0a00 && word & 0x100 != 0 {
        let opcode = (word >> 20) & 0xf;
        let opc2 = (word >> 6) & 0x3;
        let name = match (opcode, opc2) {
            (0x2, 0x0) => Some("vmul.f64"),
            (0x3, 0x0) => Some("vadd.f64"),
            (0x3, 0x1) => Some("vsub.f64"),
            (0x8, 0x0) => Some("vdiv.f64"),
            // The 1011 column distinguishes two-operand operations through
            // bits[19:16] instead of a source register.
            (0xb, _) => match (word >> 16) & 0xf {
                0x5 => Some(if word & 0x40 != 0 { "vcmpdz" } else { "vcmpd" }),
                0x8 => Some("vcvt.if"),
                0xc => Some("vcvt.fi"),
                0xd => Some("vcvt.fi"),
                0x7 => Some("vcvt.ds"),
                _ => None,
            },
            _ => None,
        };
        if let Some(name) = name {
            // Three-register arithmetic reads Vn (bits[19:16], N at bit 7) as
            // its first source; the 1011 column has no Vn operand.
            let operands = if opcode == 0xb {
                format!("d{d}, d{m}")
            } else {
                let n = ((word >> 16) & 0xf) | (((word >> 7) & 1) << 4);
                format!("d{d}, d{n}, d{m}")
            };
            return Some((conditional_mnemonic(name, condition), operands));
        }
    }
    None
}

/// Reattaches an ARM condition code to a synthesized VFP mnemonic so the
/// decoded stream keeps its original branch semantics (`vaddne` etc.). The
/// lifter treats every conditional form as the base operation because Dart's
/// AOT uses conditional VFP only inside diamonds whose arms are fused away.
fn conditional_mnemonic(base: &str, condition: u32) -> String {
    if condition == 0xe {
        base.to_owned()
    } else {
        static CONDITIONS: [&str; 15] = [
            "eq", "ne", "cs", "cc", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le",
            "al",
        ];
        match usize::try_from(condition)
            .ok()
            .and_then(|index| CONDITIONS.get(index))
        {
            Some(suffix) => format!("{base}{suffix}"),
            None => base.to_owned(),
        }
    }
}

impl Disassembler {
    pub fn new(abi: Abi) -> Result<Self> {
        let mut capstone = match abi {
            Abi::Arm64V8a => Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .detail(true)
                .build(),
            Abi::ArmeabiV7a => Capstone::new()
                .arm()
                .mode(arch::arm::ArchMode::Arm)
                .detail(true)
                .build(),
            Abi::X86_64 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .detail(true)
                .build(),
        }
        .map_err(|error| ClutterError::Analysis(format!("initialize disassembler: {error}")))?;
        capstone
            .set_skipdata(true)
            .map_err(|error| ClutterError::Analysis(format!("enable data skipping: {error}")))?;
        Ok(Self { capstone, abi })
    }

    pub fn analyze(
        &self,
        address: u64,
        bytes: &[u8],
        symbols: &BTreeMap<u64, Symbol>,
        parameter_count: Option<usize>,
        object_pool: Option<&[String]>,
        dispatch_table: Option<&DispatchTableAnalysis<'_>>,
        receiver_class: Option<&str>,
        receiver_library_uri: Option<&str>,
    ) -> Result<Disassembly> {
        let instructions = self
            .capstone
            .disasm_all(bytes, address)
            .map_err(|error| ClutterError::Analysis(format!("disassemble function: {error}")))?;
        let mut statements =
            Vec::with_capacity(instructions.len().min(MAX_RENDERED_INSTRUCTIONS) + 2);
        let mut consumed = 0usize;
        let mut skipped_data = 0usize;
        let mut omitted_non_call_instructions = 0usize;
        let mut evidence = MachineCodeEvidence {
            instruction_bytes: bytes.len(),
            ..MachineCodeEvidence::default()
        };
        let function_end = address.saturating_add(bytes.len() as u64);
        let mut block_starts = std::collections::BTreeSet::from([address]);
        let mut machine_instructions = Vec::with_capacity(instructions.len());
        let mut decoded = Vec::with_capacity(instructions.len());
        let mut pool_registers = BTreeMap::<String, (usize, String)>::new();
        let mut pool_provenance = PoolPointerProvenance::new(self.abi);
        for (index, instruction) in instructions.iter().enumerate() {
            consumed = instruction
                .address()
                .saturating_sub(address)
                .saturating_add(instruction.bytes().len() as u64)
                .try_into()
                .unwrap_or(bytes.len());
            let fallback = (self.abi == Abi::ArmeabiV7a
                && instruction.mnemonic().is_some_and(is_skipped_data))
            .then(|| decode_arm32_vfp_fallback(instruction.bytes()))
            .flatten();
            let mnemonic = fallback.as_ref().map_or_else(
                || instruction.mnemonic().unwrap_or("unknown"),
                |value| &value.0,
            );
            let operands = fallback
                .as_ref()
                .map_or_else(|| instruction.op_str().unwrap_or(""), |value| &value.1);
            let pool_index = pool_provenance.load_index(mnemonic, operands);
            let pool_value = pool_index.and_then(|index| {
                let pool = object_pool?;
                if self.abi == Abi::ArmeabiV7a && mnemonic == "vldr" {
                    // ARM32 pool words are 32 bits: an unboxed double
                    // occupies two consecutive immediate entries, low first.
                    let word = |index: usize| {
                        pool.get(index)
                            .and_then(|value| value.parse::<i64>().ok())
                            .map(|value| u64::from(value as u32))
                    };
                    return Some(((word(index + 1)? << 32) | word(index)?).to_string());
                }
                pool.get(index).cloned()
            });
            evidence.object_pool_loads += usize::from(pool_value.is_some());
            let destination = split_operands(operands).first().cloned();
            if pool_value.is_none()
                && writes_first_operand(mnemonic)
                && let Some(destination) = destination.as_deref()
            {
                pool_registers.remove(&normalize_register(destination));
            }
            if let (Some(pool_index), Some(pool_value)) = (pool_index, pool_value.as_ref())
                && let Some(destination) = destination.as_deref()
            {
                pool_registers.insert(
                    normalize_register(destination),
                    (pool_index, pool_value.clone()),
                );
            }
            pool_provenance.observe(mnemonic, operands);
            machine_instructions.push(MachineInstruction {
                address: format!("0x{:x}", instruction.address()),
                bytes: hex::encode(instruction.bytes()),
                mnemonic: mnemonic.to_owned(),
                operands: operands.to_owned(),
                object_pool_index: pool_index,
                object_pool_value: pool_value,
            });
            if is_skipped_data(mnemonic) {
                skipped_data += instruction.bytes().len();
                if index < MAX_RENDERED_INSTRUCTIONS {
                    statements.push(PseudoStatement::UnknownOperation {
                        address: format!("0x{:x}", instruction.address()),
                        bytes: hex::encode(instruction.bytes()),
                    });
                }
                continue;
            }
            decoded.push(DecodedInstruction {
                address: instruction.address(),
                next: instruction
                    .address()
                    .saturating_add(instruction.bytes().len() as u64),
                mnemonic: mnemonic.to_owned(),
                operands: operands.to_owned(),
            });
            evidence.decoded_instructions += 1;
            if let Some(target_address) = direct_call_target(mnemonic, operands) {
                evidence.direct_calls += 1;
                // `WriteBarrierWrappers` is entered at a per-register offset
                // inside the root-proven stub.
                let symbol = symbols.get(&target_address).or_else(|| {
                    symbols
                        .range(..target_address)
                        .next_back()
                        .map(|(_, symbol)| symbol)
                        .filter(|symbol| symbol.label == "stub WriteBarrierWrappers")
                });
                statements.push(PseudoStatement::DirectCall {
                    address: format!("0x{:x}", instruction.address()),
                    target_address: format!("0x{target_address:x}"),
                    target_code_address: symbol
                        .and_then(|symbol| symbol.code_address)
                        .map(|address| format!("0x{address:x}")),
                    target_entry_offset: symbol.and_then(|symbol| symbol.entry_offset),
                    target_resolution: symbol.and_then(|symbol| symbol.resolution),
                    target: symbol
                        .filter(|symbol| symbol.semantic_name)
                        .map(|symbol| symbol.label.clone()),
                    target_library_uri: symbol.and_then(|symbol| symbol.library_uri.clone()),
                    target_scope: symbol.map_or(CallTargetScope::Unknown, |symbol| symbol.scope),
                });
            } else if is_call(mnemonic) {
                evidence.indirect_calls += 1;
                let called_entry = pool_registers.get(&normalize_register(operands)).cloned();
                // Switchable-call shapes (SingleTarget/IC/megamorphic) load
                // the stub entry into the call register while the paired
                // UnlinkedCall selector — carrying the dynamic-call name —
                // rides in a scratch register. When both are recent pool
                // loads, the selector is the authoritative call evidence.
                let selector_slot = pool_registers
                    .values()
                    .rev()
                    .find(|(_, label)| label.starts_with("dynamicCall(\""))
                    .cloned();
                let resolved = match called_entry {
                    Some((index, target)) if is_named_pool_target(&target) => Some((index, target)),
                    // Stub entry missing or opaque: a live dynamicCall
                    // selector identifies the paired switchable-call site.
                    _ => selector_slot,
                };
                if let Some((pool_index, target)) = resolved {
                    statements.push(PseudoStatement::ObjectPoolCall {
                        address: format!("0x{:x}", instruction.address()),
                        expression: operands.to_owned(),
                        pool_index,
                        target: target.clone(),
                        target_scope: call_target_scope(&target, None, None),
                    });
                } else {
                    statements.push(PseudoStatement::IndirectCall {
                        address: format!("0x{:x}", instruction.address()),
                        expression: operands.to_owned(),
                    });
                }
            } else if is_return(mnemonic, operands) {
                evidence.returns += 1;
                statements.push(PseudoStatement::MachineReturn {
                    address: format!("0x{:x}", instruction.address()),
                });
                add_fallthrough_block(&mut block_starts, instruction, function_end);
            } else if let Some(conditional) = branch_kind(mnemonic) {
                let target = branch_target(operands);
                if conditional {
                    evidence.conditional_branches += 1;
                } else {
                    evidence.unconditional_branches += 1;
                }
                if let Some(target) = target
                    && (address..function_end).contains(&target)
                {
                    block_starts.insert(target);
                }
                add_fallthrough_block(&mut block_starts, instruction, function_end);
                statements.push(PseudoStatement::Branch {
                    address: format!("0x{:x}", instruction.address()),
                    target_address: target.map(|target| format!("0x{target:x}")),
                    conditional,
                });
            } else if index < MAX_RENDERED_INSTRUCTIONS {
                statements.push(PseudoStatement::Comment {
                    text: format!("0x{:x}: {mnemonic} {operands}", instruction.address())
                        .trim_end()
                        .to_owned(),
                });
            } else {
                omitted_non_call_instructions += 1;
            }
        }
        if omitted_non_call_instructions > 0 {
            statements.push(PseudoStatement::Comment {
                text: format!(
                    "{omitted_non_call_instructions} additional non-call machine instructions omitted"
                ),
            });
        }
        let trailing_unknown = bytes.len().saturating_sub(consumed);
        evidence.unknown_bytes = skipped_data.saturating_add(trailing_unknown);
        evidence.decoded_bytes = bytes.len().saturating_sub(evidence.unknown_bytes);
        evidence.basic_block_starts = if evidence.decoded_instructions == 0 {
            0
        } else {
            block_starts.len()
        };
        let control_flow = build_control_flow(address, function_end, &decoded, &block_starts);
        evidence.control_flow_edges = control_flow.len();
        evidence.reachable_basic_blocks =
            reachable_block_count(address, &control_flow, &block_starts);
        let dispatch_cache =
            dispatch_table.map(|table| (table, recover_dispatch_calls(self.abi, &decoded, table)));
        if let Some((_, calls)) = dispatch_cache.as_ref() {
            evidence.dispatch_table_calls = calls.len();
            // A selector family does not prove the receiver's runtime class.
            // Exact resolution stays zero until class-ID data flow identifies
            // one concrete table slot.
            evidence.resolved_dispatch_table_calls = 0;
            for statement in &mut statements {
                let PseudoStatement::IndirectCall {
                    address,
                    expression,
                } = statement
                else {
                    continue;
                };
                let Some(call_address) = parse_immediate(address) else {
                    continue;
                };
                let Some(call) = calls.get(&call_address) else {
                    continue;
                };
                *statement = PseudoStatement::DispatchTableCall {
                    address: address.clone(),
                    expression: expression.clone(),
                    selector_offset: call.selector_offset,
                    selector_name: call.selector_name.clone(),
                    candidate_targets: call.candidate_targets.clone(),
                    candidate_count: call.candidate_count,
                    raw_slot_target_count: call.raw_slot_target_count,
                };
            }
        } else {
            // Ensure dispatch counters are zero when table absent.
            evidence.dispatch_table_calls = 0;
            evidence.resolved_dispatch_table_calls = 0;
        }
        let semantic_statements = {
            // `parameter_count` counts visible parameters; an instance
            // member's receiver is the implicit parameter ahead of them.
            let receiver = receiver_class.map(|class| ParameterHint {
                name: "this".to_owned(),
                class_name: Some(class.to_owned()),
                class_library_uri: receiver_library_uri.map(str::to_owned),
            });
            let parameter_hints = receiver
                .into_iter()
                .chain(
                    (0..parameter_count.unwrap_or_default()).map(|index| ParameterHint {
                        name: format!("arg{index}"),
                        class_name: None,
                        class_library_uri: None,
                    }),
                )
                .collect::<Vec<_>>();
            let convention = ConventionInput {
                declared: parameter_count.map(|_| {
                    parameter_hints
                        .iter()
                        .map(|hint| {
                            if hint.name == "this" {
                                DeclaredRepresentation::Tagged
                            } else {
                                DeclaredRepresentation::Unknown
                            }
                        })
                        .collect()
                }),
                window: None,
                entry_offset: 0,
            };
            let (dispatch_table_ref, dispatch_calls_ref) = match &dispatch_cache {
                Some((table, calls)) => (Some(*table), Some(calls)),
                None => (None, None),
            };
            lift_semantics_with_names_outcome(
                self.abi,
                &parameter_hints,
                &decoded,
                &block_starts,
                symbols,
                object_pool,
                None,
                None,
                dispatch_table_ref,
                dispatch_calls_ref,
                Some(&convention),
                &[],
                &[],
            )
        };
        evidence.semantic_worklist_exhausted = semantic_statements.worklist_exhausted;
        let semantic_statements = semantic_statements.statements;
        evidence.semantic_statements = semantic_statements.len();
        // Update resolved dispatch counter from receiver-proven semantics
        // (P1).  Before this pass `resolved_dispatch_table_calls` was always 0
        // because no dispatch call was ever proven.
        if let Some((_, calls)) = dispatch_cache.as_ref() {
            let resolved = semantic_statements
                .iter()
                .filter(|stmt| match stmt {
                    // Unresolved dispatch sites keep a `dispatch <selector>`
                    // call; only a proven implementation counts as resolved.
                    crate::model::SemanticStatement::ResolvedCall { target, .. }
                        if target.starts_with("dispatch ") =>
                    {
                        false
                    }
                    crate::model::SemanticStatement::ResolvedCall { address, .. } => {
                        let addr = address.trim_start_matches("0x").trim_start_matches("0X");
                        if let Ok(v) = u64::from_str_radix(addr, 16) {
                            calls.contains_key(&v)
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
                .count();
            evidence.resolved_dispatch_table_calls = resolved;
        }
        if consumed < bytes.len() {
            statements.push(PseudoStatement::UnknownOperation {
                address: format!("0x{:x}", address + consumed as u64),
                bytes: hex::encode(&bytes[consumed..]),
            });
        }
        statements.push(PseudoStatement::ReturnUnknown);
        Ok(Disassembly {
            statements,
            evidence,
            instructions: machine_instructions,
            control_flow,
            semantic_statements,
        })
    }
}

pub(crate) fn relift_semantics(
    function: &crate::model::RecoveredFunction,
    abi: Abi,
    parameter_hints: &[ParameterHint],
    field_layout: Option<&RecoveredFieldLayout>,
    receiver_class: Option<(&str, Option<&str>)>,
    symbols: &BTreeMap<u64, Symbol>,
    dispatch_table: Option<&DispatchTableAnalysis<'_>>,
) -> SemanticLiftOutcome {
    let mut decoded = function
        .instructions
        .iter()
        .filter_map(|instruction| {
            let address = parse_immediate(&instruction.address)?;
            let byte_len = instruction.bytes.len() / 2;
            Some(DecodedInstruction {
                address,
                next: address.saturating_add(byte_len as u64),
                mnemonic: instruction.mnemonic.clone(),
                operands: instruction.operands.clone(),
            })
        })
        .collect::<Vec<_>>();
    let Some(entry) = parse_immediate(&function.address) else {
        return SemanticLiftOutcome {
            statements: function.semantic_statements.clone(),
            worklist_exhausted: function.machine_code.semantic_worklist_exhausted,
            parameter_defaults: BTreeMap::new(),
            facts: LiftFacts::default(),
        };
    };
    let suspensions = suspension_calls(function, &decoded, entry);
    if abi == Abi::X86_64 {
        skip_x64_suspend_epilogues(&mut decoded, &suspensions);
    }
    let overlay;
    let symbols = match suspend_stub_symbols(abi, &decoded, &suspensions, function.async_modifier)
    {
        Some(stubs) => {
            let mut symbols = symbols.clone();
            symbols.extend(stubs);
            overlay = symbols;
            &overlay
        }
        None => symbols,
    };
    let mut block_starts = std::collections::BTreeSet::from([entry]);
    for edge in &function.control_flow {
        if let Some(address) = parse_immediate(&edge.from) {
            block_starts.insert(address);
        }
        if let Some(address) = parse_immediate(&edge.to) {
            block_starts.insert(address);
        }
    }

    let max_pool_index = function
        .instructions
        .iter()
        .filter_map(|instruction| instruction.object_pool_index)
        .max()
        .filter(|index| *index < 1_000_000);
    let object_pool = max_pool_index.map(|max_index| {
        let mut values = vec![String::new(); max_index.saturating_add(1)];
        for instruction in &function.instructions {
            if let (Some(index), Some(value)) = (
                instruction.object_pool_index,
                instruction.object_pool_value.as_ref(),
            ) && let Some(slot) = values.get_mut(index)
            {
                *slot = value.clone();
            }
        }
        values
    });
    let dispatch_calls = dispatch_table.map(|table| recover_dispatch_calls(abi, &decoded, table));
    let exceptional_entries = function
        .code_metadata
        .as_ref()
        .map(|metadata| {
            metadata
                .try_regions()
                .into_iter()
                .filter(|region| u64::from(region.handler_pc_offset) < function.size)
                .filter_map(|region| entry.checked_add(u64::from(region.handler_pc_offset)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let exceptional_edges = function
        .code_metadata
        .as_ref()
        .map(|metadata| exceptional_edges(metadata, entry, function.size))
        .unwrap_or_default();
    let outcome = lift_semantics_with_names_outcome(
        abi,
        parameter_hints,
        &decoded,
        &block_starts,
        symbols,
        object_pool.as_deref(),
        field_layout,
        receiver_class,
        dispatch_table,
        dispatch_calls.as_ref(),
        Some(&ConventionInput::for_function(abi, function)),
        &exceptional_entries,
        &exceptional_edges,
    );
    outcome
}

/// Call instructions the compiler marked as suspension points: a pc
/// descriptor with a yield index (`SuspendInstr` records it immediately
/// before the call to its suspend stub).
fn suspension_calls(
    function: &crate::model::RecoveredFunction,
    decoded: &[DecodedInstruction],
    entry: u64,
) -> BTreeSet<u64> {
    let calls = decoded
        .iter()
        .filter(|instruction| is_call(&instruction.mnemonic))
        .map(|instruction| (instruction.address, instruction.next))
        .collect::<Vec<_>>();
    function
        .code_metadata
        .iter()
        .flat_map(|metadata| &metadata.pc_descriptors)
        .filter(|descriptor| descriptor.yield_index >= 0)
        .filter_map(|descriptor| {
            let pc = entry.checked_add(u64::from(descriptor.pc_offset))?;
            calls
                .iter()
                .find(|(address, next)| *address == pc || *next == pc)
                .map(|(address, _)| *address)
        })
        .collect()
}

/// x64 suspend stubs return past a `LeaveFrame; ret` epilogue emitted after
/// the call (`SuspendStubABI::kResumePcDistance`), so that epilogue is not
/// a function exit: execution resumes after it.
fn skip_x64_suspend_epilogues(decoded: &mut Vec<DecodedInstruction>, suspensions: &BTreeSet<u64>) {
    let mut remove = BTreeSet::new();
    for (index, instruction) in decoded.iter().enumerate() {
        if !suspensions.contains(&instruction.address) {
            continue;
        }
        let epilogue = decoded.get(index + 1..index + 4);
        let matches = epilogue.is_some_and(|epilogue| {
            epilogue[0].mnemonic == "mov"
                && epilogue[0].operands.replace(' ', "") == "rsp,rbp"
                && epilogue[1].mnemonic == "pop"
                && epilogue[1].operands.trim() == "rbp"
                && epilogue[2].mnemonic == "ret"
        });
        if matches {
            remove.extend(index + 1..index + 4);
        }
    }
    let mut index = 0usize;
    decoded.retain(|_| {
        let keep = !remove.contains(&index);
        index += 1;
        keep
    });
}

/// `ARGS_DESC_REG` and the byte displacement of `ArgumentsDescriptor`'s
/// count element (array element 1 of the descriptor, tagged).
fn arguments_descriptor(abi: Abi) -> (&'static str, i64) {
    match abi {
        Abi::Arm64V8a => ("x4", 0x13),
        Abi::ArmeabiV7a => ("r4", 0xf),
        Abi::X86_64 => ("r10", 0x13),
    }
}

/// Seeds what a prologue with optional positional parameters reads: the
/// arguments descriptor at entry, and each parameter's value under a
/// pseudo slot the descriptor-relative loads resolve to.
fn seed_optional_parameter_frame(abi: Abi, state: &mut FlowState, hints: &[ParameterHint]) {
    let marker = |text: String| Expression {
        text,
        confidence: EvidenceConfidence::High,
        complexity: 1,
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    };
    state.registers.insert(
        arguments_descriptor(abi).0.to_owned(),
        marker("aot.argumentsDescriptor".to_owned()),
    );
    for (index, hint) in hints.iter().enumerate() {
        state
            .stack
            .insert(format!("param:{index}"), marker(hint.name.clone()));
    }
}

/// Recognizes the optional-parameter prologue (`PrologueBuilder`): the
/// descriptor's argument count, `count - fixed` as a Smi, a frame base
/// scaled by it, and loads of parameter `j` at `(fixed - 1 - j) * word`
/// above the two saved words. Returns the destination and value.
fn optional_parameter_prologue(
    abi: Abi,
    mnemonic: &str,
    operands: &[String],
    registers: &BTreeMap<String, Expression>,
    stack: &BTreeMap<String, Expression>,
) -> Option<(String, Expression)> {
    let word = TargetLayout::of(abi).word_size;
    let text_of = |register: &str| registers.get(&normalize_register(register)).map(|value| value.text.as_str());
    // `(aot.argumentCount - 2F)` -> F, the fixed parameter count.
    let fixed_of = |text: &str| -> Option<i64> {
        let smi = text
            .strip_prefix("(aot.argumentCount - ")?
            .strip_suffix(')')?
            .parse::<i64>()
            .ok()?;
        (smi > 0 && smi % 2 == 0).then_some(smi / 2)
    };
    let named = |text: String| Expression {
        text,
        confidence: EvidenceConfidence::High,
        complexity: 1,
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    };
    let parameter = |fixed: i64, displacement: i64| -> Option<Expression> {
        if displacement % word != 0 {
            return None;
        }
        let index = fixed + 1 - displacement / word;
        let index = usize::try_from(index).ok()?;
        Some(
            stack
                .get(&format!("param:{index}"))
                .cloned()
                .unwrap_or_else(|| named(format!("arg{index}"))),
        )
    };
    let destination = normalize_register(operands.first()?);
    match mnemonic {
        "ldr" | "ldur" | "mov" if operands.len() == 2 && operands[1].contains('[') => {
            let memory = &operands[1];
            let inner = &memory[memory.find('[')? + 1..memory.rfind(']')?];
            // x64 scaled index: `[rbp + rcx*4 + 0x18]`.
            if abi == Abi::X86_64 && inner.contains('*') {
                let parts = inner.split('+').map(str::trim).collect::<Vec<_>>();
                let (base, index, displacement) = match parts.as_slice() {
                    [base, index] => (*base, *index, 0),
                    [base, index, displacement] => (*base, *index, signed_immediate(displacement)?),
                    _ => return None,
                };
                let (index, scale) = index.split_once('*')?;
                if normalize_register(base) != abi_frame_register(abi) || scale.trim() != "4" {
                    return None;
                }
                let index_text = text_of(index.trim())?;
                if let Some(entry) = named_entry_of(index_text) {
                    if displacement != word {
                        return None;
                    }
                    return Some((destination, stack.get(&format!("named:{entry}"))?.clone()));
                }
                let fixed = fixed_of(index_text)?;
                return Some((destination, parameter(fixed, displacement)?));
            }
            let (base, displacement) = arm_memory_address(memory)?;
            let base_text = registers.get(&base).map(|value| value.text.as_str())?;
            if base_text == "aot.argumentsDescriptor" {
                // Descriptor elements are compressed words after the array
                // header: count (1), then (name, position) pairs from 4.
                let count = arguments_descriptor(abi).1;
                let element = (displacement - count) / 4 + 1;
                if (displacement - count) % 4 != 0 || element < 1 {
                    return None;
                }
                return match element {
                    1 => Some((destination, named("aot.argumentCount".to_owned()))),
                    element if element >= 4 => {
                        let entry = (element - 4) / 2;
                        let kind = if (element - 4) % 2 == 0 { "Name" } else { "Position" };
                        Some((destination, named(format!("aot.namedArgument{kind}({entry})"))))
                    }
                    _ => None,
                };
            }
            if let Some(entry) = base_text
                .strip_prefix("aot.namedParameterBase(")
                .and_then(|rest| rest.strip_suffix(')'))
            {
                // The matched named argument sits one word above the base.
                if displacement != word {
                    return None;
                }
                return Some((destination, stack.get(&format!("named:{entry}"))?.clone()));
            }
            let fixed = base_text
                .strip_prefix("aot.parameterBase(")?
                .strip_suffix(')')?
                .parse::<i64>()
                .ok()?;
            Some((destination, parameter(fixed, displacement)?))
        }
        // ARM64 `add C, x29, wB, sxtw #2`; ARM32 `add C, fp, rB, lsl #1`:
        // the frame base for `count - fixed` supplied words.
        "add" if operands.len() == 4
            && normalize_register(&operands[1]) == abi_frame_register(abi) =>
        {
            let shift = operands[3].replace(' ', "");
            let expected = match abi {
                Abi::Arm64V8a => "sxtw#2",
                Abi::ArmeabiV7a => "lsl#1",
                Abi::X86_64 => return None,
            };
            if shift != expected {
                return None;
            }
            let index_text = text_of(&operands[2])?;
            if let Some(entry) = named_entry_of(index_text) {
                return Some((destination, named(format!("aot.namedParameterBase({entry})"))));
            }
            let fixed = fixed_of(index_text)?;
            Some((destination, named(format!("aot.parameterBase({fixed})"))))
        }
        // Decompressing a descriptor name or position keeps its meaning:
        // ARM64 `add x, x, x28, lsl #32`, x64 `add r, r, [r14 + heap base]`.
        "add" if operands.len() >= 3
            && text_of(&operands[1]).is_some_and(|text| text.starts_with("aot.namedArgument"))
            && (normalize_register(&operands[2]) == "x28"
                || arm_memory_address(&operands[2])
                    == Some(("r14".to_owned(), X64_THREAD_HEAP_BASE_OFFSET))) =>
        {
            Some((destination, registers.get(&normalize_register(&operands[1]))?.clone()))
        }
        // x64 widens the Smi before indexing.
        "movsxd" if operands.len() == 2 => {
            let value = registers.get(&normalize_register(&operands[1]))?;
            if fixed_of(&value.text).is_none() && named_entry_of(&value.text).is_none() {
                return None;
            }
            Some((destination, value.clone()))
        }
        _ => None,
    }
}

/// `(aot.argumentCount - aot.namedArgumentPosition(k))` -> `k`.
fn named_entry_of(text: &str) -> Option<&str> {
    text.strip_prefix("(aot.argumentCount - aot.namedArgumentPosition(")?
        .strip_suffix("))")
}

/// Records which parameter name the prologue compares with the descriptor's
/// `k`-th named entry, so the matching path's value load names it.
fn note_named_argument_compare(
    mnemonic: &str,
    operands: &[String],
    registers: &BTreeMap<String, Expression>,
    object_pool: Option<&[String]>,
    stack: &mut BTreeMap<String, Expression>,
) {
    if mnemonic != "cmp" || operands.len() < 2 {
        return;
    }
    let value = |operand: &str| {
        registers
            .get(&normalize_register(operand))
            .cloned()
            .or_else(|| resolve_expression(operand, registers, object_pool))
    };
    let (Some(left), Some(right)) = (value(&operands[0]), value(&operands[1])) else {
        return;
    };
    let (entry, name) = match (
        left.text.strip_prefix("aot.namedArgumentName("),
        right.text.strip_prefix("aot.namedArgumentName("),
    ) {
        (Some(entry), None) => (entry, right.text),
        (None, Some(entry)) => (entry, left.text),
        _ => return,
    };
    let Some(entry) = entry.strip_suffix(')') else {
        return;
    };
    // Pool strings print quoted; the parameter's name is the string.
    let Some(name) = name
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .filter(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$'))
    else {
        return;
    };
    stack.insert(
        format!("named:{entry}"),
        Expression {
            text: name.to_owned(),
            confidence: EvidenceConfidence::High,
            complexity: 1,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        },
    );
}

/// A merged value whose assignments are exactly one optional parameter and
/// one constant is that parameter with its default applied: the prologue's
/// missing-argument path stores `DefaultParameterValueAt`. Replaces the
/// merged value by the parameter and returns the defaults found.
fn extract_parameter_defaults(
    statements: &mut Vec<SemanticStatement>,
    parameters: &BTreeMap<String, usize>,
) -> BTreeMap<usize, String> {
    let mut values = BTreeMap::<String, BTreeSet<String>>::new();
    for statement in statements.iter() {
        if let SemanticStatement::Assign {
            variable, value, ..
        } = statement
        {
            values
                .entry(variable.clone())
                .or_default()
                .insert(value.clone());
        }
    }
    let mut defaults = BTreeMap::new();
    for (variable, values) in values {
        let values = values.into_iter().collect::<Vec<_>>();
        let [first, second] = values.as_slice() else {
            continue;
        };
        let (parameter, constant) = if parameters.contains_key(first) && is_literal(second) {
            (first, second)
        } else if parameters.contains_key(second) && is_literal(first) {
            (second, first)
        } else {
            continue;
        };
        defaults.insert(parameters[parameter], constant.clone());
        statements.retain(|statement| {
            !matches!(statement, SemanticStatement::Assign { variable: assigned, .. } if *assigned == variable)
        });
        for statement in statements.iter_mut() {
            rename_in_statement(statement, &variable, parameter);
        }
    }
    defaults
}

fn is_literal(text: &str) -> bool {
    matches!(text, "null" | "true" | "false")
        || text.parse::<f64>().is_ok()
        || (text.starts_with('\'') && text.ends_with('\'')
            || text.len() >= 2 && text.starts_with('"') && text.ends_with('"'))
            && !text.contains("${")
        || text.starts_with("snapshotInstance(") && text.ends_with(')')
}

/// Symbols for the suspend stubs this body calls at its suspension points,
/// named from its async modifier. The stub takes the awaited or yielded
/// value in `SuspendStubABI::kArgumentReg` and returns the resumed value.
fn suspend_stub_symbols(
    abi: Abi,
    decoded: &[DecodedInstruction],
    suspensions: &BTreeSet<u64>,
    modifier: Option<crate::model::AsyncModifier>,
) -> Option<BTreeMap<u64, Symbol>> {
    use crate::model::AsyncModifier;
    let label = match modifier {
        Some(AsyncModifier::Async) => "stub Await",
        Some(AsyncModifier::SyncStar) => "stub SuspendSyncStar",
        // An async* suspension is either an await or a yield.
        Some(AsyncModifier::AsyncStar) => "stub SuspendAsyncStar",
        _ => return None,
    };
    let argument = match abi {
        Abi::Arm64V8a => "x0",
        Abi::ArmeabiV7a => "r0",
        Abi::X86_64 => "rax",
    };
    let stubs = decoded
        .iter()
        .filter(|instruction| suspensions.contains(&instruction.address))
        .filter_map(|instruction| direct_call_target(&instruction.mnemonic, &instruction.operands))
        .map(|target| {
            let mut symbol = Symbol::new(label.to_owned(), None, None).with_code_identity(
                target,
                0,
                crate::model::DirectCallResolution::ExactEntry,
            );
            symbol.parameters = Some(std::sync::Arc::from(vec![ArgumentLocation::Register(
                argument,
            )]));
            (target, symbol)
        })
        .collect::<BTreeMap<_, _>>();
    (!stubs.is_empty()).then_some(stubs)
}

/// Incoming parameter locations of `function`'s compiled body, as the
/// lifter seeds them.
pub(crate) fn machine_interface(
    function: &crate::model::RecoveredFunction,
    abi: Abi,
    parameter_hints: &[ParameterHint],
) -> Option<crate::model::MachineInterface> {
    let decoded = function
        .instructions
        .iter()
        .filter_map(|instruction| {
            let address = parse_immediate(&instruction.address)?;
            (!is_skipped_data(&instruction.mnemonic)).then(|| DecodedInstruction {
                address,
                next: address.saturating_add((instruction.bytes.len() / 2) as u64),
                mnemonic: instruction.mnemonic.clone(),
                operands: instruction.operands.clone(),
            })
        })
        .collect::<Vec<_>>();
    if decoded.is_empty() {
        return None;
    }
    let convention = ConventionInput::for_function(abi, function);
    let (evidence, returns_fpu) = decoded_body_evidence(abi, &decoded, convention.entry_offset);
    let seeds = resolve_parameter_seeds(abi, parameter_hints, &convention, &evidence);
    let parameter_count_known = convention
        .declared
        .as_ref()
        .is_some_and(|declared| declared.len() == parameter_hints.len());
    Some(crate::model::MachineInterface {
        parameters: seeds
            .iter()
            .map(|seed| crate::model::MachineParameter {
                name: seed.hint.name.clone(),
                location: match seed.location {
                    ArgumentLocation::Register(register)
                    | ArgumentLocation::FpuRegister(register) => register.to_owned(),
                    ArgumentLocation::RegisterPair(low, high) => format!("{low}:{high}"),
                    ArgumentLocation::Stack { word, .. } => format!("stack[{word}]"),
                },
                representation: match seed.representation {
                    Representation::Tagged => "tagged",
                    Representation::UnboxedInt64 => "unboxed_int64",
                    Representation::UnboxedDouble => "unboxed_double",
                },
                proof: match seed.proof {
                    LocationProof::Proven => "proven",
                    LocationProof::Assumed => "assumed",
                },
            })
            .collect(),
        returns_unboxed_double: returns_fpu == Some(true),
        parameter_count_known,
    })
}

#[derive(Clone, Debug)]
struct RecoveredFieldIdentity {
    name: String,
    value_class: Option<String>,
    value_library_uri: Option<String>,
    /// True when the field name was not recovered and the access is a
    /// slot-offset placeholder for a proven receiver class.
    synthesized_slot: bool,
    /// Declared type display name from the exact Field object.
    declared_type: Option<String>,
}

/// VM-verified instance layouts keyed by declaring class and byte offset.
///
/// Field offsets are only meaningful inside a particular class. Keeping them
/// globally keyed by offset can silently turn an Array store at `+0x10` into a
/// completely unrelated application field. This index deliberately requires a
/// receiver-class proof before assigning a field name.
///
/// Names come only from exact Field metadata (`Field::TargetOffsetOf`).
/// Class slots known only from the unboxed-field bitmap stay anonymous
/// (`_slot_<offset>`) and are marked synthesized, so they never outrank or
/// masquerade as a named field. Declaration order is not used to invent
/// offsets: the compiler lays out superclass fields first, reuses inherited
/// type-argument storage, sizes slots by unboxing decisions the snapshot
/// does not record per field, and surviving Field objects can be a subset.
#[derive(Clone, Debug, Default)]
pub(crate) struct RecoveredFieldLayout {
    fields: BTreeMap<(Option<String>, String, i64), RecoveredFieldIdentity>,
    /// `(library, class) -> (library, superclass)` for inherited fields.
    superclasses: BTreeMap<(Option<String>, String), (Option<String>, String)>,
    /// Static fields keyed by `(shared table, byte offset in the table)`.
    /// `None` marks an id claimed by two different Field objects.
    statics: BTreeMap<(bool, i64), Option<StaticFieldIdentity>>,
    /// Thread offsets of the field tables, known only for an exact profile.
    field_tables: Option<FieldTableLayout>,
}

#[derive(Clone, Debug, PartialEq)]
struct StaticFieldIdentity {
    /// `Owner.name`, or `name` for a top-level field.
    name: String,
    owner: Option<String>,
    value_class: Option<String>,
    value_library_uri: Option<String>,
}

/// `Thread::field_table_values_offset`, its shared counterpart, and the
/// width of one field-table entry (`FieldTable::OffsetOf`), from
/// `runtime_offsets_extracted.h` of the exact SDK revision.
#[derive(Clone, Copy, Debug)]
struct FieldTableLayout {
    values: i64,
    shared_values: i64,
    word: i64,
    /// `AOT_Thread_object_null_offset`, `bool_true_offset` and
    /// `bool_false_offset`: canonical objects cached in the thread.
    null: i64,
    bool_true: i64,
    bool_false: i64,
}

/// One static-field access recognised from `ldr t, [THR, #table];
/// ldr/str v, [t, #id * word]`.
struct StaticFieldAccess {
    name: String,
    offset: i64,
    word: i64,
    value_class: Option<String>,
    value_library_uri: Option<String>,
    /// False for an anonymous `_static_<id>` slot.
    named: bool,
}

impl RecoveredFieldLayout {
    /// Builds the layout index from recovered Field declarations and class
    /// instance slots. Accepts declarations from every scope: field names are
    /// only promoted when the receiver class is proven, so out-of-scope
    /// Flutter/Dart SDK layouts are safe enrichment evidence.
    pub(crate) fn from_declarations(
        _abi: Abi,
        declarations: &[crate::model::RecoveredDeclaration],
    ) -> Self {
        use crate::model::RecoveredDeclarationKind;
        let mut layouts = Self::default();
        for declaration in declarations {
            match declaration.kind {
                RecoveredDeclarationKind::Field => {
                    if let Some(metadata) = declaration
                        .field_metadata
                        .as_ref()
                        .filter(|metadata| metadata.is_static)
                    {
                        if let Some(offset) = metadata.static_field_offset {
                            let declared_type = metadata.declared_type.as_ref();
                            let owner = declaration
                                .owner
                                .as_deref()
                                .filter(|owner| !matches!(*owner, "::" | "top_level"))
                                .map(crate::analysis::readable_snapshot_name);
                            let field_name =
                                crate::analysis::readable_snapshot_name(&declaration.name);
                            layouts.insert_static(
                                metadata.is_shared,
                                offset,
                                StaticFieldIdentity {
                                    name: owner.as_ref().map_or(field_name.clone(), |owner| {
                                        format!("{owner}.{field_name}")
                                    }),
                                    owner,
                                    value_class: declared_type
                                        .and_then(|value| simple_class_type(&value.display_name)),
                                    value_library_uri: declared_type
                                        .and_then(|value| value.library_uri.clone()),
                                },
                            );
                        }
                        continue;
                    }
                    let (Some(owner), Some(offset)) = (
                        declaration.owner.as_deref(),
                        declaration
                            .field_metadata
                            .as_ref()
                            .filter(|metadata| !metadata.is_static)
                            .and_then(|metadata| metadata.instance_field_offset),
                    ) else {
                        continue;
                    };
                    let declared_type = declaration
                        .field_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.declared_type.as_ref());
                    let value_class =
                        declared_type.and_then(|value| simple_class_type(&value.display_name));
                    let value_library_uri =
                        declared_type.and_then(|value| value.library_uri.clone());
                    let class_name = crate::analysis::readable_snapshot_name(owner);
                    let field_name = crate::analysis::readable_snapshot_name(&declaration.name);
                    layouts.insert(
                        declaration.library_uri.clone(),
                        class_name.clone(),
                        offset,
                        field_name.clone(),
                        value_class,
                        value_library_uri,
                    );
                    if let Some(identity) = layouts.fields.get_mut(&(
                        declaration.library_uri.clone(),
                        class_name,
                        offset,
                    )) && identity.name == field_name
                    {
                        identity.declared_type =
                            declared_type.map(|value| value.display_name.clone());
                    }
                }
                RecoveredDeclarationKind::Class => {
                    let Some(metadata) = declaration.class_metadata.as_ref() else {
                        continue;
                    };
                    let class_name = crate::analysis::readable_snapshot_name(&declaration.name);
                    if let Some(super_type) = metadata.super_type.as_ref()
                        && let Some(super_name) = simple_class_type(&super_type.display_name)
                    {
                        layouts.superclasses.insert(
                            (declaration.library_uri.clone(), class_name.clone()),
                            (super_type.library_uri.clone(), super_name),
                        );
                    }
                    for slot in &metadata.instance_slots {
                        if slot.slot_type == "type_arguments_field" {
                            continue;
                        }
                        match slot.field_name.as_deref() {
                            Some(name) => layouts.insert(
                                declaration.library_uri.clone(),
                                class_name.clone(),
                                slot.offset,
                                crate::analysis::readable_snapshot_name(name),
                                None,
                                None,
                            ),
                            None => layouts.insert_anonymous(
                                declaration.library_uri.clone(),
                                class_name.clone(),
                                slot.offset,
                            ),
                        }
                    }
                }
                RecoveredDeclarationKind::Function => {}
            }
        }
        layouts
    }

    fn insert_static(&mut self, shared: bool, offset: i64, identity: StaticFieldIdentity) {
        match self.statics.get(&(shared, offset)) {
            None => {
                self.statics.insert((shared, offset), Some(identity));
            }
            Some(Some(existing)) if *existing == identity => {}
            Some(_) => {
                self.statics.insert((shared, offset), None);
            }
        }
    }

    /// Enables static-field recognition for snapshots whose thread layout is
    /// known. `profile` is the validated root profile (`dart-3.12.2` for the
    /// exact revision, `dart-3.<minor>` for a release family); the offsets are
    /// the PRODUCT AOT `Thread` offsets from each release's
    /// `runtime_offsets_extracted.h`, identical across its patch releases.
    /// Other profiles keep static accesses as raw loads.
    pub(crate) fn with_exact_thread_layout(mut self, abi: Abi, profile: Option<&str>) -> Self {
        // [values, shared_values, null, true, false] for (32-bit ARM,
        // 64-bit compressed-pointer targets). Dart 3.4 had no shared table.
        const NONE: i64 = i64::MIN;
        let offsets: Option<([i64; 5], [i64; 5])> = match profile {
            Some("dart-3.12.2" | "dart-3.11" | "dart-3.12") => Some((
                [0x38, 0x3c, 0x40, 0x48, 0x4c],
                [0x78, 0x80, 0x88, 0x98, 0xa0],
            )),
            Some("dart-3.10") => Some((
                [0x38, 0x3c, 0x40, 0x44, 0x48],
                [0x78, 0x80, 0x88, 0x90, 0x98],
            )),
            Some("dart-3.5" | "dart-3.6" | "dart-3.7" | "dart-3.8" | "dart-3.9") => Some((
                [0x30, 0x34, 0x38, 0x3c, 0x40],
                [0x68, 0x70, 0x78, 0x80, 0x88],
            )),
            Some("dart-3.4") => Some((
                [0x30, NONE, 0x34, 0x38, 0x3c],
                [0x68, NONE, 0x70, 0x78, 0x80],
            )),
            _ => None,
        };
        self.field_tables = offsets.map(|(arm32, compressed)| {
            let (offsets, word) = match abi {
                Abi::ArmeabiV7a => (arm32, 4),
                Abi::Arm64V8a | Abi::X86_64 => (compressed, 8),
            };
            FieldTableLayout {
                values: offsets[0],
                shared_values: offsets[1],
                word,
                null: offsets[2],
                bool_true: offsets[3],
                bool_false: offsets[4],
            }
        });
        self
    }

    /// `Some(shared)` when `[thread, #displacement]` loads a field-table base.
    fn field_table_at(&self, displacement: i64) -> Option<bool> {
        let tables = self.field_tables?;
        if displacement == tables.values {
            Some(false)
        } else if displacement == tables.shared_values {
            Some(true)
        } else {
            None
        }
    }

    /// Canonical constant the thread caches at `displacement`.
    fn thread_constant(&self, displacement: i64) -> Option<(&'static str, Option<&'static str>)> {
        let tables = self.field_tables?;
        if displacement == tables.null {
            Some(("null", None))
        } else if displacement == tables.bool_true {
            Some(("true", Some("bool")))
        } else if displacement == tables.bool_false {
            Some(("false", Some("bool")))
        } else if displacement == tables.bool_false + tables.word {
            // `empty_array_` follows `bool_false_` in `CACHED_NON_VM_STUB_LIST`
            // on every supported release.
            Some(("const []", Some("_ImmutableList")))
        } else {
            None
        }
    }

    /// The static field at `offset` of a field table. Ids no Field object
    /// names become anonymous `_static_<id>` slots; ids claimed by two
    /// different fields are refused.
    fn static_field(&self, shared: bool, offset: i64) -> Option<StaticFieldAccess> {
        let word = self.field_tables?.word;
        if offset < 0 || offset % word != 0 {
            return None;
        }
        match self.statics.get(&(shared, offset)) {
            Some(Some(identity)) => Some(StaticFieldAccess {
                name: identity.name.clone(),
                offset,
                word,
                value_class: identity.value_class.clone(),
                value_library_uri: identity.value_library_uri.clone(),
                named: true,
            }),
            Some(None) => None,
            None => Some(StaticFieldAccess {
                name: format!(
                    "aot.staticField({}{})",
                    offset / word,
                    if shared { ", shared: true" } else { "" }
                ),
                offset,
                word,
                value_class: None,
                value_library_uri: None,
                named: false,
            }),
        }
    }

    /// Records an exactly named field. It replaces an anonymous slot at the
    /// same place and never adds a second offset for a name already placed
    /// exactly in this class.
    pub(crate) fn insert(
        &mut self,
        library_uri: Option<String>,
        class_name: String,
        offset: i64,
        field_name: String,
        value_class: Option<String>,
        value_library_uri: Option<String>,
    ) {
        let key = (library_uri, class_name, offset);
        if let Some(existing) = self.fields.get(&key)
            && !existing.synthesized_slot
        {
            return;
        }
        let already_placed = self.fields.iter().any(|((library, class, _), identity)| {
            *library == key.0
                && *class == key.1
                && !identity.synthesized_slot
                && identity.name == field_name
        });
        if already_placed {
            return;
        }
        self.fields.insert(
            key,
            RecoveredFieldIdentity {
                name: field_name,
                value_class,
                value_library_uri,
                synthesized_slot: false,
                declared_type: None,
            },
        );
    }

    /// Name and declared type of the field an exact Field object places at
    /// `offset` in `class_name` or a superclass. Anonymous slots yield
    /// `None`.
    pub(crate) fn exact_field(
        &self,
        class_name: &str,
        library_uri: Option<&str>,
        offset: i64,
    ) -> Option<(&str, Option<&str>)> {
        let (found, identity) = self.field(class_name, library_uri, offset)?;
        (found == offset && !identity.synthesized_slot)
            .then_some((identity.name.as_str(), identity.declared_type.as_deref()))
    }

    /// Records a slot whose storage is proven but whose name is not.
    fn insert_anonymous(&mut self, library_uri: Option<String>, class_name: String, offset: i64) {
        self.fields
            .entry((library_uri, class_name, offset))
            .or_insert_with(|| RecoveredFieldIdentity {
                name: format!("_slot_{:x}", u64::try_from(offset).unwrap_or_default()),
                value_class: None,
                value_library_uri: None,
                synthesized_slot: true,
                declared_type: None,
            });
    }

    /// Field at `displacement` of an object whose class is `class_name`,
    /// searching superclasses for inherited fields.
    /// Whether a class of this name is declared outside the SDK.
    fn has_application_class(&self, class_name: &str) -> bool {
        let application = |library: &Option<String>, name: &String| {
            name == class_name
                && library
                    .as_deref()
                    .is_some_and(|library| !library.starts_with("dart:"))
        };
        self.fields
            .keys()
            .any(|(library, name, _)| application(library, name))
            || self
                .superclasses
                .keys()
                .any(|(library, name)| application(library, name))
    }

    fn field(
        &self,
        class_name: &str,
        library_uri: Option<&str>,
        displacement: i64,
    ) -> Option<(i64, &RecoveredFieldIdentity)> {
        let mut class = (library_uri.map(str::to_owned), class_name.to_owned());
        let mut visited = BTreeSet::new();
        let mut anonymous = None;
        while visited.insert(class.clone()) {
            if let Some(found) = self.declared_field(&class.1, class.0.as_deref(), displacement) {
                if !found.1.synthesized_slot {
                    return Some(found);
                }
                // A subclass bitmap slot may cover a field a superclass
                // declares by name; keep looking before settling for it.
                anonymous.get_or_insert(found);
            }
            let Some(parent) = self.superclasses.get(&class).cloned().or_else(|| {
                // Library-less lookups accept a unique superclass link.
                let mut parents = self
                    .superclasses
                    .iter()
                    .filter(|((_, name), _)| *name == class.1)
                    .map(|(_, parent)| parent);
                let first = parents.next()?;
                parents.all(|other| other == first).then(|| first.clone())
            }) else {
                break;
            };
            class = parent;
        }
        anonymous
    }

    fn declared_field(
        &self,
        class_name: &str,
        library_uri: Option<&str>,
        displacement: i64,
    ) -> Option<(i64, &RecoveredFieldIdentity)> {
        for offset in [displacement, displacement.saturating_add(1)] {
            if let Some(identity) = self.fields.get(&(
                library_uri.map(str::to_owned),
                class_name.to_owned(),
                offset,
            )) {
                return Some((offset, identity));
            }

            // Object-pool labels do not always carry the declaring library.
            // Accept a class-name-only lookup only when every matching layout
            // agrees on the recovered field identity.
            let mut matches = self
                .fields
                .iter()
                .filter(|((_, owner, candidate_offset), _)| {
                    owner == class_name && *candidate_offset == offset
                })
                .map(|(_, identity)| identity);
            let Some(first) = matches.next() else {
                continue;
            };
            if matches.all(|candidate| {
                candidate.name == first.name
                    && candidate.value_class == first.value_class
                    && candidate.value_library_uri == first.value_library_uri
            }) {
                return Some((offset, first));
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
struct DispatchCallEvidence {
    selector_offset: usize,
    selector_name: Option<String>,
    candidate_targets: Vec<String>,
    candidate_count: usize,
    raw_slot_target_count: usize,
}

fn recover_dispatch_calls(
    abi: Abi,
    instructions: &[DecodedInstruction],
    table: &DispatchTableAnalysis<'_>,
) -> BTreeMap<u64, DispatchCallEvidence> {
    match abi {
        Abi::ArmeabiV7a => return recover_arm32_dispatch_calls(instructions, table),
        Abi::X86_64 => return recover_x64_dispatch_calls(instructions, table),
        Abi::Arm64V8a => {}
    }
    let mut constants = BTreeMap::<String, i64>::new();
    let mut selectors = BTreeMap::<String, i64>::new();
    let mut call_registers = BTreeMap::<String, usize>::new();
    let mut recovered = BTreeMap::new();

    for instruction in instructions {
        let operands = split_operands(&instruction.operands);
        match instruction.mnemonic.as_str() {
            "mov" | "movz" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                if let Some(value) = signed_immediate(&operands[1])
                    .or_else(|| constants.get(&normalize_register(&operands[1])).copied())
                {
                    constants.insert(target.clone(), value);
                } else {
                    constants.remove(&target);
                }
                selectors.remove(&target);
                call_registers.remove(&target);
            }
            "movk" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                let shift = operands
                    .get(2)
                    .and_then(|operand| shift_amount(operand))
                    .unwrap_or(0);
                if let Some(value) = signed_immediate(&operands[1])
                    && shift < 64
                {
                    let mask = !(0xffffi64 << shift);
                    let previous = constants.get(&target).copied().unwrap_or_default();
                    constants.insert(target.clone(), (previous & mask) | (value << shift));
                } else {
                    constants.remove(&target);
                }
                selectors.remove(&target);
                call_registers.remove(&target);
            }
            "add" | "sub" if operands.len() >= 3 => {
                let target = normalize_register(&operands[0]);
                let source = normalize_register(&operands[1]);
                let mut value = signed_immediate(&operands[2])
                    .or_else(|| constants.get(&normalize_register(&operands[2])).copied());
                if let Some(shift) = operands.get(3).and_then(|operand| shift_amount(operand))
                    && shift < 63
                {
                    value = value.and_then(|value| value.checked_shl(shift));
                }
                if source == "x0"
                    && let Some(mut delta) = value
                {
                    if instruction.mnemonic == "sub" {
                        delta = delta.saturating_neg();
                    }
                    selectors.insert(target.clone(), delta);
                } else {
                    selectors.remove(&target);
                }
                constants.remove(&target);
                call_registers.remove(&target);
            }
            "ldr" if operands.len() >= 2 && operands[1].to_ascii_lowercase().contains("[x21") => {
                let target = normalize_register(&operands[0]);
                let memory = operands[1].to_ascii_lowercase();
                let selector = selectors
                    .iter()
                    .find(|(register, _)| {
                        memory
                            .split(|character: char| !character.is_ascii_alphanumeric())
                            .any(|token| token == register.as_str())
                    })
                    .and_then(|(_, delta)| {
                        i64::try_from(table.origin_element)
                            .ok()?
                            .checked_add(*delta)
                            .and_then(|offset| usize::try_from(offset).ok())
                    });
                if let Some(selector) = selector {
                    call_registers.insert(target.clone(), selector);
                } else {
                    call_registers.remove(&target);
                }
                constants.remove(&target);
                selectors.remove(&target);
            }
            mnemonic if is_call(mnemonic) => {
                if let Some(register) = operands.first().map(|operand| normalize_register(operand))
                    && let Some(selector_offset) = call_registers.get(&register).copied()
                {
                    recovered.insert(
                        instruction.address,
                        dispatch_call_evidence(selector_offset, table),
                    );
                }
                constants.clear();
                selectors.clear();
                call_registers.clear();
            }
            mnemonic if branch_kind(mnemonic).is_some() => {
                constants.clear();
                selectors.clear();
                call_registers.clear();
            }
            mnemonic if writes_first_operand(mnemonic) => {
                if let Some(target) = operands.first().map(|operand| normalize_register(operand)) {
                    constants.remove(&target);
                    selectors.remove(&target);
                    call_registers.remove(&target);
                }
            }
            _ => {}
        }
    }
    recovered
}

/// x64 `EmitDispatchTableCall`: the table base is reloaded from the thread
/// into a scratch register and the call indexes it with the class id in
/// RCX (`DispatchTableNullErrorABI::kClassIdReg`):
///
///   mov rax, qword ptr [r14 + <dispatch_table_array_offset>]
///   call qword ptr [rax + rcx*8 + <(selector - kOriginElement) * 8>]
///
/// The thread offset differs across SDK revisions, so any thread-slot load
/// feeding the call's base register qualifies; the `rcx*8` index and the
/// thread-relative base together do not occur for other call shapes.
fn recover_x64_dispatch_calls(
    instructions: &[DecodedInstruction],
    table: &DispatchTableAnalysis<'_>,
) -> BTreeMap<u64, DispatchCallEvidence> {
    let mut thread_loaded = BTreeSet::<String>::new();
    let mut recovered = BTreeMap::new();
    for instruction in instructions {
        let operands = split_operands(&instruction.operands);
        if is_call(&instruction.mnemonic) {
            if let Some(selector_offset) = operands
                .first()
                .and_then(|operand| x64_dispatch_selector(operand, &thread_loaded, table))
            {
                recovered.insert(
                    instruction.address,
                    dispatch_call_evidence(selector_offset, table),
                );
            }
            thread_loaded.clear();
            continue;
        }
        if branch_kind(&instruction.mnemonic).is_some()
            || is_return(&instruction.mnemonic, &instruction.operands)
        {
            thread_loaded.clear();
            continue;
        }
        let Some(destination) = operands
            .first()
            .filter(|_| writes_first_operand(&instruction.mnemonic))
            .map(|value| normalize_register(value))
        else {
            continue;
        };
        let from_thread = instruction.mnemonic == "mov"
            && operands.len() == 2
            && operands[1].contains("qword ptr")
            && arm_memory_address(&operands[1]).is_some_and(|(base, _)| base == "r14");
        if from_thread {
            thread_loaded.insert(destination);
        } else {
            thread_loaded.remove(&destination);
        }
    }
    recovered
}

fn x64_dispatch_selector(
    operand: &str,
    thread_loaded: &BTreeSet<String>,
    table: &DispatchTableAnalysis<'_>,
) -> Option<usize> {
    let start = operand.find('[')?;
    let end = operand[start + 1..].find(']')? + start + 1;
    let inner = operand[start + 1..end].replace(' ', "");
    // `rax+rcx*8+0x24488` or `rax+rcx*8-0x80`.
    let (base, rest) = inner.split_once('+')?;
    let rest = rest.strip_prefix("rcx*8")?;
    if !thread_loaded.contains(&normalize_register(base)) {
        return None;
    }
    let displacement = if rest.is_empty() {
        0
    } else {
        signed_immediate(rest)?
    };
    if displacement % 8 != 0 {
        return None;
    }
    i64::try_from(table.origin_element)
        .ok()?
        .checked_add(displacement / 8)
        .and_then(|selector| usize::try_from(selector).ok())
}

fn recover_arm32_dispatch_calls(
    instructions: &[DecodedInstruction],
    table: &DispatchTableAnalysis<'_>,
) -> BTreeMap<u64, DispatchCallEvidence> {
    // ARM32 dedicates r7 to the class dispatch table. Large negative selector
    // offsets are encoded as an affine two-instruction address:
    //
    //   add lr, r7, r0, lsl #2
    //   ldr lr, [lr, #-0x734]
    //   blx lr
    //
    // r0 is the receiver class ID, so r7 + r0*4 denotes selector zero and the
    // signed load displacement selects the row. A second, compact form loads
    // directly from [r7, r0, lsl #2]. Only these compiler-shaped sequences are
    // accepted, and all state is killed at calls/branches.
    let mut dispatch_addresses = BTreeMap::<String, i64>::new();
    let mut call_registers = BTreeMap::<String, usize>::new();
    let mut recovered = BTreeMap::new();

    for instruction in instructions {
        let operands = split_operands(&instruction.operands);
        match instruction.mnemonic.as_str() {
            "add"
                if operands.len() >= 4
                    && normalize_register(&operands[1]) == "r7"
                    && normalize_register(&operands[2]) == "r0"
                    && shift_amount(&operands[3]) == Some(2) =>
            {
                let target = normalize_register(&operands[0]);
                dispatch_addresses.insert(target.clone(), 0);
                call_registers.remove(&target);
            }
            // Offsets beyond the 12-bit load immediate are split:
            // `add lr, lr, #0x12000` before the load (`AddImmediate`).
            "add" | "sub"
                if operands.len() == 3
                    && dispatch_addresses.contains_key(&normalize_register(&operands[1]))
                    && signed_immediate(&operands[2]).is_some() =>
            {
                let source = normalize_register(&operands[1]);
                let target = normalize_register(&operands[0]);
                let adjust = signed_immediate(&operands[2]).unwrap_or_default();
                let adjust = if instruction.mnemonic == "sub" {
                    -adjust
                } else {
                    adjust
                };
                let base = dispatch_addresses.get(&source).copied().unwrap_or_default();
                dispatch_addresses.insert(target.clone(), base + adjust);
                call_registers.remove(&target);
            }
            "ldr" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                let selector = arm32_dispatch_selector(&operands[1], &dispatch_addresses, table);
                if let Some(selector) = selector {
                    call_registers.insert(target.clone(), selector);
                } else {
                    call_registers.remove(&target);
                }
                dispatch_addresses.remove(&target);
            }
            mnemonic if is_call(mnemonic) => {
                if let Some(register) = operands.first().map(|operand| normalize_register(operand))
                    && let Some(selector_offset) = call_registers.get(&register).copied()
                {
                    recovered.insert(
                        instruction.address,
                        dispatch_call_evidence(selector_offset, table),
                    );
                }
                dispatch_addresses.clear();
                call_registers.clear();
            }
            mnemonic
                if branch_kind(mnemonic).is_some()
                    || is_return(mnemonic, &instruction.operands) =>
            {
                dispatch_addresses.clear();
                call_registers.clear();
            }
            mnemonic if writes_first_operand(mnemonic) => {
                if let Some(target) = operands.first().map(|operand| normalize_register(operand)) {
                    dispatch_addresses.remove(&target);
                    call_registers.remove(&target);
                }
            }
            _ => {}
        }
    }
    recovered
}

fn arm32_dispatch_selector(
    memory: &str,
    dispatch_addresses: &BTreeMap<String, i64>,
    table: &DispatchTableAnalysis<'_>,
) -> Option<usize> {
    let start = memory.find('[')?;
    let end = memory[start + 1..].find(']')?.saturating_add(start + 1);
    let parts = split_operands(memory.get(start + 1..end)?);
    let base = normalize_register(parts.first()?);
    let byte_delta = if base == "r7"
        && parts.len() >= 3
        && normalize_register(&parts[1]) == "r0"
        && shift_amount(&parts[2]) == Some(2)
    {
        0
    } else {
        let base_delta = dispatch_addresses.get(&base).copied()?;
        let displacement = parts.get(1).and_then(|operand| signed_immediate(operand))?;
        base_delta.checked_add(displacement)?
    };
    if byte_delta % 4 != 0 {
        return None;
    }
    i64::try_from(table.origin_element)
        .ok()?
        .checked_add(byte_delta / 4)
        .and_then(|selector| usize::try_from(selector).ok())
}

fn dispatch_call_evidence(
    selector_offset: usize,
    table: &DispatchTableAnalysis<'_>,
) -> DispatchCallEvidence {
    let raw_targets = table
        .class_ids
        .iter()
        .filter_map(|class_id| {
            let index = selector_offset.checked_add(*class_id)?;
            (table.slot_fits_receiver(index, *class_id) != Some(false))
                .then(|| table.targets.get(index))
                .flatten()
        })
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let (selector_name, candidate_targets, candidate_count) = infer_dispatch_selector(&raw_targets);
    DispatchCallEvidence {
        selector_offset,
        selector_name,
        candidate_targets,
        candidate_count,
        raw_slot_target_count: raw_targets.len(),
    }
}

/// Infers a dispatch-call selector from the table slots a call site can reach
/// (`selector_offset + class_id` for every known populated CID).
///
/// Slot multiplicity is misleading: a selector implemented by one class fills
/// exactly one slot, while a shared helper can occupy hundreds of identical
/// displaced rows. Grouping by raw slot count therefore lets one widely-shared
/// implementation outvote the true selector. Every slot resolving to the same
/// Code label is collapsed first; inference then works over *distinct
/// implementations*:
///
/// - exactly one implementation → its member name is the selector, proven;
/// - otherwise a member name wins only when it names a strict majority
///   (>= 2:1) of the distinct implementations and at least three survive
///   with readable (non-synthetic) names.
fn infer_dispatch_selector(raw_targets: &[String]) -> (Option<String>, Vec<String>, usize) {
    let mut implementations = BTreeSet::<String>::new();
    for target in raw_targets {
        if !target.is_empty() {
            implementations.insert(target.clone());
        }
    }
    if implementations.is_empty() {
        return (None, Vec::new(), 0);
    }
    if implementations.len() == 1 {
        let target = implementations.into_iter().next().expect("one entry");
        let selector = target
            .rsplit_once('.')
            .map_or(target.as_str(), |(_, name)| name)
            .to_owned();
        return (Some(selector), vec![target], 1);
    }
    let mut groups = BTreeMap::<String, std::collections::BTreeSet<String>>::new();
    for target in &implementations {
        let selector = target
            .rsplit_once('.')
            .map_or(target.as_str(), |(_, name)| name);
        groups
            .entry(selector.to_owned())
            .or_default()
            .insert(target.clone());
    }
    let mut ranked = groups.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .len()
            .cmp(&left.1.len())
            .then_with(|| left.0.cmp(&right.0))
    });
    let Some((selector, targets)) = ranked.first() else {
        return (None, Vec::new(), 0);
    };
    // Unanimous member name across every distinct implementation: the classic
    // polymorphic shape (every class names its override identically).
    if ranked.len() == 1 {
        let readable = !selector.starts_with("sub_")
            && !selector.starts_with("_iso_stub_")
            && !selector.starts_with("stub_")
            && !selector.is_empty();
        if readable {
            return (
                Some(selector.clone()),
                targets.iter().take(16).cloned().collect(),
                implementations.len(),
            );
        }
        return (None, Vec::new(), implementations.len());
    }
    let runner_up = ranked.get(1).map_or(0, |group| group.1.len());
    let readable = !selector.starts_with("sub_")
        && !selector.starts_with("_iso_stub_")
        && !selector.starts_with("stub_")
        && !selector.is_empty();
    // Disputed names need a strict 2:1 majority over the runner-up, at least
    // three implementations carrying the winning name, and enough coverage
    // that the sweep did not just graze unrelated selectors sharing the
    // offset window (a 5-of-105 win is noise from an obfuscated table).
    let dominant = readable
        && targets.len() >= 3
        && targets.len() >= runner_up.saturating_mul(2)
        && targets.len().saturating_mul(4) >= implementations.len();
    if !dominant {
        // No provable selector name. Distinct non-synthetic implementations
        // remain bounded evidence; purely opaque sweeps stay silent instead of
        // dressing synthetic labels up as candidates.
        let readable_impls = implementations
            .iter()
            .filter(|target| {
                let member = target
                    .rsplit_once('.')
                    .map_or(target.as_str(), |(_, name)| name);
                !member.starts_with("sub_")
                    && !member.starts_with("_iso_stub_")
                    && !member.starts_with("stub_")
                    && !member.is_empty()
            })
            .take(16)
            .cloned()
            .collect::<Vec<_>>();
        return (None, readable_impls, implementations.len());
    }
    (
        Some(selector.clone()),
        targets.iter().take(16).cloned().collect(),
        implementations.len(),
    )
}

fn signed_immediate(value: &str) -> Option<i64> {
    immediate_text(value)?.parse().ok()
}

fn shift_amount(value: &str) -> Option<u32> {
    value
        .trim()
        .strip_prefix("lsl")
        .map(str::trim)
        .and_then(signed_immediate)
        .and_then(|value| u32::try_from(value).ok())
}

/// Tracks registers that are provably affine aliases of Dart's object-pool
/// pointer. ARM32 and ARM64 cannot always encode the full offset of a large
/// pool in one load, so the AOT compiler commonly emits:
///
/// `add r8, r5, #0x21000; ldr r3, [r8, #0x687]`
/// `add x1, x27, #0x14, lsl #12; ldr x1, [x1, #0x7c0]`
///
/// Treating the two instructions independently loses every pool entry above
/// the immediate-load window. The map stores the byte delta from the fixed
/// pool pointer and is deliberately cleared at calls and branches rather than
/// merging unproven values across control-flow joins.
struct PoolPointerProvenance {
    abi: Abi,
    derived_offsets: BTreeMap<String, i64>,
    /// ARM32 registers holding a `movw`/`movt` constant, which the compiler
    /// adds to PP when an offset does not fit an immediate.
    constants: BTreeMap<String, i64>,
}

impl PoolPointerProvenance {
    fn new(abi: Abi) -> Self {
        Self {
            abi,
            derived_offsets: BTreeMap::new(),
            constants: BTreeMap::new(),
        }
    }

    fn load_index(&self, mnemonic: &str, operands: &str) -> Option<usize> {
        if !is_pool_load(self.abi, mnemonic, operands) {
            return None;
        }
        if let Some(index) = object_pool_index(self.abi, operands) {
            return Some(index);
        }
        if !matches!(self.abi, Abi::ArmeabiV7a | Abi::Arm64V8a) {
            return None;
        }
        let memory = split_operands(operands).get(1)?.to_owned();
        let (base, displacement) = arm_memory_address(&memory)?;
        let base_offset = self.derived_offsets.get(&base)?;
        pool_offset_to_index(self.abi, base_offset.checked_add(displacement)?)
    }

    fn observe(&mut self, mnemonic: &str, operands: &str) {
        if is_call(mnemonic) || branch_kind(mnemonic).is_some() || is_return(mnemonic, operands) {
            self.derived_offsets.clear();
            self.constants.clear();
            return;
        }
        let operands = split_operands(operands);
        let Some(target) = operands.first().map(|operand| normalize_register(operand)) else {
            return;
        };
        if self.abi == Abi::ArmeabiV7a && operands.len() == 2 {
            let immediate = signed_immediate(&operands[1]);
            match (mnemonic, immediate) {
                ("movw" | "mov", Some(value)) => {
                    self.derived_offsets.remove(&target);
                    self.constants.insert(target, value & 0xffff_ffff);
                    return;
                }
                ("movt", Some(value)) => {
                    self.derived_offsets.remove(&target);
                    match self.constants.get(&target).copied() {
                        Some(low) => {
                            let word = ((value & 0xffff) << 16) | (low & 0xffff);
                            self.constants.insert(target, i64::from(word as u32 as i32));
                        }
                        None => {
                            self.constants.remove(&target);
                        }
                    }
                    return;
                }
                _ => {}
            }
        }
        let register_delta = operands
            .get(2)
            .and_then(|operand| self.constants.get(&normalize_register(operand)).copied());
        if writes_first_operand(mnemonic) {
            self.constants.remove(&target);
        }
        if matches!(self.abi, Abi::ArmeabiV7a | Abi::Arm64V8a)
            && matches!(mnemonic, "add" | "sub")
            && operands.len() >= 3
        {
            let source = normalize_register(&operands[1]);
            let pool_pointer = match self.abi {
                Abi::ArmeabiV7a => "r5",
                Abi::Arm64V8a => "x27",
                Abi::X86_64 => "",
            };
            let source_offset = if source == pool_pointer {
                Some(0)
            } else {
                self.derived_offsets.get(&source).copied()
            };
            if let (Some(source_offset), Some(mut delta)) = (
                source_offset,
                signed_immediate(&operands[2]).or(register_delta),
            ) {
                if self.abi == Abi::Arm64V8a
                    && let Some(shift) = operands.get(3).and_then(|value| shift_amount(value))
                {
                    let Some(shifted) = delta.checked_shl(shift) else {
                        self.derived_offsets.remove(&target);
                        return;
                    };
                    delta = shifted;
                }
                if mnemonic == "sub" {
                    delta = delta.saturating_neg();
                }
                if let Some(offset) = source_offset.checked_add(delta) {
                    self.derived_offsets.insert(target, offset);
                    return;
                }
            }
        } else if matches!(mnemonic, "mov" | "mov.w") && operands.len() >= 2 {
            let source = normalize_register(&operands[1]);
            let pool_pointer = match self.abi {
                Abi::ArmeabiV7a => "r5",
                Abi::Arm64V8a => "x27",
                Abi::X86_64 => "",
            };
            let source_offset = if source == pool_pointer {
                Some(0)
            } else {
                self.derived_offsets.get(&source).copied()
            };
            if let Some(offset) = source_offset {
                self.derived_offsets.insert(target, offset);
                return;
            }
        }
        if writes_first_operand(mnemonic) {
            self.derived_offsets.remove(&target);
        }
    }
}

#[derive(Clone, Debug)]
struct DecodedInstruction {
    address: u64,
    next: u64,
    mnemonic: String,
    operands: String,
}

#[derive(Clone, Debug)]
struct Expression {
    text: String,
    confidence: EvidenceConfidence,
    complexity: usize,
    class_name: Option<String>,
    class_library_uri: Option<String>,
    /// True when the value is an untagged machine integer (a Smi/Mint
    /// payload). Dart `int` values render identically either way, so the
    /// untag step itself never appears in recovered source.
    raw: bool,
    /// True when `class_name` is the value's exact runtime class (a fresh
    /// allocation or a canonical constant) rather than a static type that
    /// subclasses and implementers also satisfy.
    exact_class: bool,
    /// Instruction that defined a call result. Distinct calls with the same
    /// display name must not merge as one value.
    definition_site: Option<u64>,
    /// ARM32 only: the register holds the high word of the 64-bit integer
    /// `text` names, whose low word travels in another register. Dart keeps
    /// unboxed `int` values in register pairs on 32-bit targets; the pair is
    /// one source value, so the high half never surfaces on its own.
    high_word: bool,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.confidence == other.confidence
            && self.class_name == other.class_name
            && self.class_library_uri == other.class_library_uri
            && self.raw == other.raw
            && self.exact_class == other.exact_class
            && self.high_word == other.high_word
            && self.definition_site == other.definition_site
    }
}

/// An array being filled element-by-element immediately after an allocation
/// stub call. Dart AOT lowers string interpolation to exactly this shape:
/// allocate an array, store literal parts and value expressions into the
/// element slots, then call `String._interpolate`.
///
/// A buffer is identified by its allocation call site. Registers, frame
/// slots and outgoing argument slots refer to it through
/// [`FlowState::aliases`], so a spill, reload or derived element pointer
/// never forks the contents.
#[derive(Clone, Debug, PartialEq)]
struct ElementBuffer {
    /// The allocation's result text, which element writes name.
    array: String,
    /// Element count from the allocation's Smi length argument, when it was
    /// a constant. Slots past the stored ones stay explicit gaps.
    length: Option<usize>,
    parts: Vec<Option<String>>,
    /// Weakest confidence among the stored element values.
    confidence: EvidenceConfidence,
}

impl ElementBuffer {
    fn new(array: String, length: Option<usize>) -> Self {
        ElementBuffer {
            array,
            length,
            parts: Vec::new(),
            confidence: EvidenceConfidence::High,
        }
    }

    /// Records `value` at element `index`. An unknown value is kept as an
    /// explicit gap, and it weakens the buffer like any low-confidence part.
    fn store(&mut self, index: usize, value: Option<&Expression>) {
        if self.parts.len() <= index {
            self.parts.resize(index + 1, None);
        }
        self.parts[index] = value
            .filter(|value| !value.text.is_empty())
            .map(|value| value.text.clone());
        self.confidence = weaker(
            self.confidence,
            value.map_or(EvidenceConfidence::Low, |value| value.confidence),
        );
    }
}

fn stack_key(slot: &str) -> String {
    format!("stk:{slot}")
}

fn outgoing_key(slot: i64) -> String {
    format!("out:{slot}")
}

/// Register/stack/buffer state carried across instructions. States meet at
/// control-flow joins by intersection: a value survives only when every
/// predecessor agrees on identical provenance.
#[derive(Clone, Debug, Default, PartialEq)]
struct FlowState {
    registers: BTreeMap<String, Expression>,
    stack: BTreeMap<String, Expression>,
    /// Tracked element buffers by allocation call site.
    buffers: BTreeMap<u64, ElementBuffer>,
    /// Locations pointing into a tracked buffer -> (allocation site, byte
    /// displacement from the tagged array pointer). Keys are register
    /// names, `stk:<slot>` for frame slots and `out:<offset>` for outgoing
    /// argument slots. A nonzero displacement is a derived element pointer
    /// such as `add dst, arrayBase, #offset`.
    aliases: BTreeMap<String, (u64, i64)>,
    /// Outgoing stack arguments for the next call: byte displacement from
    /// the stack pointer -> stored value. Dart AOT passes argument zero in
    /// a register and pushes the remaining arguments right-to-left so the
    /// last argument sits at `[SP]`.
    outgoing: BTreeMap<i64, Expression>,
    /// Bitmask of argument registers this body wrote since the last call
    /// (bit i = CPU argument register i, bit 8 + i = FPU argument register
    /// i). Calls whose callee convention is unknown may only report
    /// those registers as arguments when the caller itself established them;
    /// the incoming seeds exist for callee-side reads, not for calls.
    written_argument_registers: u16,
    /// Stack- and frame-pointer displacements from the entry stack pointer.
    deltas: FrameDeltas,
    /// Registers holding a receiver's object header word: register -> the
    /// receiver expression whose header was loaded.
    tag_words: BTreeMap<String, Expression>,
    /// Registers holding a receiver's class id extracted from its header:
    /// register -> the receiver expression. Dispatch-table and switchable
    /// calls are resolved only through the receiver this proves.
    class_ids: BTreeMap<String, Expression>,
    /// Static fields read on some path to this point. A value naming one is
    /// a snapshot of that field; it goes stale when the field is written.
    static_reads: BTreeSet<String>,
}

/// Displacements of the stack and frame pointers from the stack pointer at
/// the normal entry, when every path agrees on them. Stack slots are keyed
/// relative to the entry stack pointer so the same memory has one name
/// whether it is addressed through SP or FP, before or after the prologue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameDeltas {
    sp: Option<i64>,
    fp: Option<i64>,
}

impl Default for FrameDeltas {
    fn default() -> Self {
        Self {
            sp: Some(0),
            fp: None,
        }
    }
}

impl FrameDeltas {
    fn meet(self, other: Self) -> Self {
        Self {
            sp: self.sp.filter(|delta| other.sp == Some(*delta)),
            fp: self.fp.filter(|delta| other.fp == Some(*delta)),
        }
    }

    /// Deltas after `mnemonic operands` executes.
    fn after(self, abi: Abi, mnemonic: &str, operands: &str) -> Self {
        let layout = TargetLayout::of(abi);
        let word = layout.word_size;
        let parts = split_operands(operands);
        let mentions_sp = |part: &str| {
            arm_memory_address(part).is_some_and(|(base, _)| base == layout.stack_pointer)
        };
        let braced = |part: &str| {
            part.trim_matches(|c| c == '{' || c == '}')
                .split(',')
                .filter(|register| !register.trim().is_empty())
                .count() as i64
        };
        let mut next = self;
        match (abi, mnemonic) {
            (Abi::X86_64, "push" | "pushq") => next.sp = self.sp.map(|sp| sp - word),
            (Abi::X86_64, "pop" | "popq") => next.sp = self.sp.map(|sp| sp + word),
            (Abi::ArmeabiV7a, "push" | "vpush") | (Abi::ArmeabiV7a, "pop" | "vpop") => {
                let width = if mnemonic.starts_with('v') { 8 } else { word };
                let count = parts.iter().map(|part| braced(part)).sum::<i64>();
                let delta = if mnemonic.ends_with("push") {
                    -count * width
                } else {
                    count * width
                };
                next.sp = self.sp.map(|sp| sp + delta);
            }
            (Abi::ArmeabiV7a, _)
                if (mnemonic.starts_with("stm") || mnemonic.starts_with("ldm"))
                    && parts.first().is_some_and(|base| {
                        base.ends_with('!')
                            && normalize_register(base.trim_end_matches('!'))
                                == layout.stack_pointer
                    }) =>
            {
                let count = parts.iter().skip(1).map(|part| braced(part)).sum::<i64>();
                let descending = mnemonic.starts_with("stm");
                next.sp = self.sp.map(|sp| {
                    sp + if descending {
                        -count * word
                    } else {
                        count * word
                    }
                });
            }
            _ => {
                // Writeback addressing is ARM-only; x64 `mov [rsp], 5` is a
                // plain store.
                if abi != Abi::X86_64
                    && let Some(part) = parts.iter().find(|part| mentions_sp(part))
                {
                    if part.ends_with('!') {
                        // Pre-index writeback: SP += displacement.
                        let displacement = arm_memory_address(part).map_or(0, |(_, value)| value);
                        next.sp = self.sp.map(|sp| sp + displacement);
                    } else if let Some(post) = parts
                        .iter()
                        .position(|candidate| candidate == part)
                        .and_then(|index| parts.get(index + 1))
                        .filter(|_| operands.contains("], "))
                        .and_then(|value| signed_immediate(value))
                    {
                        next.sp = self.sp.map(|sp| sp + post);
                    }
                }
                // A memory destination such as `[rsp]` is a store, not a
                // register write.
                let destination = parts
                    .first()
                    .filter(|value| writes_first_operand(mnemonic) && !value.contains('['))
                    .map(|value| normalize_register(value));
                let source = |index: usize| parts.get(index).map(|value| normalize_register(value));
                let immediate =
                    |index: usize| parts.get(index).and_then(|value| signed_immediate(value));
                // Two-operand x64 forms read their destination.
                let (lhs, rhs) = if abi == Abi::X86_64 { (0, 1) } else { (1, 2) };
                if destination.as_deref() == Some(layout.stack_pointer) {
                    let based_on_sp = source(lhs).as_deref() == Some(layout.stack_pointer);
                    next.sp = match mnemonic {
                        "sub" | "subs" if based_on_sp => {
                            immediate(rhs).and_then(|value| self.sp.map(|sp| sp - value))
                        }
                        "add" | "adds" if based_on_sp => {
                            immediate(rhs).and_then(|value| self.sp.map(|sp| sp + value))
                        }
                        "mov" if source(1).as_deref() == Some(layout.frame_pointer) => self.fp,
                        "add" | "sub"
                            if abi != Abi::X86_64
                                && source(1).as_deref() == Some(layout.frame_pointer) =>
                        {
                            immediate(2).and_then(|value| {
                                self.fp.map(|fp| {
                                    if mnemonic == "add" {
                                        fp + value
                                    } else {
                                        fp - value
                                    }
                                })
                            })
                        }
                        _ => None,
                    };
                }
                if destination.as_deref() == Some(layout.frame_pointer) {
                    next.fp = match mnemonic {
                        "mov" if source(1).as_deref() == Some(layout.stack_pointer) => next.sp,
                        "add"
                            if abi != Abi::X86_64
                                && source(1).as_deref() == Some(layout.stack_pointer) =>
                        {
                            immediate(2).and_then(|value| next.sp.map(|sp| sp + value))
                        }
                        _ => None,
                    };
                }
                // `ldp x29, x30, [x15], #16` restores the caller's frame.
                if matches!(mnemonic, "ldp" | "pop")
                    && parts
                        .iter()
                        .any(|part| normalize_register(part) == layout.frame_pointer)
                {
                    next.fp = None;
                }
            }
        }
        if abi != Abi::X86_64
            && mnemonic == "pop"
            && parts
                .iter()
                .any(|part| part.contains("fp") || part.contains("r11"))
        {
            next.fp = None;
        }
        if abi == Abi::X86_64
            && matches!(mnemonic, "pop" | "popq")
            && parts
                .first()
                .is_some_and(|part| normalize_register(part) == layout.frame_pointer)
        {
            next.fp = None;
        }
        next
    }
}

/// Stack-slot key for a byte displacement from the entry stack pointer.
fn entry_slot_key(displacement: i64) -> String {
    format!("[entry,#{displacement}]")
}

impl FlowState {
    fn meet(left: &FlowState, right: &FlowState) -> FlowState {
        Self::meet_at(left, right, None)
    }

    /// Meets two predecessor states at the block starting at `join`. With a
    /// join address, a register or stack slot that every predecessor defines
    /// but with different values becomes one merged value named
    /// `phi_<join>_<location>`; the emission pass assigns it at the end of
    /// each predecessor. Without one, disagreeing values are dropped.
    fn meet_at(left: &FlowState, right: &FlowState, join: Option<(u64, Abi)>) -> FlowState {
        fn merge_equal<V: Clone + PartialEq>(
            left: &BTreeMap<String, V>,
            right: &BTreeMap<String, V>,
        ) -> BTreeMap<String, V> {
            left.iter()
                .filter_map(|(key, value)| {
                    right
                        .get(key)
                        .filter(|other| **other == *value)
                        .map(|_| (key.clone(), value.clone()))
                })
                .collect()
        }
        let merge_expressions =
            |left: &BTreeMap<String, Expression>, right: &BTreeMap<String, Expression>| {
                left.iter()
                    .filter_map(|(key, value)| {
                        right
                            .get(key)
                            .filter(|other| **other == *value)
                            .map(|other| {
                                (
                                    key.clone(),
                                    Expression {
                                        complexity: value.complexity.min(other.complexity),
                                        ..value.clone()
                                    },
                                )
                            })
                    })
                    .collect::<BTreeMap<_, _>>()
            };
        let merge_values = |left: &BTreeMap<String, Expression>,
                            right: &BTreeMap<String, Expression>| {
            let Some((join, _)) = join else {
                return merge_expressions(left, right);
            };
            left.iter()
                .filter_map(|(key, value)| {
                    let other = right.get(key)?;
                    if other == value {
                        return Some((
                            key.clone(),
                            Expression {
                                complexity: value.complexity.min(other.complexity),
                                ..value.clone()
                            },
                        ));
                    }
                    Some((key.clone(), phi_expression(join, key, value, other)))
                })
                .collect::<BTreeMap<_, _>>()
        };
        let merge_outgoing = |left: &BTreeMap<i64, Expression>,
                              right: &BTreeMap<i64, Expression>| {
            left.iter()
                .filter_map(|(key, value)| {
                    right
                        .get(key)
                        .filter(|other| **other == *value)
                        .map(|other| {
                            (
                                *key,
                                Expression {
                                    complexity: value.complexity.min(other.complexity),
                                    ..value.clone()
                                },
                            )
                        })
                })
                .collect::<BTreeMap<_, _>>()
        };
        // Element buffers merge part-wise: slots holding the same text on
        // every path survive; disagreeing or missing slots become explicit
        // gaps so an interpolation built across a branch keeps its proven
        // literal parts instead of being discarded wholesale.
        let mut buffers = BTreeMap::new();
        for (site, left_buffer) in &left.buffers {
            if let Some(right_buffer) = right.buffers.get(site) {
                let len = left_buffer.parts.len().max(right_buffer.parts.len());
                let mut parts = Vec::with_capacity(len);
                let mut confidence = weaker(left_buffer.confidence, right_buffer.confidence);
                for index in 0..len {
                    let l = left_buffer.parts.get(index).cloned().flatten();
                    let r = right_buffer.parts.get(index).cloned().flatten();
                    if l.is_some() && l == r {
                        parts.push(l);
                    } else {
                        parts.push(None);
                        confidence = EvidenceConfidence::Low;
                    }
                }
                buffers.insert(
                    *site,
                    ElementBuffer {
                        array: left_buffer.array.clone(),
                        length: (left_buffer.length == right_buffer.length)
                            .then_some(left_buffer.length)
                            .flatten(),
                        parts,
                        confidence,
                    },
                );
            }
        }
        let registers = merge_values(&left.registers, &right.registers);
        // A merged register was established before the join, not in the
        // straight-line code leading to the next call, so it is not an
        // outgoing argument of an unknown callee.
        let mut written_argument_registers =
            left.written_argument_registers & right.written_argument_registers;
        if let Some((join, abi)) = join {
            let layout = TargetLayout::of(abi);
            for (register, value) in &registers {
                if !is_phi_of(value, join) {
                    continue;
                }
                if let Some(bit) = layout.cpu_argument_index(register) {
                    written_argument_registers &= !(1 << bit);
                } else if let Some(bit) = layout.fpu_argument_index(register) {
                    written_argument_registers &= !(1 << (8 + bit));
                }
            }
        }
        FlowState {
            registers,
            stack: merge_values(&left.stack, &right.stack),
            buffers,
            aliases: merge_equal(&left.aliases, &right.aliases),
            outgoing: merge_outgoing(&left.outgoing, &right.outgoing),
            written_argument_registers,
            deltas: left.deltas.meet(right.deltas),
            tag_words: merge_expressions(&left.tag_words, &right.tag_words),
            class_ids: merge_expressions(&left.class_ids, &right.class_ids),
            static_reads: left
                .static_reads
                .union(&right.static_reads)
                .cloned()
                .collect(),
        }
    }
}

/// Identifier naming the merged value of `location` at the block `join`.
fn phi_name(join: u64, location: &str) -> String {
    let location = location
        .chars()
        .map(|character| match character {
            '-' => 'm',
            character if character.is_ascii_alphanumeric() => character,
            _ => '_',
        })
        .collect::<String>();
    format!("phi_{join:x}_{location}")
}

fn is_phi_of(value: &Expression, join: u64) -> bool {
    value.definition_site == Some(join) && value.text.starts_with("phi_")
}

fn phi_expression(join: u64, location: &str, left: &Expression, right: &Expression) -> Expression {
    Expression {
        text: phi_name(join, location),
        confidence: weaker(left.confidence, right.confidence),
        complexity: 1,
        class_name: (left.class_name == right.class_name)
            .then(|| left.class_name.clone())
            .flatten(),
        class_library_uri: (left.class_name == right.class_name
            && left.class_library_uri == right.class_library_uri)
            .then(|| left.class_library_uri.clone())
            .flatten(),
        raw: left.raw && right.raw,
        high_word: left.high_word && right.high_word,
        exact_class: left.exact_class && right.exact_class && left.class_name == right.class_name,
        definition_site: Some(join),
    }
}

/// Stack-pointer register for each supported ABI.
fn abi_stack_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x15",
        Abi::ArmeabiV7a => "r13",
        Abi::X86_64 => "rsp",
    }
}

/// `AOT_Thread_heap_base_offset` for PRODUCT x64 with compressed pointers
/// (`runtime_offsets_extracted.h`). Decompressing a 32-bit reference adds
/// this thread slot.
const X64_THREAD_HEAP_BASE_OFFSET: i64 = 0x58;

/// Frame-pointer register for each supported ABI.
fn abi_frame_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x29",
        Abi::ArmeabiV7a => "r11",
        Abi::X86_64 => "rbp",
    }
}

fn mentions_any(text: &str, names: &BTreeSet<String>) -> bool {
    names.iter().any(|name| text.contains(name.as_str()))
}

/// Canonical name of a scalar FPU register: ARM64 `vN.16b`/`qN`/`sN`
/// spell the register the lifter tracks as `dN`; other names pass through.
fn fpu_register(operand: &str) -> String {
    let value = operand.trim().to_ascii_lowercase();
    let stem = value.split('.').next().unwrap_or_default();
    if let Some(index) = stem
        .strip_prefix('v')
        .or_else(|| stem.strip_prefix('q'))
        .filter(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
    {
        return format!("d{index}");
    }
    normalize_register(&value)
}

fn is_vector_register(operand: &str) -> bool {
    let value = operand.trim().to_ascii_lowercase();
    value.starts_with("xmm")
        || value
            .strip_prefix('v')
            .and_then(|rest| rest.split('.').next())
            .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// Value of an FPU arithmetic operand: a register, or an x64 memory operand
/// reloading a spilled double.
fn resolve_fpu_operand(
    operand: &str,
    registers: &BTreeMap<String, Expression>,
    stack: &BTreeMap<String, Expression>,
    abi: Abi,
    deltas: FrameDeltas,
    object_pool: Option<&[String]>,
) -> Option<Expression> {
    if operand.contains('[') {
        return stack.get(&stack_slot_key(abi, operand, deltas)?).cloned();
    }
    registers
        .get(&fpu_register(operand))
        .cloned()
        .or_else(|| resolve_expression(operand, registers, object_pool))
}

/// A call to one of the object-store stubs that run a static field's
/// initializer and return its value.
fn calls_static_initializer(target: Option<u64>, symbols: &BTreeMap<u64, Symbol>) -> bool {
    target
        .and_then(|target| symbols.get(&target))
        .is_some_and(|symbol| {
            matches!(
                symbol.label.as_str(),
                "stub InitStaticField"
                    | "stub InitLateStaticField"
                    | "stub InitLateFinalStaticField"
                    | "stub InitSharedLateStaticField"
            )
        })
}

fn calls_instance_initializer(target: Option<u64>, symbols: &BTreeMap<u64, Symbol>) -> bool {
    target
        .and_then(|target| symbols.get(&target))
        .is_some_and(|symbol| {
            matches!(
                symbol.label.as_str(),
                "stub InitInstanceField"
                    | "stub InitLateInstanceField"
                    | "stub InitLateFinalInstanceField"
            )
        })
}

/// Thread register for each supported ABI.
fn abi_thread_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x26",
        Abi::ArmeabiV7a => "r10",
        Abi::X86_64 => "r14",
    }
}

/// A word-sized register load or store through `[base, #displacement]`
/// without writeback or an index register.
struct MemoryAccess {
    /// Loaded register, or stored operand.
    value: String,
    base: String,
    displacement: i64,
    store: bool,
}

fn plain_memory_operands(abi: Abi, mnemonic: &str, operands: &[String]) -> Option<MemoryAccess> {
    let (value, memory, store) = match (abi, mnemonic) {
        (Abi::X86_64, "mov" | "movq") if operands.len() == 2 => {
            if operands[0].contains('[') {
                (&operands[1], &operands[0], true)
            } else if operands[1].contains('[') {
                (&operands[0], &operands[1], false)
            } else {
                return None;
            }
        }
        (Abi::Arm64V8a | Abi::ArmeabiV7a, "ldr" | "ldur") if operands.len() == 2 => {
            (&operands[0], &operands[1], false)
        }
        (Abi::Arm64V8a | Abi::ArmeabiV7a, "str" | "stur") if operands.len() == 2 => {
            (&operands[0], &operands[1], true)
        }
        _ => return None,
    };
    if value.contains('[') || memory.contains('!') {
        return None;
    }
    // Full-width accesses only: `qword ptr` on x64, X registers on ARM64.
    let full_width = match abi {
        Abi::X86_64 => memory.contains("qword"),
        Abi::Arm64V8a => normalize_register(value).starts_with('x'),
        Abi::ArmeabiV7a => true,
    };
    if !full_width {
        return None;
    }
    let (base, displacement) = arm_memory_address(memory)?;
    // Reject register-indexed addressing such as `[r0, r1]` or `[rax + rcx*8]`.
    if memory.contains('*') || split_operands(&memory[memory.find('[')? + 1..]).len() > 2 {
        return None;
    }
    Some(MemoryAccess {
        value: value.clone(),
        base,
        displacement,
        store,
    })
}

/// Formats a memory-slot key the way Capstone prints displacements, in both
/// decimal and hexadecimal so either spelling resolves to the same slot.
fn slot_keys(base: &str, displacement: i64) -> Vec<String> {
    if displacement == 0 {
        return vec![format!("[{base}]"), format!("[{base},#0]")];
    }
    let (operator, magnitude) = if displacement < 0 {
        ('-', displacement.unsigned_abs())
    } else {
        ('+', displacement as u64)
    };
    vec![
        format!("[{base},#{displacement}]"),
        format!("[{base},#{displacement:#x}]"),
        format!("[{base},#{displacement:x}]"),
        format!("[{base}{operator}{magnitude}]"),
        format!("[{base}{operator}{magnitude:#x}]"),
    ]
}

/// x64 stack-overflow guard: `cmp rsp, qword ptr [thr + limit]; jbe slow`
/// with `slow = call stub; jmp back`. Deletes the two guarding instructions.
#[allow(clippy::too_many_arguments)]
fn fuse_x64_stack_guard(
    index: usize,
    input: &[DecodedInstruction],
    index_of: &BTreeMap<u64, usize>,
    _symbols: &BTreeMap<u64, Symbol>,
    keep: &mut [bool],
) {
    let branch = &input[index + 1];
    if branch.mnemonic != "jbe" && branch.mnemonic != "jb" && branch.mnemonic != "jnae" {
        return;
    }
    let Some(slow_address) = branch_target(&branch.operands) else {
        return;
    };
    let Some(slow_index) = index_of.get(&slow_address).copied() else {
        return;
    };
    if input
        .get(slow_index)
        .is_none_or(|slow| slow.mnemonic != "call")
    {
        return;
    }
    // The slow path may call the stub directly or through the thread's stub
    // table (`call qword ptr [thr + slot]`); both are runtime plumbing.
    let jumps_back = input.get(slow_index + 1).is_some_and(|back| {
        back.mnemonic == "jmp" && branch_target(&back.operands) == Some(branch.next)
    });
    if !jumps_back {
        return;
    }
    keep[index] = false;
    keep[index + 1] = false;
}

/// Replays the ARM immediate-building instructions (`mov`/`movk` on ARM64,
/// `movw`/`movt` on ARM32) that can initialize an object header register.
/// Returning `None` on any other write keeps the allocation fusion
/// conservative.
fn loaded_immediate(
    input: &[DecodedInstruction],
    start: usize,
    end: usize,
    register: &str,
) -> Option<u64> {
    let mut constant = None;
    for instruction in input.get(start..end)? {
        let operands = split_operands(&instruction.operands);
        if operands
            .first()
            .map(|operand| normalize_register(operand))
            .as_deref()
            != Some(register)
        {
            continue;
        }
        match instruction.mnemonic.as_str() {
            "mov" | "movz" if operands.len() >= 2 => {
                let value = u64::try_from(signed_immediate(&operands[1])?).ok()?;
                let shift = operands
                    .get(2)
                    .and_then(|operand| shift_amount(operand))
                    .unwrap_or(0);
                constant = value.checked_shl(shift);
            }
            "movk" if operands.len() >= 2 => {
                let previous = constant?;
                let value = u64::try_from(signed_immediate(&operands[1])?).ok()?;
                let shift = operands
                    .get(2)
                    .and_then(|operand| shift_amount(operand))
                    .unwrap_or(0);
                if shift >= 64 {
                    return None;
                }
                let mask = !(0xffff_u64 << shift);
                constant = Some((previous & mask) | ((value & 0xffff) << shift));
            }
            "movw" if operands.len() >= 2 => {
                constant = u64::try_from(signed_immediate(&operands[1])?)
                    .ok()
                    .map(|value| value & 0xffff);
            }
            "movt" if operands.len() >= 2 => {
                let previous = constant?;
                let value = u64::try_from(signed_immediate(&operands[1])?).ok()?;
                constant = Some((previous & 0xffff) | ((value & 0xffff) << 16));
            }
            mnemonic if writes_first_operand(mnemonic) => constant = None,
            _ => {}
        }
    }
    constant
}

/// Dart predefined class IDs are stable across VM builds (`class_id.h`). The
/// object header stores the 20-bit CID at bit 12 and a four-bit size in units
/// of object alignment (two native words) at bit 8. A boxed Double is CID 62
/// and 16 bytes.
fn is_boxed_double_tag(abi: Abi, tag: u64) -> bool {
    const CLASS_ID_SHIFT: u32 = 12;
    const CLASS_ID_MASK: u64 = (1 << 20) - 1;
    const SIZE_TAG_SHIFT: u32 = 8;
    const SIZE_TAG_MASK: u64 = 0xf;
    const DOUBLE_CID: u64 = 62;
    const DOUBLE_SIZE: u64 = 16;
    let alignment = 2 * TargetLayout::of(abi).word_size as u64;

    ((tag >> CLASS_ID_SHIFT) & CLASS_ID_MASK) == DOUBLE_CID
        && ((tag >> SIZE_TAG_SHIFT) & SIZE_TAG_MASK) == DOUBLE_SIZE / alignment
}

/// An inline `Double` box: the fast bump allocation starting at the thread
/// top load, and the slow stub path that rejoins at the value store.
struct InlineDoubleBox {
    /// Register holding the tagged box.
    result: String,
    /// FPU register whose value is boxed.
    value: String,
    /// Last instruction of the value store.
    value_store_end: usize,
    slow_index: usize,
    /// The slow path's branch back to the value store.
    slow_end: usize,
}

/// Matches the `BoxInstr` fast path for a Double on every ABI:
///
/// ```text
/// ARM64:  ldp r, lim, [thr, #top]; add r, r, #16; cmp lim, r; b.ls slow
/// ARM32:  ldr r, [thr, #top]; add r, r, #16; ldr lim, [thr, #end];
///         cmp lim, r; bls slow
/// x64:    mov r, [thr + top]; add r, 0x10; cmp r, [thr + end]; jae slow
/// then:   store r -> [thr, #top]; sub r, r, #15; store header -> [r, #-1];
///         store dN -> [r, #7]  (ARM32: add ip, r, #k; vstr dN, [ip, #7-k])
/// ```
///
/// The slow path must call a runtime stub, move its result into `r` and
/// branch back to the value store.
fn match_inline_double_box(
    abi: Abi,
    input: &[DecodedInstruction],
    index: usize,
    index_of: &BTreeMap<u64, usize>,
    is_runtime_stub_call: &dyn Fn(usize) -> bool,
) -> Option<InlineDoubleBox> {
    let thread = match abi {
        Abi::Arm64V8a => "x26",
        Abi::ArmeabiV7a => "r10",
        Abi::X86_64 => "r14",
    };
    let x64 = abi == Abi::X86_64;
    let operands = |at: usize| split_operands(&input[at].operands);
    let register = |at: usize, position: usize| {
        operands(at)
            .get(position)
            .map(|value| normalize_register(value))
    };
    let immediate =
        |at: usize, position: usize| operands(at).get(position).and_then(|v| signed_immediate(v));
    let memory = |at: usize, position: usize| {
        operands(at)
            .get(position)
            .and_then(|v| arm_memory_address(v))
    };
    // (stored operand, (base, displacement)) for a store instruction.
    let store = |at: usize| -> Option<(String, (String, i64))> {
        let parts = operands(at);
        match input[at].mnemonic.as_str() {
            "str" | "stur" | "vstr" if !x64 => {
                Some((parts.first()?.clone(), arm_memory_address(parts.last()?)?))
            }
            "mov" | "movq" | "movsd" if x64 && parts.first()?.contains('[') => {
                Some((parts.get(1)?.clone(), arm_memory_address(&parts[0])?))
            }
            _ => None,
        }
    };
    let mnemonic = |at: usize| {
        input
            .get(at)
            .map(|instruction| instruction.mnemonic.as_str())
    };

    // 1. Load the thread's allocation top.
    let (result, top_offset) = match (abi, mnemonic(index)?) {
        (Abi::Arm64V8a, "ldp") => (register(index, 0)?, memory(index, 2)?),
        (Abi::ArmeabiV7a, "ldr") | (Abi::X86_64, "mov") => (register(index, 0)?, memory(index, 1)?),
        _ => return None,
    };
    let (base, top_offset) = top_offset;
    if base != thread {
        return None;
    }
    // 2. Bump by the instance size.
    let mut cursor = index + 1;
    let bumps = mnemonic(cursor)? == "add"
        && register(cursor, 0).as_deref() == Some(result.as_str())
        && if x64 {
            immediate(cursor, 1) == Some(16)
        } else {
            register(cursor, 1).as_deref() == Some(result.as_str())
                && immediate(cursor, 2) == Some(16)
        };
    if !bumps {
        return None;
    }
    cursor += 1;
    // 3. Compare with the allocation end and branch to the slow path.
    if abi == Abi::ArmeabiV7a
        && mnemonic(cursor)? == "ldr"
        && memory(cursor, 1).is_some_and(|(base, _)| base == thread)
    {
        cursor += 1;
    }
    if mnemonic(cursor)? != "cmp"
        || !operands(cursor)
            .iter()
            .any(|operand| normalize_register(operand) == result)
    {
        return None;
    }
    cursor += 1;
    if branch_kind(mnemonic(cursor)?) != Some(true) {
        return None;
    }
    let slow_index = *index_of.get(&branch_target(&input[cursor].operands)?)?;
    if slow_index <= cursor {
        return None;
    }
    cursor += 1;
    // 4. Publish the new top, then tag the pointer.
    let (published, target) = store(cursor)?;
    if normalize_register(&published) != result || target != (thread.to_owned(), top_offset) {
        return None;
    }
    cursor += 1;
    let tags = mnemonic(cursor)? == "sub"
        && register(cursor, 0).as_deref() == Some(result.as_str())
        && if x64 {
            immediate(cursor, 1) == Some(15)
        } else {
            register(cursor, 1).as_deref() == Some(result.as_str())
                && immediate(cursor, 2) == Some(15)
        };
    if !tags {
        return None;
    }
    cursor += 1;
    // 5. The header must decode to a boxed Double.
    let header = (cursor..slow_index).find(|candidate| {
        let Some((stored, target)) = store(*candidate) else {
            return false;
        };
        if target != (result.clone(), -1) {
            return false;
        }
        let tag = if x64 {
            signed_immediate(&stored).and_then(|value| u64::try_from(value).ok())
        } else {
            loaded_immediate(input, cursor, *candidate, &normalize_register(&stored))
        };
        tag.is_some_and(|tag| is_boxed_double_tag(abi, tag))
    })?;
    // 6. The value store, which the slow path rejoins.
    let is_fpu = |operand: &str| {
        let operand = operand.trim().to_ascii_lowercase();
        operand.starts_with('d') || operand.starts_with("xmm")
    };
    let (value_store, value_store_end, value) = (header + 1..slow_index).find_map(|candidate| {
        if let Some((stored, target)) = store(candidate)
            && is_fpu(&stored)
            && target == (result.clone(), 7)
        {
            return Some((candidate, candidate, normalize_register(&stored)));
        }
        // ARM32 aligns the VFP address first: `add ip, r, #k; vstr dN, [ip, #7 - k]`.
        if abi == Abi::ArmeabiV7a
            && mnemonic(candidate)? == "add"
            && register(candidate, 1).as_deref() == Some(result.as_str())
        {
            let base = register(candidate, 0)?;
            let adjust = immediate(candidate, 2)?;
            let (stored, target) = store(candidate + 1)?;
            if input[candidate + 1].mnemonic == "vstr"
                && is_fpu(&stored)
                && target == (base, 7 - adjust)
            {
                return Some((candidate, candidate + 1, normalize_register(&stored)));
            }
        }
        None
    })?;
    // 7. Slow path: stub call, result move, branch back.
    let slow_end = (slow_index..input.len().min(slow_index + 16)).find(|candidate| {
        branch_kind(&input[*candidate].mnemonic) == Some(false)
            && branch_target(&input[*candidate].operands) == Some(input[value_store].address)
    })?;
    let has_allocation_call = (slow_index..slow_end)
        .any(|candidate| is_call(&input[candidate].mnemonic) && is_runtime_stub_call(candidate));
    let moves_result = result == abi_return_register(abi)
        || (slow_index..slow_end).any(|candidate| {
        input[candidate].mnemonic == "mov"
            && register(candidate, 0).as_deref() == Some(result.as_str())
            && register(candidate, 1).as_deref() == Some(abi_return_register(abi))
    });
    (has_allocation_call && moves_result).then_some(InlineDoubleBox {
        result,
        value,
        value_store_end,
        slow_index,
        slow_end,
    })
}

/// ARM32 keeps unboxed `int` values in register pairs. Fuses the pair
/// idioms whose branches have no source-level counterpart:
///
/// 1. Unboxing (`asr hi, x, #31; asrs lo, x, #1; blo done;
///    ldr lo, [x, #7]; ldr hi, [x, #0xb]; done:`): a Smi and a Mint yield
///    the same integer, so only the untag and the sign extension stay.
/// 2. Boxing (`lsl r, lo, #1; cmp lo, r, asr #1; cmpeq hi, r, asr #31;
///    beq done; <AllocateMint slow path>; done:`): both paths hold the same
///    integer, rewritten to a `box r, lo` pseudo-instruction.
/// 3. Signed 64-bit comparisons (`cmp hiA, hiB; blt T; bgt F;
///    cmp loA, loB; bhs F`): the high-word tests and the unsigned low-word
///    test decide one signed comparison of the full values, which the
///    lifter already names through the low words.
fn fuse_arm32_int64_idioms(
    input: &[DecodedInstruction],
    index_of: &BTreeMap<u64, usize>,
    is_runtime_stub_call: &dyn Fn(usize) -> bool,
    keep: &mut [bool],
    replacements: &mut BTreeMap<usize, DecodedInstruction>,
) {
    let operands_of = |index: usize| split_operands(&input[index].operands);
    let immediate = |operand: &str| immediate_text(operand).and_then(|value| value.parse::<i64>().ok());
    // 1. Unboxing diamonds.
    for index in 0..input.len().saturating_sub(2) {
        if !keep[index] || input[index].mnemonic != "asrs" {
            continue;
        }
        let operands = operands_of(index);
        if operands.len() != 3 || immediate(&operands[2]) != Some(1) {
            continue;
        }
        let low = normalize_register(&operands[0]);
        let source = normalize_register(&operands[1]);
        let branch = &input[index + 1];
        if low == source || !keep[index + 1] || !matches!(branch.mnemonic.as_str(), "blo" | "bcc")
        {
            continue;
        }
        let Some(join) = branch_target(&branch.operands) else {
            continue;
        };
        let loads = |at: usize, register: &str, displacement: i64| {
            input.get(at).is_some_and(|instruction| instruction.mnemonic == "ldr")
                && keep[at]
                && {
                    let loaded = operands_of(at);
                    loaded.len() == 2
                        && normalize_register(&loaded[0]) == register
                        && arm_memory_address(&loaded[1]) == Some((source.clone(), displacement))
                }
        };
        if !loads(index + 2, &low, 7) {
            continue;
        }
        let mut last = index + 2;
        if let Some(previous) = index.checked_sub(1)
            && input[previous].mnemonic == "asr"
        {
            let extended = operands_of(previous);
            if extended.len() == 3
                && normalize_register(&extended[1]) == source
                && immediate(&extended[2]) == Some(31)
                && loads(index + 3, &normalize_register(&extended[0]), 11)
            {
                last = index + 3;
            }
        }
        if input[last].next != join {
            continue;
        }
        for slot in &mut keep[index + 1..=last] {
            *slot = false;
        }
        replacements.retain(|at, _| !(index + 1..=last).contains(at));
    }
    // 2. Boxing overflow checks.
    for index in 0..input.len().saturating_sub(3) {
        if !keep[index] || input[index].mnemonic != "lsl" {
            continue;
        }
        let operands = operands_of(index);
        if operands.len() != 3 || immediate(&operands[2]) != Some(1) {
            continue;
        }
        let boxed = normalize_register(&operands[0]);
        let low = normalize_register(&operands[1]);
        let compares_back = |at: usize, register: &str, shift: &str| {
            let mnemonic = if at == index + 1 { "cmp" } else { "cmpeq" };
            input.get(at).is_some_and(|instruction| instruction.mnemonic == mnemonic)
                && {
                    let compared = operands_of(at);
                    compared.len() == 3
                        && normalize_register(&compared[0]) == register
                        && normalize_register(&compared[1]) == boxed
                        && compared[2].replace(' ', "") == shift
                }
        };
        if boxed == low || !compares_back(index + 1, &low, "asr#1") {
            continue;
        }
        let mut branch = index + 2;
        if let Some(high_compare) = input.get(branch)
            && high_compare.mnemonic == "cmpeq"
        {
            let compared = operands_of(branch);
            let Some(high) = compared.first().map(|value| normalize_register(value)) else {
                continue;
            };
            if !(compares_back(branch, &high, "asr#31") || compares_back(branch, &high, "asr#0x1f"))
            {
                continue;
            }
            branch += 1;
        }
        let Some(branch_instruction) = input.get(branch) else {
            continue;
        };
        if !matches!(branch_instruction.mnemonic.as_str(), "beq" | "bvc") {
            continue;
        }
        let Some(done) = branch_target(&branch_instruction.operands) else {
            continue;
        };
        let Some(&done_index) = index_of.get(&done) else {
            continue;
        };
        if done_index <= branch + 1 || done_index > branch + 12 {
            continue;
        }
        // The slow path allocates the Mint and fills it; nothing else.
        let slow = branch + 1..done_index;
        let allocates = slow
            .clone()
            .any(|at| is_call(&input[at].mnemonic) && is_runtime_stub_call(at));
        let only_fills = slow.clone().all(|at| {
            keep[at]
                && (matches!(input[at].mnemonic.as_str(), "str" | "mov" | "ldr")
                    || is_call(&input[at].mnemonic) && is_runtime_stub_call(at))
        });
        if !allocates || !only_fills {
            continue;
        }
        for slot in &mut keep[index + 1..done_index] {
            *slot = false;
        }
        replacements.retain(|at, _| !(index + 1..done_index).contains(at));
        replacements.insert(
            index,
            DecodedInstruction {
                address: input[index].address,
                next: input[index].next,
                mnemonic: "box".to_owned(),
                operands: format!("{boxed}, {low}"),
            },
        );
    }
    // 3. Signed 64-bit comparison towers.
    for index in 0..input.len().saturating_sub(4) {
        if !(index..=index + 4).all(|at| keep[at])
            || input[index].mnemonic != "cmp"
            || input[index + 3].mnemonic != "cmp"
        {
            continue;
        }
        let (first, second, last) = (&input[index + 1], &input[index + 2], &input[index + 4]);
        let (Some(first_target), Some(second_target), Some(last_target)) = (
            branch_target(&first.operands),
            branch_target(&second.operands),
            branch_target(&last.operands),
        ) else {
            continue;
        };
        let (less, greater) = match (first.mnemonic.as_str(), second.mnemonic.as_str()) {
            ("blt", "bgt") => (first_target, second_target),
            ("bgt", "blt") => (second_target, first_target),
            _ => continue,
        };
        let fallthrough = last.next;
        let condition = match last.mnemonic.as_str() {
            "blo" if less == last_target && greater == fallthrough => "blt",
            "bls" if less == last_target && greater == fallthrough => "ble",
            "bhi" if greater == last_target && less == fallthrough => "bgt",
            "bhs" if greater == last_target && less == fallthrough => "bge",
            _ => continue,
        };
        for slot in &mut keep[index + 1..=index + 3] {
            *slot = false;
        }
        replacements.retain(|at, _| !(index + 1..=index + 3).contains(at));
        replacements.insert(
            index,
            DecodedInstruction {
                address: input[index].address,
                next: input[index].next,
                mnemonic: "cmp".to_owned(),
                operands: input[index + 3].operands.clone(),
            },
        );
        replacements.insert(
            index + 4,
            DecodedInstruction {
                mnemonic: condition.to_owned(),
                ..last.clone()
            },
        );
    }
}

///
/// Fuses machine-level idioms that Dart AOT emits around every operation but
/// that have no source-level counterpart:
///
/// 1. Floating comparisons materialized through x64 branch diamonds or
///    ARM32 conditional loads of canonical thread-local boolean objects.
/// 2. The stack-overflow guard (`ldr limit, [THR]; cmp SP, limit; b.ls slow`).
/// 3. The Smi/Mint untag diamond (`sbfx d, s, #1, #W; tbz w(s), #0, +8;
///    ldur d, [s, #7]`). Both arms materialize the same untagged integer, so
///    the branch disappears and dataflow keeps one expression.
/// 4. The re-tag overflow check (`sbfiz d, s, #1, #W; cmp s, d, asr #1;
///    b.eq done; <allocate Mint slow path>; done:`). The checked value equals
///    the untagged result either way, so the diamond and the allocation slow
///    path collapse.
/// 5. Inline boxed-double allocation, whose fast and slow paths both produce
///    the same source-level double value.
/// 6. Compressed write-barrier checks: a store followed by tag-bit and heap
///    bounds tests that conditionally call an unnamed runtime stub.
///
/// Removing these before dataflow keeps register provenance alive across the
/// joins they create; without this the intersecting meet drops every value
/// computed across such a diamond. Only the lifting stream is filtered — the
/// complete decoded instructions remain in the machine reports.
fn fuse_machine_idioms(
    abi: Abi,
    input: &[DecodedInstruction],
    symbols: &BTreeMap<u64, Symbol>,
) -> Vec<DecodedInstruction> {
    fuse_machine_idioms_with_pool(abi, input, symbols, None)
}

/// [`fuse_machine_idioms`] with the object pool, which older releases load
/// the uninitialized-field sentinel from.
fn fuse_machine_idioms_with_pool(
    abi: Abi,
    input: &[DecodedInstruction],
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
) -> Vec<DecodedInstruction> {
    if input.is_empty() {
        return input.to_vec();
    }
    let is_arm = !matches!(abi, Abi::X86_64);
    let sp = abi_stack_register(abi);
    let thread = match abi {
        Abi::Arm64V8a => "x26",
        Abi::ArmeabiV7a => "r10",
        Abi::X86_64 => "",
    };
    let null_register = match abi {
        Abi::Arm64V8a => "x22",
        Abi::ArmeabiV7a => "r7",
        Abi::X86_64 => "",
    };
    let mut keep = vec![true; input.len()];
    let mut replacements = BTreeMap::<usize, DecodedInstruction>::new();
    let index_of: BTreeMap<u64, usize> = input
        .iter()
        .enumerate()
        .map(|(index, instruction)| (instruction.address, index))
        .collect();
    let operands_of = |index: usize| split_operands(&input[index].operands);
    let call_target =
        |index: usize| -> Option<u64> { parse_immediate(input[index].operands.split(',').next()?) };
    // Only a named VM helper or a root-proven stub may lose its machine call
    // effects. An unnamed target could be ordinary Dart code.
    let is_runtime_stub_call = |index: usize| -> bool {
        let Some(target) = call_target(index) else {
            return false;
        };
        match symbols.get(&target) {
            Some(symbol) => {
                symbol.label.starts_with("stub ")
                    || symbol.label.starts_with("stub_")
                    || symbol.label.starts_with("_iso_stub_")
            }
            // `WriteBarrierWrappers` holds one entry per object register;
            // calls land inside the root-proven stub, not at its start.
            None => symbols
                .range(..target)
                .next_back()
                .is_some_and(|(_, symbol)| symbol.label == "stub WriteBarrierWrappers"),
        }
    };

    if abi == Abi::ArmeabiV7a {
        // ARM32 adjusts a tagged object pointer into a VFP-aligned base before
        // loading or storing an unboxed field. Fold the adjacent address
        // calculation into the VFP access so field recovery sees the
        // original receiver.
        for index in 0..input.len().saturating_sub(1) {
            let access = input[index + 1].mnemonic.as_str();
            if input[index].mnemonic != "add" || !matches!(access, "vldr" | "vstr") {
                continue;
            }
            let add = operands_of(index);
            let load = operands_of(index + 1);
            if add.len() < 3 || load.len() < 2 {
                continue;
            }
            let target = normalize_register(&add[0]);
            let source = normalize_register(&add[1]);
            let Some(delta) = signed_immediate(&add[2]) else {
                continue;
            };
            let Some((load_base, displacement)) =
                load.get(1).and_then(|value| arm_memory_address(value))
            else {
                continue;
            };
            if load_base != target {
                continue;
            }
            let Some(displacement) = displacement.checked_add(delta) else {
                continue;
            };
            replacements.insert(
                index + 1,
                DecodedInstruction {
                    address: input[index + 1].address,
                    next: input[index + 1].next,
                    mnemonic: access.to_owned(),
                    operands: format!("{}, [{source}, #{displacement:#x}]", load[0]),
                },
            );
        }

        // `vcmpd; vmrs; ldrge true; ldrlt false` is ARM32's bool select.
        // Match only the exact canonical-bool thread offsets from Dart's
        // generated runtime offsets.
        for index in 0..input.len().saturating_sub(3) {
            if input[index].mnemonic != "vcmpd"
                || input[index + 1].mnemonic != "vmrs"
                || input[index + 2].mnemonic != "ldrge"
                || input[index + 3].mnemonic != "ldrlt"
            {
                continue;
            }
            let true_load = operands_of(index + 2);
            let false_load = operands_of(index + 3);
            let target = true_load.first().map(|value| normalize_register(value));
            if target.is_none()
                || false_load.first().map(|value| normalize_register(value)) != target
            {
                continue;
            }
            let (Some((true_base, true_offset)), Some((false_base, false_offset))) = (
                true_load.get(1).and_then(|value| arm_memory_address(value)),
                false_load
                    .get(1)
                    .and_then(|value| arm_memory_address(value)),
            ) else {
                continue;
            };
            if true_base != "r10"
                || false_base != "r10"
                || true_offset != 0x48
                || false_offset != 0x4c
            {
                continue;
            }
            keep[index + 1] = false;
            keep[index + 2] = false;
            keep[index + 3] = false;
            replacements.insert(
                index + 1,
                DecodedInstruction {
                    address: input[index + 1].address,
                    next: input[index + 3].next,
                    mnemonic: "cset".to_owned(),
                    operands: format!("{}, ge", target.unwrap()),
                },
            );
        }
    }

    // Pass 1: x64 emits a two-branch diamond for an ordered floating
    // comparison. `jp` sends NaN to the false arm; `jae` sends >= to the true
    // arm. Both arms load the canonical bool from the thread before joining.
    // Replace that compiler control flow with the bool value selected by the
    // comparison. Exact thread offsets keep this match version-safe.
    if abi == Abi::X86_64 {
        for index in 0..input.len().saturating_sub(5) {
            if !keep[index] || input[index].mnemonic != "comisd" {
                continue;
            }
            let unordered = &input[index + 1];
            let ordered = &input[index + 2];
            if unordered.mnemonic != "jp" || ordered.mnemonic != "jae" {
                continue;
            }
            let (Some(false_address), Some(true_address)) = (
                branch_target(&unordered.operands),
                branch_target(&ordered.operands),
            ) else {
                continue;
            };
            let (Some(false_index), Some(true_index)) = (
                index_of.get(&false_address).copied(),
                index_of.get(&true_address).copied(),
            ) else {
                continue;
            };
            let Some(false_jump_index) = false_index.checked_add(1) else {
                continue;
            };
            let Some(false_load) = input.get(false_index) else {
                continue;
            };
            let Some(false_jump) = input.get(false_jump_index) else {
                continue;
            };
            let Some(true_load) = input.get(true_index) else {
                continue;
            };
            if false_load.mnemonic != "mov"
                || false_jump.mnemonic != "jmp"
                || true_load.mnemonic != "mov"
            {
                continue;
            }
            let false_operands = operands_of(false_index);
            let true_operands = operands_of(true_index);
            let target = false_operands
                .first()
                .map(|value| normalize_register(value));
            if target.is_none()
                || true_operands.first().map(|value| normalize_register(value)) != target
            {
                continue;
            }
            let (Some((false_base, false_offset)), Some((true_base, true_offset))) = (
                false_operands
                    .get(1)
                    .and_then(|value| arm_memory_address(value)),
                true_operands
                    .get(1)
                    .and_then(|value| arm_memory_address(value)),
            ) else {
                continue;
            };
            let Some(join_address) = branch_target(&false_jump.operands) else {
                continue;
            };
            if false_base != "r14"
                || true_base != "r14"
                || false_offset != 0xa0
                || true_offset != 0x98
                || true_load.next != join_address
                || !index_of.contains_key(&join_address)
            {
                continue;
            }
            keep[index + 1] = false;
            keep[index + 2] = false;
            keep[false_index] = false;
            keep[false_jump_index] = false;
            keep[true_index] = false;
            replacements.insert(
                index + 1,
                DecodedInstruction {
                    address: unordered.address,
                    next: join_address,
                    mnemonic: "cset".to_owned(),
                    operands: format!("{}, ge", target.unwrap()),
                },
            );
        }
    }

    // Lazy static initialization (`LoadStaticFieldInstr` with
    // `calls_initializer`): the field value in the result register is
    // compared with the sentinel, and only the sentinel path loads the Field
    // and calls a root-proven init stub, which leaves the initialized value
    // in the same register. Both paths therefore produce the field value.
    // Late instance fields (`LoadFieldInstr` with `calls_initializer`) have
    // the same shape around an instance init stub.
    //
    //   [ldr tmp, [THR, #sentinel]]; cmp res, tmp|[THR + sentinel];
    //   b.ne done; ldr field_reg, [PP, #field]; bl InitLate...Field; done:
    let result_register = abi_return_register(abi);
    for call in 0..input.len() {
        if !keep[call]
            || !(calls_static_initializer(call_target(call), symbols)
                || calls_instance_initializer(call_target(call), symbols))
        {
            continue;
        }
        let Some(branch) = (call.saturating_sub(4)..call).rev().find(|&index| {
            branch_kind(&input[index].mnemonic) == Some(true)
                && branch_target(&input[index].operands) == Some(input[call].next)
        }) else {
            continue;
        };
        let not_equal = matches!(input[branch].mnemonic.as_str(), "b.ne" | "bne" | "jne");
        let setup_is_pure = (branch + 1..call).all(|index| {
            matches!(
                input[index].mnemonic.as_str(),
                "ldr" | "mov" | "movq" | "add"
            ) && operands_of(index)
                .first()
                .is_some_and(|target| normalize_register(target) != result_register)
        });
        let Some(compare) = branch.checked_sub(1) else {
            continue;
        };
        let compared = operands_of(compare);
        if !not_equal
            || !setup_is_pure
            || input[compare].mnemonic != "cmp"
            || compared
                .first()
                .is_none_or(|value| normalize_register(value) != result_register)
        {
            continue;
        }
        // ARM loads the sentinel into a scratch register, from the thread or
        // (in some releases) from the object pool.
        let sentinel_load = compare.checked_sub(1).filter(|&index| {
            let loaded = operands_of(index);
            is_arm
                && input[index].mnemonic == "ldr"
                && loaded.len() == 2
                && compared.get(1).is_some_and(|value| {
                    normalize_register(value) == normalize_register(&loaded[0])
                })
                && (arm_memory_address(&loaded[1])
                    .is_some_and(|(base, _)| base == normalize_register(thread))
                    || object_pool_index(abi, &input[index].operands)
                        .and_then(|pool_index| object_pool?.get(pool_index))
                        .is_some_and(|label| label == "uninitializedSentinel"))
        });
        let sentinel_in_memory = !is_arm
            && compared
                .get(1)
                .and_then(|value| arm_memory_address(value))
                .is_some_and(|(base, _)| base == abi_thread_register(abi));
        if is_arm && sentinel_load.is_none() || !is_arm && !sentinel_in_memory {
            continue;
        }
        // An instance field's init stub stays: it names the Field (and so
        // the receiver's class) that the preceding load reads, and the
        // lifter turns the call into that field read.
        let last = if calls_instance_initializer(call_target(call), symbols) {
            branch
        } else {
            call
        };
        for index in sentinel_load.unwrap_or(compare)..=last {
            keep[index] = false;
        }
    }

    // Pass 2: Smi/Mint untag diamonds. (ARM encodes them with sbfx/tbz;
    // x64 uses sar/test handled by transfer functions instead.)
    for index in (0..input.len().saturating_sub(2)).filter(|_| is_arm) {
        if !keep[index] || input[index].mnemonic != "sbfx" {
            continue;
        }
        let operands = operands_of(index);
        if operands.len() < 4 {
            continue;
        }
        let destination = normalize_register(&operands[0]);
        let source = normalize_register(&operands[1]);
        let (Some(shift), Some(width)) =
            (immediate_text(&operands[2]), immediate_text(&operands[3]))
        else {
            continue;
        };
        if shift != "1" || !matches!(width.as_str(), "31" | "63" | "32" | "64") {
            continue;
        }
        let test = &input[index + 1];
        let load = &input[index + 2];
        if test.mnemonic != "tbz" && test.mnemonic != "tbnz" {
            continue;
        }
        let test_operands = split_operands(&test.operands);
        if normalize_register(test_operands.first().unwrap_or(&String::new())) != source
            || test_operands.get(1).map(String::as_str) != Some("#0")
        {
            continue;
        }
        if load.mnemonic != "ldur" && load.mnemonic != "ldr" {
            continue;
        }
        let load_operands = split_operands(&load.operands);
        if normalize_register(load_operands.first().unwrap_or(&String::new())) != destination {
            continue;
        }
        let Some((load_base, load_displacement)) = load_operands
            .get(1)
            .and_then(|value| arm_memory_address(value))
        else {
            continue;
        };
        if load_base != source || load_displacement != 7 {
            continue;
        }
        // The test must jump exactly over the Mint load.
        let Some(branch_target) = branch_target(&test.operands) else {
            continue;
        };
        if branch_target != load.next {
            continue;
        }
        keep[index + 1] = false;
        keep[index + 2] = false;
    }

    // Pass 3: re-tag overflow-check diamonds with their Mint-allocation slow
    // paths. `sbfiz` stays: the transfer function treats it as a value-
    // preserving re-tag.
    for index in (0..input.len().saturating_sub(2)).filter(|_| is_arm) {
        if !keep[index] || input[index].mnemonic != "sbfiz" {
            continue;
        }
        let operands = operands_of(index);
        if operands.len() < 4 {
            continue;
        }
        let source = normalize_register(&operands[1]);
        let destination = normalize_register(&operands[0]);
        let (Some(shift), Some(width)) =
            (immediate_text(&operands[2]), immediate_text(&operands[3]))
        else {
            continue;
        };
        if shift != "1" || !matches!(width.as_str(), "31" | "63" | "32" | "64") {
            continue;
        }
        let compare = &input[index + 1];
        if compare.mnemonic != "cmp" {
            continue;
        }
        let compare_operands = operands_of(index + 1);
        if normalize_register(compare_operands.first().unwrap_or(&String::new())) != source {
            continue;
        }
        // Capstone prints `cmp x2, x0, asr #1` as three operands.
        let shifted_matches = compare_operands
            .get(1)
            .is_some_and(|value| normalize_register(value) == destination)
            && compare_operands
                .iter()
                .skip(2)
                .any(|value| value.replace(' ', "") == "asr#1");
        if !shifted_matches {
            continue;
        }
        let branch = &input[index + 2];
        if branch_kind(&branch.mnemonic) != Some(true) {
            continue;
        }
        // Only fold the equality exit of the overflow check.
        if !branch.mnemonic.starts_with("b.eq") {
            continue;
        }
        let Some(merge_address) = branch_target(&branch.operands) else {
            continue;
        };
        if merge_address <= branch.next {
            continue;
        }
        keep[index + 1] = false;
        keep[index + 2] = false;
        // Drop the allocation slow path up to the merge point.
        for instruction in input[index + 3..].iter() {
            if instruction.address >= merge_address {
                break;
            }
            let slow_index = index_of.get(&instruction.address).copied();
            match slow_index {
                Some(slow_index) if keep[slow_index] => keep[slow_index] = false,
                _ => break,
            }
        }
    }

    if abi == Abi::ArmeabiV7a {
        fuse_arm32_int64_idioms(
            input,
            &index_of,
            &is_runtime_stub_call,
            &mut keep,
            &mut replacements,
        );
    }

    // Pass 3b: x64 `BoxInt64`. The Smi fast path doubles the value and the
    // range check branches past a Mint allocation whose payload is the same
    // value:
    //   lea D, [S + S]; mov T, S; sar T, 0x1e; add T, 1; cmp T, 2; jb merge
    //   call <AllocateMint stub>; mov qword ptr [D + 7], S
    //   merge:
    // Both paths leave D holding the boxed `S`, so the `lea` becomes a plain
    // move and the check with its slow path is dropped. The scratch
    // register's own arithmetic stays.
    for index in (0..input.len().saturating_sub(6)).filter(|_| abi == Abi::X86_64) {
        if !keep[index] || input[index].mnemonic != "lea" {
            continue;
        }
        let lea = operands_of(index);
        let (Some(destination), Some(address)) = (lea.first(), lea.get(1)) else {
            continue;
        };
        let destination = normalize_register(destination);
        let Some(source) = address
            .trim()
            .strip_prefix('[')
            .and_then(|inner| inner.strip_suffix(']'))
            .and_then(|inner| inner.split_once(" + "))
            .filter(|(left, right)| left.trim() == right.trim())
            .map(|(left, _)| normalize_register(left))
        else {
            continue;
        };
        let compare = &input[index + 4];
        let branch = &input[index + 5];
        let scratch = operands_of(index + 1);
        let shape = input[index + 1].mnemonic == "mov"
            && scratch.get(1).map(|value| normalize_register(value)) == Some(source.clone())
            && input[index + 2].mnemonic == "sar"
            && input[index + 3].mnemonic == "add"
            && compare.mnemonic == "cmp"
            && operands_of(index + 4).get(1).map(String::as_str) == Some("2")
            && branch.mnemonic == "jb";
        let Some(merge) = shape.then(|| branch_target(&branch.operands)).flatten() else {
            continue;
        };
        let Some(&merge_index) = index_of.get(&merge) else {
            continue;
        };
        let slow = index + 6..merge_index;
        let payload = format!("[{destination} + 7]");
        let slow_is_box = !slow.is_empty()
            && slow.clone().all(|at| keep[at])
            && slow.clone().any(|at| is_call(&input[at].mnemonic) && is_runtime_stub_call(at))
            && slow.clone().any(|at| {
                input[at].mnemonic == "mov"
                    && input[at].operands.contains(&payload)
                    && operands_of(at).get(1).map(|value| normalize_register(value))
                        == Some(source.clone())
            });
        if !slow_is_box {
            continue;
        }
        keep[index + 4] = false;
        keep[index + 5] = false;
        for at in slow {
            keep[at] = false;
        }
        replacements.insert(
            index,
            DecodedInstruction {
                address: input[index].address,
                next: input[index].next,
                mnemonic: "mov".to_owned(),
                operands: format!("{destination}, {source}"),
            },
        );
    }

    // Pass 4: inline boxed-double allocation. The fast bump allocation
    // and the out-of-line allocation stub merge at the same value store. The
    // decoded VM header must prove CID 62 and a 16-byte object before this is
    // treated as a source-level box operation.
    for index in 0..input.len() {
        if !keep[index] {
            continue;
        }
        let Some(fused) =
            match_inline_double_box(abi, input, index, &index_of, &is_runtime_stub_call)
        else {
            continue;
        };
        for keep_slot in &mut keep[index..=fused.value_store_end] {
            *keep_slot = false;
        }
        for keep_slot in &mut keep[fused.slow_index..=fused.slow_end] {
            *keep_slot = false;
        }
        // Earlier rewrites inside the fused range (the ARM32 VFP address
        // fold of the value store) must not resurrect it.
        replacements.retain(|at, _| {
            !(index..=fused.value_store_end).contains(at)
                && !(fused.slow_index..=fused.slow_end).contains(at)
        });
        replacements.insert(
            index,
            DecodedInstruction {
                address: input[index].address,
                next: input[index].next,
                mnemonic: "fmov".to_owned(),
                operands: format!("{}, {}", fused.result, fused.value),
            },
        );
    }

    // Pass 5: stack-overflow guards.
    {
        for index in 0..input.len().saturating_sub(2) {
            if !keep[index] {
                continue;
            }
            let is_thread_load = input[index].mnemonic == "ldr" || input[index].mnemonic == "ldur";
            let is_x64_compare = abi == Abi::X86_64
                && input[index].mnemonic == "cmp"
                && input[index].operands.to_ascii_lowercase().contains("rsp");
            if !is_thread_load && !is_x64_compare {
                continue;
            }
            if is_x64_compare {
                fuse_x64_stack_guard(index, input, &index_of, symbols, &mut keep);
                continue;
            }
            let load_operands = operands_of(index);
            let Some((base, _)) = load_operands.get(1).and_then(|v| arm_memory_address(v)) else {
                continue;
            };
            if base != thread {
                continue;
            }
            let compare = &input[index.saturating_add(1)];
            if abi == Abi::X86_64 {
                // Shape: `cmp rsp, qword ptr [thr + limit]; jbe slow`.
                if compare.mnemonic != "cmp"
                    || !compare.operands.to_ascii_lowercase().contains("rsp")
                {
                    continue;
                }
                let branch = &input[index + 2];
                if branch.mnemonic != "jbe" && branch.mnemonic != "jb" && branch.mnemonic != "jnae"
                {
                    continue;
                }
                let Some(slow_address) = branch_target(&branch.operands) else {
                    continue;
                };
                let Some(slow_index) = index_of.get(&slow_address).copied() else {
                    continue;
                };
                if input
                    .get(slow_index)
                    .is_none_or(|slow| slow.mnemonic != "call")
                    || !is_runtime_stub_call(slow_index)
                {
                    continue;
                }
                let jumps_back = input.get(slow_index + 1).is_some_and(|back| {
                    matches!(back.mnemonic.as_str(), "jmp")
                        && branch_target(&back.operands) == Some(branch.next)
                });
                if !jumps_back {
                    continue;
                }
                keep[index] = false;
                keep[index + 1] = false;
                keep[index + 2] = false;
                continue;
            }
            if compare.mnemonic != "cmp" && compare.mnemonic != "cmp.w" {
                continue;
            }
            let compare_operands = operands_of(index + 1);
            let left = normalize_register(compare_operands.first().unwrap_or(&String::new()));
            let sp = sp.to_owned();
            let matches_sp = |register: &str| register == sp;
            let right_matches_sp = compare_operands.get(1).is_some_and(|value| {
                matches_sp(&normalize_register(
                    value.split(',').next().unwrap_or(value),
                ))
            });
            if !((matches_sp(&left) || right_matches_sp)
                && compare_operands.len() >= 2
                && (matches_sp(&left) != right_matches_sp || compare_operands.len() == 2))
            {
                continue;
            }
            let branch = &input[index + 2];
            if branch.mnemonic != "b.ls" && branch.mnemonic != "b.lo" && branch.mnemonic != "bls" {
                continue;
            }
            let Some(slow_address) = branch_target(&branch.operands) else {
                continue;
            };
            let Some(slow_index) = index_of.get(&slow_address).copied() else {
                continue;
            };
            // The slow path must be a stub call followed by a jump back.
            if input
                .get(slow_index)
                .is_none_or(|slow| slow.mnemonic != "bl")
            {
                continue;
            }
            if !is_runtime_stub_call(slow_index) {
                continue;
            }
            let back = input.get(slow_index + 1);
            let jumps_back = back.is_some_and(|back| {
                branch_kind(&back.mnemonic) == Some(false)
                    && branch_target(&back.operands) == Some(branch.next)
            });
            if !jumps_back {
                continue;
            }
            keep[index] = false;
            keep[index + 1] = false;
            keep[index + 2] = false;
        }
    }

    // Pass 6: compressed write-barrier tests ahead of runtime-stub calls
    // (ARM64 `bl`, ARM32 conditional `blne`, x64 `call`).
    for call_index in 0..input.len() {
        if !keep[call_index]
            || !matches!(input[call_index].mnemonic.as_str(), "bl" | "blne" | "call")
        {
            continue;
        }
        if !is_runtime_stub_call(call_index) {
            continue;
        }
        // Frameless ARM64 code saves the link register around the stub call:
        // `str x30, [x15, #-8]!; bl stub; ldr x30, [x15], #8`.
        let saves_link = abi == Abi::Arm64V8a
            && call_index > 0
            && input[call_index - 1].mnemonic == "str"
            && input[call_index - 1].operands.replace(' ', "") == "x30,[x15,#-8]!"
            && input
                .get(call_index + 1)
                .is_some_and(|restore| {
                    restore.mnemonic == "ldr"
                        && restore.operands.replace(' ', "") == "x30,[x15],#8"
                });
        let (first_scan, last) = if saves_link {
            (call_index - 1, call_index + 1)
        } else {
            (call_index, call_index)
        };
        let join = input[last].next;
        let mut start = None;
        let mut scan = first_scan;
        // Walk back over the bounded barrier-test window.
        for _step in 0..8 {
            let index = scan.checked_sub(1);
            let Some(index) = index else {
                break;
            };
            if !keep[index] {
                break;
            }
            let mnemonic = input[index].mnemonic.as_str();
            if branch_kind(mnemonic) == Some(true) {
                // Barrier tests can chain multiple tag/heap checks; every
                // conditional branch that shares the call's join point
                // belongs to the barrier.
                let matches_join = branch_target(&input[index].operands) == Some(join);
                if matches_join {
                    start = Some(index);
                    scan = index;
                    continue;
                }
                break;
            }
            let is_barrier_test = matches!(
                mnemonic,
                "tbz" | "tbnz" | "tst" | "and" | "orr" | "shr" | "sar" | "je" | "jne" | "test"
            ) || (mnemonic == "cmp"
                && operands_of(index).iter().any(|operand| {
                    normalize_register(operand.split(',').next().unwrap_or(operand))
                        == null_register
                }))
                || (matches!(mnemonic, "ldurb" | "ldur" | "ldrb")
                    && input[index].operands.contains("#-1]"))
                // ARM32 loads the barrier mask from the thread.
                || (abi == Abi::ArmeabiV7a
                    && mnemonic == "ldr"
                    && operands_of(index)
                        .get(1)
                        .and_then(|operand| arm_memory_address(operand))
                        .is_some_and(|(base, _)| base == thread))
                || (mnemonic == "mov" && input[index].operands.contains("- 1]"))
                || matches!(mnemonic, "shr" | "sar");
            if !is_barrier_test {
                break;
            }
            scan = index;
        }
        let Some(mut start) = start else {
            continue;
        };
        // The Smi test feeding the first barrier branch belongs to it too,
        // unless that branch tests the bit itself (`tbz`, `cbz`).
        if let Some(previous) = start.checked_sub(1)
            && !matches!(
                input[start].mnemonic.as_str(),
                "tbz" | "tbnz" | "cbz" | "cbnz"
            )
            && keep[previous]
            && matches!(input[previous].mnemonic.as_str(), "tst" | "test")
        {
            start = previous;
        }
        for keep_slot in &mut keep[start..=last] {
            *keep_slot = false;
        }
    }

    // Pass 7: Dart's `%` on signed integers adjusts a negative remainder by
    // re-adding the divisor in an out-of-line tail block. The comparison and
    // branch have no source-level counterpart once msub produced the
    // remainder; the adjustment block becomes unreachable and stays only in
    // the machine reports.
    let zero_register = match abi {
        Abi::Arm64V8a => "xzr",
        Abi::ArmeabiV7a => "",
        Abi::X86_64 => "",
    };
    if !zero_register.is_empty() {
        for index in 0..input.len().saturating_sub(3) {
            if !keep[index] || !keep[index + 1] {
                continue;
            }
            if input[index].mnemonic != "sdiv" || input[index + 1].mnemonic != "msub" {
                continue;
            }
            let division_operands = operands_of(index);
            let msub_operands = operands_of(index + 1);
            if division_operands.len() < 3 || msub_operands.len() < 4 {
                continue;
            }
            // msub dst, quotient, divisor, dividend with matching registers.
            let quotient = normalize_register(&division_operands[0]);
            let divisor = normalize_register(&division_operands[2]);
            if normalize_register(&msub_operands[1]) != quotient
                || normalize_register(&msub_operands[2]) != divisor
                || normalize_register(&division_operands[1])
                    != normalize_register(&msub_operands[3])
            {
                continue;
            }
            let remainder = normalize_register(&msub_operands[0]);
            let compare = &input[index + 2];
            if compare.mnemonic != "cmp" {
                continue;
            }
            let compare_operands = operands_of(index + 2);
            if normalize_register(compare_operands.first().unwrap_or(&String::new())) != remainder
                || compare_operands
                    .get(1)
                    .map(|value| normalize_register(value))
                    != Some(zero_register.to_owned())
            {
                continue;
            }
            let branch = &input[index + 3];
            if branch.mnemonic != "b.lt" && branch.mnemonic != "b.mi" && branch.mnemonic != "blt" {
                continue;
            }
            let Some(adjust_address) = branch_target(&branch.operands) else {
                continue;
            };
            let Some(adjust_index) = index_of.get(&adjust_address).copied() else {
                continue;
            };
            // The adjustment block must be `add remainder, remainder, divisor`
            // followed by a jump back past this branch.
            let add_operands = operands_of(adjust_index);
            let jumps_back = input.get(adjust_index + 1).is_some_and(|jump| {
                branch_kind(&jump.mnemonic) == Some(false)
                    && branch_target(&jump.operands) == Some(branch.next)
            });
            if input[adjust_index].mnemonic != "add"
                || normalize_register(add_operands.first().unwrap_or(&String::new())) != remainder
                || add_operands.get(1).map(|v| normalize_register(v)) != Some(remainder.clone())
                || add_operands.get(2).map(|v| normalize_register(v)) != Some(divisor)
                || !jumps_back
            {
                continue;
            }
            keep[index + 2] = false;
            keep[index + 3] = false;
        }
    }

    input
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| {
            replacements
                .remove(&index)
                .or_else(|| keep[index].then(|| instruction.clone()))
        })
        .collect()
}

fn build_control_flow(
    function_start: u64,
    function_end: u64,
    instructions: &[DecodedInstruction],
    block_starts: &std::collections::BTreeSet<u64>,
) -> Vec<ControlFlowEdge> {
    let mut edges = std::collections::BTreeSet::<(u64, u64, u8)>::new();
    for (index, block_start) in block_starts.iter().copied().enumerate() {
        let block_end = block_starts
            .iter()
            .nth(index + 1)
            .copied()
            .unwrap_or(function_end);
        let Some(last) = instructions
            .iter()
            .rev()
            .find(|instruction| (block_start..block_end).contains(&instruction.address))
        else {
            continue;
        };
        if is_return(&last.mnemonic, &last.operands) || is_trap(&last.mnemonic) {
            continue;
        }
        match branch_kind(&last.mnemonic) {
            Some(true) => {
                if let Some(target) = branch_target(&last.operands)
                    && (function_start..function_end).contains(&target)
                {
                    edges.insert((block_start, target, 2));
                }
                if last.next < function_end {
                    edges.insert((block_start, last.next, 3));
                }
            }
            Some(false) => {
                if let Some(target) = branch_target(&last.operands)
                    && (function_start..function_end).contains(&target)
                {
                    edges.insert((block_start, target, 1));
                }
            }
            None if last.next < function_end => {
                edges.insert((block_start, last.next, 0));
            }
            None => {}
        }
    }
    edges
        .into_iter()
        .map(|(from, to, kind)| ControlFlowEdge {
            from: format!("0x{from:x}"),
            to: format!("0x{to:x}"),
            kind: match kind {
                0 => ControlFlowEdgeKind::Fallthrough,
                1 => ControlFlowEdgeKind::Branch,
                2 => ControlFlowEdgeKind::ConditionalTrue,
                _ => ControlFlowEdgeKind::ConditionalFalse,
            },
        })
        .collect()
}

fn reachable_block_count(
    entry: u64,
    edges: &[ControlFlowEdge],
    block_starts: &std::collections::BTreeSet<u64>,
) -> usize {
    if block_starts.is_empty() {
        return 0;
    }
    let mut pending = vec![format!("0x{entry:x}")];
    let mut visited = std::collections::BTreeSet::new();
    while let Some(block) = pending.pop() {
        if !visited.insert(block.clone()) {
            continue;
        }
        pending.extend(
            edges
                .iter()
                .filter(|edge| edge.from == block)
                .map(|edge| edge.to.clone()),
        );
    }
    visited.len()
}

fn lift_semantics(
    abi: Abi,
    parameter_count: Option<usize>,
    instructions: &[DecodedInstruction],
    block_starts: &std::collections::BTreeSet<u64>,
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
) -> Vec<SemanticStatement> {
    let parameter_hints = (0..parameter_count.unwrap_or_default())
        .map(|index| ParameterHint {
            name: format!("arg{index}"),
            class_name: None,
            class_library_uri: None,
        })
        .collect::<Vec<_>>();
    lift_semantics_with_names(
        abi,
        &parameter_hints,
        instructions,
        block_starts,
        symbols,
        object_pool,
        None,
        None,
        None,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn lift_semantics_with_names(
    abi: Abi,
    parameter_hints: &[ParameterHint],
    instructions: &[DecodedInstruction],
    block_starts: &std::collections::BTreeSet<u64>,
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
    field_layout: Option<&RecoveredFieldLayout>,
    receiver_class: Option<(&str, Option<&str>)>,
    dispatch_table: Option<&DispatchTableAnalysis<'_>>,
    dispatch_calls: Option<&std::collections::BTreeMap<u64, DispatchCallEvidence>>,
    convention: Option<&ConventionInput>,
) -> Vec<SemanticStatement> {
    lift_semantics_with_names_outcome(
        abi,
        parameter_hints,
        instructions,
        block_starts,
        symbols,
        object_pool,
        field_layout,
        receiver_class,
        dispatch_table,
        dispatch_calls,
        convention,
        &[],
        &[],
    )
    .statements
}

#[allow(clippy::too_many_arguments)]
fn lift_semantics_with_names_outcome(
    abi: Abi,
    parameter_hints: &[ParameterHint],
    instructions: &[DecodedInstruction],
    block_starts: &std::collections::BTreeSet<u64>,
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
    field_layout: Option<&RecoveredFieldLayout>,
    receiver_class: Option<(&str, Option<&str>)>,
    dispatch_table: Option<&DispatchTableAnalysis<'_>>,
    dispatch_calls: Option<&std::collections::BTreeMap<u64, DispatchCallEvidence>>,
    convention: Option<&ConventionInput>,
    exceptional_entries: &[u64],
    exceptional_edges: &[ExceptionalEdge],
) -> SemanticLiftOutcome {
    let convention = convention
        .cloned()
        .unwrap_or_else(|| ConventionInput::from_hint_count(parameter_hints.len()));
    let (evidence, returns_fpu) =
        decoded_body_evidence(abi, instructions, convention.entry_offset);
    let seeds = resolve_parameter_seeds(abi, parameter_hints, &convention, &evidence);
    let fused = fuse_machine_idioms_with_pool(abi, instructions, symbols, object_pool);
    let pool_loads = recover_object_pool_loads(abi, &fused);
    // Recompute block boundaries over the fused stream: fusion can move
    // branches mid-block relative to the original CFG, so every branch and
    // return must terminate its block again.
    let live_addresses = fused
        .iter()
        .map(|i| i.address)
        .collect::<std::collections::BTreeSet<_>>();
    let mut fused_starts = block_starts
        .iter()
        .copied()
        .filter(|start| live_addresses.contains(start))
        .collect::<std::collections::BTreeSet<_>>();
    if let Some(first) = fused.first() {
        fused_starts.insert(first.address);
    }
    fused_starts.extend(
        exceptional_entries
            .iter()
            .copied()
            .chain(exceptional_edges.iter().map(|edge| edge.throw_return))
            .filter(|entry| live_addresses.contains(entry)),
    );
    for instruction in &fused {
        let ends_block = branch_kind(&instruction.mnemonic).is_some()
            || is_return(&instruction.mnemonic, &instruction.operands)
            || is_trap(&instruction.mnemonic);
        if ends_block && live_addresses.contains(&instruction.next) {
            fused_starts.insert(instruction.next);
        }
    }
    let blocks = LifterBlocks::build(&fused, &fused_starts);
    let exceptional_blocks = exceptional_entries
        .iter()
        .filter_map(|entry| blocks.starts.binary_search(entry).ok())
        .collect::<BTreeSet<_>>();
    // (throwing block, handler block, moves): the throwing block is the one
    // ending with the call whose return address starts the next block.
    let edges = exceptional_edges
        .iter()
        .filter_map(|edge| {
            let after = blocks.starts.binary_search(&edge.throw_return).ok()?;
            let handler = blocks.starts.binary_search(&edge.handler).ok()?;
            let thrower = after.checked_sub(1)?;
            let end = blocks.instruction_end(thrower);
            let last = fused.get(end.checked_sub(1)?)?;
            (end > blocks.instruction_start(thrower)
                && is_call(&last.mnemonic)
                && last.next == edge.throw_return
                && exceptional_blocks.contains(&handler))
            .then(|| (thrower, handler, edge.moves.as_slice()))
        })
        .collect::<Vec<_>>();
    // Receiver classes calls prove (a late field's init stub names its
    // Field, a direct call names its member's class) type the same values
    // throughout the body, so a round that finds new ones is lifted again.
    let type_hints = std::cell::RefCell::new(BTreeMap::new());
    let lift_facts = std::cell::RefCell::new(LiftFacts::default());
    let lift_round = || -> Option<Vec<SemanticStatement>> {
        let hints = type_hints.borrow();
        let context = LiftContext {
            dispatch_table,
            dispatch_calls,
            type_hints: &hints,
            lift_facts: None,
        };
        let mut entry_state = entry_flow_state(abi, &seeds, receiver_class);
        seed_optional_parameter_frame(abi, &mut entry_state, parameter_hints);
        let (converged, worklist_exhausted) = solve_block_states(
            &blocks,
            &fused,
            abi,
            symbols,
            object_pool,
            &pool_loads,
            field_layout,
            &entry_state,
            &exceptional_blocks,
            &edges,
            MAX_SEMANTIC_WORKLIST_VISITS,
            context,
        );
        if worklist_exhausted {
            return None;
        }
        // Only emit statements for blocks reachable from the entry. Unreachable
        // ranges are allocation slow paths or stub tails whose machine evidence
        // stays in the reports; emitting them would fabricate unreachable Dart.
        let successors = block_successors(&blocks, &fused);
        let mut reachable = vec![false; blocks.starts.len()];
        if !reachable.is_empty() {
            reachable[0] = true;
            let mut pending = vec![0usize];
            for &handler in &exceptional_blocks {
                if !reachable[handler] {
                    reachable[handler] = true;
                    pending.push(handler);
                }
            }
            while let Some(index) = pending.pop() {
                for successor in &successors[index] {
                    if !reachable[*successor] {
                        reachable[*successor] = true;
                        pending.push(*successor);
                    }
                }
            }
        }
        if std::env::var("CLUTTER_DEBUG_LIFT")
            .ok()
            .and_then(|v| u64::from_str_radix(&v, 16).ok())
            == fused.first().map(|i| i.address)
        {
            for (i, st) in converged.iter().enumerate() {
                eprintln!("BLOCK {:x} reach={}", blocks.starts[i], reachable[i]);
                for (k, v) in &st.registers {
                    eprintln!("   r {k} = {} ", v.text);
                }
                for (k, v) in &st.stack {
                    eprintln!("   s {k} = {} ", v.text);
                }
                let mut out = st.clone();
                simulate_range(
                    &mut out,
                    &fused,
                    Some(blocks.instruction_start(i)..blocks.instruction_end(i)),
                    abi,
                    symbols,
                    object_pool,
                    &pool_loads,
                    field_layout,
                    None,
                    context,
                    false,
                );
                for (k, v) in &out.registers {
                    eprintln!("   OUT r {k} = {} ", v.text);
                }
                for (k, v) in &out.stack {
                    eprintln!("   OUT s {k} = {} ", v.text);
                }
            }
        }
        let mut statements = Vec::new();
        for (block_index, _) in blocks.starts.iter().copied().enumerate() {
            if !reachable[block_index] {
                continue;
            }
            let Some(state) = converged.get(block_index) else {
                continue;
            };
            let mut state = state.clone();
            simulate_range(
                &mut state,
                &fused,
                Some(blocks.instruction_start(block_index)..blocks.instruction_end(block_index)),
                abi,
                symbols,
                object_pool,
                &pool_loads,
                field_layout,
                Some(&mut statements),
                LiftContext {
                    lift_facts: Some(&lift_facts),
                    ..context
                },
                returns_fpu == Some(true),
            );
            let end = blocks.instruction_end(block_index);
            let address = if end > blocks.instruction_start(block_index) {
                fused[end - 1].address
            } else {
                blocks.starts[block_index]
            };
            for (_, handler, moves) in edges
                .iter()
                .filter(|(thrower, _, _)| *thrower == block_index)
            {
                if let Some(input) = converged.get(*handler) {
                    statements.extend(phi_assignments(
                        blocks.starts[*handler],
                        input,
                        &exceptional_state(abi, &state, moves, object_pool),
                        address,
                    ));
                }
            }
            for successor in &successors[block_index] {
                if exceptional_blocks.contains(successor) {
                    continue;
                }
                if let Some(input) = converged.get(*successor) {
                    statements.extend(phi_assignments(
                        blocks.starts[*successor],
                        input,
                        &state,
                        address,
                    ));
                }
            }
        }
        Some(statements)
    };
    let mut rounds = 0;
    let mut statements = loop {
        *lift_facts.borrow_mut() = LiftFacts::default();
        let Some(statements) = lift_round() else {
            // A partial fixpoint can make a stale expression look certain at
            // a call or return. Keep the machine listing, but withhold
            // semantic claims for this body until the analysis finishes
            // within budget.
            return SemanticLiftOutcome {
                statements: Vec::new(),
                worklist_exhausted: true,
                parameter_defaults: BTreeMap::new(),
                facts: LiftFacts::default(),
            };
        };
        rounds += 1;
        let new_hints = receiver_type_hints(
            &std::mem::take(&mut lift_facts.borrow_mut().receivers),
            &type_hints.borrow(),
        );
        if new_hints.is_empty() || rounds == MAX_TYPE_HINT_ROUNDS {
            break statements;
        }
        type_hints.borrow_mut().extend(new_hints);
    };
    let worklist_exhausted = false;
    let optional_names = parameter_hints
        .iter()
        .enumerate()
        .map(|(index, hint)| (hint.name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut parameter_defaults = BTreeMap::new();
    loop {
        let found = extract_parameter_defaults(&mut statements, &optional_names);
        let before = statements.len();
        remove_trivial_phis(&mut statements);
        if found.is_empty() && statements.len() == before {
            break;
        }
        parameter_defaults.extend(found);
    }
    retain_used_phi_assignments(&mut statements);
    if !parameter_defaults.is_empty() {
        // With the defaults applied, the prologue's supplied-count tests
        // select nothing a source statement depends on.
        statements.retain(|statement| {
            !matches!(statement, SemanticStatement::Condition { expression, .. }
                if expression.contains("aot.argumentCount")
                    || expression.contains("aot.namedArgument"))
        });
    }
    SemanticLiftOutcome {
        statements,
        worklist_exhausted,
        parameter_defaults,
        facts: lift_facts.into_inner(),
    }
}

/// Assignments of each merged value at `join` from a predecessor's output,
/// ordered so an assignment reading another merged value of the same join
/// runs before that value is overwritten (a loop counter feeding a second
/// merged value). A true swap cycle keeps discovery order.
fn phi_assignments(
    join: u64,
    input: &FlowState,
    output: &FlowState,
    address: u64,
) -> Vec<SemanticStatement> {
    let mut pending = Vec::new();
    for (inputs, outputs) in [
        (&input.registers, &output.registers),
        (&input.stack, &output.stack),
    ] {
        for (location, merged) in inputs {
            if !is_phi_of(merged, join) {
                continue;
            }
            let Some(value) = outputs.get(location) else {
                continue;
            };
            if value.text == merged.text {
                continue;
            }
            pending.push((merged.text.clone(), value.clone()));
        }
    }
    let mut ordered = Vec::with_capacity(pending.len());
    while !pending.is_empty() {
        let ready = (0..pending.len())
            .find(|&candidate| {
                let variable = &pending[candidate].0;
                pending.iter().enumerate().all(|(other, (_, value))| {
                    other == candidate || !contains_identifier(&value.text, variable)
                })
            })
            .unwrap_or(0);
        ordered.push(pending.remove(ready));
    }
    ordered
        .into_iter()
        .map(|(variable, value)| SemanticStatement::Assign {
            variable,
            value: value.text,
            confidence: value.confidence,
            address: format!("0x{address:x}"),
        })
        .collect()
}

/// Replaces a merged value whose every assignment carries the same value
/// with that value (Braun et al.'s trivial-phi removal). Merged locations
/// stay merged during the fixpoint for termination, so a join whose inputs
/// finally agree still names a merged value until this pass removes it.
fn remove_trivial_phis(statements: &mut Vec<SemanticStatement>) {
    loop {
        let mut values = BTreeMap::<String, BTreeSet<String>>::new();
        for statement in statements.iter() {
            if let SemanticStatement::Assign {
                variable, value, ..
            } = statement
                && value != variable
            {
                values
                    .entry(variable.clone())
                    .or_default()
                    .insert(value.clone());
            }
        }
        let Some((variable, value)) = values.into_iter().find_map(|(variable, values)| {
            let mut values = values.into_iter();
            let value = values.next()?;
            (values.next().is_none() && !contains_identifier(&value, &variable))
                .then_some((variable, value))
        }) else {
            return;
        };
        statements.retain(|statement| {
            !matches!(statement, SemanticStatement::Assign { variable: assigned, .. } if *assigned == variable)
        });
        for statement in statements.iter_mut() {
            rename_in_statement(statement, &variable, &value);
        }
    }
}

fn rename_in_statement(statement: &mut SemanticStatement, from: &str, to: &str) {
    let rename = |text: &mut String| {
        if contains_identifier(text, from) {
            *text = replace_identifier(text, from, to);
        }
    };
    match statement {
        SemanticStatement::Return { expression, .. }
        | SemanticStatement::Condition { expression, .. } => rename(expression),
        SemanticStatement::ResolvedCall {
            target, arguments, ..
        } => {
            rename(target);
            arguments.iter_mut().for_each(rename);
        }
        SemanticStatement::FieldRead {
            receiver,
            expression,
            ..
        } => {
            rename(receiver);
            rename(expression);
        }
        SemanticStatement::FieldWrite {
            receiver, value, ..
        } => {
            rename(receiver);
            rename(value);
        }
        SemanticStatement::StaticFieldWrite { value, .. } => rename(value),
        SemanticStatement::Assign {
            variable, value, ..
        } => {
            rename(variable);
            rename(value);
        }
        SemanticStatement::StringInterpolation { parts, .. } => parts.iter_mut().for_each(rename),
        SemanticStatement::Throw {
            expression,
            stack_trace,
            ..
        } => {
            rename(expression);
            stack_trace.iter_mut().for_each(rename);
        }
        SemanticStatement::StaticFieldRead { .. } => {}
    }
}

fn replace_identifier(text: &str, from: &str, to: &str) -> String {
    let is_part = |character: char| character.is_ascii_alphanumeric() || character == '_';
    let mut output = String::with_capacity(text.len());
    let mut rest = 0usize;
    for (start, _) in text.match_indices(from) {
        let end = start + from.len();
        if start < rest
            || text[..start].chars().next_back().is_some_and(is_part)
            || text[end..].chars().next().is_some_and(is_part)
        {
            continue;
        }
        output.push_str(&text[rest..start]);
        output.push_str(to);
        rest = end;
    }
    output.push_str(&text[rest..]);
    output
}

/// Drops merged-value assignments nothing reads. Machine scratch registers
/// merge at almost every join; only values a statement consumes (directly,
/// or through another kept assignment) are source-level variables.
fn retain_used_phi_assignments(statements: &mut Vec<SemanticStatement>) {
    let mut used = BTreeSet::<String>::new();
    let mut texts = Vec::new();
    for statement in statements.iter() {
        if !matches!(statement, SemanticStatement::Assign { .. }) {
            texts.push(format!("{statement:?}"));
        }
    }
    let assignments = statements
        .iter()
        .filter_map(|statement| match statement {
            SemanticStatement::Assign {
                variable, value, ..
            } => Some((variable.clone(), value.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut candidates = assignments
        .iter()
        .map(|(variable, _)| variable.clone())
        .collect::<BTreeSet<_>>();
    loop {
        let newly_used = candidates
            .iter()
            .filter(|variable| texts.iter().any(|text| contains_identifier(text, variable)))
            .cloned()
            .collect::<Vec<_>>();
        if newly_used.is_empty() {
            break;
        }
        for variable in newly_used {
            candidates.remove(&variable);
            texts.extend(
                assignments
                    .iter()
                    .filter(|(assigned, _)| *assigned == variable)
                    .map(|(_, value)| value.clone()),
            );
            used.insert(variable);
        }
    }
    statements.retain(|statement| match statement {
        SemanticStatement::Assign { variable, .. } => used.contains(variable),
        _ => true,
    });
}

/// Whether `text` contains `identifier` as a whole identifier token.
fn contains_identifier(text: &str, identifier: &str) -> bool {
    let is_part = |character: char| character.is_ascii_alphanumeric() || character == '_';
    text.match_indices(identifier).any(|(start, _)| {
        let end = start + identifier.len();
        !text[..start].chars().next_back().is_some_and(is_part)
            && !text[end..].chars().next().is_some_and(is_part)
    })
}

/// A throw from the call returning to `throw_return` enters `handler` after
/// the VM applies `moves` (decoded catch-entry move maps).
#[derive(Clone, Debug)]
pub(crate) struct ExceptionalEdge {
    throw_return: u64,
    handler: u64,
    moves: Vec<crate::model::CatchEntryMove>,
}

/// Exceptional edges from every pc descriptor inside a try whose handler is
/// a real (non-generated) catch block. The descriptor's pc is the return
/// address of a call that may throw; the move map for that pc, when the
/// snapshot kept one, says how catch-block variables are populated.
fn exceptional_edges(
    metadata: &crate::model::RecoveredCodeMetadata,
    entry: u64,
    size: u64,
) -> Vec<ExceptionalEdge> {
    let moves = metadata
        .catch_entry_moves
        .iter()
        .map(|entry| (entry.pc_offset, &entry.moves))
        .collect::<BTreeMap<_, _>>();
    let mut edges = BTreeMap::new();
    for descriptor in &metadata.pc_descriptors {
        let Ok(try_index) = usize::try_from(descriptor.try_index) else {
            continue;
        };
        let Some(handler) = metadata
            .exception_handlers
            .iter()
            .find(|handler| handler.try_index == try_index && !handler.is_generated)
        else {
            continue;
        };
        if u64::from(handler.handler_pc_offset) >= size || u64::from(descriptor.pc_offset) > size {
            continue;
        }
        edges.entry(descriptor.pc_offset).or_insert_with(|| ExceptionalEdge {
            throw_return: entry + u64::from(descriptor.pc_offset),
            handler: entry + u64::from(handler.handler_pc_offset),
            moves: moves
                .get(&descriptor.pc_offset)
                .map(|moves| (*moves).clone())
                .unwrap_or_default(),
        });
    }
    edges.into_values().collect()
}

/// State at a handler entry for one throwing call: frame slots as the call
/// left them with the catch-entry moves applied, the exception and stack
/// trace in their fixed registers, and no other register value.
fn exceptional_state(
    abi: Abi,
    thrown_from: &FlowState,
    moves: &[crate::model::CatchEntryMove],
    object_pool: Option<&[String]>,
) -> FlowState {
    use crate::model::CatchMoveSource;
    let word = TargetLayout::of(abi).word_size;
    let slot_key = |slot: i32| {
        // AOT frames: variable slot `s` lives at `fp - (s + 1) * word`
        // (`FrameSlotForVariableIndex` with `first_local_from_fp == -1`).
        let fp = thrown_from.deltas.fp?;
        Some(entry_slot_key(fp - (i64::from(slot) + 1) * word))
    };
    let mut stack = thrown_from.stack.clone();
    let values = moves
        .iter()
        .map(|entry| {
            let value = match entry.kind {
                CatchMoveSource::Constant => usize::try_from(entry.source)
                    .ok()
                    .and_then(|index| object_pool?.get(index))
                    .filter(|value| !value.is_empty())
                    .map(|value| Expression {
                        text: value.clone(),
                        confidence: EvidenceConfidence::High,
                        complexity: 1,
                        class_name: snapshot_instance_class(value),
                        class_library_uri: None,
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: snapshot_instance_class(value).is_some(),
                    }),
                // Unboxed sources are boxed into the same value.
                CatchMoveSource::Int64Pair => None,
                _ => slot_key(entry.source).and_then(|key| thrown_from.stack.get(&key).cloned()),
            };
            (slot_key(entry.destination), value)
        })
        .collect::<Vec<_>>();
    for (destination, value) in values {
        let Some(destination) = destination else {
            continue;
        };
        match value {
            Some(value) => {
                stack.insert(destination, value);
            }
            None => {
                stack.remove(&destination);
            }
        }
    }
    let (exception, stack_trace) = match abi {
        Abi::Arm64V8a => ("x0", "x1"),
        Abi::ArmeabiV7a => ("r0", "r1"),
        Abi::X86_64 => ("rax", "rdx"),
    };
    let named = |text: &str| Expression {
        text: text.to_owned(),
        confidence: EvidenceConfidence::High,
        complexity: 1,
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    };
    // Registers the VM reserves (null, thread, pool) keep their values.
    let layout = TargetLayout::of(abi);
    let mut registers = thrown_from
        .registers
        .iter()
        .filter(|(register, _)| !layout.is_clobbered_by_call(register))
        .map(|(register, value)| (register.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    registers.insert(exception.to_owned(), named("e"));
    registers.insert(stack_trace.to_owned(), named("stackTrace"));
    FlowState {
        registers,
        stack,
        deltas: thrown_from.deltas,
        static_reads: thrown_from.static_reads.clone(),
        ..FlowState::default()
    }
}

/// Handler entry state with no decoded throwing site: only the exception
/// registers and the VM's reserved registers are known.
fn handler_entry_state(abi: Abi, entry: &FlowState) -> FlowState {
    let reserved = FlowState {
        registers: entry.registers.clone(),
        deltas: FrameDeltas { sp: None, fp: None },
        ..FlowState::default()
    };
    exceptional_state(abi, &reserved, &[], None)
}

/// Basic-block partition of a decoded instruction stream.
struct LifterBlocks {
    /// Block start addresses, sorted. Each entry owns instructions until the
    /// next start; instruction ranges are parallel to `starts`.
    starts: Vec<u64>,
    ranges: Vec<(usize, usize)>,
}

impl LifterBlocks {
    fn build(
        instructions: &[DecodedInstruction],
        block_starts: &std::collections::BTreeSet<u64>,
    ) -> Self {
        let first = instructions
            .first()
            .map_or(0, |instruction| instruction.address);
        let mut starts = block_starts
            .iter()
            .copied()
            .filter(|start| {
                instructions
                    .first()
                    .is_some_and(|instruction| instruction.address <= *start)
                    && instructions
                        .last()
                        .is_some_and(|instruction| *start <= instruction.address)
            })
            .collect::<Vec<_>>();
        if !starts.contains(&(first)) && !instructions.is_empty() {
            starts.push(first);
        }
        starts.sort_unstable();
        let mut ranges = Vec::with_capacity(starts.len());
        for (index, start) in starts.iter().copied().enumerate() {
            let end = starts.get(index + 1).copied();
            let begin = instructions.partition_point(|instruction| instruction.address < start);
            let finish = end
                .map(|end| instructions.partition_point(|instruction| instruction.address < end))
                .unwrap_or(instructions.len());
            ranges.push((begin, finish));
        }
        Self { starts, ranges }
    }

    fn instruction_start(&self, index: usize) -> usize {
        self.ranges[index].0
    }

    fn instruction_end(&self, index: usize) -> usize {
        self.ranges[index].1
    }
}

/// Liveness spelling of a register token: CPU registers use the lifter's
/// normalized names, FPU registers collapse onto the scalar-double register
/// that carries a Dart double argument (`d0` on ARM, `xmm1` on x64). Returns
/// every argument-relevant register the token overlaps.
fn liveness_registers(abi: Abi, token: &str) -> Vec<String> {
    let token = token.trim().trim_start_matches('#').to_ascii_lowercase();
    let token = token.split('.').next().unwrap_or_default();
    let numbered = |prefix: &str| {
        token
            .strip_prefix(prefix)
            .filter(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
            .and_then(|digits| digits.parse::<u32>().ok())
    };
    match abi {
        Abi::Arm64V8a => {
            if let Some(index) = numbered("x").or_else(|| numbered("w")).filter(|i| *i <= 30) {
                return vec![format!("x{index}")];
            }
            for prefix in ["d", "s", "h", "b", "q", "v"] {
                if let Some(index) = numbered(prefix).filter(|i| *i <= 31) {
                    return vec![format!("d{index}")];
                }
            }
            Vec::new()
        }
        Abi::ArmeabiV7a => {
            if let Some(index) = numbered("d").filter(|i| *i <= 31) {
                return vec![format!("d{index}")];
            }
            if let Some(index) = numbered("s").filter(|i| *i <= 31) {
                return vec![format!("d{}", index / 2)];
            }
            if let Some(index) = numbered("q").filter(|i| *i <= 15) {
                return vec![format!("d{}", index * 2), format!("d{}", index * 2 + 1)];
            }
            let normalized = normalize_register(token);
            if numbered("r").is_some()
                || matches!(token, "sb" | "sl" | "fp" | "ip" | "sp" | "lr" | "pc")
            {
                return vec![normalized];
            }
            Vec::new()
        }
        Abi::X86_64 => {
            if let Some(index) = numbered("xmm").or_else(|| numbered("ymm")) {
                return vec![format!("xmm{index}")];
            }
            let normalized = normalize_register(token);
            const NAMES: &[&str] = &[
                "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "r8", "r9", "r10", "r11",
                "r12", "r13", "r14", "r15",
            ];
            if NAMES.contains(&normalized.as_str()) {
                return vec![normalized];
            }
            // `r8b`/`r9w`-style byte and word spellings.
            if let Some(stripped) = token
                .strip_suffix('b')
                .filter(|value| value.starts_with('r'))
                && NAMES.contains(&stripped)
            {
                return vec![stripped.to_owned()];
            }
            Vec::new()
        }
    }
}

/// Every register an operand mentions, in liveness spelling.
fn operand_registers(abi: Abi, operand: &str) -> Vec<String> {
    operand
        .split(|character: char| !character.is_ascii_alphanumeric())
        .flat_map(|token| liveness_registers(abi, token))
        .collect()
}

/// Register reads and writes of one instruction, as far as argument
/// liveness needs them. Calls are reported separately because they define
/// every call-clobbered register.
struct RegisterEffects {
    reads: Vec<String>,
    writes: Vec<String>,
    is_call: bool,
}

fn register_effects(abi: Abi, mnemonic: &str, operands: &str) -> RegisterEffects {
    let parts = split_operands(operands);
    let registers_of = |part: &String| operand_registers(abi, part);
    let all_reads = || parts.iter().flat_map(registers_of).collect::<Vec<_>>();
    let mut effects = RegisterEffects {
        reads: Vec::new(),
        writes: Vec::new(),
        is_call: is_call(mnemonic),
    };
    if effects.is_call {
        // Only an indirect call's target register is read here; argument
        // registers are the callee's business.
        if direct_call_target(mnemonic, operands).is_none() {
            effects.reads = all_reads();
        }
        return effects;
    }
    let first_is_memory = parts.first().is_some_and(|part| part.contains('['));
    let write_first_read_rest = |effects: &mut RegisterEffects| {
        if let Some((first, rest)) = parts.split_first() {
            effects.writes = registers_of(first);
            effects.reads = rest.iter().flat_map(registers_of).collect();
        }
    };
    let read_write_first = |effects: &mut RegisterEffects| {
        effects.reads = all_reads();
        effects.writes = parts.first().map(registers_of).unwrap_or_default();
    };
    match abi {
        Abi::X86_64 => {
            let same_operands = parts.len() == 2 && parts[0] == parts[1];
            let base = mnemonic.trim_end_matches('q');
            if matches!(mnemonic, "xor" | "sub" | "pxor" | "xorps" | "xorpd" | "sbb")
                && same_operands
            {
                effects.writes = registers_of(&parts[0]);
                if mnemonic == "sbb" {
                    effects.reads = registers_of(&parts[0]);
                }
            } else if matches!(
                base,
                "cmp"
                    | "test"
                    | "ucomisd"
                    | "comisd"
                    | "ucomiss"
                    | "comiss"
                    | "bt"
                    | "push"
                    | "jmp"
                    | "ret"
                    | "nop"
                    | "ptest"
            ) || branch_kind(mnemonic).is_some()
            {
                effects.reads = all_reads();
            } else if base == "pop" {
                effects.writes = all_reads();
            } else if mnemonic.starts_with("cmov") || matches!(mnemonic, "xchg" | "xadd") {
                read_write_first(&mut effects);
                if mnemonic == "xchg" {
                    effects.writes = all_reads();
                }
            } else if matches!(mnemonic, "cqo" | "cdq" | "cdqe" | "cwd") {
                effects.reads = vec!["rax".to_owned()];
                effects.writes = vec!["rdx".to_owned(), "rax".to_owned()];
            } else if matches!(mnemonic, "idiv" | "div" | "mul" | "imul") && parts.len() == 1 {
                effects.reads = all_reads();
                effects.reads.extend(["rax".to_owned(), "rdx".to_owned()]);
                effects.writes = vec!["rax".to_owned(), "rdx".to_owned()];
            } else if first_is_memory {
                effects.reads = all_reads();
            } else if matches!(
                base,
                "mov"
                    | "movabs"
                    | "movzx"
                    | "movsx"
                    | "movsxd"
                    | "lea"
                    | "movd"
                    | "movsd"
                    | "movss"
                    | "movaps"
                    | "movapd"
                    | "movups"
                    | "movupd"
                    | "movdqa"
                    | "movdqu"
                    | "cvtsi2sd"
                    | "cvttsd2si"
                    | "cvtsd2si"
                    | "cvtsd2ss"
                    | "cvtss2sd"
                    | "sqrtsd"
                    | "bsr"
                    | "bsf"
                    | "popcnt"
                    | "lzcnt"
                    | "tzcnt"
                    | "movmskpd"
                    | "andnpd"
            ) || mnemonic.starts_with("set")
                || (mnemonic == "imul" && parts.len() == 3)
            {
                write_first_read_rest(&mut effects);
            } else {
                read_write_first(&mut effects);
            }
            // Shifts by `cl` read rcx implicitly through the operand text.
        }
        Abi::Arm64V8a | Abi::ArmeabiV7a => {
            let base = mnemonic.split('.').next().unwrap_or(mnemonic);
            let conditional_arm32 = abi == Abi::ArmeabiV7a && arm32_condition_suffix(base);
            if base.starts_with("st") || base.starts_with("vst") || base == "push" {
                // `stxr`-family status registers never carry Dart arguments.
                effects.reads = all_reads();
            } else if matches!(
                base,
                "cmp"
                    | "cmn"
                    | "tst"
                    | "teq"
                    | "fcmp"
                    | "fcmpe"
                    | "ccmp"
                    | "ccmn"
                    | "vcmp"
                    | "vcmpe"
                    | "cbz"
                    | "cbnz"
                    | "tbz"
                    | "tbnz"
                    | "br"
                    | "bx"
                    | "ret"
                    | "dmb"
                    | "dsb"
                    | "isb"
                    | "prfm"
                    | "pld"
                    | "nop"
                    | "vmrs"
                    | "b"
            ) || branch_kind(mnemonic).is_some()
            {
                effects.reads = all_reads();
            } else if base == "pop" || base.starts_with("ldm") || base.starts_with("vldm") {
                // `ldm r0, {r1, r2}` / `pop {r4, pc}`: braced registers are
                // written, the base is read.
                for part in &parts {
                    if part.starts_with('{') {
                        effects.writes.extend(registers_of(part));
                    } else {
                        effects.reads.extend(registers_of(part));
                    }
                }
            } else if matches!(base, "ldp" | "ldnp" | "ldaxp" | "ldxp" | "ldrd" | "ldpsw")
                || matches!(base, "umull" | "smull" | "umlal" | "smlal")
                || (base == "vmov"
                    && parts.len() == 3
                    && [&parts[0], &parts[1]].iter().all(|part| {
                        operand_registers(abi, part)
                            .iter()
                            .all(|register| !register.starts_with('d'))
                    }))
            {
                effects.writes = parts.iter().take(2).flat_map(registers_of).collect();
                effects.reads = parts.iter().skip(2).flat_map(registers_of).collect();
                if matches!(base, "umlal" | "smlal") {
                    effects.reads.extend(effects.writes.clone());
                }
            } else if matches!(
                base,
                "movk" | "bfi" | "bfxil" | "bfm" | "movt" | "vmla" | "vmls"
            ) || base.starts_with("fmla")
                || base.starts_with("fmls")
            {
                read_write_first(&mut effects);
            } else {
                write_first_read_rest(&mut effects);
                if conditional_arm32 {
                    // A predicated write leaves the old value on the
                    // not-executed path.
                    effects.reads.extend(effects.writes.clone());
                }
            }
        }
    }
    effects
}

/// ARM32 condition-code suffix on a data-processing mnemonic (`moveq`).
fn arm32_condition_suffix(mnemonic: &str) -> bool {
    const CONDITIONS: &[&str] = &[
        "eq", "ne", "cs", "hs", "cc", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt",
        "le",
    ];
    mnemonic.len() > 2
        && CONDITIONS.contains(&&mnemonic[mnemonic.len() - 2..])
        && !matches!(
            mnemonic,
            "bls" | "bhs" | "blo" | "bcs" | "bcc" | "bvs" | "bvc"
        )
        && !mnemonic.starts_with('b')
        && !matches!(mnemonic, "teq" | "vmrs")
}

/// Callee-side constraints on where parameters arrive: argument registers
/// that are live at the normal entry, incoming stack words read through the
/// frame pointer (or the untouched entry stack pointer), and whether returns
/// deliver their value in the FPU return register.
pub(crate) fn body_evidence(
    abi: Abi,
    instructions: &[crate::model::MachineInstruction],
    entry_offset: u64,
) -> (
    crate::analysis::calling_convention::BodyEvidence,
    Option<bool>,
) {
    let decoded = instructions
        .iter()
        .filter_map(|instruction| {
            let address = parse_immediate(&instruction.address)?;
            (!is_skipped_data(&instruction.mnemonic)).then(|| DecodedInstruction {
                address,
                next: address.saturating_add((instruction.bytes.len() / 2) as u64),
                mnemonic: instruction.mnemonic.clone(),
                operands: instruction.operands.clone(),
            })
        })
        .collect::<Vec<_>>();
    decoded_body_evidence(abi, &decoded, entry_offset)
}

fn decoded_body_evidence(
    abi: Abi,
    decoded: &[DecodedInstruction],
    entry_offset: u64,
) -> (
    crate::analysis::calling_convention::BodyEvidence,
    Option<bool>,
) {
    use crate::analysis::calling_convention::{BodyEvidence, TargetLayout};
    let layout = TargetLayout::of(abi);
    let mut evidence = BodyEvidence {
        live_cpu: vec![false; layout.cpu_arguments.len()],
        live_fpu: vec![false; layout.fpu_arguments.len()],
        stack_words: BTreeSet::new(),
    };
    let Some(first) = decoded.first() else {
        return (evidence, None);
    };
    let entry = first.address.saturating_add(entry_offset);
    let end = decoded.last().map_or(entry, |instruction| instruction.next);
    let mut starts = BTreeSet::from([first.address]);
    if decoded
        .iter()
        .any(|instruction| instruction.address == entry)
    {
        starts.insert(entry);
    }
    for instruction in decoded {
        let terminates = branch_kind(&instruction.mnemonic).is_some()
            || is_return(&instruction.mnemonic, &instruction.operands);
        if terminates {
            starts.insert(instruction.next);
            if let Some(target) = branch_target(&instruction.operands)
                && (first.address..end).contains(&target)
            {
                starts.insert(target);
            }
        }
    }
    let blocks = LifterBlocks::build(decoded, &starts);
    let successors = block_successors(&blocks, decoded);
    let tracked = |register: &str| {
        layout.cpu_argument_index(register).is_some()
            || layout.fpu_argument_index(register).is_some()
    };
    // Per-block upward-exposed uses and definitions over argument registers.
    let mut uses = vec![BTreeSet::<String>::new(); blocks.starts.len()];
    let mut defs = vec![BTreeSet::<String>::new(); blocks.starts.len()];
    for index in 0..blocks.starts.len() {
        for instruction in &decoded[blocks.instruction_start(index)..blocks.instruction_end(index)]
        {
            let effects = register_effects(abi, &instruction.mnemonic, &instruction.operands);
            for read in effects.reads.iter().filter(|register| tracked(register)) {
                if !defs[index].contains(read) {
                    uses[index].insert(read.clone());
                }
            }
            if effects.is_call {
                for register in layout.cpu_arguments.iter().chain(layout.fpu_arguments) {
                    defs[index].insert((*register).to_owned());
                }
            }
            for write in effects
                .writes
                .into_iter()
                .filter(|register| tracked(register))
            {
                defs[index].insert(write);
            }
        }
    }
    let mut live_in = uses.clone();
    let mut changed = true;
    let mut rounds = 0usize;
    while changed && rounds < 4 * blocks.starts.len() + 8 {
        changed = false;
        rounds += 1;
        for index in (0..blocks.starts.len()).rev() {
            let mut next = uses[index].clone();
            for successor in &successors[index] {
                for register in &live_in[*successor] {
                    if !defs[index].contains(register) {
                        next.insert(register.clone());
                    }
                }
            }
            if next != live_in[index] {
                live_in[index] = next;
                changed = true;
            }
        }
    }
    if let Ok(entry_block) = blocks.starts.binary_search(&entry) {
        for register in &live_in[entry_block] {
            if let Some(index) = layout.cpu_argument_index(register) {
                evidence.live_cpu[index] = true;
            }
            if let Some(index) = layout.fpu_argument_index(register) {
                evidence.live_fpu[index] = true;
            }
        }
    }
    // Incoming stack words: entry-relative slots at or above the parameter
    // area, addressed through either pointer while its delta is known.
    // Deltas follow the instruction stream from the normal entry; a branch
    // target reached with other deltas simply stops contributing.
    // `None` marks code not reached by fall-through.
    let mut current = Some(FrameDeltas::default());
    let mut block_deltas = BTreeMap::<u64, FrameDeltas>::new();
    for instruction in decoded
        .iter()
        .filter(|instruction| instruction.address >= entry)
    {
        if let Some(known) = block_deltas.get(&instruction.address) {
            current = Some(current.map_or(*known, |deltas| deltas.meet(*known)));
        }
        let Some(deltas) = current else {
            continue;
        };
        for part in split_operands(&instruction.operands) {
            let Some(key) = stack_slot_key(abi, &part, deltas) else {
                continue;
            };
            let Some(displacement) = key
                .strip_prefix("[entry,#")
                .and_then(|rest| rest.strip_suffix(']'))
                .and_then(|value| value.parse::<i64>().ok())
            else {
                continue;
            };
            let relative = displacement - layout.incoming_entry_sp_offset();
            if relative >= 0 && relative % layout.word_size == 0 {
                evidence
                    .stack_words
                    .insert((relative / layout.word_size) as usize);
            }
        }
        let next = deltas.after(abi, &instruction.mnemonic, &instruction.operands);
        if let Some(target) = branch_target(&instruction.operands)
            .filter(|_| branch_kind(&instruction.mnemonic).is_some())
        {
            block_deltas
                .entry(target)
                .and_modify(|existing| *existing = existing.meet(next))
                .or_insert(next);
        }
        current = (branch_kind(&instruction.mnemonic) != Some(false)
            && !is_return(&instruction.mnemonic, &instruction.operands))
        .then_some(next);
    }
    (evidence, fpu_return(abi, decoded))
}

/// Whether the body returns its value in the FPU return register: every
/// return reached by a straight-line tail that defines exactly one of the
/// two return registers agrees on the FPU one.
fn fpu_return(abi: Abi, decoded: &[DecodedInstruction]) -> Option<bool> {
    use crate::analysis::calling_convention::TargetLayout;
    let layout = TargetLayout::of(abi);
    let mut verdict = None;
    for (index, instruction) in decoded.iter().enumerate() {
        if !is_return(&instruction.mnemonic, &instruction.operands) {
            continue;
        }
        let mut cpu = false;
        let mut fpu = false;
        for previous in decoded[..index].iter().rev().take(24) {
            if branch_kind(&previous.mnemonic).is_some() || is_call(&previous.mnemonic) {
                break;
            }
            let effects = register_effects(abi, &previous.mnemonic, &previous.operands);
            cpu |= effects
                .writes
                .iter()
                .any(|register| register == layout.return_register);
            fpu |= effects
                .writes
                .iter()
                .any(|register| register == layout.fpu_return_register);
            if cpu || fpu {
                break;
            }
        }
        let this = match (cpu, fpu) {
            (true, false) => false,
            (false, true) => true,
            _ => continue,
        };
        match verdict {
            None => verdict = Some(this),
            Some(previous) if previous != this => return None,
            Some(_) => {}
        }
    }
    verdict
}

/// What the lifter knows about a body's calling convention before looking
/// at its instructions.
#[derive(Clone, Debug)]
pub(crate) struct ConventionInput {
    /// Declared representation per parameter, implicit ones first. `None`
    /// when the parameter count did not survive.
    pub declared: Option<Vec<DeclaredRepresentation>>,
    /// Kind-derived bounds on the register window; `None` leaves it open.
    pub window: Option<RegisterWindow>,
    /// Byte offset of the normal entry past a monomorphic-checked prefix.
    pub entry_offset: u64,
}

impl ConventionInput {
    /// Hints of known count with no surviving kind or types.
    fn from_hint_count(count: usize) -> Self {
        Self {
            declared: Some(vec![DeclaredRepresentation::Unknown; count]),
            window: None,
            entry_offset: 0,
        }
    }

    pub(crate) fn for_function(abi: Abi, function: &crate::model::RecoveredFunction) -> Self {
        use crate::analysis::calling_convention::{
            declared_parameters, fixed_parameter_count, is_generic,
        };
        let layout = TargetLayout::of(abi);
        Self {
            declared: declared_parameters(function),
            window: Some(RegisterWindow::for_function(
                function.kind,
                is_generic(function),
                fixed_parameter_count(function),
                layout,
            )),
            entry_offset: function
                .code_metadata
                .as_ref()
                .filter(|metadata| metadata.has_monomorphic_entrypoint)
                .map_or(0, |_| layout.polymorphic_entry_offset),
        }
    }
}

/// One incoming parameter with the single location it arrives in.
#[derive(Clone, Debug)]
struct ParameterSeed {
    hint: ParameterHint,
    location: ArgumentLocation,
    representation: Representation,
    proof: LocationProof,
}

/// Resolves every parameter hint to one location. With an unknown
/// parameter count, stack positions of the known hints cannot be derived
/// (they count from the last parameter), so only register placements are
/// kept and every observed incoming read nothing explains becomes a
/// further placeholder parameter.
fn resolve_parameter_seeds(
    abi: Abi,
    parameter_hints: &[ParameterHint],
    input: &ConventionInput,
    evidence: &BodyEvidence,
) -> Vec<ParameterSeed> {
    let layout = TargetLayout::of(abi);
    let count_known = input
        .declared
        .as_ref()
        .is_some_and(|declared| declared.len() == parameter_hints.len());
    let declared = if count_known {
        input.declared.clone().unwrap_or_default()
    } else {
        parameter_hints
            .iter()
            .map(|hint| {
                if matches!(hint.name.as_str(), "this" | "closureContext") {
                    DeclaredRepresentation::Tagged
                } else {
                    DeclaredRepresentation::Unknown
                }
            })
            .collect()
    };
    let window = input.window.unwrap_or(RegisterWindow {
        min: 0,
        max: declared.len(),
    });
    let resolved: ResolvedParameters = crate::analysis::calling_convention::resolve_parameters(
        layout, window, &declared, evidence,
    );
    let mut seeds = parameter_hints
        .iter()
        .zip(&resolved.parameters)
        .filter(|(_, parameter)| {
            count_known || !matches!(parameter.location, ArgumentLocation::Stack { .. })
        })
        .map(|(hint, parameter)| ParameterSeed {
            hint: hint.clone(),
            location: parameter.location,
            representation: parameter.representation,
            proof: parameter.proof,
        })
        .collect::<Vec<_>>();
    if count_known {
        return seeds;
    }
    let implicit = parameter_hints
        .iter()
        .take_while(|hint| {
            hint.name == "this"
                || hint.name == "closureContext"
                || hint.name.starts_with("implicitArg")
        })
        .count();
    let mut next_visible = parameter_hints.len().saturating_sub(implicit);
    let mut placeholder = |location: ArgumentLocation, representation: Representation| {
        let seed = ParameterSeed {
            hint: ParameterHint {
                name: format!("arg{next_visible}"),
                class_name: (representation == Representation::UnboxedDouble)
                    .then(|| "double".to_owned()),
                class_library_uri: (representation == Representation::UnboxedDouble)
                    .then(|| "dart:core".to_owned()),
            },
            location,
            representation,
            proof: LocationProof::Assumed,
        };
        next_visible += 1;
        seed
    };
    for index in &resolved.unexplained_cpu {
        seeds.push(placeholder(
            ArgumentLocation::Register(layout.cpu_arguments[*index]),
            Representation::Tagged,
        ));
    }
    for index in &resolved.unexplained_fpu {
        seeds.push(placeholder(
            ArgumentLocation::FpuRegister(layout.fpu_arguments[*index]),
            Representation::UnboxedDouble,
        ));
    }
    // Stack words count from the last parameter, which an unknown count
    // leaves unplaced. Assume the observed words are the whole stacked
    // tail: the known leading hints take the highest words and every lower
    // observed word becomes a placeholder parameter after them.
    let stacked_hints = parameter_hints
        .iter()
        .zip(&resolved.parameters)
        .filter(|(_, parameter)| matches!(parameter.location, ArgumentLocation::Stack { .. }))
        .map(|(hint, _)| hint.clone())
        .collect::<Vec<_>>();
    let observed_words = evidence.stack_words.last().map_or(0, |highest| highest + 1);
    if observed_words > 0 {
        let total = observed_words.max(stacked_hints.len());
        for (position, hint) in stacked_hints.iter().enumerate() {
            seeds.push(ParameterSeed {
                hint: hint.clone(),
                location: ArgumentLocation::Stack {
                    word: total - 1 - position,
                    words: 1,
                },
                representation: Representation::Tagged,
                proof: LocationProof::Assumed,
            });
        }
        for word in (0..total.saturating_sub(stacked_hints.len())).rev() {
            if evidence.stack_words.contains(&word) {
                seeds.push(placeholder(
                    ArgumentLocation::Stack { word, words: 1 },
                    Representation::Tagged,
                ));
            }
        }
    }
    seeds
}

fn entry_flow_state(
    abi: Abi,
    seeds: &[ParameterSeed],
    receiver_class: Option<(&str, Option<&str>)>,
) -> FlowState {
    let layout = TargetLayout::of(abi);
    let mut registers = BTreeMap::new();
    let mut stack = BTreeMap::new();
    for seed in seeds {
        let receiver = (seed.hint.name == "this")
            .then_some(receiver_class)
            .flatten();
        let (mut class_name, mut class_library_uri) = receiver.map_or(
            (
                seed.hint.class_name.clone(),
                seed.hint.class_library_uri.clone(),
            ),
            |(name, uri)| (Some(name.to_owned()), uri.map(str::to_owned)),
        );
        match seed.representation {
            Representation::UnboxedDouble => {
                class_name = Some("double".to_owned());
                class_library_uri = Some("dart:core".to_owned());
            }
            Representation::UnboxedInt64 => {
                class_name = Some("int".to_owned());
                class_library_uri = Some("dart:core".to_owned());
            }
            Representation::Tagged => {}
        }
        let expression = Expression {
            text: seed.hint.name.clone(),
            confidence: match seed.proof {
                LocationProof::Proven => EvidenceConfidence::High,
                LocationProof::Assumed => EvidenceConfidence::Medium,
            },
            complexity: 1,
            class_name,
            class_library_uri,
            raw: seed.representation == Representation::UnboxedInt64,
            definition_site: None,
            high_word: false,
            exact_class: false,
        };
        match seed.location {
            ArgumentLocation::Register(register) | ArgumentLocation::FpuRegister(register) => {
                registers.insert(register.to_owned(), expression);
            }
            ArgumentLocation::RegisterPair(low, high) => {
                registers.insert(high.to_owned(), high_word_of(expression.clone()));
                registers.insert(low.to_owned(), expression);
            }
            ArgumentLocation::Stack { .. } => {
                let entry = seed
                    .location
                    .entry_sp_displacement(layout)
                    .unwrap_or_default();
                stack.insert(entry_slot_key(entry), expression.clone());
                // Frame-pointer reads whose frame set-up was not tracked
                // (idiom fusion, unusual prologues) keep the literal key.
                let frame = seed.location.frame_displacement(layout).unwrap_or_default();
                for key in slot_keys(layout.frame_pointer, frame) {
                    stack.insert(key, expression.clone());
                }
            }
        }
    }
    if abi == Abi::Arm64V8a {
        // The VM keeps the null constant in a fixed register on ARM64.
        registers.insert(
            "x22".to_owned(),
            Expression {
                text: "null".to_owned(),
                confidence: EvidenceConfidence::High,
                complexity: 1,
                class_name: Some("Null".to_owned()),
                class_library_uri: Some("dart:core".to_owned()),
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            },
        );
    }
    FlowState {
        registers,
        stack,
        ..FlowState::default()
    }
}

/// Worklist fixpoint computing the meet-over-predecessors input state for
/// every basic block. The lattice shrinks monotonically (meet only removes),
/// so iteration terminates; loop joins keep a value only when every path
/// around the loop provably carries an identical expression.
#[allow(clippy::too_many_arguments)]
fn solve_block_states(
    blocks: &LifterBlocks,
    instructions: &[DecodedInstruction],
    abi: Abi,
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
    pool_loads: &BTreeMap<u64, usize>,
    field_layout: Option<&RecoveredFieldLayout>,
    entry_state: &FlowState,
    exceptional_blocks: &BTreeSet<usize>,
    edges: &[(usize, usize, &[crate::model::CatchEntryMove])],
    visit_budget: usize,
    context: LiftContext<'_, '_>,
) -> (Vec<FlowState>, bool) {
    if blocks.starts.is_empty() {
        return (Vec::new(), false);
    }
    // Successor lists per block index.
    let successors = block_successors(blocks, instructions);
    let predecessors = {
        let mut predecessors = vec![Vec::<usize>::new(); blocks.starts.len()];
        for (index, successors) in successors.iter().enumerate() {
            for successor in successors {
                if *successor < predecessors.len() {
                    predecessors[*successor].push(index);
                }
            }
        }
        predecessors
    };
    let mut inputs = vec![None::<FlowState>; blocks.starts.len()];
    let mut outputs = vec![None::<FlowState>; blocks.starts.len()];
    // Locations each join has merged. A merged location stays merged even
    // when its predecessors later agree, so joins in a cycle cannot flip
    // between a merged and a plain value forever.
    let mut merged_locations = vec![BTreeMap::<(bool, String), Expression>::new(); blocks.starts.len()];
    inputs[0] = Some(entry_state.clone());
    let mut pending = std::collections::VecDeque::from([0usize]);
    // A queued join already observes the latest outputs of all predecessors.
    // Scheduling it once per incoming edge wastes visits and can exhaust the
    // safety budget even when the dataflow has converged.
    let mut queued = vec![false; blocks.starts.len()];
    queued[0] = true;
    for &handler in exceptional_blocks {
        if handler < inputs.len() && handler != 0 {
            inputs[handler] = Some(handler_entry_state(abi, entry_state));
            pending.push_back(handler);
            queued[handler] = true;
        }
    }
    let mut visits = 0usize;
    while let Some(index) = pending.pop_front() {
        queued[index] = false;
        if visits >= visit_budget {
            // Partial inputs may still contain values from predecessors not
            // yet processed. Drop every such value before the emission pass.
            return (vec![FlowState::default(); blocks.starts.len()], true);
        }
        visits += 1;
        let mut input = if index == 0 {
            entry_state.clone()
        } else if exceptional_blocks.contains(&index) {
            // Meet over every throwing call that has been simulated.
            let mut merged: Option<FlowState> = None;
            for (thrower, _, moves) in edges.iter().filter(|(_, handler, _)| *handler == index) {
                let Some(output) = &outputs[*thrower] else {
                    continue;
                };
                let state = exceptional_state(abi, output, moves, object_pool);
                merged = Some(match merged {
                    Some(current) => {
                        FlowState::meet_at(&current, &state, Some((blocks.starts[index], abi)))
                    }
                    None => state,
                });
            }
            match merged {
                Some(mut merged) => {
                    stick_phis(&mut merged, blocks.starts[index], &mut merged_locations[index]);
                    merged
                }
                // Known throwing calls not simulated yet: wait for them, as
                // an ordinary join waits for its predecessors. A bottom state
                // here would flow around an enclosing loop and never recover.
                None if edges.iter().any(|(_, handler, _)| *handler == index) => continue,
                None => handler_entry_state(abi, entry_state),
            }
        } else {
            FlowState::default()
        };
        let predecessors = &predecessors[index];
        if index != 0 && !exceptional_blocks.contains(&index) {
            let mut merged: Option<FlowState> = None;
            for predecessor in predecessors {
                let Some(output) = &outputs[*predecessor] else {
                    continue;
                };
                merged = Some(match merged {
                    Some(current) => {
                        FlowState::meet_at(&current, output, Some((blocks.starts[index], abi)))
                    }
                    None => (*output).clone(),
                });
            }
            match merged {
                Some(mut merged) => {
                    stick_phis(&mut merged, blocks.starts[index], &mut merged_locations[index]);
                    input = merged;
                }
                // Not every predecessor has been simulated yet; retry later
                // when the worklist revisits via that predecessor.
                None if outputs.iter().any(std::option::Option::is_some) => continue,
                None => {}
            }
        }
        let mut working = input.clone();
        simulate_range(
            &mut working,
            instructions,
            Some(blocks.instruction_start(index)..blocks.instruction_end(index)),
            abi,
            symbols,
            object_pool,
            pool_loads,
            field_layout,
            None,
            context,
            false,
        );
        let changed = outputs[index].as_ref() != Some(&working);
        inputs[index] = Some(input);
        outputs[index] = Some(working);
        if changed {
            for successor in &successors[index] {
                if !queued[*successor] {
                    pending.push_back(*successor);
                    queued[*successor] = true;
                }
            }
            for (_, handler, _) in edges.iter().filter(|(thrower, _, _)| *thrower == index) {
                if !queued[*handler] {
                    pending.push_back(*handler);
                    queued[*handler] = true;
                }
            }
        }
    }
    let states = inputs
        .into_iter()
        .zip(outputs)
        .map(|(input, _)| input.unwrap_or_default())
        .collect();
    (states, false)
}

/// Keeps merged values monotone across solver iterations: a location that
/// merged at `join` stays merged, and its confidence and type facts only
/// weaken, whatever its predecessors carry on a later visit.
fn stick_phis(
    state: &mut FlowState,
    join: u64,
    merged: &mut BTreeMap<(bool, String), Expression>,
) {
    for (is_stack, values) in [(false, &mut state.registers), (true, &mut state.stack)] {
        for (location, value) in values.iter_mut() {
            let key = (is_stack, location.clone());
            let previous = merged.get(&key);
            if previous.is_none() && !is_phi_of(value, join) {
                continue;
            }
            let incoming = if is_phi_of(value, join) {
                value.clone()
            } else {
                phi_expression(join, location, value, value)
            };
            let sticky = match previous {
                Some(previous) => phi_expression(join, location, previous, &incoming),
                None => incoming,
            };
            merged.insert(key, sticky.clone());
            *value = sticky;
        }
    }
}

fn block_successors(blocks: &LifterBlocks, instructions: &[DecodedInstruction]) -> Vec<Vec<usize>> {
    let mut successors = vec![Vec::<usize>::new(); blocks.starts.len()];
    for index in 0..blocks.starts.len() {
        let range = blocks.ranges[index];
        let Some(last) = instructions[range.0..range.1].last() else {
            // Idiom fusion can delete every instruction of a block; treat the
            // emptied block as pure fall-through so the chain stays intact.
            if index + 1 < blocks.starts.len() && !successors[index].contains(&(index + 1)) {
                successors[index].push(index + 1);
            }
            continue;
        };
        let mut push = |address: u64| {
            if let Some(successor) = blocks.starts.binary_search(&address).ok()
                && !successors[index].contains(&successor)
            {
                successors[index].push(successor);
            }
        };
        let push_fallthrough = |successors: &mut Vec<Vec<usize>>| {
            if index + 1 < blocks.starts.len() && !successors[index].contains(&(index + 1)) {
                successors[index].push(index + 1);
            }
        };
        if is_return(&last.mnemonic, &last.operands) || is_trap(&last.mnemonic) {
            continue;
        }
        match branch_kind(&last.mnemonic) {
            Some(true) => {
                if let Some(target) = branch_target(&last.operands) {
                    push(target);
                }
                push_fallthrough(&mut successors);
            }
            Some(false) => {
                if let Some(target) = branch_target(&last.operands) {
                    push(target);
                } else {
                    push_fallthrough(&mut successors);
                }
            }
            None => push_fallthrough(&mut successors),
        }
    }
    successors
}

/// Sequential transfer-function pass over one instruction range. With an emit
/// sink this produces semantic statements; without one it only advances the
/// state (used by the fixpoint).
#[allow(clippy::too_many_arguments)]
/// Program-wide facts and per-body hints the block transfer function
/// consults, identically in the fixpoint and the emission pass.
#[derive(Clone, Copy)]
struct LiftContext<'a, 't> {
    dispatch_table: Option<&'a DispatchTableAnalysis<'t>>,
    dispatch_calls: Option<&'a BTreeMap<u64, DispatchCallEvidence>>,
    /// Value text -> `(class, library)` proven for the untyped value that
    /// text names (see [`receiver_type_hints`]).
    type_hints: &'a BTreeMap<String, (String, Option<String>)>,
    /// Collects class facts while emitting.
    lift_facts: Option<&'a std::cell::RefCell<LiftFacts>>,
}

impl LiftContext<'static, 'static> {
    /// No program-wide facts and no hints.
    #[cfg(test)]
    fn bare() -> Self {
        static NO_HINTS: BTreeMap<String, (String, Option<String>)> = BTreeMap::new();
        LiftContext {
            dispatch_table: None,
            dispatch_calls: None,
            type_hints: &NO_HINTS,
            lift_facts: None,
        }
    }
}

/// Class facts one lifting round proves, keyed by value text or callee.
#[derive(Clone, Debug, Default)]
pub(crate) struct LiftFacts {
    /// `(value text, class, library)` of receivers a call proves.
    pub receivers: Vec<(String, String, Option<String>)>,
    /// `(callee code address, parameter index, class)` of every register
    /// argument a direct call passes; `None` for an untyped value.
    pub arguments: Vec<(u64, usize, Option<(String, Option<String>)>)>,
    /// Class of the value each return leaves in the result register.
    pub returns: Vec<Option<(String, Option<String>)>>,
}

/// Classes a whole-program lifting pass proves for anonymous interfaces.
#[derive(Clone, Debug, Default)]
pub(crate) struct InferredClasses {
    /// Function address -> parameter index -> class every typed direct
    /// caller passes.
    pub parameters: BTreeMap<u64, BTreeMap<usize, (String, Option<String>)>>,
    /// Function address -> class every return of its body yields.
    pub results: BTreeMap<u64, (String, Option<String>)>,
}

/// Agreement across direct call sites and returns.
///
/// A parameter is typed only for a body reachable solely by direct calls
/// (not a dispatch-table row, closure or tear-off), when the call sites
/// passing a typed value all pass the same class and outnumber those
/// passing an untyped one. A result is typed when every return yields the
/// same class, ignoring `null`.
pub(crate) fn infer_interprocedural_classes(facts: &[(u64, bool, LiftFacts)]) -> InferredClasses {
    type Class = (String, Option<String>);
    // (typed class or conflict, typed count, untyped count)
    let mut arguments = BTreeMap::<(u64, usize), (Option<Option<&Class>>, usize, usize)>::new();
    for (_, _, function_facts) in facts {
        for (callee, index, class) in &function_facts.arguments {
            let entry = arguments.entry((*callee, *index)).or_insert((None, 0, 0));
            match class {
                None => entry.2 += 1,
                Some(class) => {
                    entry.1 += 1;
                    entry.0 = match entry.0 {
                        None => Some(Some(class)),
                        Some(Some(existing)) if existing.0 == class.0 => Some(Some(existing)),
                        Some(_) => Some(None),
                    };
                }
            }
        }
    }
    let direct_only = facts
        .iter()
        .filter(|(_, indirect, _)| !indirect)
        .map(|(address, _, _)| *address)
        .collect::<BTreeSet<_>>();
    let mut inferred = InferredClasses::default();
    for ((callee, index), (class, typed, untyped)) in arguments {
        if let Some(Some(class)) = class
            && typed >= untyped
            && direct_only.contains(&callee)
        {
            inferred
                .parameters
                .entry(callee)
                .or_default()
                .insert(index, class.clone());
        }
    }
    for (address, _, function_facts) in facts {
        let mut result: Option<&Class> = None;
        let mut agrees = !function_facts.returns.is_empty();
        for class in &function_facts.returns {
            match class {
                Some(class) if class.0 == "Null" => {}
                Some(class) if result.is_none_or(|existing| existing.0 == class.0) => {
                    result = Some(class);
                }
                _ => agrees = false,
            }
        }
        if agrees && let Some(class) = result {
            inferred.results.insert(*address, class.clone());
        }
    }
    inferred
}

/// A value's class as a fact; `Object` proves nothing.
fn value_class(value: &Expression) -> Option<(String, Option<String>)> {
    value
        .class_name
        .clone()
        .filter(|class| class != "Object" && !value.high_word)
        .map(|class| (class, value.class_library_uri.clone()))
}

/// Lifting rounds per body: the first collects receiver evidence; later
/// rounds apply it, and may prove receivers reached through typed values.
const MAX_TYPE_HINT_ROUNDS: usize = 3;

/// Classes for untyped values that every piece of evidence agrees on and
/// that no earlier round already applied. Merged values are skipped: their
/// text names one join, not one value.
fn receiver_type_hints(
    evidence: &[(String, String, Option<String>)],
    known: &BTreeMap<String, (String, Option<String>)>,
) -> BTreeMap<String, (String, Option<String>)> {
    let mut proven = BTreeMap::<&str, Option<(&str, Option<&str>)>>::new();
    for (text, class, library) in evidence {
        let register = text.len() >= 2
            && matches!(text.as_bytes()[0], b'r' | b'x' | b'w' | b'd' | b's' | b'v')
            && text[1..].chars().all(|character| character.is_ascii_digit());
        if register
            || text.starts_with("phi_")
            || text.starts_with("local")
            || matches!(text.as_str(), "null" | "true" | "false")
            || known.contains_key(text)
        {
            continue;
        }
        let fact = Some((class.as_str(), library.as_deref()));
        match proven.get(text.as_str()) {
            None => {
                proven.insert(text, fact);
            }
            Some(existing) if *existing == fact => {}
            Some(_) => {
                proven.insert(text, None);
            }
        }
    }
    proven
        .into_iter()
        .filter_map(|(text, fact)| {
            let (class, library) = fact?;
            Some((text.to_owned(), (class.to_owned(), library.map(str::to_owned))))
        })
        .collect()
}

fn simulate_range(
    state: &mut FlowState,
    instructions: &[DecodedInstruction],
    range: Option<std::ops::Range<usize>>,
    abi: Abi,
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
    pool_loads: &BTreeMap<u64, usize>,
    field_layout: Option<&RecoveredFieldLayout>,
    mut emit: Option<&mut Vec<SemanticStatement>>,
    context: LiftContext<'_, '_>,
    returns_unboxed_double: bool,
) {
    let LiftContext {
        dispatch_table,
        dispatch_calls,
        type_hints,
        lift_facts,
    } = context;
    let return_register = abi_return_register(abi);
    // Register holding this body's own result at a return.
    let result_register = if returns_unboxed_double {
        TargetLayout::of(abi).fpu_return_register
    } else {
        return_register
    };
    let begin = range.as_ref().map_or(0, |range| range.start);
    let end = range.as_ref().map_or(instructions.len(), |range| range.end);
    let mut last_comparison: Option<Expression> = None;
    // Registers holding an object-pool entry loaded in this block, by pool
    // index: stub calls read the pool object itself (a closure's Function)
    // rather than its display text.
    let mut pool_registers = BTreeMap::<String, usize>::new();
    // ARM32: the value the last flag-setting low-word operation (`adds`,
    // `subs`, `rsbs`) computed, whose carry the next `adc`/`sbc`/`rsc`
    // folds into the high word of the same int64 pair.
    let mut last_carry: Option<Expression> = None;
    // Pending integer division (quotient register, dividend, divisor) used to
    // pair a following multiply-subtract into Dart's `%`.
    let mut last_division: Option<(String, String, String)> = None;
    // 32-bit (compressed) reference loads awaiting decompression:
    // destination register -> (base register, load displacement).
    let mut compressed_loads = BTreeMap::<String, (String, i64)>::new();
    // Registers holding `array + index * element size`: register ->
    // (array, index). A load at the Array data offset through one reads
    // `array[index]`.
    let mut element_pointers = BTreeMap::<String, (Expression, Expression)>::new();
    // Compressed element loads awaiting decompression: register -> read.
    let mut pending_elements = BTreeMap::<String, Expression>::new();
    let stack_register = abi_stack_register(abi);
    // Registers holding a field-table base loaded from the thread, and
    // whether it is the shared table.
    let mut field_table_bases = BTreeMap::<String, bool>::new();
    let FlowState {
        registers,
        stack,
        buffers,
        aliases,
        ..
    } = state;
    macro_rules! push_statement {
        ($statement:expr) => {
            if let Some(sink) = emit.as_deref_mut() {
                sink.push($statement);
            }
        };
    }
    // Argument registers written in this block since its last call: the
    // parallel move feeding a call's fixed argument registers sits in the
    // call's own block, unlike values that merely flow in from predecessors.
    let mut block_written_argument_registers = 0u16;
    let mut after_call = false;
    for instruction in instructions[begin..end].iter() {
        if std::mem::replace(&mut after_call, is_call(&instruction.mnemonic)) {
            block_written_argument_registers = 0;
        }
        if !type_hints.is_empty() {
            for value in registers.values_mut() {
                apply_type_hint(value, type_hints);
            }
        }
        let mut operands = split_operands(&instruction.operands);
        // x64 two-operand arithmetic reads its destination: lift
        // `add rdx, src` as `add rdx, rdx, src` so it shares the ARM arms
        // (including heap-base decompression). `xor` keeps its own
        // two-operand arm for boolean negation.
        if abi == Abi::X86_64
            && operands.len() == 2
            && matches!(
                instruction.mnemonic.as_str(),
                "add" | "sub" | "imul" | "and" | "or"
            )
            && !operands[0].contains('[')
        {
            operands.insert(1, operands[0].clone());
        }
        // Any write to the stack pointer retires the outgoing argument area
        // (prologue frame allocation or epilogue restore).
        let destination_register = operands
            .first()
            .filter(|_| writes_first_operand(&instruction.mnemonic))
            .map(|target| normalize_register(target));
        if let Some(destination) = destination_register.as_deref() {
            pool_registers.remove(destination);
            if let Some(index) = pool_loads.get(&instruction.address) {
                pool_registers.insert(destination.to_owned(), *index);
            }
            let layout = TargetLayout::of(abi);
            if let Some(bit) = layout.cpu_argument_index(destination) {
                state.written_argument_registers |= 1 << bit;
                block_written_argument_registers |= 1 << bit;
            } else if let Some(bit) = layout.fpu_argument_index(destination) {
                state.written_argument_registers |= 1 << (8 + bit);
                block_written_argument_registers |= 1 << (8 + bit);
            }
        }
        // A load may overwrite its own element-pointer base
        // (`ldr ip, [ip, #0xb]`): the pointer stays visible to this
        // instruction only.
        let displaced_element = destination_register.as_deref().and_then(|destination| {
            element_pointers
                .remove(destination)
                .map(|pointer| (destination.to_owned(), pointer))
        });
        // Likewise the decompressing add writes the register it reads.
        let displaced_pending = destination_register.as_deref().and_then(|destination| {
            pending_elements
                .remove(destination)
                .map(|element| (destination.to_owned(), element))
        });
        // The class id a dispatch call indexes with, read before the call
        // clobbers it; switchable calls check the receiver loaded into
        // their own receiver register instead.
        let dispatch_receiver = is_call(&instruction.mnemonic)
            .then(|| {
                state
                    .class_ids
                    .get(dispatch_class_id_register(abi))
                    .cloned()
            })
            .flatten();
        let switchable_receiver = is_call(&instruction.mnemonic)
            .then(|| registers.get(switchable_receiver_register(abi)).cloned())
            .flatten();
        track_class_ids(
            abi,
            &instruction.mnemonic,
            &operands,
            registers,
            &mut state.tag_words,
            &mut state.class_ids,
        );
        // Slot keys in this instruction use the deltas before it executes
        // (pre-index writeback addresses `old SP + displacement`).
        let deltas = state.deltas;
        state.deltas = deltas.after(abi, &instruction.mnemonic, &instruction.operands);
        if state.deltas.sp != deltas.sp
            || state.deltas.sp.is_none() && moves_stack_pointer(abi, instruction)
        {
            state.outgoing.clear();
            aliases.retain(|key, _| !key.starts_with("out:"));
            // Literal stack-pointer-relative slot names no longer denote the
            // same memory once the pointer moves.
            let prefix = format!("[{stack_register}");
            stack.retain(|key, _| !key.starts_with(&prefix));
        }
        let stored_register_list = instruction
            .mnemonic
            .starts_with("stm")
            .then(|| operands.get(1).and_then(|list| register_list(list)))
            .flatten();
        if moves_stack_pointer(abi, instruction) {
            // Pushes and frame set-up are not outgoing argument stores.
        } else if let Some(((base, displacement), stored_operands)) = match instruction
            .mnemonic
            .as_str()
        {
            "str" | "stur" | "stp" => operands
                .last()
                .and_then(|value| arm_memory_address(value))
                .map(|address| (address, &operands[..operands.len().saturating_sub(1)])),
            "mov" | "movq" if operands.first().is_some_and(|value| value.contains('[')) => operands
                .first()
                .and_then(|value| arm_memory_address(value))
                .map(|address| (address, &operands[1..])),
            // ARM32 `stm sp, {a, b}` stores ascending from SP.
            "stm" | "stmia" if abi == Abi::ArmeabiV7a => operands
                .first()
                .filter(|base| !base.ends_with('!'))
                .zip(stored_register_list.as_deref())
                .map(|(base, registers)| ((normalize_register(base), 0), registers)),
            _ => None,
        } && base == stack_register
            && displacement >= 0
            && deltas.sp.is_none_or(|delta| delta < 0)
        {
            // Outgoing argument store for the next call: after the frame is
            // set up, every non-negative stack-pointer displacement lies in
            // the reserved outgoing-argument area (locals and spills are
            // frame-pointer relative).
            let width = TargetLayout::of(abi).word_size;
            for (stored, operand) in stored_operands.iter().enumerate() {
                let slot = displacement + stored as i64 * width;
                if let Some(value) = resolve_expression(operand, registers, object_pool) {
                    state.outgoing.insert(slot, value);
                } else {
                    state.outgoing.remove(&slot);
                }
                match aliases.get(&normalize_register(operand)).copied() {
                    Some(alias) => {
                        aliases.insert(outgoing_key(slot), alias);
                    }
                    None => {
                        aliases.remove(&outgoing_key(slot));
                    }
                }
            }
            continue;
        }
        note_named_argument_compare(&instruction.mnemonic, &operands, registers, object_pool, stack);
        if let Some((target, value)) =
            optional_parameter_prologue(abi, &instruction.mnemonic, &operands, registers, stack)
        {
            compressed_loads.remove(&target);
            aliases.remove(&target);
            registers.insert(target, value);
            continue;
        }
        // A Dart callee may assign any static field, so spilled snapshots of
        // one are no longer proven current. VM stubs do not run Dart code.
        if is_call(&instruction.mnemonic)
            && !state.static_reads.is_empty()
            && !direct_call_target(&instruction.mnemonic, &instruction.operands)
                .and_then(|target| symbols.get(&target))
                .is_some_and(|symbol| {
                    // A suspension runs arbitrary Dart code before resuming.
                    symbol.label.starts_with("stub ")
                        && !symbol.label.starts_with("stub Await")
                        && !symbol.label.starts_with("stub Suspend")
                })
        {
            for value in stack.values_mut() {
                if mentions_any(&value.text, &state.static_reads) {
                    value.confidence = EvidenceConfidence::Low;
                }
            }
        }
        if let Some(layout) = field_layout.filter(|layout| layout.field_tables.is_some()) {
            let memory = plain_memory_operands(abi, &instruction.mnemonic, &operands);
            let access = memory.as_ref().and_then(|access| {
                let shared = *field_table_bases.get(&access.base)?;
                Some((layout.static_field(shared, access.displacement)?, shared))
            });
            if is_call(&instruction.mnemonic) {
                field_table_bases.clear();
            } else if let Some(destination) = destination_register.as_deref() {
                field_table_bases.remove(destination);
            }
            if let Some(memory) = memory.as_ref().filter(|memory| !memory.store)
                && memory.base == abi_thread_register(abi)
                && let Some(shared) = layout.field_table_at(memory.displacement)
            {
                let target = normalize_register(&memory.value);
                registers.remove(&target);
                compressed_loads.remove(&target);
                aliases.remove(&target);
                field_table_bases.insert(target, shared);
                continue;
            }
            if let Some(memory) = memory.as_ref().filter(|memory| !memory.store)
                && memory.base == abi_thread_register(abi)
                && let Some((text, class_name)) = layout.thread_constant(memory.displacement)
            {
                let target = normalize_register(&memory.value);
                compressed_loads.remove(&target);
                aliases.remove(&target);
                registers.insert(
                    target,
                    Expression {
                        text: text.to_owned(),
                        confidence: EvidenceConfidence::High,
                        complexity: 1,
                        class_name: class_name.map(str::to_owned),
                        class_library_uri: class_name.map(|_| "dart:core".to_owned()),
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: class_name.is_some(),
                    },
                );
                continue;
            }
            if let (Some(memory), Some((field, shared))) = (memory, access) {
                let confidence = if field.named {
                    EvidenceConfidence::High
                } else {
                    EvidenceConfidence::Low
                };
                let field_id = field.offset / field.word;
                let address = format!("0x{:x}", instruction.address);
                if memory.store {
                    let value = resolve_expression(&memory.value, registers, object_pool);
                    // Earlier reads of this field no longer describe it.
                    registers.retain(|_, value| !value.text.contains(&field.name));
                    stack.retain(|_, value| !value.text.contains(&field.name));
                    push_statement!(SemanticStatement::StaticFieldWrite {
                        field: field.name,
                        field_id,
                        shared,
                        confidence: value.as_ref().map_or(EvidenceConfidence::Low, |value| {
                            weaker(confidence, value.confidence)
                        }),
                        value: value.map_or_else(
                            || format!("aot.unresolvedRegister('{}')", memory.value),
                            |value| value.text,
                        ),
                        address,
                    });
                } else {
                    let target = normalize_register(&memory.value);
                    push_statement!(SemanticStatement::StaticFieldRead {
                        field: field.name.clone(),
                        field_id,
                        shared,
                        confidence,
                        address,
                    });
                    compressed_loads.remove(&target);
                    aliases.remove(&target);
                    state.static_reads.insert(field.name.clone());
                    registers.insert(
                        target,
                        Expression {
                            text: field.name,
                            confidence,
                            complexity: 1,
                            class_name: field.value_class,
                            class_library_uri: field.value_library_uri,
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                }
                continue;
            }
        }
        match instruction.mnemonic.as_str() {
            // ARM64 scalar double arithmetic and x64 SSE2 two-operand forms.
            "fadd" | "fsub" | "fmul" | "fdiv" | "addsd" | "subsd" | "mulsd" | "divsd"
                if operands.len() >= 2 =>
            {
                let target = fpu_register(&operands[0]);
                let (left, right) = if operands.len() >= 3 {
                    (&operands[1], &operands[2])
                } else {
                    (&operands[0], &operands[1])
                };
                let operator = match instruction.mnemonic.as_str() {
                    "fadd" | "addsd" => "+",
                    "fsub" | "subsd" => "-",
                    "fmul" | "mulsd" => "*",
                    _ => "/",
                };
                let value = resolve_fpu_operand(left, registers, stack, abi, deltas, object_pool)
                    .zip(resolve_fpu_operand(right, registers, stack, abi, deltas, object_pool))
                    .and_then(|(left, right)| binary_expression(left, operator, right))
                    .map(|expression| Expression {
                        class_name: Some("double".to_owned()),
                        class_library_uri: Some("dart:core".to_owned()),
                        ..expression
                    });
                match value {
                    Some(value) => {
                        registers.insert(target, value);
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            // Integer to double: Dart's implicit `toDouble()` in mixed
            // arithmetic keeps the integer expression's text.
            "scvtf" | "cvtsi2sd" if operands.len() >= 2 => {
                let target = fpu_register(&operands[0]);
                match resolve_expression(&operands[1], registers, object_pool) {
                    Some(value) => {
                        registers.insert(
                            target,
                            Expression {
                                class_name: Some("double".to_owned()),
                                class_library_uri: Some("dart:core".to_owned()),
                                high_word: false,
                                exact_class: false,
                                ..value
                            },
                        );
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            // `eor v0.16b, v0.16b, v0.16b` / `xorps xmm0, xmm0` clear to 0.0.
            "eor" | "xorps" | "xorpd" | "pxor"
                if operands.len() >= 2
                    && is_vector_register(&operands[0])
                    && operands
                        .iter()
                        .all(|operand| fpu_register(operand) == fpu_register(&operands[0])) =>
            {
                registers.insert(
                    fpu_register(&operands[0]),
                    Expression {
                        text: "0.0".to_owned(),
                        confidence: EvidenceConfidence::High,
                        complexity: 1,
                        class_name: Some("double".to_owned()),
                        class_library_uri: Some("dart:core".to_owned()),
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: false,
                    },
                );
            }
            // ARM32 NEON register copy (`vorr q0, q2, q2`): the scalar double
            // in the low half of qM moves to the low half of qD.
            "vorr"
                if operands.len() == 3
                    && operands[1] == operands[2]
                    && operands[0].starts_with('q')
                    && operands[1].starts_with('q') =>
            {
                let low_half = |operand: &str| {
                    operand[1..]
                        .parse::<u32>()
                        .ok()
                        .map(|index| format!("d{}", index * 2))
                };
                if let (Some(target), Some(source)) = (low_half(&operands[0]), low_half(&operands[1])) {
                    match registers.get(&source).cloned() {
                        Some(value) => {
                            registers.insert(target, value);
                        }
                        None => {
                            registers.remove(&target);
                        }
                    }
                }
            }
            // ARM64 vector register copy (`mov v0.16b, v1.16b`).
            "mov" if operands.len() == 2 && abi == Abi::Arm64V8a && is_vector_register(&operands[0]) => {
                let target = fpu_register(&operands[0]);
                match registers.get(&fpu_register(&operands[1])).cloned() {
                    Some(value) => {
                        registers.insert(target, value);
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            "fmov" | "vmovd" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                let value = floating_immediate_text(&operands[1])
                    .map(|text| Expression {
                        text,
                        confidence: EvidenceConfidence::High,
                        complexity: 1,
                        class_name: Some("double".to_owned()),
                        class_library_uri: Some("dart:core".to_owned()),
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: false,
                    })
                    .or_else(|| resolve_expression(&operands[1], registers, object_pool));
                match value {
                    Some(value) => {
                        registers.insert(target, value);
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            // FPU stores: ARM32 `vstr dN, [base, #d]`, x64 `movsd [base + d], xmmN`.
            "vstr" | "movsd" | "movaps" | "movapd"
                if operands.len() >= 2
                    && (instruction.mnemonic == "vstr" || operands[0].contains('[')) =>
            {
                let (value_operand, address) = if instruction.mnemonic == "vstr" {
                    (&operands[0], &operands[1])
                } else {
                    (&operands[1], &operands[0])
                };
                let value = resolve_expression(value_operand, registers, object_pool);
                if let Some(slot) = stack_slot_key(abi, address, deltas) {
                    match value {
                        Some(value) => {
                            stack.insert(slot, value);
                        }
                        None => {
                            stack.remove(&slot);
                        }
                    }
                } else if let Some((base, displacement)) = arm_memory_address(address)
                    && fill_pending_box(registers, &base, displacement, value.as_ref())
                {
                } else if let Some((base, displacement)) = arm_memory_address(address)
                    && let Some(receiver) = registers.get(&base).cloned()
                    && let Some(value) = value
                    && let Some((field_offset, field)) =
                        recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                {
                    let confidence = if field.synthesized_slot {
                        EvidenceConfidence::Low
                    } else {
                        weaker(receiver.confidence, value.confidence)
                    };
                    push_statement!(SemanticStatement::FieldWrite {
                        receiver: receiver.text,
                        field: field.name.clone(),
                        offset: field_offset,
                        value: value.text,
                        confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                }
            }
            "movsd" | "vldr" | "movaps" | "movapd" | "vmov.f64" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                let source = &operands[1];
                let value = if let Some(slot) = stack_slot_key(abi, source, deltas) {
                    // A double reloaded from its spill slot.
                    stack.get(&slot).cloned()
                } else if let Some(index) = pool_loads.get(&instruction.address).copied() {
                    object_pool
                        .and_then(|pool| pool.get(index))
                        .and_then(|value| object_pool_f64_expression(value))
                } else if let Some((base, displacement)) = arm_memory_address(source)
                    && let Some(receiver) = registers.get(&base).cloned()
                    && let Some((field_offset, field)) =
                        recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                {
                    let expression = field_expression(&receiver.text, &field.name);
                    let confidence = if field.synthesized_slot {
                        EvidenceConfidence::Low
                    } else {
                        receiver.confidence
                    };
                    push_statement!(SemanticStatement::FieldRead {
                        receiver: receiver.text.clone(),
                        field: field.name.clone(),
                        offset: field_offset,
                        expression: expression.clone(),
                        confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                    Some(Expression {
                        text: expression,
                        confidence,
                        complexity: receiver.complexity.saturating_add(1),
                        class_name: Some("double".to_owned()),
                        class_library_uri: Some("dart:core".to_owned()),
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: false,
                    })
                } else {
                    resolve_expression(source, registers, object_pool)
                };
                match value {
                    Some(value) => {
                        registers.insert(target, value);
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            // Fused ARM32 int64 boxing (see `fuse_arm32_int64_idioms`).
            "box" if operands.len() == 2 => {
                let target = normalize_register(&operands[0]);
                match resolve_expression(&operands[1], registers, object_pool) {
                    Some(value) => {
                        registers.insert(
                            target,
                            Expression {
                                raw: false,
                                high_word: false,
                                ..value
                            },
                        );
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            "mov" | "movz" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                // Shifted register moves: ARM32 emits Smi untag/re-tag as
                // `mov rd, rs, asr #1` / `mov rd, rs, lsl #1`.
                if operands.len() >= 3 {
                    let shift = operands[2].replace(' ', "");
                    let source = resolve_expression(&operands[1], registers, object_pool);
                    match (source, shift.as_str()) {
                        (Some(value), text)
                            if text.ends_with("asr#1") || text.ends_with("asr#0x1") =>
                        {
                            let untagged = !value.raw;
                            registers.insert(
                                target.clone(),
                                Expression {
                                    complexity: value.complexity.saturating_add(1),
                                    raw: true,
                                    definition_site: None,
                                    high_word: false,
                                    exact_class: false,
                                    text: if untagged {
                                        value.text.clone()
                                    } else {
                                        format!("({} >> 1)", value.text)
                                    },
                                    ..value
                                },
                            );
                            compressed_loads.remove(&target);
                            continue;
                        }
                        (Some(value), text)
                            if text.ends_with("lsl#1") || text.ends_with("lsl#0x1") =>
                        {
                            // A one-bit left shift before a store is a re-tag;
                            // the stored value is the source expression.
                            registers.insert(target.clone(), value);
                            compressed_loads.remove(&target);
                            continue;
                        }
                        _ => {}
                    }
                }
                // Memory-destination move (x64 stores): spill to frame
                // slots, fill tracked element buffers, or write fields.
                if operands[0].contains('[') {
                    if let Some(slot) = stack_slot_key(abi, &operands[0], deltas) {
                        let source_register = normalize_register(&operands[1]);
                        // Spilling a tracked array keeps it alive across calls.
                        spill_alias(aliases, &source_register, &slot);
                        // An untracked value still overwrites the slot: a
                        // stale entry would fabricate the value reloaded.
                        match resolve_expression(&operands[1], registers, object_pool) {
                            Some(value) => {
                                stack.insert(slot, value);
                            }
                            None => {
                                stack.remove(&slot);
                            }
                        }
                        continue;
                    }
                    // `mov dword ptr [array + index*4 + 0xf], value`.
                    if let Some((base, index, scale, displacement)) = operands
                        .first()
                        .and_then(|value| x64_scaled_index_address(value))
                        && matches!(scale, 2 | 4)
                        && let Some(array) = registers.get(&base).cloned()
                        && let Some(index) = registers.get(&index).cloned()
                        && let Some(value) =
                            resolve_expression(&operands[1], registers, object_pool)
                        && let Some(write) = element_write(
                            abi,
                            &(
                                array,
                                if scale == 2 {
                                    untagged_index(index)
                                } else {
                                    index
                                },
                            ),
                            displacement,
                            &value,
                            instruction.address,
                        )
                    {
                        push_statement!(write);
                        continue;
                    }
                    if let Some((base, displacement)) =
                        operands.first().and_then(|value| arm_memory_address(value))
                    {
                        let value = resolve_expression(&operands[1], registers, object_pool);
                        // A boxed integer's payload store (x64 spells it
                        // `mov [rax + 7], rcx`) makes the box that value.
                        if fill_pending_box(registers, &base, displacement, value.as_ref()) {
                            continue;
                        }
                        if let Some(write) = store_element(
                            abi,
                            buffers,
                            aliases,
                            &base,
                            displacement,
                            value.as_ref(),
                        ) {
                            if let Some(write) = write {
                                push_statement!(write.statement(instruction.address));
                            }
                            continue;
                        }
                        if let Some(value) = value
                            && let Some(receiver) = registers.get(&base).cloned()
                            && let Some((field_offset, field)) =
                                recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                        {
                            let confidence = if field.synthesized_slot {
                                EvidenceConfidence::Low
                            } else {
                                weaker(receiver.confidence, value.confidence)
                            };
                            push_statement!(SemanticStatement::FieldWrite {
                                receiver: receiver.text,
                                field: field.name.clone(),
                                offset: field_offset,
                                value: value.text,
                                confidence,
                                address: format!("0x{:x}", instruction.address),
                            });
                        }
                    }
                    compressed_loads.remove(&normalize_register(
                        operands.first().unwrap_or(&String::new()),
                    ));
                    continue;
                }
                let source_operand = &operands[1];
                if source_operand.contains('[') {
                    // x64 memory-source move: route through the same slot,
                    // pool, and field resolution as the ARM load arms.
                    aliases.remove(&target);
                    if let Some(slot) = stack_slot_key(abi, source_operand, deltas) {
                        reload_alias(aliases, &target, &slot);
                        if let Some(value) = stack.get(&slot).cloned() {
                            registers.insert(target.clone(), value);
                        } else {
                            let displacement = arm_memory_address(source_operand)
                                .map(|(_, displacement)| displacement)
                                .unwrap_or_default();
                            registers.insert(
                                target.clone(),
                                Expression {
                                    text: format!("local{:x}", displacement.unsigned_abs()),
                                    confidence: EvidenceConfidence::Low,
                                    complexity: 1,
                                    class_name: None,
                                    class_library_uri: None,
                                    raw: false,
                                    definition_site: None,
                                    high_word: false,
                                    exact_class: false,
                                },
                            );
                        }
                        compressed_loads.remove(&target);
                        continue;
                    }
                    if let Some(index) = pool_loads.get(&instruction.address).copied()
                        && let Some(value) = object_pool.and_then(|pool| pool.get(index))
                    {
                        registers.insert(
                            target.clone(),
                            Expression {
                                text: pool_value_text(value),
                                confidence: EvidenceConfidence::High,
                                complexity: 1,
                                class_name: snapshot_instance_class(value),
                                class_library_uri: None,
                                raw: false,
                                // A canonical instance's class is its
                                // runtime class.
                                definition_site: None,
                                high_word: false,
                                exact_class: snapshot_instance_class(value).is_some(),
                            },
                        );
                        compressed_loads.remove(&target);
                        continue;
                    }
                    // 32-bit destinations are compressed-reference loads.
                    let raw_destination = operands[0].trim().to_ascii_lowercase();
                    let is_32bit_destination = raw_destination.starts_with('e')
                        || (raw_destination.starts_with('r')
                            && raw_destination.ends_with('d')
                            && raw_destination.len() > 2
                            && raw_destination[1..raw_destination.len() - 1]
                                .chars()
                                .all(|character| character.is_ascii_digit()));
                    // `mov edi, dword ptr [array + index*4 + 0xf]`: an Array
                    // element, decompressed by the following heap-base add.
                    if is_32bit_destination
                        && let Some((base, index, scale, displacement)) =
                            x64_scaled_index_address(source_operand)
                        && matches!(scale, 2 | 4)
                        && let Some(array) = registers.get(&base).cloned()
                    {
                        // An untracked index keeps its register name, which
                        // renders as an explicit unresolved register.
                        let index = registers.get(&index).cloned().unwrap_or(Expression {
                            text: index,
                            confidence: EvidenceConfidence::Low,
                            complexity: 1,
                            class_name: None,
                            class_library_uri: None,
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        });
                        let index = if scale == 2 {
                            untagged_index(index)
                        } else {
                            index
                        };
                        match element_read(abi, &(array, index), displacement) {
                            Some(element) => {
                                pending_elements.insert(target.clone(), element);
                            }
                            None => {
                                pending_elements.remove(&target);
                            }
                        }
                        compressed_loads.remove(&target);
                        registers.remove(&target);
                        continue;
                    }
                    if is_32bit_destination
                        && !pool_loads.contains_key(&instruction.address)
                        && let Some((base, displacement)) = arm_memory_address(source_operand)
                    {
                        compressed_loads.insert(target.clone(), (base, displacement));
                        registers.remove(&target);
                        continue;
                    }
                    compressed_loads.remove(&target);
                    // A full-width load from an object is an unboxed field
                    // (references are compressed): resolve it like ARM does.
                    if source_operand.contains("qword")
                        && let Some((base, displacement)) = arm_memory_address(source_operand)
                        && let Some(receiver) = registers.get(&base).cloned()
                        && let Some((field_offset, field)) =
                            recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                    {
                        let expression = field_expression(&receiver.text, &field.name);
                        let confidence = if field.synthesized_slot {
                            EvidenceConfidence::Low
                        } else {
                            receiver.confidence
                        };
                        registers.insert(
                            target.clone(),
                            Expression {
                                text: expression.clone(),
                                confidence,
                                complexity: receiver.complexity.saturating_add(1),
                                class_name: field.value_class.clone(),
                                class_library_uri: field.value_library_uri.clone(),
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                        push_statement!(SemanticStatement::FieldRead {
                            receiver: receiver.text,
                            field: field.name.clone(),
                            offset: field_offset,
                            expression,
                            confidence,
                            address: format!("0x{:x}", instruction.address),
                        });
                        continue;
                    }
                    registers.remove(&target);
                    continue;
                }

                if let Some(value) = resolve_expression(&operands[1], registers, object_pool) {
                    registers.insert(target.clone(), value);
                } else {
                    registers.remove(&target);
                }
                compressed_loads.remove(&target);
                // A register copy carries an in-flight element buffer.
                let source = normalize_register(&operands[1]);
                if source != target {
                    match aliases.get(&source).copied() {
                        Some(alias) => {
                            aliases.insert(target.clone(), alias);
                        }
                        None => {
                            aliases.remove(&target);
                        }
                    }
                }
            }
            "movk" if operands.len() >= 2 => {
                registers.remove(&normalize_register(&operands[0]));
            }
            "sdiv" | "udiv" if operands.len() >= 3 => {
                let target = normalize_register(&operands[0]);
                last_division = Some((
                    target.clone(),
                    normalize_register(&operands[1]),
                    normalize_register(&operands[2]),
                ));
                if let (Some(left), Some(right)) = (
                    resolve_expression(&operands[1], registers, object_pool),
                    resolve_expression(&operands[2], registers, object_pool),
                ) {
                    // Dart `~/` is truncating integer division; optimized
                    // code also uses it for provably non-negative `%`
                    // quotients.
                    let raw = left.raw || right.raw;
                    registers.insert(
                        target,
                        Expression {
                            text: format!("({} ~/ {})", left.text, right.text),
                            confidence: weaker(left.confidence, right.confidence),
                            complexity: left
                                .complexity
                                .saturating_add(right.complexity)
                                .saturating_add(1),
                            class_name: None,
                            class_library_uri: None,
                            raw,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                } else {
                    registers.remove(&target);
                }
            }
            "neg" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                match resolve_expression(&operands[1], registers, object_pool) {
                    Some(value) if value.complexity < 32 => {
                        registers.insert(
                            target,
                            Expression {
                                text: format!("(-{})", value.text),
                                confidence: value.confidence,
                                complexity: value.complexity.saturating_add(1),
                                class_name: None,
                                class_library_uri: None,
                                raw: value.raw,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    _ => {
                        registers.remove(&target);
                    }
                }
            }
            "mvn" | "orn" | "bic" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                match resolve_expression(&operands[1], registers, object_pool) {
                    Some(value) if value.complexity < 32 => {
                        registers.insert(
                            target,
                            Expression {
                                text: format!("(~{})", value.text),
                                confidence: value.confidence,
                                complexity: value.complexity.saturating_add(1),
                                class_name: None,
                                class_library_uri: None,
                                raw: value.raw,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    _ => {
                        registers.remove(&target);
                    }
                }
            }
            // Multiply-subtract: with the matching preceding `sdiv` this is
            // Dart's `%` (remainder); standalone it stays arithmetic.
            "msub" | "msubl" if operands.len() >= 4 => {
                let target = normalize_register(&operands[0]);
                let product_register = normalize_register(&operands[1]);
                let is_remainder_of_last_division =
                    last_division
                        .as_ref()
                        .is_some_and(|(quotient, dividend, divisor)| {
                            quotient == &product_register
                                && normalize_register(&operands[2]) == *divisor
                                && normalize_register(&operands[3]) == *dividend
                        });
                if let (Some(product), Some(divisor), Some(minuend)) = (
                    resolve_expression(&operands[1], registers, object_pool),
                    resolve_expression(&operands[2], registers, object_pool),
                    resolve_expression(&operands[3], registers, object_pool),
                ) {
                    let text = if is_remainder_of_last_division {
                        format!("({} % {})", minuend.text, divisor.text)
                    } else {
                        format!("({} - ({} * {}))", minuend.text, product.text, divisor.text)
                    };
                    registers.insert(
                        target,
                        Expression {
                            text,
                            confidence: weaker(
                                minuend.confidence,
                                weaker(product.confidence, divisor.confidence),
                            ),
                            complexity: minuend
                                .complexity
                                .saturating_add(product.complexity)
                                .saturating_add(divisor.complexity),
                            class_name: None,
                            class_library_uri: None,
                            raw: minuend.raw || product.raw || divisor.raw,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                } else {
                    registers.remove(&target);
                }
                last_division = None;
            }
            // Condition set/select: materialize the pending comparison as a
            // Dart bool (or a conditional expression for selects).
            "cset" | "csetm" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                let condition = operands
                    .get(1)
                    .and_then(|code| comparison_from_condition_code(code, &last_comparison));
                match condition {
                    Some(text) => {
                        registers.insert(
                            target,
                            Expression {
                                text,
                                confidence: last_comparison
                                    .as_ref()
                                    .map_or(EvidenceConfidence::Low, |c| c.confidence),
                                complexity: last_comparison
                                    .as_ref()
                                    .map_or(2, |c| c.complexity.saturating_add(1)),
                                class_name: Some("bool".to_owned()),
                                class_library_uri: Some("dart:core".to_owned()),
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            "csinc"
                if operands.len() >= 4
                    && operands[1].trim() == "xzr"
                    && operands[2].trim() == "xzr" =>
            {
                // csinc xd, xzr, xzr, inv-cond == cset xd, cond
                let target = normalize_register(&operands[0]);
                let inverted = operands
                    .get(3)
                    .map(|code| invert_condition_code(code.trim()))
                    .unwrap_or_default();
                let condition = inverted
                    .as_deref()
                    .and_then(|code| comparison_from_condition_code(code, &last_comparison));
                match condition {
                    Some(text) => {
                        registers.insert(
                            target,
                            Expression {
                                text,
                                confidence: last_comparison
                                    .as_ref()
                                    .map_or(EvidenceConfidence::Low, |c| c.confidence),
                                complexity: last_comparison
                                    .as_ref()
                                    .map_or(2, |c| c.complexity.saturating_add(1)),
                                class_name: Some("bool".to_owned()),
                                class_library_uri: Some("dart:core".to_owned()),
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    None => {
                        registers.remove(&target);
                    }
                }
            }
            "csel" if operands.len() >= 4 => {
                let target = normalize_register(&operands[0]);
                let condition = operands
                    .get(3)
                    .and_then(|code| comparison_from_condition_code(code, &last_comparison));
                if let (Some(test), Some(left), Some(right)) = (
                    condition,
                    resolve_expression(&operands[1], registers, object_pool),
                    resolve_expression(&operands[2], registers, object_pool),
                ) {
                    let text = match (left.text.as_str(), right.text.as_str()) {
                        ("true", "false") => test,
                        ("false", "true") => format!("!({test})"),
                        _ => format!("({} ? {} : {})", test, left.text, right.text),
                    };
                    registers.insert(
                        target,
                        Expression {
                            text,
                            confidence: weaker(
                                last_comparison
                                    .as_ref()
                                    .map_or(EvidenceConfidence::Low, |value| value.confidence),
                                weaker(left.confidence, right.confidence),
                            ),
                            complexity: left
                                .complexity
                                .saturating_add(right.complexity)
                                .saturating_add(2),
                            class_name: if left.class_name == right.class_name {
                                left.class_name
                            } else {
                                None
                            },
                            class_library_uri: if left.class_library_uri == right.class_library_uri
                            {
                                left.class_library_uri
                            } else {
                                None
                            },
                            raw: left.raw && right.raw,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                } else {
                    registers.remove(&target);
                }
            }
            // ARM32 `adds`/`subs` compute the low word of an int64 register
            // pair (the carry feeds `adc`/`sbc`); the lifter carries an int64
            // value in its low register, so they lift as the full operation.
            "add" | "sub" | "mul" | "imul" | "and" | "orr" | "eor" | "or" | "xor" | "adds"
            | "subs" | "rsb" | "rsbs" | "adc" | "adcs" | "sbc" | "sbcs" | "rsc" | "rscs"
                if operands.len() >= 3 =>
            {
                let target = normalize_register(&operands[0]);
                if abi == Abi::ArmeabiV7a {
                    let left = resolve_expression(&operands[1], registers, object_pool);
                    let right = resolve_expression(&operands[2], registers, object_pool);
                    let high =
                        |value: &Option<Expression>| value.as_ref().is_some_and(|v| v.high_word);
                    let mnemonic = instruction.mnemonic.as_str();
                    if matches!(mnemonic, "adc" | "adcs" | "sbc" | "sbcs" | "rsc" | "rscs") {
                        // The carry-consuming half of an int64 operation
                        // produces the high word of the value its
                        // flag-setting low half computed.
                        match last_carry.take() {
                            Some(low) if high(&left) || high(&right) => {
                                registers.insert(target.clone(), high_word_of(low));
                            }
                            _ => {
                                registers.remove(&target);
                            }
                        }
                        aliases.remove(&target);
                        continue;
                    }
                    if high(&left) || high(&right) {
                        // Both halves of a bitwise pair operation name the
                        // same int64 value; anything else on a high word
                        // has no pair-level meaning the lifter tracks.
                        let both = high(&left) && high(&right);
                        let operator = match mnemonic {
                            "and" => Some("&"),
                            "orr" => Some("|"),
                            "eor" => Some("^"),
                            _ => None,
                        };
                        match (both, operator, left, right) {
                            (true, Some(operator), Some(left), Some(right)) => {
                                let left = Expression {
                                    high_word: false,
                                    ..left
                                };
                                let right = Expression {
                                    high_word: false,
                                    ..right
                                };
                                match binary_expression(left, operator, right) {
                                    Some(value) => {
                                        registers.insert(target.clone(), high_word_of(value));
                                    }
                                    None => {
                                        registers.remove(&target);
                                    }
                                }
                            }
                            _ => {
                                registers.remove(&target);
                            }
                        }
                        aliases.remove(&target);
                        continue;
                    }
                    if matches!(mnemonic, "rsb" | "rsbs") {
                        // Reverse subtract: `rsb d, a, b` computes `b - a`.
                        let value = match (left, right) {
                            (Some(left), Some(right)) => raw_arithmetic(
                                binary_expression(right.clone(), "-", left.clone()),
                                &left,
                                &right,
                            ),
                            _ => None,
                        };
                        if mnemonic == "rsbs" {
                            last_carry = value.clone();
                        }
                        match value {
                            Some(value) => {
                                registers.insert(target.clone(), value);
                            }
                            None => {
                                registers.remove(&target);
                            }
                        }
                        aliases.remove(&target);
                        continue;
                    }
                }
                // Element address: `add d, array, index, lsl #k`. Array
                // elements are 4 bytes on every Android target (compressed
                // pointers on 64-bit), so `lsl #2` scales an untagged index
                // and `lsl #1` a Smi one; either way the index expression
                // names the Dart value.
                if abi != Abi::X86_64
                    && instruction.mnemonic == "add"
                    && let Some(shift) = operands.get(3).and_then(|shift| shift_amount(shift))
                    && matches!(shift, 1 | 2)
                    && let Some(array) = resolve_expression(&operands[1], registers, object_pool)
                    && let Some(index) = resolve_expression(&operands[2], registers, object_pool)
                    && !array.high_word
                    && !index.high_word
                {
                    registers.insert(
                        target.clone(),
                        Expression {
                            text: format!("({} + ({} << {shift}))", array.text, index.text),
                            confidence: weaker(array.confidence, index.confidence),
                            complexity: array.complexity.saturating_add(index.complexity),
                            class_name: None,
                            class_library_uri: None,
                            raw: true,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                    let index = if shift == 1 {
                        untagged_index(index)
                    } else {
                        index
                    };
                    element_pointers.insert(target.clone(), (array, index));
                    aliases.remove(&target);
                    continue;
                }
                // x64 decompression: `mov eax, dword ptr [recv+off];
                // add rax, qword ptr [r14+0x58]` adds the heap base held in
                // the thread. Treat it exactly like the ARM shift idiom.
                if abi == Abi::X86_64
                    && instruction.mnemonic == "add"
                    && arm_memory_address(&operands[2])
                        == Some(("r14".to_owned(), X64_THREAD_HEAP_BASE_OFFSET))
                {
                    let source = normalize_register(&operands[1]);
                    if let Some(element) = pending_elements.remove(&source).or_else(|| {
                        displaced_pending
                            .clone()
                            .filter(|(register, _)| *register == source)
                            .map(|(_, element)| element)
                    }) {
                        registers.insert(target.clone(), element);
                        compressed_loads.remove(&source);
                        aliases.remove(&target);
                        continue;
                    }
                    if let Some((base_register, displacement)) =
                        compressed_loads.get(&source).cloned()
                        && let Some(receiver) = registers.get(&base_register).cloned()
                        && let Some((field_offset, field)) =
                            recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                    {
                        let expression = field_expression(&receiver.text, &field.name);
                        let confidence = if field.synthesized_slot {
                            EvidenceConfidence::Low
                        } else {
                            receiver.confidence
                        };
                        push_statement!(SemanticStatement::FieldRead {
                            receiver: receiver.text.clone(),
                            field: field.name.clone(),
                            offset: field_offset,
                            expression: expression.clone(),
                            confidence,
                            address: format!("0x{:x}", instruction.address),
                        });
                        registers.insert(
                            target.clone(),
                            Expression {
                                text: expression,
                                confidence,
                                complexity: receiver.complexity.saturating_add(1),
                                class_name: field.value_class.clone(),
                                class_library_uri: field.value_library_uri.clone(),
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                        compressed_loads.remove(&source);
                        aliases.remove(&target);
                        continue;
                    }
                }
                // Compressed-pointer decompression idiom:
                // `ldur wN, [recv, #off]; add Xd, XN, x28, lsl #32` loads a
                // reference field. Preserve the field expression through the
                // decompression instead of treating it as arithmetic.
                if instruction.mnemonic == "add"
                    && normalize_register(&operands[2]) == "x28"
                    // Capstone prints the shift as its own operand.
                    && operands.get(3).and_then(|shift| shift_amount(shift)) == Some(32)
                {
                    let source = normalize_register(&operands[1]);
                    if let Some(element) = pending_elements.remove(&source).or_else(|| {
                        displaced_pending
                            .clone()
                            .filter(|(register, _)| *register == source)
                            .map(|(_, element)| element)
                    }) {
                        registers.insert(target.clone(), element);
                        compressed_loads.remove(&source);
                        aliases.remove(&target);
                        continue;
                    }
                    if let Some((base_register, displacement)) =
                        compressed_loads.get(&source).cloned()
                        && let Some(receiver) = registers.get(&base_register).cloned()
                    {
                        if let Some(context) = closure_context_read(abi, &receiver, displacement) {
                            registers.insert(target.clone(), context);
                            compressed_loads.remove(&source);
                            aliases.remove(&target);
                            continue;
                        }
                        // Slot placeholders keep provenance alive when the
                        // snapshot tree-shook the receiver's Field objects;
                        // without them every decompressed read degrades into
                        // anonymous locals.
                        let resolved =
                            recovered_field_or_slot(field_layout, &receiver, displacement, abi);
                        if let Some((field_offset, field)) = resolved {
                            let expression = field_expression(&receiver.text, &field.name);
                            let confidence = if field.synthesized_slot {
                                EvidenceConfidence::Low
                            } else {
                                receiver.confidence
                            };
                            registers.insert(
                                target.clone(),
                                Expression {
                                    text: expression.clone(),
                                    confidence,
                                    complexity: receiver.complexity.saturating_add(1),
                                    class_name: field.value_class.clone(),
                                    class_library_uri: field.value_library_uri.clone(),
                                    raw: false,
                                    definition_site: None,
                                    high_word: false,
                                    exact_class: false,
                                },
                            );
                            push_statement!(SemanticStatement::FieldRead {
                                receiver: receiver.text,
                                field: field.name.clone(),
                                offset: field_offset,
                                expression,
                                confidence,
                                address: format!("0x{:x}", instruction.address),
                            });
                            compressed_loads.remove(&source);
                            aliases.remove(&target);
                            continue;
                        }
                    }
                }
                let left = resolve_expression(&operands[1], registers, object_pool);
                let right = resolve_expression(&operands[2], registers, object_pool);
                if let (Some(left), Some(right)) = (left, right) {
                    let operator = match instruction.mnemonic.as_str() {
                        "add" | "adds" => "+",
                        "sub" | "subs" => "-",
                        "mul" | "imul" => "*",
                        "and" => "&",
                        "orr" | "or" => "|",
                        "eor" | "xor" => "^",
                        _ => "^",
                    };
                    if matches!(instruction.mnemonic.as_str(), "eor" | "xor")
                        && left.class_name.as_deref() == Some("bool")
                        && right.text == if abi == Abi::ArmeabiV7a { "8" } else { "16" }
                    {
                        registers.insert(
                            target.clone(),
                            Expression {
                                text: format!("!({})", left.text),
                                confidence: left.confidence,
                                complexity: left.complexity.saturating_add(1),
                                class_name: Some("bool".to_owned()),
                                class_library_uri: Some("dart:core".to_owned()),
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    } else if left.text == "null"
                        && instruction.mnemonic == "add"
                        && right.text.parse::<i64>().is_ok()
                    {
                        // Tagged canonical booleans live at fixed offsets
                        // from null on ARM64 (pointer_tagging.h):
                        // true = +0x20, false = +0x30.
                        let constant = match right.text.as_str() {
                            "32" => Some("true"),
                            "48" => Some("false"),
                            _ => None,
                        };
                        match constant {
                            Some(value) => {
                                registers.insert(
                                    target.clone(),
                                    Expression {
                                        text: value.to_owned(),
                                        confidence: EvidenceConfidence::High,
                                        complexity: 1,
                                        class_name: Some("bool".to_owned()),
                                        class_library_uri: Some("dart:core".to_owned()),
                                        raw: false,
                                        definition_site: None,
                                        high_word: false,
                                        exact_class: false,
                                    },
                                );
                            }
                            // Unknown null-relative arithmetic is not a
                            // recoverable Dart expression.
                            None => {
                                registers.remove(&target);
                            }
                        }
                    } else if let Some(expression) = raw_arithmetic(
                        binary_expression(left.clone(), operator, right.clone()),
                        &left,
                        &right,
                    ) {
                        if matches!(instruction.mnemonic.as_str(), "adds" | "subs") {
                            last_carry = Some(expression.clone());
                        }
                        registers.insert(target.clone(), expression);
                    } else {
                        registers.remove(&target);
                    }
                } else {
                    registers.remove(&target);
                }
                if abi == Abi::ArmeabiV7a
                    && matches!(instruction.mnemonic.as_str(), "adds" | "subs")
                    && !registers.contains_key(&target)
                {
                    last_carry = None;
                }
                // Derived element pointers: `add dst, arrBase, #offset`
                // extends buffer provenance with a byte displacement.
                aliases.remove(&target);
                if instruction.mnemonic == "add"
                    && let Some(delta) = signed_immediate(&operands[2])
                {
                    let source = normalize_register(&operands[1]);
                    if let Some((site, extra)) = aliases.get(&source).copied() {
                        aliases.insert(target.clone(), (site, extra.saturating_add(delta)));
                    }
                }
            }
            // `xor r, r` is x64's register-clearing idiom: zero, whatever r held.
            "xor"
                if abi == Abi::X86_64
                    && operands.len() == 2
                    && is_register_spelling(&operands[0])
                    && normalize_register(&operands[0]) == normalize_register(&operands[1]) =>
            {
                let target = normalize_register(&operands[0]);
                compressed_loads.remove(&target);
                aliases.remove(&target);
                registers.insert(
                    target,
                    Expression {
                        text: "0".to_owned(),
                        confidence: EvidenceConfidence::High,
                        complexity: 1,
                        class_name: None,
                        class_library_uri: None,
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: false,
                    },
                );
            }
            "xor" if abi == Abi::X86_64 && operands.len() == 2 => {
                let target = normalize_register(&operands[0]);
                let left = resolve_expression(&operands[0], registers, object_pool);
                let right = resolve_expression(&operands[1], registers, object_pool);
                match (left, right) {
                    (Some(left), Some(right))
                        if left.class_name.as_deref() == Some("bool") && right.text == "16" =>
                    {
                        registers.insert(
                            target,
                            Expression {
                                text: format!("!({})", left.text),
                                confidence: left.confidence,
                                complexity: left.complexity.saturating_add(1),
                                class_name: Some("bool".to_owned()),
                                class_library_uri: Some("dart:core".to_owned()),
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    (Some(left), Some(right)) => {
                        registers.insert(
                            target,
                            Expression {
                                text: format!("({} ^ {})", left.text, right.text),
                                confidence: weaker(left.confidence, right.confidence),
                                complexity: left
                                    .complexity
                                    .saturating_add(right.complexity)
                                    .saturating_add(1),
                                class_name: None,
                                class_library_uri: None,
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    _ => {
                        registers.remove(&target);
                    }
                }
            }
            "asr" | "lsr" | "lsl" | "sar" | "shr" | "shl" | "asrs" | "lsrs" | "lsls"
                if operands.len() >= 3 =>
            {
                let target = normalize_register(&operands[0]);
                if let Some(value) = resolve_expression(&operands[1], registers, object_pool)
                    && let Some(shift) = immediate_text(&operands[2])
                {
                    if abi == Abi::ArmeabiV7a && matches!(instruction.mnemonic.as_str(), "asr" | "asrs")
                    {
                        if value.high_word {
                            // Arithmetic on one half of an int64 pair; the
                            // pair-level result is not tracked.
                            registers.remove(&target);
                            continue;
                        }
                        // `asr hi, lo, #31` sign-extends a 32-bit value into
                        // the high word of its int64 register pair.
                        if shift == "31" {
                            registers.insert(target, high_word_of(value));
                            continue;
                        }
                        // ARM32 Smi untag (`asr(s) d, s, #1`): the payload
                        // IS the source value.
                        if shift == "1" && !value.raw {
                            registers.insert(
                                target,
                                Expression {
                                    raw: true,
                                    definition_site: None,
                                    exact_class: false,
                                    complexity: value.complexity.saturating_add(1),
                                    ..value
                                },
                            );
                            continue;
                        }
                    }
                    if value.high_word {
                        registers.remove(&target);
                        continue;
                    }
                    if instruction.mnemonic == "sar" && shift == "1" && !value.raw {
                        // x64 Smi untag: the payload IS the source value.
                        registers.insert(
                            target,
                            Expression {
                                raw: true,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                                complexity: value.complexity.saturating_add(1),
                                ..value
                            },
                        );
                        continue;
                    }
                    // Re-tagging an untagged integer (`lsl/shl #1`) preserves
                    // the source-level value.
                    let int_derived =
                        value.raw || value.text.contains(" % ") || value.text.contains(" ~/ ");
                    if matches!(instruction.mnemonic.as_str(), "lsl" | "lsls" | "shl")
                        && shift == "1"
                        && int_derived
                    {
                        registers.insert(target, Expression { raw: false, ..value });
                        continue;
                    }
                    let operator = if matches!(instruction.mnemonic.as_str(), "lsl" | "lsls" | "shl")
                    {
                        "<<"
                    } else {
                        ">>"
                    };
                    if value.complexity < 32 {
                        registers.insert(
                            target,
                            Expression {
                                text: format!("({} {operator} {shift})", value.text),
                                confidence: value.confidence,
                                complexity: value.complexity + 1,
                                class_name: None,
                                class_library_uri: None,
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    } else {
                        registers.remove(&target);
                    }
                } else {
                    registers.remove(&target);
                }
            }
            "sete" | "setne" | "setl" | "setg" | "setle" | "setge" | "setb" | "setbe" | "seta"
            | "setae" | "sets" | "setns"
                if !operands.is_empty() =>
            {
                let target = normalize_register(&operands[0]);
                let code = &instruction.mnemonic[3..];
                let code = match code {
                    "e" => Some("eq"),
                    "ne" => Some("ne"),
                    "l" => Some("lt"),
                    "le" => Some("le"),
                    "g" => Some("gt"),
                    "ge" => Some("ge"),
                    "b" => Some("lo"),
                    "be" => Some("ls"),
                    "a" => Some("hi"),
                    "ae" => Some("hs"),
                    "s" => Some("mi"),
                    "ns" => Some("pl"),
                    _ => None,
                };
                let comparison = code.and_then(condition_code_operator).and_then(|operator| {
                    last_comparison.as_ref().and_then(|comparison| {
                        comparison
                            .text
                            .split_once(COMPARISON_SEPARATOR)
                            .map(|(left, right)| comparison_text(left, operator, right))
                    })
                });
                if let Some(text) = comparison {
                    registers.insert(
                        target,
                        Expression {
                            text,
                            confidence: last_comparison
                                .as_ref()
                                .map_or(EvidenceConfidence::Low, |c| c.confidence),
                            complexity: last_comparison
                                .as_ref()
                                .map_or(2, |c| c.complexity.saturating_add(1)),
                            class_name: Some("bool".to_owned()),
                            class_library_uri: Some("dart:core".to_owned()),
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                } else {
                    registers.remove(&target);
                }
            }
            "sbfx" | "ubfx" if operands.len() >= 4 => {
                let target = normalize_register(&operands[0]);
                match (
                    resolve_expression(&operands[1], registers, object_pool),
                    immediate_text(&operands[2]),
                    immediate_text(&operands[3]),
                ) {
                    (Some(value), Some(lsb), Some(width))
                        if value.complexity < 32 && width.parse::<usize>().is_ok() =>
                    {
                        if instruction.mnemonic == "sbfx"
                            && lsb == "1"
                            && matches!(width.as_str(), "31" | "63")
                            && !value.raw
                        {
                            // Smi untag of an ordinary Dart value: the
                            // payload IS the source-level integer, so the
                            // expression text is unchanged; only provenance
                            // switches to untagged.
                            registers.insert(
                                target,
                                Expression {
                                    raw: true,
                                    definition_site: None,
                                    high_word: false,
                                    exact_class: false,
                                    complexity: value.complexity.saturating_add(1),
                                    ..value.clone()
                                },
                            );
                            continue;
                        }
                        let expression = if instruction.mnemonic == "sbfx" && lsb == "1" {
                            format!("({} >> 1)", value.text)
                        } else {
                            let mask = match width.parse::<u32>() {
                                Ok(width) if width < 64 => (1u64 << width).wrapping_sub(1),
                                _ => u64::MAX,
                            };
                            format!("(({} >> {}) & {:#x})", value.text, lsb, mask)
                        };
                        registers.insert(
                            target,
                            Expression {
                                text: expression,
                                confidence: value.confidence,
                                complexity: value.complexity.saturating_add(1),
                                class_name: None,
                                class_library_uri: None,
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                    _ => {
                        registers.remove(&target);
                    }
                }
            }
            "sbfiz" | "ubfiz" if operands.len() >= 4 => {
                let target = normalize_register(&operands[0]);
                match (
                    resolve_expression(&operands[1], registers, object_pool),
                    immediate_text(&operands[2]),
                    immediate_text(&operands[3]),
                ) {
                    // A one-bit left insert is the Smi re-tag; the stored or
                    // returned tagged word carries the same numeric value.
                    (Some(value), Some(lsb), Some(width))
                        if lsb == "1" && matches!(width.as_str(), "31" | "63") =>
                    {
                        registers.insert(target, value);
                    }
                    _ => {
                        registers.remove(&target);
                    }
                }
            }
            // ARM32 VFP double arithmetic decoded by the fallback decoder.
            // Both operands and the destination live in the same dN register
            // file, so this mirrors the integer binary-expression path with
            // `double` result provenance.
            "vadd.f64" | "vsub.f64" | "vmul.f64" | "vdiv.f64" if operands.len() >= 3 => {
                let target = normalize_register(&operands[0]);
                let operator = match instruction.mnemonic.as_str() {
                    "vadd.f64" => "+",
                    "vsub.f64" => "-",
                    "vmul.f64" => "*",
                    _ => "/",
                };
                let left = resolve_expression(&operands[1], registers, object_pool);
                let right = resolve_expression(&operands[2], registers, object_pool);
                if let (Some(left), Some(right)) = (left, right) {
                    if let Some(expression) = binary_expression(left, operator, right) {
                        let expression = Expression {
                            class_name: Some("double".to_owned()),
                            class_library_uri: Some("dart:core".to_owned()),
                            ..expression
                        };
                        registers.insert(target, expression);
                    } else {
                        registers.remove(&target);
                    }
                } else {
                    registers.remove(&target);
                }
            }
            "fcmp" | "comisd" | "vcmpd" | "vcmpdz" if operands.len() >= 2 => {
                let left = resolve_expression(&operands[0], registers, object_pool);
                let right = resolve_expression(&operands[1], registers, object_pool);
                last_comparison = match (left, right) {
                    (Some(left), Some(right)) => Some(Expression {
                        text: pending_comparison(&left.text, &right.text),
                        confidence: weaker(left.confidence, right.confidence),
                        complexity: left.complexity.saturating_add(right.complexity),
                        class_name: None,
                        class_library_uri: None,
                        raw: false,
                        definition_site: None,
                        high_word: false,
                        exact_class: false,
                    }),
                    _ => None,
                };
            }
            "cmp" | "cmn" | "tst" | "test" if operands.len() >= 2 => {
                // An untracked register operand keeps its machine name so the
                // comparison — and therefore the whole branch diamond —
                // stays recoverable instead of silently disappearing.
                let resolve_or_named = |operand: &str| {
                    resolve_expression(operand, registers, object_pool).or_else(|| {
                        let candidate = normalize_register(operand);
                        registers
                            .contains_key(&candidate)
                            .then(|| candidate.clone())
                            .or_else(|| is_register_spelling(operand).then_some(candidate))
                            .map(|name| Expression {
                                text: name,
                                confidence: EvidenceConfidence::Low,
                                complexity: 1,
                                class_name: None,
                                class_library_uri: None,
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            })
                    })
                };
                let left = resolve_or_named(&operands[0]);
                let right = resolve_or_named(&operands[1]);
                last_comparison = match (left, right) {
                    (Some(left), Some(right)) => {
                        let text = if matches!(instruction.mnemonic.as_str(), "tst" | "test") {
                            pending_comparison(&format!("({} & {})", left.text, right.text), "0")
                        } else {
                            pending_comparison(&left.text, &right.text)
                        };
                        Some(Expression {
                            text,
                            confidence: weaker(left.confidence, right.confidence),
                            complexity: left.complexity.saturating_add(right.complexity),
                            class_name: None,
                            class_library_uri: None,
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        })
                    }
                    _ => None,
                };
            }
            // A store pair into frame slots writes two consecutive words; an
            // untracked half still overwrites its slot.
            "stp" if operands.len() >= 3 && stack_slot_key(abi, &operands[2], deltas).is_some() => {
                let word = TargetLayout::of(abi).word_size;
                if let Some((base, displacement)) = arm_memory_address(&operands[2]) {
                    for (index, source) in operands[..2].iter().enumerate() {
                        let address = format!("[{base}, #{}]", displacement + index as i64 * word);
                        let Some(slot) = stack_slot_key(abi, &address, deltas) else {
                            continue;
                        };
                        spill_alias(aliases, &normalize_register(source), &slot);
                        match resolve_expression(source, registers, object_pool) {
                            Some(value) => {
                                stack.insert(slot, value);
                            }
                            None => {
                                stack.remove(&slot);
                            }
                        }
                    }
                }
            }
            "stur" | "str" if operands.len() >= 2 => {
                if let Some(slot) = stack_slot_key(abi, &operands[1], deltas) {
                    let source = normalize_register(&operands[0]);
                    // Spilling a tracked array keeps it alive across calls.
                    spill_alias(aliases, &source, &slot);
                    // An untracked value still overwrites the slot: a stale
                    // entry would fabricate the value reloaded.
                    match resolve_expression(&operands[0], registers, object_pool) {
                        Some(value) => {
                            stack.insert(slot, value);
                        }
                        None => {
                            stack.remove(&slot);
                        }
                    }
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1]) {
                    let value = resolve_expression(&operands[0], registers, object_pool);
                    if let Some(write) = value.as_ref().and_then(|value| {
                        element_pointer(&element_pointers, &displaced_element, &base).and_then(
                            |pointer| {
                                element_write(
                                    abi,
                                    pointer,
                                    displacement,
                                    value,
                                    instruction.address,
                                )
                            },
                        )
                    }) {
                        // `array[index] = value` through an element pointer.
                        push_statement!(write);
                    } else if fill_pending_box(registers, &base, displacement, value.as_ref()) {
                    } else if let Some(write) =
                        store_element(abi, buffers, aliases, &base, displacement, value.as_ref())
                    {
                        if let Some(write) = write {
                            push_statement!(write.statement(instruction.address));
                        }
                    } else if let Some(receiver) = registers.get(&base).cloned()
                        && let Some(value) = value.filter(|value| !value.high_word)
                        && let Some((field_offset, field)) =
                            recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                    {
                        let confidence = if field.synthesized_slot {
                            EvidenceConfidence::Low
                        } else {
                            weaker(receiver.confidence, value.confidence)
                        };
                        push_statement!(SemanticStatement::FieldWrite {
                            receiver: receiver.text,
                            field: field.name.clone(),
                            offset: field_offset,
                            value: value.text,
                            confidence,
                            address: format!("0x{:x}", instruction.address),
                        });
                    }
                }
            }
            "ldur" | "ldr" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                // Track 32-bit reference loads for the decompression idiom.
                if operands[0].trim().to_ascii_lowercase().starts_with('w') {
                    let trackable = stack_slot_key(abi, &operands[1], deltas).is_none()
                        && !pool_loads.contains_key(&instruction.address)
                        && arm_memory_address(&operands[1]).is_some();
                    if trackable {
                        if let Some((base, displacement)) = arm_memory_address(&operands[1]) {
                            if let Some(element) =
                                element_pointer(&element_pointers, &displaced_element, &base)
                                    .and_then(|pointer| element_read(abi, pointer, displacement))
                            {
                                pending_elements.insert(target.clone(), element);
                            }
                            compressed_loads.insert(target.clone(), (base, displacement));
                        }
                    } else {
                        compressed_loads.remove(&target);
                    }
                } else {
                    compressed_loads.remove(&target);
                }
                if let Some(value) = arm_memory_address(&operands[1])
                    .filter(|(base, displacement)| base == stack_register && *displacement >= 0)
                    .and_then(|(_, displacement)| state.outgoing.get(&displacement).cloned())
                {
                    // Reading back a staged outgoing argument (a switchable
                    // call reloads its receiver from the argument area).
                    aliases.remove(&target);
                    registers.insert(target, value);
                } else if let Some(slot) = stack_slot_key(abi, &operands[1], deltas) {
                    // Reloading a spilled array re-attaches buffer tracking.
                    reload_alias(aliases, &target, &slot);
                    if let Some(value) = stack.get(&slot).cloned() {
                        registers.insert(target.clone(), value);
                    } else {
                        // The slot's store did not provably reach this join
                        // (locals are reused across loops). Surface a stable
                        // low-confidence local so computations over it stay
                        // visible instead of silently disappearing.
                        let displacement = arm_memory_address(&operands[1])
                            .map(|(_, displacement)| displacement)
                            .unwrap_or_default();
                        registers.insert(
                            target,
                            Expression {
                                text: format!("local{:x}", displacement.unsigned_abs()),
                                confidence: EvidenceConfidence::Low,
                                complexity: 1,
                                class_name: None,
                                class_library_uri: None,
                                raw: false,
                                definition_site: None,
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                } else if let Some(index) = pool_loads.get(&instruction.address).copied()
                    && let Some(value) = object_pool.and_then(|pool| pool.get(index))
                {
                    registers.insert(
                        target,
                        Expression {
                            text: pool_value_text(value),
                            confidence: EvidenceConfidence::High,
                            complexity: 1,
                            class_name: snapshot_instance_class(value),
                            class_library_uri: None,
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: snapshot_instance_class(value).is_some(),
                        },
                    );
                } else if abi == Abi::ArmeabiV7a
                    && let Some(element) =
                        arm_memory_address(&operands[1]).and_then(|(base, displacement)| {
                            element_pointer(&element_pointers, &displaced_element, &base)
                                .and_then(|pointer| element_read(abi, pointer, displacement))
                        })
                {
                    // ARM32 references are full words: no decompression.
                    registers.insert(target, element);
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1])
                    && displacement == code_entry_point_displacement(abi)
                    && let Some(code) = registers
                        .get(&base)
                        .filter(|code| is_pool_code_label(&code.text))
                        .cloned()
                {
                    // `Code::entry_point_` of a pool-loaded Code object: the
                    // register now holds that code's entry, so a following
                    // indirect call targets the same stub.
                    registers.insert(target, code);
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1])
                    && let Some(context) = registers
                        .get(&base)
                        .and_then(|receiver| closure_context_read(abi, receiver, displacement))
                {
                    registers.insert(target, context);
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1])
                    && let Some(receiver) = registers.get(&base).cloned()
                    && let Some((field_offset, field)) =
                        recovered_field_or_slot(field_layout, &receiver, displacement, abi)
                {
                    let expression = field_expression(&receiver.text, &field.name);
                    let confidence = if field.synthesized_slot {
                        EvidenceConfidence::Low
                    } else {
                        receiver.confidence
                    };
                    registers.insert(
                        target.clone(),
                        Expression {
                            text: expression.clone(),
                            confidence,
                            complexity: receiver.complexity.saturating_add(1),
                            class_name: field.value_class.clone(),
                            class_library_uri: field.value_library_uri.clone(),
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                    push_statement!(SemanticStatement::FieldRead {
                        receiver: receiver.text,
                        field: field.name.clone(),
                        offset: field_offset,
                        expression,
                        confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                } else {
                    registers.remove(&target);
                }
            }
            mnemonic if direct_call_target(mnemonic, &instruction.operands).is_some() => {
                let target_address =
                    direct_call_target(mnemonic, &instruction.operands).unwrap_or_default();
                let symbol = symbols.get(&target_address);
                let target = symbol
                    .map(|symbol| symbol.label.clone())
                    .unwrap_or_else(|| format!("sub_{target_address:x}"));
                if is_interpolate_target(&target)
                    && let Some((literal, confidence)) =
                        take_interpolation_literal(abi, buffers, aliases)
                {
                    push_statement!(SemanticStatement::StringInterpolation {
                        parts: vec![literal.clone()],
                        confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                    last_comparison = None;
                    last_division = None;
                    kill_caller_saved(abi, registers);
                    state.outgoing.clear();
                    state.written_argument_registers = 0;
                    retain_frame_buffers(buffers, aliases);
                    registers.insert(
                        return_register.to_owned(),
                        Expression {
                            text: literal,
                            confidence,
                            complexity: 1,
                            class_name: Some("String".to_owned()),
                            class_library_uri: Some("dart:core".to_owned()),
                            raw: false,
                            definition_site: None,
                            high_word: false,
                            exact_class: false,
                        },
                    );
                    continue;
                }
                if let Some(sink) = lift_facts {
                    let mut facts = sink.borrow_mut();
                    if let Some((receiver, class, library)) = proven_call_receiver(
                        abi,
                        &target,
                        symbol,
                        registers,
                        object_pool,
                        &pool_registers,
                    ) && receiver
                        .class_name
                        .as_deref()
                        .is_none_or(|class| class == "Object")
                        && !receiver.high_word
                    {
                        facts
                            .receivers
                            .push((receiver.text.clone(), class, library));
                    }
                    if let Some(symbol) = symbol
                        && let Some(callee) = symbol.code_address
                        && let Some(parameters) = symbol.parameters.as_deref()
                    {
                        for (index, location) in parameters.iter().enumerate() {
                            let ArgumentLocation::Register(register) = location else {
                                continue;
                            };
                            facts.arguments.push((
                                callee,
                                index,
                                registers.get(*register).and_then(value_class),
                            ));
                        }
                    }
                }
                if let Some(semantics) = target.strip_prefix("stub ").and_then(|stub| {
                    runtime_stub_semantics(abi, stub, registers, &pool_registers, object_pool)
                }) {
                    last_comparison = None;
                    last_division = None;
                    match semantics {
                        StubSemantics::Throw {
                            exception,
                            stack_trace,
                        } => {
                            push_statement!(SemanticStatement::Throw {
                                confidence: exception.confidence,
                                expression: exception.text,
                                stack_trace: stack_trace.map(|value| value.text),
                                address: format!("0x{:x}", instruction.address),
                            });
                            kill_caller_saved(abi, registers);
                        }
                        StubSemantics::Value {
                            value,
                            preserves_registers,
                        } => {
                            if !preserves_registers {
                                kill_caller_saved(abi, registers);
                            }
                            registers.insert(
                                return_register.to_owned(),
                                Expression {
                                    definition_site: Some(instruction.address),
                                    ..value
                                },
                            );
                        }
                    }
                    state.outgoing.clear();
                    state.written_argument_registers = 0;
                    retain_frame_buffers(buffers, aliases);
                    continue;
                }
                let stub_arguments = allocation_stub_arguments(abi, &target, registers);
                // VM stubs that create values get a name unique to their call
                // site, so two arrays in one body are never the same value.
                let result_text = if stub_arguments.is_some() {
                    site_result_name(&target, instruction.address)
                } else {
                    format!("{}_result", sanitize_semantic_name(&target))
                };
                let arguments = stub_arguments.unwrap_or_else(|| {
                    collect_call_arguments(
                        abi,
                        registers,
                        &state.outgoing,
                        state.written_argument_registers,
                        symbol,
                    )
                });
                push_statement!(SemanticStatement::ResolvedCall {
                    target: target.clone(),
                    arguments: arguments.clone(),
                    confidence: EvidenceConfidence::Medium,
                    address: format!("0x{:x}", instruction.address),
                });
                last_comparison = None;
                last_division = None;
                // Read the allocation length before the call clobbers it.
                let allocated_length = registers
                    .get(allocate_array_length_register(abi))
                    .and_then(constant_array_length);
                // A late/lazy field initialization stub returns the field's
                // value: the `if (f == sentinel) f = init()` sequence around
                // it is the compiled form of reading that field. Its ABI
                // (`InitInstanceFieldABI`, `InitStaticFieldABI`) passes the
                // instance and the Field in the first two argument
                // registers, whether or not this body just wrote them.
                let staged = init_field_registers(abi)
                    .iter()
                    .filter_map(|register| registers.get(*register).map(|value| value.text.clone()))
                    .collect::<Vec<_>>();
                let late_read = late_field_read(&target, &staged)
                    .or_else(|| late_field_read(&target, &arguments));
                kill_caller_saved(abi, registers);
                state.outgoing.clear();
                state.written_argument_registers = 0;
                // Register-resident bookkeeping dies at the call; spilled
                // arrays survive on the stack.
                retain_frame_buffers(buffers, aliases);
                if looks_like_allocation_stub(&target) {
                    // Tentative buffer; it survives only while subsequent
                    // stores match the compressed Array element layout.
                    buffers.insert(
                        instruction.address,
                        ElementBuffer::new(result_text.clone(), allocated_length),
                    );
                    aliases.insert(return_register.to_owned(), (instruction.address, 0));
                }
                let text = late_read.unwrap_or(result_text);
                registers.insert(
                    call_result_register(abi, symbol).to_owned(),
                    Expression {
                        text,
                        confidence: EvidenceConfidence::Low,
                        complexity: 1,
                        class_name: symbol
                            .and_then(|symbol| symbol.result_class.clone())
                            .or_else(|| {
                                (target == "stub AllocateContext").then(|| "Context".to_owned())
                            }),
                        // The VM's Context lives in no Dart library;
                        // `dart:core` keeps application classes that share
                        // the name from matching it.
                        class_library_uri: symbol
                            .and_then(|symbol| symbol.result_library_uri.clone())
                            .or_else(|| {
                                (target == "stub AllocateContext").then(|| "dart:core".to_owned())
                            }),
                        raw: false,
                        // Only a per-class allocation stub proves the
                        // runtime class; factories and methods may return
                        // any subtype of their declared result.
                        definition_site: Some(instruction.address),
                        high_word: false,
                        exact_class: symbol.is_some_and(|symbol| {
                            symbol.allocation_stub && symbol.result_class.is_some()
                        }),
                    },
                );
            }
            mnemonic if is_call(mnemonic) => {
                // --- P1 receiver-proven virtual dispatch (class-ID dataflow) ---
                let dispatch_resolved: Option<String> =
                    if let (Some(table), Some(calls)) = (dispatch_table, dispatch_calls) {
                        calls.get(&instruction.address).and_then(|call| {
                            resolve_dispatch_via_receiver(
                                table,
                                call.selector_offset,
                                dispatch_receiver.as_ref(),
                            )
                        })
                    } else {
                        None
                    };
                if let Some(target_text) = dispatch_resolved {
                    let arguments = collect_call_arguments(
                        abi,
                        registers,
                        &state.outgoing,
                        state.written_argument_registers,
                        None,
                    );
                    kill_caller_saved(abi, registers);
                    state.outgoing.clear();
                    retain_frame_buffers(buffers, aliases);
                    last_comparison = None;
                    push_statement!(SemanticStatement::ResolvedCall {
                        target: target_text.clone(),
                        arguments,
                        confidence: EvidenceConfidence::High,
                        address: format!("0x{:x}", instruction.address),
                    });
                    let (result_class, result_library_uri) =
                        resolved_result_class(dispatch_table, &target_text);
                    registers.insert(
                        return_register.to_owned(),
                        Expression {
                            text: format!("{}_result", sanitize_semantic_name(&target_text)),
                            confidence: EvidenceConfidence::Low,
                            complexity: 1,
                            class_name: result_class,
                            class_library_uri: result_library_uri,
                            raw: false,
                            definition_site: Some(instruction.address),
                            high_word: false,
                            exact_class: false,
                        },
                    );
                } else if operands
                    .first()
                    .and_then(|operand| registers.get(&normalize_register(operand)))
                    .is_some_and(|target| target.text.starts_with("resetPoolEntry("))
                    && registers
                        .get(native_entry_register(abi))
                        .is_some_and(|entry| entry.text.starts_with("nativePoolEntry("))
                {
                    // `NativeCallInstr`: the arguments sit on the stack above
                    // a null result slot at `[SP]`, which the native call
                    // fills and the body reloads.
                    let arguments = state
                        .outgoing
                        .iter()
                        .rev()
                        .filter(|(displacement, _)| **displacement > 0)
                        .map(|(_, value)| value.text.clone())
                        .collect::<Vec<_>>();
                    kill_caller_saved(abi, registers);
                    state.outgoing.clear();
                    state.written_argument_registers = 0;
                    retain_frame_buffers(buffers, aliases);
                    last_comparison = None;
                    push_statement!(SemanticStatement::ResolvedCall {
                        target: "native call".to_owned(),
                        arguments,
                        confidence: EvidenceConfidence::Medium,
                        address: format!("0x{:x}", instruction.address),
                    });
                    let result = Expression {
                        text: "native_call_result".to_owned(),
                        confidence: EvidenceConfidence::Medium,
                        complexity: 1,
                        class_name: None,
                        class_library_uri: None,
                        raw: false,
                        definition_site: Some(instruction.address),
                        high_word: false,
                        exact_class: false,
                    };
                    if let Some(slot) = deltas.sp.map(entry_slot_key) {
                        stack.insert(slot, result.clone());
                    }
                    registers.insert(return_register.to_owned(), result);
                } else {
                    let target = operands
                        .first()
                        .and_then(|operand| resolve_expression(operand, registers, object_pool))
                        .filter(|target| is_named_pool_target(&target.text));
                    // IC / megamorphic fallback (P1): when the call used a
                    // `dynamicCall("foo")` selector slot and the pool target is
                    // opaque, try receiver-proven method lookup via the global
                    // symbol table.  This narrows `call(dynamicCall)` sites that
                    // dispatch through the IC stub.
                    let ic_resolved: Option<String> = if target.is_none() {
                        find_ic_selector(registers).and_then(|selector| {
                            resolve_ic_target_via_receiver(
                                &selector,
                                switchable_receiver.as_ref(),
                                symbols,
                                dispatch_table,
                            )
                        })
                    } else {
                        None
                    };
                    let stub_arguments = target
                        .as_ref()
                        .and_then(|target| allocation_stub_arguments(abi, &target.text, registers));
                    let site_named = stub_arguments.is_some();
                    let arguments = stub_arguments.unwrap_or_else(|| {
                        collect_call_arguments(
                            abi,
                            registers,
                            &state.outgoing,
                            state.written_argument_registers,
                            None,
                        )
                    });
                    let block_arguments = dispatch_call_arguments(
                        abi,
                        registers,
                        &state.outgoing,
                        state.written_argument_registers & block_written_argument_registers,
                        dispatch_receiver.as_ref(),
                    );
                    kill_caller_saved(abi, registers);
                    state.outgoing.clear();
                    retain_frame_buffers(buffers, aliases);
                    last_comparison = None;
                    if let Some(ic_target) = ic_resolved {
                        push_statement!(SemanticStatement::ResolvedCall {
                            target: ic_target.clone(),
                            arguments: arguments.clone(),
                            confidence: EvidenceConfidence::High,
                            address: format!("0x{:x}", instruction.address),
                        });
                        let (result_class, result_library_uri) =
                            resolved_result_class(dispatch_table, &ic_target);
                        registers.insert(
                            return_register.to_owned(),
                            Expression {
                                text: format!("{}_result", sanitize_semantic_name(&ic_target)),
                                confidence: EvidenceConfidence::Low,
                                complexity: 1,
                                class_name: result_class,
                                class_library_uri: result_library_uri,
                                raw: false,
                                definition_site: Some(instruction.address),
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    } else if let Some(target) = target {
                        push_statement!(SemanticStatement::ResolvedCall {
                            target: target.text.clone(),
                            arguments,
                            confidence: EvidenceConfidence::Medium,
                            address: format!("0x{:x}", instruction.address),
                        });
                        let (result_class, result_library_uri) =
                            resolved_result_class(dispatch_table, &target.text);
                        registers.insert(
                            return_register.to_owned(),
                            Expression {
                                text: if site_named {
                                    site_result_name(&target.text, instruction.address)
                                } else {
                                    format!("{}_result", sanitize_semantic_name(&target.text))
                                },
                                confidence: EvidenceConfidence::Low,
                                complexity: 1,
                                class_name: result_class,
                                class_library_uri: result_library_uri,
                                raw: false,
                                definition_site: Some(instruction.address),
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    } else if let Some(call) =
                        dispatch_calls.and_then(|calls| calls.get(&instruction.address))
                    {
                        // A dispatch-table call whose implementation the
                        // receiver does not prove: keep the call and its
                        // result, naming the selector when the table does.
                        let target = match call.selector_name.as_deref() {
                            Some(name) => format!("dispatch {} {name}", call.selector_offset),
                            None => format!("dispatch {}", call.selector_offset),
                        };
                        let receiver = dispatch_receiver.as_ref().map(|value| value.text.clone());
                        let call_arguments = receiver
                            .iter()
                            .cloned()
                            .chain(block_arguments)
                            .collect::<Vec<_>>();
                        push_statement!(SemanticStatement::ResolvedCall {
                            target: target.clone(),
                            arguments: call_arguments,
                            confidence: EvidenceConfidence::Low,
                            address: format!("0x{:x}", instruction.address),
                        });
                        registers.insert(
                            return_register.to_owned(),
                            Expression {
                                text: format!("{}_result", sanitize_semantic_name(&target)),
                                confidence: EvidenceConfidence::Low,
                                complexity: 1,
                                class_name: None,
                                class_library_uri: None,
                                raw: false,
                                definition_site: Some(instruction.address),
                                high_word: false,
                                exact_class: false,
                            },
                        );
                    }
                }
            }
            mnemonic if is_return(mnemonic, &instruction.operands) => {
                if let Some(sink) = lift_facts {
                    sink.borrow_mut()
                        .returns
                        .push(registers.get(result_register).and_then(value_class));
                }
                if let Some(value) = registers.get(result_register)
                    && !value.text.starts_with("pool[")
                {
                    push_statement!(SemanticStatement::Return {
                        expression: value.text.clone(),
                        confidence: value.confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                }
            }
            mnemonic if branch_kind(mnemonic) == Some(true) => {
                let condition = branch_condition(
                    mnemonic,
                    &operands,
                    registers,
                    last_comparison.as_ref(),
                    object_pool,
                );
                if std::env::var("CLUTTER_DEBUG_COND").is_ok() {
                    eprintln!(
                        "COND 0x{:x} {} ops={:?} cmp={:?} -> {:?}",
                        instruction.address,
                        mnemonic,
                        operands,
                        last_comparison.as_ref().map(|c| &c.text),
                        condition.as_ref().map(|c| &c.text)
                    );
                }
                if let Some(condition) = condition {
                    let target = branch_target(&instruction.operands);
                    push_statement!(SemanticStatement::Condition {
                        expression: condition.text,
                        true_target: target.map(|target| format!("0x{target:x}")),
                        false_target: Some(format!("0x{:x}", instruction.next)),
                        confidence: condition.confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                }
                last_comparison = None;
            }
            // Derived element pointers on x64: `lea r13, [rdx + 0x13]`.
            "lea" if operands.len() >= 2 && operands[1].contains('[') => {
                let target = normalize_register(&operands[0]);
                let base_alias = operands
                    .last()
                    .and_then(|value| arm_memory_address(value))
                    .and_then(|(base, displacement)| {
                        aliases
                            .get(&base)
                            .map(|(site, extra)| (*site, extra.saturating_add(displacement)))
                    });
                registers.remove(&target);
                compressed_loads.remove(&target);
                match base_alias {
                    Some(alias) => {
                        aliases.insert(target, alias);
                    }
                    None => {
                        aliases.remove(&target);
                    }
                }
            }
            mnemonic if writes_first_operand(mnemonic) => {
                if let Some(target) = operands.first() {
                    let target = normalize_register(target);
                    registers.remove(&target);
                    aliases.remove(&target);
                    compressed_loads.remove(&target);
                }
            }
            _ => {}
        }
    }
}

/// Compressed-pointer `Array` element slot for a store displacement.
/// ARM64 keeps the tagged payload at `data - 1`; both Android ABIs use
/// 4-byte compressed elements. ARM32's header is one word smaller.
fn element_index(abi: Abi, displacement: i64) -> Option<usize> {
    let (base, stride) = match abi {
        // Compressed-pointer builds pack elements at 4 bytes on every target.
        Abi::Arm64V8a | Abi::X86_64 => (15i64, 4i64),
        Abi::ArmeabiV7a => (11i64, 4i64),
    };
    let relative = displacement.checked_sub(base)?;
    (relative >= 0 && relative % stride == 0)
        .then_some(relative / stride)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < 128)
}

fn is_interpolate_target(target: &str) -> bool {
    let name = target.rsplit('.').next().unwrap_or(target);
    matches!(name, "_interpolate" | "interpolate")
}

fn looks_like_allocation_stub(target: &str) -> bool {
    target.starts_with("sub_")
        || target.contains("AllocateArray")
        || target.contains("NewArray")
        || target.contains("AllocateObject")
}

/// Exact inputs of the VM allocation stubs whose results Dart code keeps,
/// read from their fixed ABI registers (`constants_<arch>.h`) instead of
/// from whichever argument registers the caller happened to write:
///
/// - `AllocateContext`: the variable count as a raw integer
///   (R1 / X1 / R10).
/// - `AllocateArray`: `AllocateArrayABI` type arguments, then the Smi
///   length.
/// - `AllocateGrowableArray`: the `AllocateObjectABI` type arguments.
/// - `Allocate<T>Array` typed data: the Smi length in
///   `AllocateTypedDataArrayABI::kLengthReg` (R4 / X4 / RAX).
/// - `AllocateRecord2` / `AllocateRecord3`: the field values in
///   `AllocateSmallRecordABI` order.
/// - `InstanceOf`: the tested instance and the destination type.
/// - `InstantiateTypeArguments*`: the `InstantiationABI` type arguments.
fn allocation_stub_arguments(
    abi: Abi,
    target: &str,
    registers: &BTreeMap<String, Expression>,
) -> Option<Vec<String>> {
    let stub = target.strip_prefix("stub ")?;
    let value = |register: &str| {
        registers
            .get(register)
            .map_or_else(|| register.to_owned(), |value| value.text.clone())
    };
    let (context_count, array_types, array_length, object_types, typed_length, record) =
        match abi {
            Abi::ArmeabiV7a => ("r1", "r1", "r2", "r3", "r4", ["r2", "r3", "r4"]),
            Abi::Arm64V8a => ("x1", "x1", "x2", "x1", "x4", ["x2", "x3", "x4"]),
            Abi::X86_64 => ("r10", "rbx", "r10", "rdx", "rax", ["rbx", "rdx", "rcx"]),
        };
    // `TypeTestABI`: the tested instance and the destination type.
    let (instance, destination_type) = match abi {
        Abi::ArmeabiV7a => ("r0", "r8"),
        Abi::Arm64V8a => ("x0", "x8"),
        Abi::X86_64 => ("rax", "rbx"),
    };
    // `InstantiationABI`: uninstantiated, instantiator and function type
    // arguments.
    let instantiation = match abi {
        Abi::ArmeabiV7a => ["r3", "r2", "r1"],
        Abi::Arm64V8a => ["x3", "x2", "x1"],
        Abi::X86_64 => ["rbx", "rdx", "rcx"],
    };
    match stub {
        "InstanceOf" => Some(vec![value(instance), value(destination_type)]),
        "InstantiateTypeArguments"
        | "InstantiateTypeArgumentsMayShareInstantiatorTA"
        | "InstantiateTypeArgumentsMayShareFunctionTA" => Some(
            instantiation
                .iter()
                .map(|register| value(register))
                .collect(),
        ),
        "AllocateContext" => Some(vec![value(context_count)]),
        "AllocateArray" => Some(vec![value(array_types), value(array_length)]),
        "AllocateGrowableArray" => Some(vec![value(object_types)]),
        "AllocateRecord2" => Some(record[..2].iter().map(|register| value(register)).collect()),
        "AllocateRecord3" => Some(record.iter().map(|register| value(register)).collect()),
        _ if typed_data_list_class(stub).is_some() => Some(vec![value(typed_length)]),
        _ => None,
    }
}

/// The result name of a value-producing VM stub called at `address`:
/// `stub_AllocateArray_1a2b_result`. Unique per call site, so values the
/// same stub creates stay distinct in text-keyed IR and aliases.
pub(crate) fn site_result_name(target: &str, address: u64) -> String {
    format!("{}_{address:x}_result", sanitize_semantic_name(target))
}

/// The `dart:typed_data` list a typed-data allocation stub creates.
pub(crate) fn typed_data_list_class(stub: &str) -> Option<&'static str> {
    Some(match stub {
        "AllocateInt8Array" => "Int8List",
        "AllocateUint8Array" => "Uint8List",
        "AllocateUint8ClampedArray" => "Uint8ClampedList",
        "AllocateInt16Array" => "Int16List",
        "AllocateUint16Array" => "Uint16List",
        "AllocateInt32Array" => "Int32List",
        "AllocateUint32Array" => "Uint32List",
        "AllocateInt64Array" => "Int64List",
        "AllocateUint64Array" => "Uint64List",
        "AllocateFloat32Array" => "Float32List",
        "AllocateFloat64Array" => "Float64List",
        "AllocateFloat32x4Array" => "Float32x4List",
        "AllocateInt32x4Array" => "Int32x4List",
        "AllocateFloat64x2Array" => "Float64x2List",
        _ => return None,
    })
}

/// Register holding the Smi element count for the `AllocateArray` stub
/// (`AllocateArrayABI::kLengthReg`).
fn allocate_array_length_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x2",
        Abi::ArmeabiV7a => "r2",
        Abi::X86_64 => "r10",
    }
}

/// Element count of a constant Smi length argument.
fn constant_array_length(value: &Expression) -> Option<usize> {
    // Immediate expressions are rendered in decimal (`immediate_text`).
    let tagged = value.text.parse::<u64>().ok()?;
    (tagged % 2 == 0)
        .then_some(tagged / 2)
        .and_then(|length| usize::try_from(length).ok())
        .filter(|length| *length <= 128)
}

/// Makes a spill slot refer to the buffer `register` points at, or clears
/// the slot's buffer reference when the stored value is not a buffer base.
fn spill_alias(aliases: &mut BTreeMap<String, (u64, i64)>, register: &str, slot: &str) {
    match aliases.get(register).copied() {
        Some((site, 0)) => {
            aliases.insert(stack_key(slot), (site, 0));
        }
        _ => {
            aliases.remove(&stack_key(slot));
        }
    }
}

/// Makes `register` refer to the buffer spilled in `slot`, if any.
fn reload_alias(aliases: &mut BTreeMap<String, (u64, i64)>, register: &str, slot: &str) {
    match aliases.get(&stack_key(slot)).copied() {
        Some(alias) => {
            aliases.insert(register.to_owned(), alias);
        }
        None => {
            aliases.remove(register);
        }
    }
}

/// Stores `value` through `base + displacement` when `base` points into a
/// tracked buffer. Returns whether the store was consumed as buffer
/// traffic. A store outside the element layout invalidates the buffer,
/// since the allocation was then not an interpolation array.
fn store_element(
    abi: Abi,
    buffers: &mut BTreeMap<u64, ElementBuffer>,
    aliases: &BTreeMap<String, (u64, i64)>,
    base: &str,
    displacement: i64,
    value: Option<&Expression>,
) -> Option<Option<ElementWrite>> {
    let (site, extra) = aliases.get(base).copied()?;
    let buffer = buffers.get_mut(&site)?;
    let offset = extra.saturating_add(displacement);
    match element_index(abi, offset)
        .filter(|index| buffer.length.is_none_or(|length| *index < length))
    {
        Some(index) => {
            buffer.store(index, value);
            Some(value.map(|value| ElementWrite {
                array: buffer.array.clone(),
                index,
                offset: offset.saturating_add(1),
                value: value.clone(),
            }))
        }
        None => {
            buffers.remove(&site);
            Some(None)
        }
    }
}

/// A store into a tracked array's element `index` (untagged byte `offset`).
struct ElementWrite {
    array: String,
    index: usize,
    offset: i64,
    value: Expression,
}

impl ElementWrite {
    /// The write as IR: a field write whose field is `[index]`, which the
    /// renderer spells as an index assignment. Interpolation buffers keep
    /// their writes too; the renderer drops an array nothing else reads.
    fn statement(self, address: u64) -> SemanticStatement {
        SemanticStatement::FieldWrite {
            receiver: self.array,
            field: format!("[{}]", self.index),
            offset: self.offset,
            confidence: weaker(self.value.confidence, EvidenceConfidence::Medium),
            value: self.value.text,
            address: format!("0x{address:x}"),
        }
    }
}

/// Drops register and outgoing-slot references at a call; buffers spilled
/// to the frame survive it.
fn retain_frame_buffers(
    buffers: &mut BTreeMap<u64, ElementBuffer>,
    aliases: &mut BTreeMap<String, (u64, i64)>,
) {
    aliases.retain(|key, _| key.starts_with("stk:"));
    buffers.retain(|site, _| aliases.values().any(|(alias, _)| alias == site));
}

/// Consumes the buffer passed as `_interpolate`'s argument and renders it as
/// a Dart string literal. Literal parts are JSON-decoded pool strings;
/// unresolved parts, including allocated slots never stored, become explicit
/// placeholders instead of being guessed. The result is only as strong as
/// its weakest part.
fn take_interpolation_literal(
    abi: Abi,
    buffers: &mut BTreeMap<u64, ElementBuffer>,
    aliases: &BTreeMap<String, (u64, i64)>,
) -> Option<(String, EvidenceConfidence)> {
    let first_register = TargetLayout::of(abi).cpu_arguments.first().copied();
    let (site, displacement) = aliases
        .get(&outgoing_key(0))
        .or_else(|| first_register.and_then(|register| aliases.get(register)))
        .copied()?;
    if displacement != 0 {
        return None;
    }
    let mut buffer = buffers.remove(&site)?;
    if buffer.parts.iter().flatten().count() == 0 {
        return None;
    }
    let mut confidence = buffer.confidence;
    match buffer.length {
        Some(length) if length >= buffer.parts.len() => buffer.parts.resize(length, None),
        Some(_) => return None,
        // Without the allocated length a trailing element may be missing.
        None => confidence = weaker(confidence, EvidenceConfidence::Medium),
    }
    if buffer.parts.iter().any(Option::is_none) {
        confidence = EvidenceConfidence::Low;
    }
    let mut output = String::from("'");
    for part in &buffer.parts {
        match part {
            Some(text) => {
                // A `…` after the closing quote marks a label cut short: the
                // rest of the literal is unknown, not an expression.
                let (encoded, truncated) = text
                    .strip_suffix('…')
                    .filter(|encoded| encoded.ends_with('"'))
                    .map_or((text.as_str(), false), |encoded| (encoded, true));
                if let Some(decoded) = decode_json_string(encoded) {
                    for character in decoded.chars() {
                        match character {
                            '$' => output.push_str("\\$"),
                            '\'' => output.push_str("\\'"),
                            '\\' => output.push_str("\\\\"),
                            '\n' => output.push_str("\\n"),
                            '\r' => output.push_str("\\r"),
                            other if needs_unicode_escape(other) => {
                                output.push_str(&format!("\\u{{{:x}}}", other as u32));
                            }
                            other => output.push(other),
                        }
                    }
                    if truncated {
                        output.push_str("${aot.unresolvedValue('truncated literal')}");
                        confidence = weaker(confidence, EvidenceConfidence::Medium);
                    }
                } else {
                    // An expression part renders as an interpolation.
                    output.push_str("${");
                    output.push_str(strip_outer_parens(text));
                    output.push('}');
                }
            }
            None => output.push_str("${aot.unresolvedValue('interpolated part')}"),
        }
    }
    output.push('\'');
    Some((output, confidence))
}

/// Characters generated source must spell as `\u{…}` escapes: control
/// characters and the bidirectional formatting characters that make text
/// display differently from how it parses ("Trojan Source").
pub(crate) fn needs_unicode_escape(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
        )
}

fn decode_json_string(value: &str) -> Option<String> {
    serde_json::from_str(value).ok()
}

/// Strips one balanced outer parenthesisation, leaving expressions such as
/// `snapshotRef(371)` untouched.
fn strip_outer_parens(text: &str) -> &str {
    let trimmed = text.trim();
    if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
        return trimmed;
    }
    let mut depth = 0usize;
    for (index, character) in trimmed.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 && index != trimmed.len() - 1 {
                    return trimmed;
                }
            }
            _ => {}
        }
    }
    &trimmed[1..trimmed.len() - 1]
}

/// Extracts the simple class name of a declared type when it can carry
/// instance fields (excludes primitives and non-instance types).
pub(crate) fn simple_class_type(display_name: &str) -> Option<String> {
    let value = display_name.trim_end_matches('?');
    let root = value.split('<').next()?.trim();
    if root.is_empty()
        || root.contains([' ', '(', ')', '[', ']', '{', '}', ','])
        || matches!(
            root,
            "dynamic" | "void" | "Never" | "Null" | "bool" | "double" | "int" | "num" | "String"
        )
    {
        return None;
    }
    Some(root.to_owned())
}

/// An incoming argument's stable name plus any surviving declared class.
/// Class provenance lets member reads on that parameter resolve real field
/// names instead of raw slot offsets.
#[derive(Clone, Debug)]
pub(crate) struct ParameterHint {
    pub name: String,
    pub class_name: Option<String>,
    pub class_library_uri: Option<String>,
}

/// Stable register-level names for a function's incoming arguments. The
/// implicit receiver/tear-off parameters occupy the leading slots; visible
/// parameter names and declared classes come from the resolved signature
/// when available.
pub(crate) fn semantic_parameter_hints(
    function: &crate::model::RecoveredFunction,
) -> Vec<ParameterHint> {
    use crate::model::RecoveredFunctionKind;
    let signature = function.signature.as_ref();
    let implicit = signature
        .map(|signature| signature.implicit_parameter_count)
        .or_else(|| {
            function
                .vm_evidence
                .as_ref()
                .and_then(|evidence| evidence.implicit_parameter_count)
        })
        .unwrap_or_else(|| {
            // Without a surviving signature, Dart's AOT calling convention
            // still gives a proven instance member an implicit receiver in
            // argument slot zero.
            usize::from(
                function.is_static == Some(false)
                    && function
                        .owner
                        .as_deref()
                        .is_some_and(|owner| !matches!(owner, "::" | "top_level")),
            )
        });
    let visible = signature
        .map(|signature| {
            signature
                .fixed_parameter_count
                .saturating_add(signature.optional_parameter_count)
        })
        .or(function.parameter_count)
        .unwrap_or_default();
    let mut names = Vec::with_capacity(implicit.saturating_add(visible));
    for index in 0..implicit {
        let has_instance_owner = function
            .owner
            .as_deref()
            .is_some_and(|owner| !matches!(owner, "::" | "top_level"));
        let name = if index == 0
            && matches!(
                function.kind,
                Some(RecoveredFunctionKind::Closure | RecoveredFunctionKind::ImplicitClosure)
            )
        {
            "closureContext".to_owned()
        } else if index == 0
            && has_instance_owner
            && function
                .vm_evidence
                .as_ref()
                .is_none_or(|evidence| evidence.is_static != Some(true))
        {
            "this".to_owned()
        } else {
            format!("implicitArg{index}")
        };
        // A closure body's first argument is the Closure object itself;
        // its captured variables live in the Context it points to.
        let closure = name == "closureContext";
        names.push(ParameterHint {
            name,
            class_name: closure.then(|| "_Closure".to_owned()),
            class_library_uri: closure.then(|| "dart:core".to_owned()),
        });
    }
    let resolved = signature.and_then(|signature| signature.resolved.as_ref());
    for index in 0..visible {
        let parameter = resolved.and_then(|resolved| resolved.parameters.get(index));
        let name = parameter
            .and_then(|parameter| parameter.name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("arg{index}"));
        let declared = parameter.and_then(|parameter| parameter.declared_type.as_ref());
        let class = declared.and_then(|type_| simple_class_type(&type_.display_name));
        let class_library_uri = class
            .as_ref()
            .and_then(|_| declared.and_then(|type_| type_.library_uri.clone()));
        names.push(ParameterHint {
            name,
            class_name: class,
            class_library_uri,
        });
    }
    names
}

/// Result classes by call-target label. A label several functions share
/// keeps a class only when all of them declare the same one.
pub(crate) fn label_result_classes(
    symbols: &BTreeMap<u64, Symbol>,
) -> BTreeMap<String, (String, Option<String>)> {
    let mut classes = BTreeMap::<String, Option<(String, Option<String>)>>::new();
    for symbol in symbols.values().filter(|symbol| symbol.semantic_name) {
        let result = symbol
            .result_class
            .clone()
            .map(|class| (class, symbol.result_library_uri.clone()));
        match classes.get(&symbol.label) {
            None => {
                classes.insert(symbol.label.clone(), result);
            }
            Some(existing) if *existing == result => {}
            Some(_) => {
                classes.insert(symbol.label.clone(), None);
            }
        }
    }
    classes
        .into_iter()
        .filter_map(|(label, result)| Some((label, result?)))
        .collect()
}

/// The class whose instance member `function` is, when its receiver is the
/// first argument: a non-static method, accessor or constructor body.
fn instance_member_owner(
    function: &crate::model::RecoveredFunction,
) -> Option<(String, Option<String>)> {
    use crate::model::RecoveredFunctionKind as Kind;
    let owner = function
        .owner
        .as_deref()
        .filter(|owner| !matches!(*owner, "::" | "top_level"))?;
    let member = matches!(
        function.kind?,
        Kind::Regular
            | Kind::Getter
            | Kind::Setter
            | Kind::Constructor
            | Kind::ImplicitGetter
            | Kind::ImplicitSetter
            | Kind::MethodExtractor
    );
    let has_receiver = function.is_static == Some(false)
        && function
            .signature
            .as_ref()
            .is_none_or(|signature| signature.implicit_parameter_count > 0);
    (member && has_receiver).then(|| {
        (
            crate::analysis::readable_snapshot_name(owner),
            function.library_uri.clone(),
        )
    })
}

/// Builds the call-target symbol table from recovered functions, plus the
/// qualified-name → library candidates used to attribute recovered indirect
/// calls.
#[allow(clippy::type_complexity)]
pub(crate) fn build_function_symbols(
    abi: Abi,
    functions: &[crate::model::RecoveredFunction],
    application_package: Option<&str>,
) -> (
    BTreeMap<u64, Symbol>,
    BTreeMap<String, BTreeSet<Option<String>>>,
) {
    let mut symbols = BTreeMap::<u64, Symbol>::new();
    let mut target_library_candidates = BTreeMap::<String, BTreeSet<Option<String>>>::new();
    for function in functions {
        let Some(address) = parse_immediate(&function.address) else {
            continue;
        };
        // `name_source` alone says whether a name is Clutter's: an
        // application function may itself be called `sub_1000`.
        let semantic = function.name_source != crate::model::RecoveredNameSource::Synthetic
            && function.name != "unknownFunction";
        if semantic {
            let qualified = match function.owner.as_deref() {
                Some(owner) if !matches!(owner, "::" | "top_level") => {
                    format!("{owner}.{}", function.name)
                }
                _ => function.name.clone(),
            };
            target_library_candidates
                .entry(qualified)
                .or_default()
                .insert(function.library_uri.clone());
        }
        // Constructors return their owner; getters and methods return their
        // declared type when it survived tree-shaking. Either way the class
        // seeds receiver provenance so chained member reads resolve fields.
        let return_type = function
            .signature
            .as_ref()
            .and_then(|signature| signature.resolved.as_ref())
            .and_then(|resolved| resolved.return_type.as_ref());
        let (result_class, result_library_uri) = match function.kind {
            Some(crate::model::RecoveredFunctionKind::Constructor) => (
                function
                    .owner
                    .clone()
                    .map(|owner| crate::analysis::readable_snapshot_name(&owner)),
                function.library_uri.clone(),
            ),
            _ => match return_type
                .and_then(|return_type| simple_class_type(&return_type.display_name))
            {
                Some(class) => (
                    Some(class),
                    return_type.and_then(|return_type| return_type.library_uri.clone()),
                ),
                None => (None, None),
            },
        };
        let receiver_class = instance_member_owner(function);
        let symbol = if semantic {
            let label = match function.owner.as_deref() {
                Some(owner) if !matches!(owner, "::" | "top_level") => {
                    format!("{owner}.{}", function.name)
                }
                _ => function.name.clone(),
            };
            let mut symbol =
                Symbol::new(label, function.library_uri.clone(), application_package)
                    .with_code_identity(address, 0, crate::model::DirectCallResolution::ExactEntry)
                    .with_result_class(result_class);
            symbol.result_library_uri = result_library_uri;
            symbol.receiver_class = receiver_class;
            symbol
        } else {
            Symbol::code_boundary(address)
        };
        let symbol = with_callee_convention(abi, function, symbol);
        insert_preferred(&mut symbols, address, symbol.clone());
        if let Some(offset) = function
            .code_metadata
            .as_ref()
            .and_then(|metadata| metadata.unchecked_entry_offset)
            .filter(|offset| *offset > 0 && *offset < function.size)
        {
            let unchecked = symbol.with_code_identity(
                address,
                offset,
                crate::model::DirectCallResolution::UncheckedEntry,
            );
            insert_preferred(&mut symbols, address.saturating_add(offset), unchecked);
        }
    }
    (symbols, target_library_candidates)
}

/// Attaches the parameter locations a call to `function` must use.
///
/// Per-class allocation stubs (a frameless tail branch into the shared
/// allocation stub) take no Dart arguments. Other bodies need a surviving
/// parameter count; their fixed parameters are placed by the kind rules
/// and the body's own reads (see `calling_convention::resolve_parameters`).
fn with_callee_convention(
    abi: Abi,
    function: &crate::model::RecoveredFunction,
    mut symbol: Symbol,
) -> Symbol {
    use crate::analysis::calling_convention::{fixed_parameter_count, resolve_parameters};
    if is_allocation_stub(function) {
        symbol.parameters = Some(std::sync::Arc::from(Vec::new()));
        symbol.allocation_stub = true;
        return symbol;
    }
    let convention = ConventionInput::for_function(abi, function);
    let (evidence, returns_fpu) =
        body_evidence(abi, &function.instructions, convention.entry_offset);
    symbol.returns_fpu = returns_fpu == Some(true);
    let layout = TargetLayout::of(abi);
    let Some(declared) = convention.declared.as_ref() else {
        symbol.parameters = observed_register_parameters(layout, &evidence);
        return symbol;
    };
    let window = convention.window.unwrap_or(RegisterWindow {
        min: 0,
        max: declared.len(),
    });
    let resolved = resolve_parameters(layout, window, declared, &evidence);
    let fixed = fixed_parameter_count(function)
        .unwrap_or(declared.len())
        .min(resolved.parameters.len());
    symbol.parameters = Some(
        resolved.parameters[..fixed]
            .iter()
            .map(|parameter| parameter.location)
            .collect(),
    );
    symbol
}

/// Parameter locations of a body whose parameter count did not survive,
/// when its own reads leave no ordering ambiguity: every live argument
/// register belongs to one register class and they form a prefix of that
/// class's sequence, and no incoming stack word is read. Interleaving CPU
/// and FPU parameters cannot be recovered from reads alone.
///
/// A body that reads incoming stack words but no argument register uses
/// the all-stack convention (`MaxNumberOfParametersInRegisters() == 0`:
/// closures, dispatchers, overrides pinned to a stack signature); its
/// parameters extend at least to the farthest word it reads.
fn observed_register_parameters(
    layout: &TargetLayout,
    evidence: &BodyEvidence,
) -> Option<std::sync::Arc<[ArgumentLocation]>> {
    if !evidence.stack_words.is_empty() {
        let registers_live = evidence.live_cpu.iter().chain(&evidence.live_fpu).any(|live| *live);
        let count = evidence.stack_words.last()?.checked_add(1)?;
        return (!registers_live).then(|| {
            (0..count)
                .rev()
                .map(|word| ArgumentLocation::Stack { word, words: 1 })
                .collect()
        });
    }
    let prefix = |live: &[bool]| {
        let count = live.iter().filter(|live| **live).count();
        live.iter().take(count).all(|live| *live).then_some(count)
    };
    let cpu = prefix(&evidence.live_cpu)?;
    let fpu = prefix(&evidence.live_fpu)?;
    match (cpu, fpu) {
        (0, 0) => None,
        (count, 0) => Some(
            layout.cpu_arguments[..count]
                .iter()
                .map(|register| ArgumentLocation::Register(register))
                .collect(),
        ),
        (0, count) => Some(
            layout.fpu_arguments[..count]
                .iter()
                .map(|register| ArgumentLocation::FpuRegister(register))
                .collect(),
        ),
        _ => None,
    }
}

/// A per-class allocation stub: a code object owned by a Class rather than
/// a Function, lowered to a frameless tail branch into the shared
/// `AllocateObject` stub after loading the class's tags.
fn is_allocation_stub(function: &crate::model::RecoveredFunction) -> bool {
    if function.kind.is_some()
        || function.instructions.is_empty()
        || function.instructions.len() > 6
    {
        return false;
    }
    let end = parse_immediate(&function.address).map(|start| start.saturating_add(function.size));
    // Alignment padding (`int3`, `brk`, `udf`) may follow the tail branch.
    let Some(last) = function.instructions.iter().rev().find(|instruction| {
        !matches!(
            instruction.mnemonic.as_str(),
            "int3" | "brk" | "udf" | "nop" | "bkpt"
        ) && !is_skipped_data(&instruction.mnemonic)
    }) else {
        return false;
    };
    branch_kind(&last.mnemonic) == Some(false)
        && branch_target(&last.operands)
            .zip(parse_immediate(&function.address).zip(end))
            .is_some_and(|(target, (start, end))| !(start..end).contains(&target))
        && function
            .instructions
            .iter()
            .all(|instruction| !is_call(&instruction.mnemonic))
}

fn insert_preferred(symbols: &mut BTreeMap<u64, Symbol>, address: u64, symbol: Symbol) {
    match symbols.get(&address) {
        Some(existing) if existing.semantic_name || !symbol.semantic_name => {}
        _ => {
            symbols.insert(address, symbol);
        }
    }
}

fn recover_object_pool_loads(
    abi: Abi,
    instructions: &[DecodedInstruction],
) -> BTreeMap<u64, usize> {
    let mut provenance = PoolPointerProvenance::new(abi);
    let mut loads = BTreeMap::new();
    for instruction in instructions {
        if let Some(index) = provenance.load_index(&instruction.mnemonic, &instruction.operands) {
            loads.insert(instruction.address, index);
        }
        provenance.observe(&instruction.mnemonic, &instruction.operands);
    }
    loads
}

fn abi_return_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x0",
        Abi::ArmeabiV7a => "r0",
        Abi::X86_64 => "rax",
    }
}

/// Applies a call's register clobbers. Dart code keeps no values in
/// callee-saved registers across calls: the allocator blocks every
/// allocatable CPU and FPU register, so only the VM's reserved registers
/// (thread, pool, null, frame and stack pointers) survive.
fn kill_caller_saved(abi: Abi, registers: &mut BTreeMap<String, Expression>) {
    let layout = TargetLayout::of(abi);
    registers.retain(|register, _| !layout.is_clobbered_by_call(register));
}

/// Register that receives a call's result.
fn call_result_register(abi: Abi, callee: Option<&Symbol>) -> &'static str {
    let layout = TargetLayout::of(abi);
    if callee.is_some_and(|symbol| symbol.returns_fpu) {
        layout.fpu_return_register
    } else {
        layout.return_register
    }
}

fn split_operands(value: &str) -> Vec<String> {
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut values = Vec::new();
    for (index, character) in value.char_indices() {
        match character {
            '[' | '{' => depth += 1,
            ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                values.push(value[start..index].trim().to_owned());
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < value.len() {
        values.push(value[start..].trim().to_owned());
    }
    values
}

fn resolve_expression(
    operand: &str,
    registers: &BTreeMap<String, Expression>,
    object_pool: Option<&[String]>,
) -> Option<Expression> {
    let register = normalize_register(operand);
    if let Some(value) = registers.get(&register) {
        return Some(value.clone());
    }
    immediate_text(operand)
        .map(|value| Expression {
            text: value,
            confidence: EvidenceConfidence::Low,
            complexity: 1,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        })
        .or_else(|| {
            let index = operand
                .strip_prefix("pool[")?
                .strip_suffix(']')?
                .parse::<usize>()
                .ok()?;
            Some(Expression {
                text: pool_value_text(object_pool?.get(index)?),
                confidence: EvidenceConfidence::Medium,
                complexity: 1,
                class_name: object_pool
                    .and_then(|pool| pool.get(index))
                    .and_then(|value| snapshot_instance_class(value)),
                class_library_uri: None,
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: object_pool
                    .and_then(|pool| pool.get(index))
                    .is_some_and(|value| snapshot_instance_class(value).is_some()),
            })
        })
}

/// Recognizes bare machine-register spellings (`w3`, `x0`, `rax`) so
/// comparisons can keep untracked values visible by name.
fn is_register_spelling(value: &str) -> bool {
    let value = value.trim().trim_start_matches('#');
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        && value
            .chars()
            .next()
            .is_some_and(|character| matches!(character, 'x' | 'w' | 'r' | 'e'))
}

fn normalize_register(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    const ARM_ALIASES: &[(&str, &str)] = &[
        ("sb", "r9"),
        ("sl", "r10"),
        ("fp", "r11"),
        ("ip", "r12"),
        ("sp", "r13"),
        ("lr", "r14"),
        ("pc", "r15"),
    ];
    if let Some((_, canonical)) = ARM_ALIASES.iter().find(|(alias, _)| *alias == value) {
        return (*canonical).to_owned();
    }
    // x86 sub-register spellings collapse onto their 64-bit name so
    // provenance survives `mov eax, ...` vs `mov rax, ...`.
    const X86_ALIASES: &[(&str, &str)] = &[
        ("eax", "rax"),
        ("ebx", "rbx"),
        ("ecx", "rcx"),
        ("edx", "rdx"),
        ("esi", "rsi"),
        ("edi", "rdi"),
        ("ebp", "rbp"),
        ("esp", "rsp"),
        ("sil", "rsi"),
        ("dil", "rdi"),
        ("bpl", "rbp"),
        ("spl", "rsp"),
        ("al", "rax"),
        ("bl", "rbx"),
        ("cl", "rcx"),
        ("dl", "rdx"),
        ("ax", "rax"),
        ("bx", "rbx"),
        ("cx", "rcx"),
        ("dx", "rdx"),
    ];
    if let Some((_, canonical)) = X86_ALIASES.iter().find(|(alias, _)| *alias == value) {
        return (*canonical).to_owned();
    }
    if value.len() >= 4
        && value.starts_with('r')
        && value.ends_with('d')
        && let Ok(index) = value[1..value.len() - 1].parse::<u8>()
    {
        return format!("r{index}");
    }
    if value.len() == 4
        && value.starts_with('r')
        && value.ends_with('w')
        && let Ok(index) = value[1..value.len() - 1].parse::<u8>()
    {
        return format!("r{index}");
    }
    value
        .strip_prefix('w')
        .and_then(|suffix| suffix.parse::<u8>().ok())
        .map_or(value.clone(), |index| format!("x{index}"))
}

fn immediate_text(value: &str) -> Option<String> {
    let value = value.trim().trim_start_matches('#');
    let (negative, value) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    let value = value.strip_prefix('+').unwrap_or(value);
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        i128::from_str_radix(hex, 16).ok()?
    } else {
        value.parse::<i128>().ok()?
    };
    Some(if negative {
        format!("-{parsed}")
    } else {
        parsed.to_string()
    })
}

fn floating_immediate_text(value: &str) -> Option<String> {
    let parsed = value.trim().trim_start_matches('#').parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    let mut text = parsed.to_string();
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    Some(text)
}

/// Interprets an x64 `movsd` immediate-pool entry as its IEEE-754 payload.
/// Snapshot recovery renders raw `Immediate64` entries as decimal bit
/// patterns; the floating load opcode is the type evidence that makes this
/// conversion unambiguous.
fn object_pool_f64_expression(value: &str) -> Option<Expression> {
    let bits = value
        .parse::<u64>()
        .ok()
        .or_else(|| value.parse::<i64>().ok().map(|value| value as u64))?;
    let parsed = f64::from_bits(bits);
    let text = floating_immediate_text(&parsed.to_string())?;
    Some(Expression {
        text,
        confidence: EvidenceConfidence::High,
        complexity: 1,
        class_name: Some("double".to_owned()),
        class_library_uri: Some("dart:core".to_owned()),
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    })
}

/// Classifies a memory operand as a named stack slot addressed through the
/// frame or stack pointer. With known pointer deltas the key is relative to
/// the entry stack pointer; otherwise it is the literal base/displacement.
fn stack_slot_key(abi: Abi, value: &str, deltas: FrameDeltas) -> Option<String> {
    let (base, displacement) = arm_memory_address(value)?;
    let delta = if base == abi_stack_register(abi) {
        deltas.sp
    } else if base == abi_frame_register(abi) {
        deltas.fp
    } else {
        return None;
    };
    match delta {
        Some(delta) => Some(entry_slot_key(delta + displacement)),
        None => slot_keys(&base, displacement).into_iter().next(),
    }
}

/// Whether an instruction adjusts the stack pointer (frame set-up,
/// teardown, pushes and pops) rather than storing through it.
fn moves_stack_pointer(abi: Abi, instruction: &DecodedInstruction) -> bool {
    let stack_register = abi_stack_register(abi);
    let parts = split_operands(&instruction.operands);
    matches!(instruction.mnemonic.as_str(), "push" | "pop" | "pushq" | "popq" | "vpush" | "vpop")
        || parts
            .first()
            .filter(|_| writes_first_operand(&instruction.mnemonic))
            .is_some_and(|destination| {
                // `mov [rsp], reg` writes memory, not the stack pointer.
                !destination.contains('[')
                    && normalize_register(destination.trim_end_matches('!')) == stack_register
            })
        // ARM pre/post-index writeback; x64 has no writeback addressing,
        // and its `[rsp], reg` store operands must not match.
        || abi != Abi::X86_64
            && parts.iter().any(|part| {
                arm_memory_address(part).is_some_and(|(base, _)| base == stack_register)
                    && (part.ends_with('!') || instruction.operands.contains("], "))
            })
}

/// Arguments of a Dart call in parameter order.
///
/// With a known callee convention each fixed parameter is read from its own
/// location, including values the caller merely forwards from its incoming
/// arguments; stacked arguments (fixed overflow and every optional
/// argument) follow in the order the caller allocated them, highest stack
/// word first. Without one, only argument registers this body wrote since
/// the previous call are reported, then the stack.
/// Arguments of a dispatch-table call after its receiver. Instance calls
/// fill the CPU argument registers from the receiver's onward without gaps
/// (`ComputeCallingConvention`), so the register arguments are the written
/// prefix after the receiver's register; stacked arguments follow. A final
/// register copy of the receiver is the class-id load's scratch register.
fn dispatch_call_arguments(
    abi: Abi,
    registers: &BTreeMap<String, Expression>,
    outgoing: &BTreeMap<i64, Expression>,
    written_argument_registers: u16,
    receiver: Option<&Expression>,
) -> Vec<String> {
    let mut stacked = outgoing.iter().collect::<Vec<_>>();
    stacked.sort_by(|left, right| right.0.cmp(left.0));
    // The receiver in the first stacked slot marks the all-stack convention.
    if let Some(((_, first), rest)) = stacked.split_first()
        && receiver.is_some_and(|receiver| receiver.text == first.text)
    {
        return rest.iter().map(|(_, value)| value.text.clone()).collect();
    }
    let layout = TargetLayout::of(abi);
    let mut arguments = Vec::new();
    for (index, register) in layout.cpu_arguments.iter().enumerate().skip(1) {
        match registers.get(*register) {
            Some(value) if written_argument_registers & (1 << index) != 0 && !value.high_word => {
                arguments.push(value.text.clone())
            }
            _ => break,
        }
    }
    if stacked.is_empty()
        && arguments.last().map(String::as_str) == receiver.map(|value| value.text.as_str())
    {
        arguments.pop();
    }
    arguments.extend(stacked.into_iter().map(|(_, value)| value.text.clone()));
    arguments
}

fn collect_call_arguments(
    abi: Abi,
    registers: &BTreeMap<String, Expression>,
    outgoing: &BTreeMap<i64, Expression>,
    written_argument_registers: u16,
    callee: Option<&Symbol>,
) -> Vec<String> {
    let layout = TargetLayout::of(abi);
    let mut stacked = outgoing.iter().collect::<Vec<_>>();
    stacked.sort_by(|left, right| right.0.cmp(left.0));
    let mut stacked = stacked.into_iter().map(|(_, value)| value.text.clone());
    let mut arguments = Vec::new();
    if let Some(parameters) = callee.and_then(|symbol| symbol.parameters.as_deref()) {
        for location in parameters {
            let register = match *location {
                ArgumentLocation::Register(register)
                | ArgumentLocation::FpuRegister(register)
                | ArgumentLocation::RegisterPair(register, _) => register,
                ArgumentLocation::Stack { .. } => {
                    arguments.push(
                        stacked
                            .next()
                            .unwrap_or_else(|| "aot.unresolvedValue('stack argument')".to_owned()),
                    );
                    continue;
                }
            };
            // A callee whose pair parameters were not recognized still
            // lists the high word's register on its own.
            if let Some(value) = registers.get(register)
                && value.high_word
                && arguments.last() == Some(&value.text)
            {
                continue;
            }
            // An untracked register keeps its machine name, which renders
            // as an explicit unresolved register.
            arguments.push(
                registers
                    .get(register)
                    .map_or_else(|| register.to_owned(), |value| value.text.clone()),
            );
        }
        arguments.extend(stacked);
        return arguments;
    }
    for (index, register) in layout.cpu_arguments.iter().enumerate() {
        // Holes are tolerated: static calls leave the receiver slot unused
        // while further arguments still travel in their own registers.
        // The high word of an int64 pair travels with its low word, which
        // already names the whole value.
        if written_argument_registers & (1 << index) != 0
            && let Some(value) = registers.get(*register)
            && !value.high_word
        {
            arguments.push(value.text.clone());
        }
    }
    // FPU argument registers are not reported for an unknown callee: the
    // same registers stage every double operation (boxing, comparisons), so
    // a write proves nothing about the call.
    arguments.extend(stacked);
    // A register-only call with an exact retained arity cannot have extra
    // value arguments. With outgoing stack writes, their relationship to
    // recently written registers is still unknown.
    if outgoing.is_empty()
        && let Some(count) = callee.and_then(|symbol| symbol.value_argument_count)
    {
        arguments.truncate(count);
    }
    arguments
}

/// The high word of `value`'s int64 register pair on ARM32.
fn high_word_of(value: Expression) -> Expression {
    Expression {
        high_word: true,
        exact_class: false,
        ..value
    }
}

/// Integer arithmetic over untagged operands stays untagged: re-tagging the
/// result (`lsl #1`) then preserves the source value instead of rendering a
/// machine shift. A constant operand does not change the representation.
fn raw_arithmetic(
    result: Option<Expression>,
    left: &Expression,
    right: &Expression,
) -> Option<Expression> {
    let untagged = |value: &Expression| value.raw || value.text.parse::<i64>().is_ok();
    result.map(|result| Expression {
        raw: (left.raw || right.raw) && untagged(left) && untagged(right),
        ..result
    })
}

fn binary_expression(left: Expression, operator: &str, right: Expression) -> Option<Expression> {
    let complexity = left
        .complexity
        .saturating_add(right.complexity)
        .saturating_add(1);
    if complexity > 32 {
        return None;
    }
    Some(Expression {
        text: format!("({} {operator} {})", left.text, right.text),
        confidence: weaker(left.confidence, right.confidence),
        complexity,
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    })
}

fn snapshot_instance_class(value: &str) -> Option<String> {
    value
        .strip_prefix("snapshotInstance(")?
        .split_once(')')
        .map(|(label, _)| {
            label
                .rsplit_once('@')
                .map_or(label, |(class_name, _)| class_name)
                .to_owned()
        })
}

fn weaker(left: EvidenceConfidence, right: EvidenceConfidence) -> EvidenceConfidence {
    match (left, right) {
        (EvidenceConfidence::Low, _) | (_, EvidenceConfidence::Low) => EvidenceConfidence::Low,
        (EvidenceConfidence::Medium, _) | (_, EvidenceConfidence::Medium) => {
            EvidenceConfidence::Medium
        }
        _ => EvidenceConfidence::High,
    }
}

fn sanitize_semantic_name(value: &str) -> String {
    value
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

fn object_pool_index(abi: Abi, operands: &str) -> Option<usize> {
    let lower = operands.to_ascii_lowercase();
    let uses_pool_pointer = match abi {
        Abi::Arm64V8a => lower.contains("[x27"),
        Abi::ArmeabiV7a => lower.contains("[r5"),
        Abi::X86_64 => lower.contains("[r15"),
    };
    if !uses_pool_pointer {
        return None;
    }
    let offset = i64::try_from(parse_memory_offset(&lower)?).ok()?;
    pool_offset_to_index(abi, offset)
}

fn pool_offset_to_index(abi: Abi, offset: i64) -> Option<usize> {
    // ARM32 keeps PP as a tagged heap-object pointer. Consequently the first
    // pool payload word is addressed at ObjectPool::data_offset() -
    // kHeapObjectTag = 8 - 1 = 7. This is why real AOT loads use offsets such
    // as #0x13 and #0x8c3 rather than word-aligned values.
    // x64 keeps the tagged -1 addressing: pool data starts at 16 - 1.
    let (first_entry_offset, word_size) = match abi {
        Abi::ArmeabiV7a => (7i64, 4i64),
        Abi::Arm64V8a => (16i64, 8i64),
        Abi::X86_64 => (15i64, 8i64),
    };
    let relative = offset.checked_sub(first_entry_offset)?;
    (relative >= 0 && relative % word_size == 0)
        .then(|| usize::try_from(relative / word_size).ok())
        .flatten()
}

fn is_pool_load(abi: Abi, mnemonic: &str, operands: &str) -> bool {
    match abi {
        Abi::ArmeabiV7a => mnemonic.starts_with("ldr") || mnemonic == "vldr",
        Abi::Arm64V8a => mnemonic.starts_with("ldr"),
        Abi::X86_64 => {
            matches!(mnemonic, "mov" | "movq" | "movsd")
                && split_operands(operands)
                    .get(1)
                    .is_some_and(|operand| operand.contains('['))
        }
    }
}

fn arm_memory_address(value: &str) -> Option<(String, i64)> {
    let start = value.find('[')?;
    let end = value[start + 1..].find(']')?.saturating_add(start + 1);
    let inner = value.get(start + 1..end)?;
    // ARM prints `[x29, #8]`; x64 prints `[rbp - 8]` (space-separated).
    let operands = split_operands(inner);
    let (base_text, displacement_text) = if operands.len() >= 2 {
        (operands[0].as_str(), Some(operands[1].clone()))
    } else {
        let mut tokens = operands.first()?.split_whitespace();
        let base = tokens.next()?;
        let rest = tokens.collect::<Vec<_>>().join("");
        (base, if rest.is_empty() { None } else { Some(rest) })
    };
    let base = normalize_register(base_text);
    let displacement = displacement_text
        .as_deref()
        .map_or(Some(0), signed_immediate)?;
    Some((base, displacement))
}

/// Resolves a field access on a receiver with proven class. When the exact
/// Field declaration did not survive tree-shaking, the access still surfaces
/// as a low-confidence slot placeholder instead of being dropped — the
/// arithmetic feeding it remains visible while no invented member name is.
fn recovered_field_or_slot(
    field_layout: Option<&RecoveredFieldLayout>,
    receiver: &Expression,
    displacement: i64,
    abi: Abi,
) -> Option<(i64, RecoveredFieldIdentity)> {
    if let Some((offset, identity)) = recovered_field(field_layout, receiver, displacement) {
        let mut identity = identity.clone();
        // The retained `_Closure._context` field types its captured scope.
        if receiver.class_name.as_deref() == Some("_Closure")
            && receiver.class_library_uri.as_deref() == Some("dart:core")
            && identity.name == "_context"
        {
            identity.value_class = Some("Context".to_owned());
            identity.value_library_uri = Some("dart:core".to_owned());
        }
        return Some((offset, identity));
    }
    // Machine operands address a tagged heap pointer, so the displacement is
    // one byte below the VM's object-layout offset. Keep matching in machine
    // coordinates, but expose the untagged offset in semantic IR and names.
    // Android 64-bit targets use compressed pointers: an 8-byte header and
    // 4-byte slots. ARM32 has a one-word header.
    let (first_field, stride) = match abi {
        Abi::Arm64V8a | Abi::X86_64 => (7i64, 4i64),
        Abi::ArmeabiV7a => (3i64, 4i64),
    };
    if displacement < first_field
        || (displacement - first_field) % stride != 0
        || displacement > 4096
    {
        return None;
    }
    if receiver.class_library_uri.as_deref() == Some("dart:core") {
        match receiver.class_name.as_deref() {
            Some("Context") => return context_slot(abi, displacement),
            Some("_Closure") => return closure_slot(abi, displacement),
            _ => {}
        }
    }
    if let Some(class_name) = receiver.class_name.as_deref() {
        // Only the SDK's containers qualify: an application class may be
        // named `Map` or `String`, and dropping its field accesses would
        // erase its state. VM contexts were named above.
        let sdk_class = match receiver.class_library_uri.as_deref() {
            Some(library) => library.starts_with("dart:"),
            None => !field_layout.is_some_and(|layout| layout.has_application_class(class_name)),
        };
        if sdk_class
            && matches!(
                class_name,
                "Array" | "_GrowableList" | "_ImmutableList" | "String" | "Map" | "Set"
            )
        {
            // Container internals have their own meaning; never placeholder them.
            return None;
        }
    }
    // P2: even when receiver class is not proven, a strided field-like
    // displacement is still surfaced as a low-confidence `_slot_` instead of
    // being dropped. This closes the 7:1 write/read asymmetry on obfuscated
    // builds where `this` parameter type did not survive and also gives
    // the lifter a stable receiver expression to propagate.
    let offset = displacement.checked_add(1)?;
    Some((
        offset,
        RecoveredFieldIdentity {
            name: format!("_slot_{offset:x}"),
            value_class: None,
            value_library_uri: None,
            synthesized_slot: true,
            declared_type: None,
        },
    ))
}

/// Names a closure `Context` slot: its parent context or a captured
/// variable by index. `runtime_offsets_extracted.h` (AOT, product) puts the
/// parent at 0x8 and the variables at 0xc on ARM32, and at 0xc / 0x10 with
/// the compressed pointers of the Android 64-bit targets.
fn context_slot(abi: Abi, displacement: i64) -> Option<(i64, RecoveredFieldIdentity)> {
    let (parent, variables) = match abi {
        Abi::ArmeabiV7a => (0x8i64, 0xci64),
        Abi::Arm64V8a | Abi::X86_64 => (0xc, 0x10),
    };
    let offset = displacement.checked_add(1)?;
    let name = if offset == parent {
        "parent".to_owned()
    } else if offset >= variables && (offset - variables) % 4 == 0 {
        format!("captured{}", (offset - variables) / 4)
    } else {
        return None;
    };
    Some((
        offset,
        RecoveredFieldIdentity {
            name,
            value_class: (offset == parent).then(|| "Context".to_owned()),
            value_library_uri: (offset == parent).then(|| "dart:core".to_owned()),
            synthesized_slot: true,
            declared_type: None,
        },
    ))
}

/// Names the `Closure` slots a closure body reads: its Function and its
/// captured Context (`AOT_Closure_function_offset` / `_context_offset`:
/// 0x10 / 0x14 on ARM32, 0x14 / 0x18 with compressed pointers).
fn closure_slot(abi: Abi, displacement: i64) -> Option<(i64, RecoveredFieldIdentity)> {
    let (function, context) = match abi {
        Abi::ArmeabiV7a => (0x10i64, 0x14i64),
        Abi::Arm64V8a | Abi::X86_64 => (0x14, 0x18),
    };
    let offset = displacement.checked_add(1)?;
    let (name, value_class) = if offset == context {
        ("context", Some("Context".to_owned()))
    } else if offset == function {
        ("function", None)
    } else {
        return None;
    };
    Some((
        offset,
        RecoveredFieldIdentity {
            name: name.to_owned(),
            value_library_uri: value_class.as_ref().map(|_| "dart:core".to_owned()),
            value_class,
            synthesized_slot: true,
            declared_type: None,
        },
    ))
}

fn recovered_field<'a>(
    field_layout: Option<&'a RecoveredFieldLayout>,
    receiver: &Expression,
    displacement: i64,
) -> Option<(i64, &'a RecoveredFieldIdentity)> {
    let layout = field_layout?;
    layout.field(
        receiver.class_name.as_deref()?,
        receiver.class_library_uri.as_deref(),
        displacement,
    )
}

fn field_expression(receiver: &str, field: &str) -> String {
    format!("{receiver}.{field}")
}

/// The declared result class of a call resolved to `target`'s label.
fn resolved_result_class(
    table: Option<&DispatchTableAnalysis<'_>>,
    target: &str,
) -> (Option<String>, Option<String>) {
    match table
        .and_then(|table| table.label_results)
        .and_then(|results| results.get(target))
    {
        Some((class, library)) => (Some(class.clone()), library.clone()),
        None => (result_class_from_target(target), None),
    }
}

fn result_class_from_target(target: &str) -> Option<String> {
    if target.starts_with("_iso_stub_") || target.starts_with("stub_") {
        return None;
    }
    let target = target
        .split_once(".dart.")
        .map_or(target, |(_, suffix)| suffix)
        .trim_end_matches('.');
    let parts = target.split('.').collect::<Vec<_>>();
    let class = match parts.as_slice() {
        [class] if looks_like_class_name(class) => *class,
        [class, constructor, ..] if class == constructor && looks_like_class_name(class) => *class,
        [_, class, constructor, ..] if class == constructor && looks_like_class_name(class) => {
            *class
        }
        _ => return None,
    };
    Some(class.to_owned())
}

fn looks_like_class_name(value: &str) -> bool {
    value
        .trim_start_matches('_')
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase() || character == '$')
}

/// Separates the operands of a pending `cmp` until a branch or conditional
/// select picks the operator. A control character never occurs in lifted
/// text (string labels are JSON-escaped), so compared string constants
/// cannot move the split.
const COMPARISON_SEPARATOR: &str = " \u{1f} ";

fn pending_comparison(left: &str, right: &str) -> String {
    format!("{left}{COMPARISON_SEPARATOR}{right}")
}

/// `left operator right`, parenthesizing operands that are themselves
/// comparisons or logical expressions: Dart does not chain `==`.
fn comparison_text(left: &str, operator: &str, right: &str) -> String {
    let operand = |text: &str| {
        let binary = [" == ", " != ", " < ", " > ", " <= ", " >= ", " && ", " || ", " is "]
            .iter()
            .any(|operator| text.contains(operator));
        if binary && !is_wrapped_in_parentheses(text) {
            format!("({text})")
        } else {
            text.to_owned()
        }
    };
    format!("{} {operator} {}", operand(left), operand(right))
}

/// Whether one pair of parentheses encloses all of `text`.
fn is_wrapped_in_parentheses(text: &str) -> bool {
    let Some(inner) = text.strip_prefix('(').and_then(|rest| rest.strip_suffix(')')) else {
        return false;
    };
    let mut depth = 0i32;
    for character in inner.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

/// Maps an ARM condition code to the comparison operator it selects.
fn condition_code_operator(code: &str) -> Option<&'static str> {
    match code {
        "eq" | "z" => Some("=="),
        "ne" | "nz" => Some("!="),
        "lt" | "lo" | "cc" | "mi" => Some("<"),
        "le" | "ls" => Some("<="),
        "gt" | "hi" => Some(">"),
        "ge" | "hs" | "cs" | "pl" => Some(">="),
        _ => None,
    }
}

fn invert_condition_code(code: &str) -> Option<String> {
    let inverted = match code {
        "eq" => "ne",
        "ne" => "eq",
        "lt" => "ge",
        "ge" => "lt",
        "le" => "gt",
        "gt" => "le",
        "lo" => "hs",
        "hs" => "lo",
        "ls" => "hi",
        "hi" => "ls",
        "cc" => "cs",
        "cs" => "cc",
        "mi" => "pl",
        "pl" => "mi",
        _ => return None,
    };
    Some(inverted.to_owned())
}

/// Renders a pending `cmp` comparison for a condition-code consumer such as
/// `cset`/`csel`.
fn comparison_from_condition_code(code: &str, comparison: &Option<Expression>) -> Option<String> {
    let operator = condition_code_operator(code.trim().trim_start_matches("al"))?;
    let comparison = comparison.as_ref()?;
    let (left, right) = comparison.text.split_once(COMPARISON_SEPARATOR)?;
    Some(comparison_text(left, operator, right))
}

fn branch_condition(
    mnemonic: &str,
    operands: &[String],
    registers: &BTreeMap<String, Expression>,
    comparison: Option<&Expression>,
    object_pool: Option<&[String]>,
) -> Option<Expression> {
    if matches!(mnemonic, "cbz" | "cbnz") {
        // Like `cmp`, an untracked register keeps its machine name so the
        // branch diamond, and the values merged after it, stay structured.
        let operand = operands.first()?;
        let value = resolve_expression(operand, registers, object_pool).or_else(|| {
            is_register_spelling(operand).then(|| Expression {
                text: normalize_register(operand),
                confidence: EvidenceConfidence::Low,
                complexity: 1,
                class_name: None,
                class_library_uri: None,
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            })
        })?;
        let operator = if mnemonic == "cbz" { "==" } else { "!=" };
        return Some(Expression {
            text: format!("{} {operator} 0", value.text),
            confidence: value.confidence,
            complexity: value.complexity.saturating_add(1),
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        });
    }
    if matches!(mnemonic, "tbz" | "tbnz") {
        let value = resolve_expression(operands.first()?, registers, object_pool)?;
        let bit = operands.get(1).and_then(|value| immediate_text(value))?;
        // Dart's EmitBoolTest discriminates canonical booleans by the
        // object-alignment bit (pointer_tagging.h kBoolValueBitPosition:
        // `true` sits at null+0x20, `false` at null+0x30). A `tbz` takes the
        // branch when the value IS `true`; a `tbnz` when it is NOT true.
        // Rendering that machine fact keeps recovered predicates readable and
        // correctly polarized instead of an opaque bit test.
        if bit == "4" {
            let text = if mnemonic == "tbz" {
                value.text.clone()
            } else {
                format!("!({})", value.text)
            };
            return Some(Expression {
                text,
                confidence: EvidenceConfidence::Medium,
                complexity: value.complexity.saturating_add(1),
                class_name: Some("bool".to_owned()),
                class_library_uri: Some("dart:core".to_owned()),
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            });
        }
        let operator = if mnemonic == "tbz" { "==" } else { "!=" };
        return Some(Expression {
            text: format!("({} & (1 << {bit})) {operator} 0", value.text),
            confidence: value.confidence,
            complexity: value.complexity.saturating_add(2),
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        });
    }
    let comparison = comparison?;
    let (left, right) = comparison.text.split_once(COMPARISON_SEPARATOR)?;
    // ARM spells conditional branches `b<cc>` / `b.<cc>`; x86 spells them
    // `j<cc>`. Normalize both to the bare condition code.
    let condition = mnemonic
        .trim_start_matches("b.")
        .trim_start_matches('b')
        .trim_start_matches('j')
        .trim_end_matches(".w");
    // EmitBoolTest lowers to a masked test followed by a flags branch:
    // equality means the value IS `true`, inequality NOT. The pending
    // comparison text is `(<value> & mask) ? 0`; the mask is the
    // object-alignment bit (16 on 64-bit targets, 8 on ARM32).
    if matches!(condition, "e" | "z" | "ne" | "nz")
        && let Some((inner, mask)) = left
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
            .and_then(|rest| rest.rsplit_once(" & "))
        && matches!(immediate_text(mask).as_deref(), Some("16") | Some("8"))
    {
        let text = if matches!(condition, "e" | "z") {
            inner.to_owned()
        } else {
            format!("!({inner})")
        };
        return Some(Expression {
            text,
            confidence: EvidenceConfidence::Medium,
            complexity: comparison.complexity.saturating_add(1),
            class_name: Some("bool".to_owned()),
            class_library_uri: Some("dart:core".to_owned()),
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        });
    }
    let operator = match condition {
        "eq" | "z" | "e" => "==",
        "ne" | "nz" => "!=",
        "lt" | "lo" | "cc" | "l" | "b" | "s" => "<",
        "le" | "ls" | "be" => "<=",
        "gt" | "hi" | "a" => ">",
        "ge" | "hs" | "cs" | "ae" | "ns" => ">=",
        "mi" => "<",
        "pl" => ">=",
        _ => return None,
    };
    Some(Expression {
        text: comparison_text(left, operator, right),
        confidence: comparison.confidence,
        complexity: comparison.complexity.saturating_add(1),
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    })
}

fn parse_memory_offset(operands: &str) -> Option<u64> {
    let marker = operands
        .find("#0x")
        .map(|index| (index + 3, 16))
        .or_else(|| operands.find("+ 0x").map(|index| (index + 4, 16)))
        .or_else(|| operands.find(", #").map(|index| (index + 3, 10)))?;
    let value = operands[marker.0..]
        .chars()
        .take_while(|character| character.is_ascii_hexdigit())
        .collect::<String>();
    u64::from_str_radix(&value, marker.1).ok()
}

/// Register `NativeCallInstr` loads the native function entry into
/// (`LoadNativeEntry`): R9 on ARM32, R5 on ARM64, RBX on x64.
fn native_entry_register(abi: Abi) -> &'static str {
    match abi {
        Abi::ArmeabiV7a => "r9",
        Abi::Arm64V8a => "x5",
        Abi::X86_64 => "rbx",
    }
}

/// The Dart index an access at `displacement` through an element pointer
/// reads or writes.
fn element_access_index(
    abi: Abi,
    (_, index): &(Expression, Expression),
    displacement: i64,
) -> Option<String> {
    let data = match abi {
        Abi::ArmeabiV7a => 0xb,
        Abi::Arm64V8a | Abi::X86_64 => 0xf,
    };
    let relative = displacement.checked_sub(data)?;
    if relative % 4 != 0 || !(-64..=256).contains(&relative) {
        return None;
    }
    Some(match relative / 4 {
        0 => index.text.clone(),
        // Brackets delimit the index, so the sum needs no parentheses.
        offset if offset > 0 => format!("{} + {offset}", index.text),
        offset => format!("{} - {}", index.text, -offset),
    })
}

/// The Dart index a Smi-tagged index register holds. Texts denote Dart
/// values, except where the lifter kept the tagging itself: `(i << 1)` is
/// index `i`, `((i << 1) + 8)` is `i + 4`.
fn untagged_index(index: Expression) -> Expression {
    let text = &index.text;
    let untagged = if let Some(inner) = text
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(" << 1)"))
        .filter(|inner| !inner.contains(' ') || is_wrapped_in_parentheses(inner))
    {
        Some(inner.to_owned())
    } else if let Some(rest) = text.strip_prefix("((")
        && let Some((tagged, tail)) = rest.split_once(" << 1) ")
        && let Some((operator, constant)) = tail.strip_suffix(')').and_then(|tail| tail.split_once(' '))
        && matches!(operator, "+" | "-")
        && let Ok(constant) = constant.parse::<i64>()
        && constant % 2 == 0
        && (!tagged.contains(' ') || is_wrapped_in_parentheses(tagged))
    {
        Some(format!("({tagged} {operator} {})", constant / 2))
    } else {
        None
    };
    match untagged {
        Some(text) => Expression { text, ..index },
        None => index,
    }
}

/// A store through an element pointer, as an `[index]` field write.
fn element_write(
    abi: Abi,
    pointer: &(Expression, Expression),
    displacement: i64,
    value: &Expression,
    address: u64,
) -> Option<SemanticStatement> {
    let index = element_access_index(abi, pointer, displacement)?;
    Some(SemanticStatement::FieldWrite {
        receiver: pointer.0.text.clone(),
        field: format!("[{index}]"),
        offset: displacement.saturating_add(1),
        value: value.text.clone(),
        confidence: weaker(weaker(pointer.0.confidence, pointer.1.confidence), value.confidence),
        address: format!("0x{address:x}"),
    })
}

/// The element pointer `base` names, including one the current instruction
/// displaced by overwriting its register.
fn element_pointer<'a>(
    element_pointers: &'a BTreeMap<String, (Expression, Expression)>,
    displaced: &'a Option<(String, (Expression, Expression))>,
    base: &str,
) -> Option<&'a (Expression, Expression)> {
    element_pointers.get(base).or_else(|| {
        displaced
            .as_ref()
            .filter(|(register, _)| register == base)
            .map(|(_, pointer)| pointer)
    })
}

/// `array[index]` for a load through an element pointer. The Array data
/// starts at `AOT_Array_data_offset` (0x10 with compressed pointers, 0xc on
/// ARM32; one less tagged); another 4-byte-aligned displacement reads a
/// constant offset from `index`, as `list[i - 1]` compiles.
fn element_read(
    abi: Abi,
    pointer: &(Expression, Expression),
    displacement: i64,
) -> Option<Expression> {
    let index_text = element_access_index(abi, pointer, displacement)?;
    let (array, index) = pointer;
    Some(Expression {
        text: format!("{}[{index_text}]", array.text),
        confidence: weaker(array.confidence, index.confidence),
        complexity: array.complexity.saturating_add(index.complexity).saturating_add(1),
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    })
}

/// `(base, index, scale, displacement)` of an x64 `[base + index*scale +
/// displacement]` operand.
fn x64_scaled_index_address(operand: &str) -> Option<(String, String, i64, i64)> {
    let start = operand.find('[')?;
    let end = operand[start..].find(']')? + start;
    let terms = operand[start + 1..end]
        .split('+')
        .map(str::trim)
        .collect::<Vec<_>>();
    let [base, scaled, displacement] = terms.as_slice() else {
        return None;
    };
    let (index, scale) = scaled.split_once('*')?;
    Some((
        normalize_register(base),
        normalize_register(index),
        scale.trim().parse().ok()?,
        signed_immediate(displacement)?,
    ))
}

/// Tagged displacement of `Code::entry_point_`, the first word after the
/// object header (`AOT_Code_entry_point_offset`: 0x4 on ARM32, 0x8 on the
/// 64-bit targets, whose Code objects keep full-width fields).
fn code_entry_point_displacement(abi: Abi) -> i64 {
    match abi {
        Abi::ArmeabiV7a => 3,
        Abi::Arm64V8a | Abi::X86_64 => 7,
    }
}

/// Pool labels that denote a Code object: a named VM or object-store stub,
/// or an entry the loader resets to a stub (native-call and switchable-call
/// trampolines).
fn is_pool_code_label(value: &str) -> bool {
    value.starts_with("stub ") || value.starts_with("resetPoolEntry(")
}

/// The value text of a pool label. The ` nestedStrings[...]` suffix lists
/// strings reachable from a constant for the source-literal evidence; it is
/// data from the snapshot and must never become part of an expression.
fn pool_value_text(label: &str) -> String {
    label
        .split_once(" nestedStrings[")
        .map_or(label, |(value, _)| value)
        .to_owned()
}

fn is_named_pool_target(value: &str) -> bool {
    !value.starts_with("snapshotRef(")
        && !value.starts_with("snapshotClass(")
        && !value.starts_with("snapshotType(")
        && !value.starts_with("snapshotField(")
        && !value.starts_with("snapshotInstance(")
        && !value.starts_with("snapshotLibrary(")
        && !value.starts_with("nativePoolEntry(")
        && !value.starts_with("resetPoolEntry(")
        && !value.starts_with("dynamicCall(")
        && !value.starts_with('"')
        && value.parse::<i64>().is_err()
}

/// Registers of an ARM `{r0, r1}` list, in ascending store order. Ranges
/// (`{r0-r3}`) are left undecoded.
fn register_list(list: &str) -> Option<Vec<String>> {
    let inner = list.trim().strip_prefix('{')?.strip_suffix('}')?;
    let registers = inner
        .split(',')
        .map(|register| register.trim().to_owned())
        .collect::<Vec<_>>();
    registers
        .iter()
        .all(|register| !register.is_empty() && !register.contains('-'))
        .then_some(registers)
}

fn writes_first_operand(mnemonic: &str) -> bool {
    // ARM32 conditional compares (`cmpeq`, `tstne`) only set flags.
    let compare_with_condition = mnemonic.len() == 5
        && matches!(&mnemonic[..3], "cmp" | "cmn" | "tst" | "teq")
        && condition_code_operator(&mnemonic[3..]).is_some();
    !compare_with_condition
        && !matches!(
            mnemonic,
            "cmp"
                | "cmn"
                | "tst"
                | "teq"
                | "str"
                | "stur"
                | "stp"
                | "strb"
                | "strh"
                | "strd"
                | "sturb"
                | "sturh"
                | "stlr"
                | "stlrb"
                | "stlrh"
                | "vstr"
                // Store-multiple writes memory; its base changes only with
                // `!` writeback, which frame tracking handles separately.
                | "stm"
                | "stmia"
                | "stmib"
                | "stmda"
                | "stmdb"
                | "vstmia"
                | "vstmdb"
                | "b"
                | "bl"
                | "blr"
                | "ret"
                | "cbz"
                | "cbnz"
                | "tbz"
                | "tbnz"
        )
        && branch_kind(mnemonic).is_none()
}

fn is_skipped_data(mnemonic: &str) -> bool {
    mnemonic.starts_with('.')
}

fn direct_call_target(mnemonic: &str, operands: &str) -> Option<u64> {
    if !is_call(mnemonic) {
        return None;
    }
    parse_immediate(operands.split(',').next()?)
}

fn is_call(mnemonic: &str) -> bool {
    matches!(mnemonic, "bl" | "blx" | "blr" | "call" | "callq")
}

/// Register holding the receiver's class id at a dispatch-table call
/// (`DispatchTableNullErrorABI::kClassIdReg`).
fn dispatch_class_id_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x0",
        Abi::ArmeabiV7a => "r0",
        Abi::X86_64 => "rcx",
    }
}

/// Register a switchable (IC) call loads its receiver into before calling
/// through the pool (`EmitInstanceCallAOT`).
fn switchable_receiver_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x0",
        Abi::ArmeabiV7a => "r0",
        Abi::X86_64 => "rdx",
    }
}

/// Follows `LoadClassId`: a header-word load `[receiver - kHeapObjectTag]`
/// followed by extracting the 20-bit class-id field at bit 12 (`ubfx` on
/// ARM, `shr` of the 32-bit header on x64). Register copies carry both
/// facts; any other write kills them.
fn track_class_ids(
    abi: Abi,
    mnemonic: &str,
    operands: &[String],
    registers: &BTreeMap<String, Expression>,
    tag_words: &mut BTreeMap<String, Expression>,
    class_ids: &mut BTreeMap<String, Expression>,
) {
    if is_call(mnemonic) {
        tag_words.clear();
        class_ids.clear();
        return;
    }
    let Some(destination) = operands
        .first()
        .filter(|_| writes_first_operand(mnemonic))
        .map(|value| normalize_register(value))
    else {
        return;
    };
    let source = |index: usize| operands.get(index).map(|value| normalize_register(value));
    let immediate = |index: usize| {
        operands
            .get(index)
            .and_then(|value| signed_immediate(value))
    };
    // ARM32 `LoadTaggedClassIdMayBeSmi` predicates the header load and
    // bitfield extract on a heap object and moves the Smi class id on the
    // other path: `tst r0, #1; ldrne r1, [r0, #-1]; ubfxne r1, r1, #12, #20;
    // moveq r1, #cid`. Both paths yield the same receiver's class id.
    let predicated = abi == Abi::ArmeabiV7a && arm32_condition_suffix(mnemonic);
    let mnemonic = if predicated {
        &mnemonic[..mnemonic.len() - 2]
    } else {
        mnemonic
    };
    if predicated
        && mnemonic == "mov"
        && immediate(1).is_some()
        && class_ids.contains_key(&destination)
    {
        return;
    }
    let mut tag = None;
    let mut class_id = None;
    match mnemonic {
        "ldur" | "ldr" | "mov" | "movl" if operands.len() == 2 && operands[1].contains('[') => {
            if let Some((base, -1)) = arm_memory_address(&operands[1]) {
                tag = registers.get(&base).cloned();
            }
        }
        "ubfx" if operands.len() == 4 && immediate(2) == Some(12) && immediate(3) == Some(20) => {
            class_id = source(1).and_then(|register| tag_words.get(&register).cloned());
        }
        "shr" | "shrl" if abi == Abi::X86_64 && operands.len() == 2 && immediate(1) == Some(12) => {
            class_id = tag_words.get(&destination).cloned();
        }
        "mov" | "movq" if operands.len() == 2 && !operands[1].contains('[') => {
            if let Some(register) = source(1) {
                tag = tag_words.get(&register).cloned();
                class_id = class_ids.get(&register).cloned();
            }
        }
        _ => {}
    }
    tag_words.remove(&destination);
    class_ids.remove(&destination);
    if let Some(tag) = tag {
        tag_words.insert(destination.clone(), tag);
    }
    if let Some(class_id) = class_id {
        class_ids.insert(destination, class_id);
    }
}

/// The single concrete class id of an exactly-typed receiver.
fn exact_receiver_class_id(
    table: &DispatchTableAnalysis<'_>,
    receiver: &Expression,
) -> Option<usize> {
    if !receiver.exact_class {
        return None;
    }
    receiver_class_id(table, receiver)
}

/// The unique class id named by a value's (exact or static) class.
fn receiver_class_id(table: &DispatchTableAnalysis<'_>, receiver: &Expression) -> Option<usize> {
    let class_name = receiver
        .class_name
        .as_deref()
        .filter(|name| !name.is_empty())?;
    // A receiver's library can be spelled differently from the class's own
    // (a `dart:core` type name against an obfuscated library token), so a
    // qualified miss falls back to the name alone.
    let qualified = table.qualified_to_cids.and_then(|qualified| {
        qualified.get(&(
            Some(receiver.class_library_uri.clone()?),
            class_name.to_owned(),
        ))
    });
    let cids =
        qualified.or_else(|| table.name_to_cids.and_then(|names| names.get(class_name)))?;
    // Name reuse across libraries (or obfuscation) leaves several classes.
    match cids.as_slice() {
        [cid] => Some(*cid),
        _ => None,
    }
}

/// Resolves a dispatch-table call to one implementation.
///
/// The receiver is the value whose class id the call indexed with, not
/// any class-typed value that happens to be live. An exact class consults
/// only its own table slot `selector_offset + cid`. A static type admits
/// every concrete subtype (subclasses, implementers and mixin users), so
/// each of their slots must hold an implementation the subtype inherits and
/// all of them must agree: rows are packed with holes, so a slot owned by an
/// unrelated class is not selector membership and defeats the proof.
fn resolve_dispatch_via_receiver(
    table: &DispatchTableAnalysis<'_>,
    selector_offset: usize,
    receiver: Option<&Expression>,
) -> Option<String> {
    let receiver = receiver?;
    if let Some(cid) = exact_receiver_class_id(table, receiver) {
        return table
            .targets
            .get(selector_offset.checked_add(cid)?)?
            .clone()
            .filter(|label| !label.is_empty());
    }
    let static_cid = receiver_class_id(table, receiver)?;
    let subtypes = table.subtype_cids?.get(&static_cid)?;
    let mut implementation: Option<&String> = None;
    for &class_id in subtypes {
        let index = selector_offset.checked_add(class_id)?;
        let target = table.targets.get(index)?.as_ref()?;
        if target.is_empty() || table.slot_fits_receiver(index, class_id) == Some(false) {
            return None;
        }
        match implementation {
            None => implementation = Some(target),
            Some(existing) if existing == target => {}
            Some(_) => return None,
        }
    }
    implementation.cloned()
}

/// The selector of the `dynamicCall(...)` ICData pool label a register holds.
/// Only a label that *is* the ICData entry counts: a string constant whose
/// text merely contains `dynamicCall(` must not forge a selector.
fn find_ic_selector(registers: &BTreeMap<String, Expression>) -> Option<String> {
    registers.values().find_map(|expression| {
        // The selector is JSON-encoded (`unlinked_call_label`).
        let encoded = expression.text.strip_prefix("dynamicCall(")?;
        let selector = serde_json::Deserializer::from_str(encoded)
            .into_iter::<String>()
            .next()?
            .ok()?;
        (!selector.is_empty()).then_some(selector)
    })
}

/// Resolves a switchable call to the implementation its receiver's exact
/// class declares for `selector`. Only the receiver the call itself loaded
/// is considered, its class must be exact and the only class of that name,
/// and the class must declare the member itself: an inherited
/// implementation would need the class hierarchy to prove.
fn resolve_ic_target_via_receiver(
    selector: &str,
    receiver: Option<&Expression>,
    symbols: &BTreeMap<u64, Symbol>,
    table: Option<&DispatchTableAnalysis<'_>>,
) -> Option<String> {
    let receiver = receiver?;
    let table = table?;
    exact_receiver_class_id(table, receiver)?;
    let class_name = receiver.class_name.as_deref()?;
    if table
        .name_to_cids
        .and_then(|names| names.get(class_name))
        .is_none_or(|cids| cids.len() != 1)
    {
        return None;
    }
    let expected = format!("{class_name}.{selector}");
    let mut matches = symbols
        .values()
        .filter(|symbol| symbol.semantic_name && symbol.label == expected)
        .filter(|symbol| {
            receiver.class_library_uri.is_none()
                || symbol.library_uri.is_none()
                || symbol.library_uri == receiver.class_library_uri
        })
        .map(|symbol| symbol.label.clone())
        .collect::<BTreeSet<_>>();
    (matches.len() == 1).then(|| matches.pop_first()).flatten()
}

/// Types an untyped value by the class evidence proved for its text.
fn apply_type_hint(value: &mut Expression, hints: &BTreeMap<String, (String, Option<String>)>) {
    if value
        .class_name
        .as_deref()
        .is_some_and(|class| class != "Object")
        || value.high_word
    {
        return;
    }
    if let Some((class, library)) = hints.get(&value.text) {
        value.class_name = Some(class.clone());
        value.class_library_uri = library.clone();
    }
}

/// The receiver a direct call proves the class of, with that class:
/// an instance member's receiver (its first parameter), or the instance a
/// late field's init stub initializes (`InitInstanceFieldABI::kInstanceReg`
/// is R1/X1/RBX; the Field's pool label is `Owner.name`).
fn proven_call_receiver<'r>(
    abi: Abi,
    target: &str,
    symbol: Option<&Symbol>,
    registers: &'r BTreeMap<String, Expression>,
    object_pool: Option<&[String]>,
    pool_registers: &BTreeMap<String, usize>,
) -> Option<(&'r Expression, String, Option<String>)> {
    let [instance, field] = init_field_registers(abi);
    if matches!(
        target,
        "stub InitInstanceField" | "stub InitLateInstanceField" | "stub InitLateFinalInstanceField"
    ) {
        let label = pool_registers
            .get(field)
            .and_then(|index| object_pool?.get(*index))?;
        let (owner, name) = label.split_once('.')?;
        if owner.is_empty()
            || name.is_empty()
            || name.contains('.')
            || !looks_like_identifier(owner)
        {
            return None;
        }
        return Some((registers.get(instance)?, owner.to_owned(), None));
    }
    let symbol = symbol?;
    let (class, library) = symbol.receiver_class.clone()?;
    let register = match symbol.parameters.as_deref()?.first()? {
        ArgumentLocation::Register(register) => *register,
        _ => return None,
    };
    Some((registers.get(register)?, class, library))
}

fn looks_like_identifier(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '$')
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        })
}

/// The context a closure allocated in this body captured, read back
/// through the closure (`UntaggedClosure::context_`: the fifth field after
/// the header, at 0x14 on ARM32 and 0x18 with compressed pointers).
fn closure_context_read(abi: Abi, closure: &Expression, displacement: i64) -> Option<Expression> {
    let context_offset = match abi {
        Abi::ArmeabiV7a => 0x14,
        Abi::Arm64V8a | Abi::X86_64 => 0x18,
    };
    if displacement.checked_add(1)? != context_offset {
        return None;
    }
    let (_, context) = closure_allocation_parts(&closure.text)?;
    Some(Expression {
        text: context.unwrap_or("null").to_owned(),
        confidence: closure.confidence,
        complexity: 1,
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    })
}

/// `(function, context)` of an `aot.closure('<function>'[, context])`
/// value, with the function label unescaped.
fn closure_allocation_parts(text: &str) -> Option<(String, Option<&str>)> {
    let rest = text.strip_prefix("aot.closure('")?.strip_suffix(')')?;
    let mut function = String::new();
    let mut characters = rest.char_indices();
    loop {
        let (index, character) = characters.next()?;
        match character {
            '\\' => function.push(characters.next()?.1),
            '\'' => {
                let tail = &rest[index + 1..];
                return match tail.strip_prefix(", ") {
                    Some(context) if !context.is_empty() => Some((function, Some(context))),
                    _ if tail.is_empty() => Some((function, None)),
                    _ => None,
                };
            }
            character => function.push(character),
        }
    }
}

/// A value produced by an out-of-line box allocation stub whose payload
/// store has not been seen yet.
const UNFILLED_BOX: &str = "aot.unresolvedValue('unfilled box')";

/// What a call to a VM stub means at the Dart level, when the stub's
/// register ABI (`ThrowABI`, `ReThrowABI`, `AllocateClosureABI`,
/// `AllocateBoxABI`; identical across the supported releases) carries it.
enum StubSemantics {
    Throw {
        exception: Expression,
        stack_trace: Option<Expression>,
    },
    /// The stub's result, left in the return register. Box allocation
    /// stubs preserve every other register (the compiler's slow path saves
    /// the live ones around them).
    Value {
        value: Expression,
        preserves_registers: bool,
    },
}

fn runtime_stub_semantics(
    abi: Abi,
    stub: &str,
    registers: &BTreeMap<String, Expression>,
    pool_registers: &BTreeMap<String, usize>,
    object_pool: Option<&[String]>,
) -> Option<StubSemantics> {
    let (first, second) = match abi {
        Abi::Arm64V8a => ("x1", "x2"),
        Abi::ArmeabiV7a => ("r1", "r2"),
        Abi::X86_64 => ("rbx", "rdx"),
    };
    let value_of = |register: &str| {
        registers
            .get(register)
            .cloned()
            .unwrap_or_else(|| Expression {
                text: register.to_owned(),
                confidence: EvidenceConfidence::Low,
                complexity: 1,
                class_name: None,
                class_library_uri: None,
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            })
    };
    let plain = |text: String, confidence: EvidenceConfidence| Expression {
        text,
        confidence,
        complexity: 1,
        class_name: None,
        class_library_uri: None,
        raw: false,
        definition_site: None,
        high_word: false,
        exact_class: false,
    };
    match stub {
        "Throw" => Some(StubSemantics::Throw {
            exception: value_of(abi_return_register(abi)),
            stack_trace: None,
        }),
        "ReThrow" => Some(StubSemantics::Throw {
            exception: value_of(abi_return_register(abi)),
            // ReThrowABI::kStackTraceReg is R1/X1 on ARM and RBX on x64.
            stack_trace: Some(value_of(first)),
        }),
        "AllocateClosure"
        | "AllocateClosureGeneric"
        | "AllocateClosureTa"
        | "AllocateClosureTaGeneric" => {
            // AllocateClosureABI: kFunctionReg = R1/X1/RBX, kContextReg =
            // R2/X2/RDX. Only a pool-loaded Function names the closure.
            let function = pool_registers
                .get(first)
                .and_then(|index| object_pool?.get(*index))
                .filter(|label| !label.is_empty())?;
            let context = registers
                .get(second)
                .filter(|context| context.text != "null" && !context.high_word);
            let quoted = function.replace('\\', "\\\\").replace('\'', "\\'");
            let text = match context {
                Some(context) => format!("aot.closure('{quoted}', {})", context.text),
                None => format!("aot.closure('{quoted}')"),
            };
            Some(StubSemantics::Value {
                value: Expression {
                    class_name: Some("Function".to_owned()),
                    class_library_uri: Some("dart:core".to_owned()),
                    ..plain(text, EvidenceConfidence::Medium)
                },
                preserves_registers: false,
            })
        }
        stub if stub.starts_with("AllocateMint")
            || matches!(
                stub,
                "AllocateDouble" | "AllocateFloat32x4" | "AllocateFloat64x2" | "AllocateInt32x4"
            ) =>
        {
            Some(StubSemantics::Value {
                value: plain(UNFILLED_BOX.to_owned(), EvidenceConfidence::Low),
                preserves_registers: true,
            })
        }
        _ => None,
    }
}

/// Stores the payload of a box whose allocation stub result `base` holds:
/// the box is the stored value itself. Also swallows the high-word store
/// of an ARM32 Mint. Returns whether the store was consumed.
fn fill_pending_box(
    registers: &mut BTreeMap<String, Expression>,
    base: &str,
    displacement: i64,
    value: Option<&Expression>,
) -> bool {
    let Some(pending) = registers
        .get(base)
        .filter(|pending| pending.text == UNFILLED_BOX)
    else {
        return false;
    };
    // The payload starts at offset 8 on every target (tagged address - 1).
    if displacement != 7 {
        return false;
    }
    let definition_site = pending.definition_site;
    match value {
        Some(value) => {
            registers.insert(
                base.to_owned(),
                Expression {
                    raw: false,
                    high_word: false,
                    definition_site,
                    ..value.clone()
                },
            );
        }
        None => {
            registers.remove(base);
        }
    }
    true
}

/// `(instance, Field)` registers of the field initialization stubs: R1/R2
/// on ARM, RBX/RDX on x64.
fn init_field_registers(abi: Abi) -> [&'static str; 2] {
    match abi {
        Abi::Arm64V8a => ["x1", "x2"],
        Abi::ArmeabiV7a => ["r1", "r2"],
        Abi::X86_64 => ["rbx", "rdx"],
    }
}

/// The field read a late or lazily initialized field's init stub performs.
/// `InitInstanceFieldABI` passes (instance, Field) and `InitStaticFieldABI`
/// passes the Field last; the pool labels a retained Field `Owner.name`.
fn late_field_read(target: &str, arguments: &[String]) -> Option<String> {
    let field = arguments.last()?;
    let field_label = !field.is_empty()
        && field.split('.').count() <= 2
        && field.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '$' | '.')
        })
        && !field.ends_with("_result")
        && !field.starts_with(|character: char| character.is_ascii_digit())
        && !is_machine_value_name(field);
    if !field_label {
        return None;
    }
    match target {
        "stub InitLateFinalInstanceField"
        | "stub InitLateInstanceField"
        | "stub InitInstanceField" => {
            let [instance, _] = arguments else {
                return None;
            };
            let name = field.rsplit('.').next()?;
            Some(format!("{instance}.{name}"))
        }
        "stub InitLateFinalStaticField"
        | "stub InitLateStaticField"
        | "stub InitStaticField"
        | "stub InitSharedLateFinalStaticField"
        | "stub InitSharedLateStaticField"
        | "stub InitSharedStaticField" => Some(field.clone()),
        _ => None,
    }
}

/// Lifter-internal value names that are not program identifiers.
fn is_machine_value_name(value: &str) -> bool {
    let register = value.len() >= 2
        && matches!(value.as_bytes()[0], b'r' | b'x' | b'w' | b'd' | b's' | b'v')
        && value[1..].chars().all(|character| character.is_ascii_digit());
    register
        || value.starts_with("arg")
        || value.starts_with("phi_")
        || value.starts_with("local")
        || value.starts_with("merged")
        || matches!(value, "this" | "null" | "true" | "false" | "closureContext")
}

/// A breakpoint trap: Dart emits one after every call that cannot return
/// (`Throw`, `ReThrow`, error stubs), so nothing falls through it.
fn is_trap(mnemonic: &str) -> bool {
    matches!(mnemonic, "bkpt" | "brk" | "udf" | "int3" | "ud2")
}

fn is_return(mnemonic: &str, operands: &str) -> bool {
    mnemonic.starts_with("ret")
        || (mnemonic == "bx" && operands.trim() == "lr")
        || ((mnemonic == "pop" || mnemonic == "pop.w") && operands.contains("pc"))
}

fn branch_kind(mnemonic: &str) -> Option<bool> {
    if matches!(mnemonic, "b" | "b.w" | "jmp" | "jmpq") {
        return Some(false);
    }
    let arm_condition = mnemonic.starts_with("b.")
        || (mnemonic.starts_with('b')
            && matches!(
                mnemonic.trim_start_matches('b').trim_end_matches(".w"),
                "eq" | "ne"
                    | "cs"
                    | "hs"
                    | "cc"
                    | "lo"
                    | "mi"
                    | "pl"
                    | "vs"
                    | "vc"
                    | "hi"
                    | "ls"
                    | "ge"
                    | "lt"
                    | "gt"
                    | "le"
            ));
    let test_branch = matches!(mnemonic, "cbz" | "cbnz" | "tbz" | "tbnz");
    let x86_condition = mnemonic.starts_with('j') && !matches!(mnemonic, "jmp" | "jmpq");
    (arm_condition || test_branch || x86_condition).then_some(true)
}

fn branch_target(operands: &str) -> Option<u64> {
    parse_immediate(operands.rsplit(',').next()?)
}

fn parse_immediate(value: &str) -> Option<u64> {
    let value = value
        .trim()
        .trim_start_matches('#')
        .trim_start_matches("0x");
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u64::from_str_radix(value, 16).ok()
}

/// Canonical register spelling (`w4` → `x4`, `r10d` → `r10`) for evidence
/// passes outside the lifter.
pub(crate) fn normalized_register(value: &str) -> String {
    normalize_register(value)
}

/// Public wrapper for address-string parsing outside the disassembler
/// (source-band attribution in the snapshot recovery layer).
pub(crate) fn parse_immediate_public(value: &str) -> Option<u64> {
    parse_immediate(value)
}

fn add_fallthrough_block(
    block_starts: &mut std::collections::BTreeSet<u64>,
    instruction: &capstone::Insn<'_>,
    function_end: u64,
) {
    let next = instruction
        .address()
        .saturating_add(instruction.bytes().len() as u64);
    if next < function_end {
        block_starts.insert(next);
    }
}

pub(crate) fn call_target_scope(
    label: &str,
    library_uri: Option<&str>,
    application_package: Option<&str>,
) -> CallTargetScope {
    if label.starts_with("stub ") || label.starts_with("_iso_stub_") || label.starts_with("stub_") {
        return CallTargetScope::Runtime;
    }
    let Some(uri) = library_uri else {
        return CallTargetScope::Unknown;
    };
    if uri.starts_with("dart:") {
        CallTargetScope::DartSdk
    } else if uri.starts_with("package:flutter/") {
        CallTargetScope::FlutterSdk
    } else if application_package
        .is_some_and(|package| uri.starts_with(&format!("package:{package}/")))
    {
        CallTargetScope::Application
    } else if uri.starts_with("package:") {
        CallTargetScope::Package
    } else {
        CallTargetScope::Unknown
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::ParameterHint;
    use crate::model::{Abi, ControlFlowEdgeKind, PseudoStatement, SemanticStatement};

    use super::{
        DecodedInstruction, Disassembler, DispatchTableAnalysis, Expression,
        MAX_RENDERED_INSTRUCTIONS, RecoveredFieldLayout, Symbol, branch_kind, build_control_flow,
        direct_call_target, infer_dispatch_selector, lift_semantics, lift_semantics_with_names,
        late_field_read, object_pool_index, reachable_block_count, recover_dispatch_calls,
        recover_object_pool_loads,
    };
    use crate::model::EvidenceConfidence;

    fn instruction(address: u64, mnemonic: &str, operands: &str) -> DecodedInstruction {
        DecodedInstruction {
            address,
            next: address + 4,
            mnemonic: mnemonic.to_owned(),
            operands: operands.to_owned(),
        }
    }

    #[test]
    fn bool_test_bit_branches_recover_polarized_conditions() {
        let mut registers = BTreeMap::new();
        registers.insert(
            "x0".to_owned(),
            super::Expression {
                text: "isEmptyResult".to_owned(),
                confidence: EvidenceConfidence::Low,
                complexity: 1,
                class_name: Some("bool".to_owned()),
                class_library_uri: Some("dart:core".to_owned()),
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            },
        );
        // tbz takes the branch when the value IS `true`.
        let taken_true = super::branch_condition(
            "tbz",
            &["x0".to_owned(), "#4".to_owned(), "#0x10".to_owned()],
            &registers,
            None,
            None,
        )
        .expect("bool test should build a condition");
        assert_eq!(taken_true.text, "isEmptyResult");
        // tbnz takes the branch when the value is NOT true.
        let taken_false = super::branch_condition(
            "tbnz",
            &["x0".to_owned(), "#4".to_owned(), "#0x10".to_owned()],
            &registers,
            None,
            None,
        )
        .expect("bool test should build a condition");
        assert_eq!(taken_false.text, "!(isEmptyResult)");
    }

    #[test]
    fn x86_bool_mask_branches_recover_polarized_conditions() {
        let comparison = Some(super::Expression {
            text: super::pending_comparison("(w0 & 16)", "0"),
            confidence: EvidenceConfidence::Low,
            complexity: 2,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        });
        let equal = super::branch_condition(
            "je",
            &[String::new()],
            &BTreeMap::new(),
            comparison.as_ref(),
            None,
        )
        .expect("masked equality is a boolean test");
        assert_eq!(equal.text, "w0");
        let unequal_comparison = Expression {
            text: super::pending_comparison("(w0 & 16)", "0"),
            confidence: EvidenceConfidence::Low,
            complexity: 2,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        };
        let unequal = super::branch_condition(
            "jne",
            &[String::new()],
            &BTreeMap::new(),
            Some(&unequal_comparison),
            None,
        )
        .expect("masked inequality is a boolean test");
        assert_eq!(unequal.text, "!(w0)");
    }

    #[test]
    fn canonical_boolean_constants_map_to_their_null_offsets() {
        // pointer_tagging.h: kTrueOffsetFromNull = +0x20, kFalseOffsetFromNull = +0x30.
        let instructions = [
            instruction(0x1000, "add", "x0, x22, #0x20"),
            instruction(0x1004, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &instructions,
            &BTreeSet::from([0x1000]),
            &BTreeMap::new(),
            None,
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. } if expression == "true"
            )),
            "statements: {statements:?}"
        );
        let instructions = [
            instruction(0x1000, "add", "x0, x22, #0x30"),
            instruction(0x1004, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &instructions,
            &BTreeSet::from([0x1000]),
            &BTreeMap::new(),
            None,
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. } if expression == "false"
            )),
            "statements: {statements:?}"
        );
    }

    #[test]
    fn parses_immediate_call_targets_only() {
        assert_eq!(direct_call_target("bl", "#0x1234"), Some(0x1234));
        assert_eq!(direct_call_target("call", "0x5678"), Some(0x5678));
        assert_eq!(direct_call_target("blr", "x16"), None);
        assert_eq!(branch_kind("b.ls"), Some(true));
        assert_eq!(branch_kind("jmp"), Some(false));
    }

    #[test]
    fn retains_direct_calls_after_instruction_comment_limit() {
        let mut bytes = [0x1f, 0x20, 0x03, 0xd5].repeat(MAX_RENDERED_INSTRUCTIONS + 1);
        bytes.extend([0x00, 0x00, 0x00, 0x94]);
        let statements = Disassembler::new(Abi::Arm64V8a)
            .unwrap()
            .analyze(
                0x1000,
                &bytes,
                &BTreeMap::new(),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        assert!(
            statements
                .statements
                .iter()
                .any(|statement| matches!(statement, PseudoStatement::DirectCall { .. }))
        );
    }

    #[test]
    fn distinguishes_indirect_calls_and_machine_returns() {
        // blr x16; ret
        let bytes = [0x00, 0x02, 0x3f, 0xd6, 0xc0, 0x03, 0x5f, 0xd6];
        let disassembly = Disassembler::new(Abi::Arm64V8a)
            .unwrap()
            .analyze(
                0x1000,
                &bytes,
                &BTreeMap::new(),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        assert_eq!(disassembly.evidence.indirect_calls, 1);
        assert_eq!(disassembly.evidence.direct_calls, 0);
        assert_eq!(disassembly.evidence.returns, 1);
        assert!(
            disassembly
                .statements
                .iter()
                .any(|statement| { matches!(statement, PseudoStatement::IndirectCall { .. }) })
        );
    }

    #[test]
    fn resolves_switchable_calls_through_their_selector_load() {
        // Switchable-call shape: the UnlinkedCall selector rides in scratch
        // register x16 while the stub entry lands in the call register x17.
        // ldr x16,[x27,#16]; ldr x17,[x27,#24]; blr x17
        let bytes = [
            0x70, 0x0b, 0x40, 0xf9, 0x71, 0x0f, 0x40, 0xf9, 0x20, 0x02, 0x3f, 0xd6,
        ];
        let pool = vec![
            "dynamicCall(\"isEmpty\", arity=2)".to_owned(),
            // Unnamed stub Code renders as an opaque reference.
            "snapshotRef(41)".to_owned(),
        ];
        let disassembly = Disassembler::new(Abi::Arm64V8a)
            .unwrap()
            .analyze(
                0x1000,
                &bytes,
                &BTreeMap::new(),
                None,
                Some(&pool),
                None,
                None,
                None,
            )
            .unwrap();

        let selector_call = disassembly
            .statements
            .iter()
            .find_map(|statement| match statement {
                PseudoStatement::ObjectPoolCall {
                    pool_index, target, ..
                } => (*pool_index == 0).then(|| target.clone()),
                _ => None,
            });
        assert_eq!(
            selector_call.as_deref(),
            Some("dynamicCall(\"isEmpty\", arity=2)")
        );
    }

    #[test]
    fn decodes_vfp_fallback_and_continues_to_arm_return() {
        // Capstone reports the first valid VFP immediate as skip-data in this
        // mode. The Dart-derived fallback decodes it and still resumes at the
        // following `bx lr`.
        let bytes = [0x00, 0x0b, 0xb0, 0xee, 0x1e, 0xff, 0x2f, 0xe1];
        let disassembly = Disassembler::new(Abi::ArmeabiV7a)
            .unwrap()
            .analyze(
                0x1000,
                &bytes,
                &BTreeMap::new(),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        assert_eq!(disassembly.evidence.unknown_bytes, 0);
        assert_eq!(disassembly.evidence.decoded_bytes, 8);
        assert_eq!(disassembly.evidence.returns, 1);
    }

    #[test]
    fn builds_conditional_cfg_and_reachability() {
        let instructions = [
            instruction(0x1000, "cbz", "x0, #0x1008"),
            instruction(0x1004, "ret", ""),
            instruction(0x1008, "ret", ""),
        ];
        let blocks = BTreeSet::from([0x1000, 0x1004, 0x1008]);
        let edges = build_control_flow(0x1000, 0x100c, &instructions, &blocks);

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|edge| {
            edge.from == "0x1000"
                && edge.to == "0x1008"
                && edge.kind == ControlFlowEdgeKind::ConditionalTrue
        }));
        assert!(edges.iter().any(|edge| {
            edge.from == "0x1000"
                && edge.to == "0x1004"
                && edge.kind == ControlFlowEdgeKind::ConditionalFalse
        }));
        assert_eq!(reachable_block_count(0x1000, &edges, &blocks), 3);
    }

    #[test]
    fn maps_object_pool_offsets_for_each_abi() {
        assert_eq!(
            object_pool_index(Abi::Arm64V8a, "x16, [x27, #0x20]"),
            Some(2)
        );
        assert_eq!(
            object_pool_index(Abi::ArmeabiV7a, "r3, [r5, #0x13]"),
            Some(3)
        );
        // x64 pool data starts at 16 - 1 (tagged): entries at 15 + 8k.
        assert_eq!(object_pool_index(Abi::X86_64, "rax, [r15 + 0x27]"), Some(3));
        assert_eq!(object_pool_index(Abi::Arm64V8a, "x0, [x29, #0x20]"), None);

        let floating_load = [instruction(
            0x1000,
            "movsd",
            "xmm0, qword ptr [r15 + 0xac7]",
        )];
        assert_eq!(
            recover_object_pool_loads(Abi::X86_64, &floating_load).get(&0x1000),
            Some(&343)
        );
    }

    #[test]
    fn recovers_split_arm32_pool_offsets_and_invalidates_at_branches() {
        let instructions = [
            instruction(0x1000, "add", "r8, r5, #0x21000"),
            instruction(0x1004, "ldr", "r3, [r8, #0x687]"),
            instruction(0x1008, "b", "#0x1010"),
            instruction(0x100c, "ldr", "r4, [r8, #0x68b]"),
            instruction(0x1010, "ldr", "lr, [r5, #0x1a7]"),
        ];
        let loads = recover_object_pool_loads(Abi::ArmeabiV7a, &instructions);

        assert_eq!(loads.get(&0x1004), Some(&34208));
        assert_eq!(loads.get(&0x100c), None);
        assert_eq!(loads.get(&0x1010), Some(&104));
    }

    #[test]
    fn recovers_shifted_arm64_pool_offsets_and_invalidates_at_branches() {
        let instructions = [
            instruction(0x1000, "add", "x1, x27, #0x14, lsl #12"),
            instruction(0x1004, "ldr", "x1, [x1, #0x7c0]"),
            instruction(0x1008, "b", "#0x1010"),
            instruction(0x100c, "ldr", "x2, [x1, #0x10]"),
            instruction(0x1010, "ldr", "x3, [x27, #0x20]"),
        ];
        let loads = recover_object_pool_loads(Abi::Arm64V8a, &instructions);

        assert_eq!(loads.get(&0x1004), Some(&10486));
        assert_eq!(loads.get(&0x100c), None);
        assert_eq!(loads.get(&0x1010), Some(&2));
    }

    #[test]
    fn resolves_named_indirect_pool_calls() {
        let instructions = [
            instruction(0x1000, "ldr", "x16, [x27, #0x20]"),
            instruction(0x1004, "blr", "x16"),
            instruction(0x1008, "ret", ""),
        ];
        let blocks = BTreeSet::from([0x1000]);
        let pool = [
            "zero".to_owned(),
            "one".to_owned(),
            "Widget.build".to_owned(),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &instructions,
            &blocks,
            &BTreeMap::new(),
            Some(&pool),
        );

        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::ResolvedCall { target, .. } if target == "Widget.build"
        )));
    }

    #[test]
    fn assigns_vm_field_names_only_with_receiver_class_proof() {
        let instructions = [
            instruction(0x1000, "ldr", "x2, [x1, #0x7]"),
            instruction(0x1004, "str", "x2, [x1, #0xb]"),
            instruction(0x1008, "ret", ""),
        ];
        let blocks = BTreeSet::from([0x1000]);
        let mut layout = RecoveredFieldLayout::default();
        layout.insert(
            Some("package:app/model.dart".to_owned()),
            "Profile".to_owned(),
            8,
            "name".to_owned(),
            None,
            None,
        );
        layout.insert(
            Some("package:app/model.dart".to_owned()),
            "Profile".to_owned(),
            12,
            "displayName".to_owned(),
            None,
            None,
        );
        let statements = lift_semantics_with_names(
            Abi::Arm64V8a,
            &[ParameterHint {
                name: "this".to_owned(),
                class_name: None,
                class_library_uri: None,
            }],
            &instructions,
            &blocks,
            &BTreeMap::new(),
            None,
            Some(&layout),
            Some(("Profile", Some("package:app/model.dart"))),
            None,
            None,
            None,
        );
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::FieldRead { field, offset: 8, .. } if field == "name"
        )));
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::FieldWrite { field, offset: 12, .. }
                if field == "displayName"
        )));

        let unrelated = lift_semantics_with_names(
            Abi::Arm64V8a,
            &[ParameterHint {
                name: "this".to_owned(),
                class_name: None,
                class_library_uri: None,
            }],
            &instructions,
            &blocks,
            &BTreeMap::new(),
            None,
            Some(&layout),
            Some(("Unrelated", Some("package:app/model.dart"))),
            None,
            None,
            None,
        );
        assert!(!unrelated.iter().any(|statement| matches!(
            statement,
            SemanticStatement::FieldRead { field, .. }
                | SemanticStatement::FieldWrite { field, .. }
                if !field.starts_with("_slot_")
        )));
    }

    #[test]
    fn lifts_arm32_calls_through_derived_pool_pointers() {
        let instructions = [
            instruction(0x1000, "add", "r8, r5, #0x1000"),
            instruction(0x1004, "ldr", "lr, [r8, #0x13]"),
            instruction(0x1008, "blx", "lr"),
            instruction(0x100c, "bx", "lr"),
        ];
        let blocks = BTreeSet::from([0x1000]);
        let mut pool = vec!["snapshotRef(0)".to_owned(); 1028];
        pool[1027] = "TerminalProvider.check".to_owned();
        let statements = lift_semantics(
            Abi::ArmeabiV7a,
            None,
            &instructions,
            &blocks,
            &BTreeMap::new(),
            Some(&pool),
        );

        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::ResolvedCall { target, .. }
                if target == "TerminalProvider.check"
        )));
    }

    #[test]
    fn lifts_arm32_native_calls_with_stacked_arguments_and_result_slot() {
        // `_Double._toString` in Dart 3.9 ARM32: the receiver and a null
        // result slot are stored with `stm`, R9 holds the native entry, and
        // the stub reached through `Code::entry_point_` fills `[SP]`.
        let instructions = [
            instruction(0x1000, "push", "{fp, lr}"),
            instruction(0x1004, "add", "fp, sp, #0"),
            instruction(0x1008, "sub", "sp, sp, #0xc"),
            instruction(0x100c, "ldr", "lr, [fp, #8]"),
            instruction(0x1010, "ldr", "sb, [sl, #0x38]"),
            instruction(0x1014, "stm", "sp, {sb, lr}"),
            instruction(0x1018, "add", "r2, sp, #4"),
            instruction(0x101c, "mov", "r1, #1"),
            instruction(0x1020, "ldr", "sb, [r5, #0x13]"),
            instruction(0x1024, "ldr", "lr, [r5, #0x17]"),
            instruction(0x1028, "ldr", "lr, [lr, #3]"),
            instruction(0x102c, "blx", "lr"),
            instruction(0x1030, "ldr", "r0, [sp]"),
            instruction(0x1034, "sub", "sp, fp, #0"),
            instruction(0x1038, "pop", "{fp, pc}"),
        ];
        let blocks = BTreeSet::from([0x1000]);
        let mut pool = vec!["snapshotRef(0)".to_owned(); 8];
        pool[3] = "nativePoolEntry(3)".to_owned();
        pool[4] = "resetPoolEntry(4)".to_owned();
        let statements = lift_semantics(
            Abi::ArmeabiV7a,
            Some(1),
            &instructions,
            &blocks,
            &BTreeMap::new(),
            Some(&pool),
        );

        assert!(matches!(
            statements.as_slice(),
            [
                SemanticStatement::ResolvedCall { target, arguments, .. },
                SemanticStatement::Return { expression, .. },
            ] if target == "native call" && arguments == &["arg0".to_owned()]
                && expression == "native_call_result"
        ));
    }

    #[test]
    fn arm32_store_multiple_keeps_stack_arguments_and_frame() {
        let instructions = [
            instruction(0x1000, "push", "{fp, lr}"),
            instruction(0x1004, "add", "fp, sp, #0"),
            instruction(0x1008, "sub", "sp, sp, #8"),
            instruction(0x100c, "mov", "r4, #4"),
            instruction(0x1010, "mov", "r6, #6"),
            instruction(0x1014, "stm", "sp, {r4, r6}"),
            instruction(0x1018, "bl", "#0x2000"),
            instruction(0x101c, "str", "r0, [sp]"),
            instruction(0x1020, "ldr", "r0, [sp]"),
            instruction(0x1024, "sub", "sp, fp, #0"),
            instruction(0x1028, "pop", "{fp, pc}"),
        ];
        let blocks = BTreeSet::from([0x1000]);
        let statements = lift_semantics(
            Abi::ArmeabiV7a,
            None,
            &instructions,
            &blocks,
            &BTreeMap::new(),
            None,
        );

        // `[sp]` holds the last argument; the frame stays tracked, so the
        // stored call result is read back rather than an anonymous slot.
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::ResolvedCall { target, arguments, .. }
                if target == "sub_2000" && arguments == &["6".to_owned(), "4".to_owned()]
        )));
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "sub_2000_result"
        )));
    }

    #[test]
    fn recovers_arm64_class_dispatch_selector_candidates() {
        let instructions = [
            instruction(0x1000, "mov", "x17, #0x20"),
            instruction(0x1004, "add", "x30, x0, x17"),
            instruction(0x1008, "ldr", "x30, [x21, x30, lsl #3]"),
            instruction(0x100c, "blr", "x30"),
        ];
        let mut targets = vec![None; 4131];
        targets[4129] = Some("Alpha.render".to_owned());
        targets[4130] = Some("Beta.render".to_owned());
        let table = DispatchTableAnalysis {
            origin_element: 4096,
            targets: &targets,
            class_ids: &[1, 2],
            cid_to_name: None,
            name_to_cids: None,
            qualified_to_cids: None,
            super_cids: None,
            target_owner_cids: &[],
            subtype_cids: None,
            label_results: None,
        };

        let calls = recover_dispatch_calls(Abi::Arm64V8a, &instructions, &table);
        let call = calls.get(&0x100c).unwrap();
        assert_eq!(call.selector_offset, 4128);
        assert_eq!(call.candidate_count, 2);
        assert_eq!(
            call.candidate_targets,
            vec!["Alpha.render".to_owned(), "Beta.render".to_owned()]
        );
    }

    #[test]
    fn recovers_arm32_split_and_direct_dispatch_sequences() {
        let instructions = [
            instruction(0x1000, "add", "lr, r7, r0, lsl #2"),
            instruction(0x1004, "ldr", "lr, [lr, #-0x10]"),
            instruction(0x1008, "blx", "lr"),
            instruction(0x100c, "ldr", "lr, [r7, r0, lsl #2]"),
            instruction(0x1010, "blx", "lr"),
        ];
        let mut targets = vec![None; 1026];
        targets[1020] = Some("Alpha.compare".to_owned());
        targets[1021] = Some("Beta.compare".to_owned());
        targets[1024] = Some("Alpha.render".to_owned());
        targets[1025] = Some("Beta.render".to_owned());
        let table = DispatchTableAnalysis {
            origin_element: 1023,
            targets: &targets,
            class_ids: &[1, 2],
            cid_to_name: None,
            name_to_cids: None,
            qualified_to_cids: None,
            super_cids: None,
            target_owner_cids: &[],
            subtype_cids: None,
            label_results: None,
        };

        let calls = recover_dispatch_calls(Abi::ArmeabiV7a, &instructions, &table);
        let split = calls.get(&0x1008).unwrap();
        assert_eq!(split.selector_offset, 1019);
        assert_eq!(split.selector_name.as_deref(), Some("compare"));
        let direct = calls.get(&0x1010).unwrap();
        assert_eq!(direct.selector_offset, 1023);
        assert_eq!(direct.selector_name.as_deref(), Some("render"));
    }

    #[test]
    fn rejects_sparse_selector_names_in_mostly_opaque_dispatch_rows() {
        let mut targets = (0..100)
            .map(|index| format!("sub_{index:x}"))
            .collect::<Vec<_>>();
        targets.extend((0..100).map(|index| format!("Class{}.==", index % 5)));

        // A 5-of-105 implementation win is grazing unrelated selectors that
        // share the offset window: no proven name, but the five readable
        // implementations stay as bounded evidence while the 100 synthetic
        // `sub_*` labels are filtered out of the candidate list.
        let (selector, candidates, count) = infer_dispatch_selector(&targets);
        assert_eq!(selector, None);
        assert!(!candidates.is_empty());
        assert!(
            candidates
                .iter()
                .all(|candidate| !candidate.starts_with("sub_"))
        );
        assert_eq!(count, 105);
    }

    #[test]
    fn merged_values_with_one_incoming_value_are_replaced() {
        use super::{remove_trivial_phis, replace_identifier};
        let assign = |variable: &str, value: &str| SemanticStatement::Assign {
            variable: variable.to_owned(),
            value: value.to_owned(),
            confidence: EvidenceConfidence::High,
            address: "0x1000".to_owned(),
        };
        let mut statements = vec![
            assign("phi_10_x0", "0"),
            assign("phi_20_x1", "phi_10_x0"),
            assign("phi_20_x1", "phi_10_x0"),
            assign("phi_10_x0", "(phi_20_x1 + 1)"),
            SemanticStatement::Return {
                expression: "phi_20_x1".to_owned(),
                confidence: EvidenceConfidence::High,
                address: "0x1010".to_owned(),
            },
        ];
        remove_trivial_phis(&mut statements);
        assert!(!format!("{statements:?}").contains("phi_20_x1"));
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Assign { variable, value, .. }
                if variable == "phi_10_x0" && value == "(phi_10_x0 + 1)"
        )));
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "phi_10_x0"
        )));
        assert_eq!(replace_identifier("phi_1 + phi_10", "phi_1", "a"), "a + phi_10");
    }

    #[test]
    fn loop_counters_merge_at_the_header() {
        let counting = [
            instruction(0x1000, "mov", "x0, #0"),
            instruction(0x1004, "cmp", "x0, x1"),
            instruction(0x1008, "b.ge", "#0x1014"),
            instruction(0x100c, "add", "x0, x0, #2"),
            instruction(0x1010, "b", "#0x1004"),
            instruction(0x1014, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &counting,
            &BTreeSet::from([0x1000, 0x1004, 0x100c, 0x1014]),
            &BTreeMap::new(),
            None,
        );
        let assigned = statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Assign {
                    variable, value, ..
                } if variable == "phi_1004_x0" => Some(value.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(assigned, vec!["0", "(phi_1004_x0 + 2)"]);
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "phi_1004_x0"
        )));
    }

    #[test]
    fn kills_caller_saved_values_and_disagreeing_joins() {
        // Values survive provable straight-line control flow.
        let straight_line = [
            instruction(0x1000, "mov", "x0, #1"),
            instruction(0x1004, "b", "#0x1008"),
            instruction(0x1008, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &straight_line,
            &BTreeSet::from([0x1000, 0x1008]),
            &BTreeMap::new(),
            None,
        );
        assert!(
            statements
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::Return { .. }))
        );

        // A join whose predecessors disagree merges into one named value
        // that each predecessor assigns.
        let diamond = [
            instruction(0x1000, "mov", "x0, #1"),
            instruction(0x1004, "cbz", "x1, #0x100c"),
            instruction(0x1008, "mov", "x0, #2"),
            instruction(0x100c, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &diamond,
            &BTreeSet::from([0x1000, 0x1008, 0x100c]),
            &BTreeMap::new(),
            None,
        );
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "phi_100c_x0"
        )));
        let assigned = statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Assign {
                    variable,
                    value,
                    address,
                    ..
                } if variable == "phi_100c_x0" => Some((address.as_str(), value.as_str())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(assigned, vec![("0x1004", "1"), ("0x1008", "2")]);

        // Caller-saved registers are still killed across calls.
        let call_sequence = [
            instruction(0x1000, "mov", "x1, #1"),
            instruction(0x1004, "bl", "#0x2000"),
            instruction(0x1008, "mov", "x0, x1"),
            instruction(0x100c, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &call_sequence,
            &BTreeSet::from([0x1000]),
            &BTreeMap::new(),
            None,
        );
        assert!(
            !statements
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::Return { .. }))
        );
    }

    #[test]
    fn rebuilds_string_interpolation_from_allocation_pattern() {
        // mov x2, #4; bl <alloc stub>; store "Added "; store pool part;
        // pass the array at [SP]; bl _interpolate — the AOT lowering of
        // 'Added ${...}'.
        let statements = lift_interpolation(&[
            instruction(0x1010, "mov", "x2, #4"),
            instruction(0x1014, "bl", "#0x9000"),
            instruction(0x1018, "ldr", "x16, [x27, #0x20]"),
            instruction(0x101c, "stur", "w16, [x0, #0xf]"),
            instruction(0x1020, "ldr", "x17, [x27, #0x28]"),
            instruction(0x1024, "stur", "w17, [x0, #0x13]"),
            instruction(0x1028, "str", "x0, [x15]"),
            instruction(0x102c, "bl", "#0x8000"),
            instruction(0x1030, "ret", ""),
        ]);
        assert_eq!(
            interpolations(&statements),
            vec![("'Added world'".to_owned(), EvidenceConfidence::High)],
            "{statements:?}"
        );
    }

    #[test]
    fn interpolation_follows_the_passed_array() {
        // Array A (two parts) is spilled, then array B (three parts) is
        // filled. Only A is passed; B's larger fill must not win.
        let statements = lift_interpolation(&[
            instruction(0x1010, "mov", "x2, #4"),
            instruction(0x1014, "bl", "#0x9000"),
            instruction(0x1018, "stur", "x0, [x29, #-8]"),
            instruction(0x101c, "ldr", "x16, [x27, #0x20]"),
            instruction(0x1020, "stur", "w16, [x0, #0xf]"),
            instruction(0x1024, "ldr", "x16, [x27, #0x28]"),
            instruction(0x1028, "stur", "w16, [x0, #0x13]"),
            instruction(0x102c, "mov", "x2, #6"),
            instruction(0x1030, "bl", "#0x9000"),
            instruction(0x1034, "ldr", "x16, [x27, #0x28]"),
            instruction(0x1038, "stur", "w16, [x0, #0xf]"),
            instruction(0x103c, "stur", "w16, [x0, #0x13]"),
            instruction(0x1040, "stur", "w16, [x0, #0x17]"),
            instruction(0x1044, "ldur", "x16, [x29, #-8]"),
            instruction(0x1048, "str", "x16, [x15]"),
            instruction(0x104c, "bl", "#0x8000"),
            instruction(0x1050, "ret", ""),
        ]);
        assert_eq!(
            interpolations(&statements),
            vec![("'Added world'".to_owned(), EvidenceConfidence::High)],
            "{statements:?}"
        );
    }

    #[test]
    fn interpolation_keeps_unstored_trailing_slot() {
        // Two elements are allocated but only the literal is stored; the
        // missing value stays an explicit gap and weakens the claim.
        let statements = lift_interpolation(&[
            instruction(0x1010, "mov", "x2, #4"),
            instruction(0x1014, "bl", "#0x9000"),
            instruction(0x1018, "ldr", "x16, [x27, #0x20]"),
            instruction(0x101c, "stur", "w16, [x0, #0xf]"),
            instruction(0x1020, "str", "x0, [x15]"),
            instruction(0x1024, "bl", "#0x8000"),
            instruction(0x1028, "ret", ""),
        ]);
        assert_eq!(
            interpolations(&statements),
            vec![(
                "'Added ${aot.unresolvedValue('interpolated part')}'".to_owned(),
                EvidenceConfidence::Low
            )],
            "{statements:?}"
        );
    }

    #[test]
    fn interpolation_requires_the_array_argument() {
        // The filled array never reaches an argument location.
        let statements = lift_interpolation(&[
            instruction(0x1010, "mov", "x2, #2"),
            instruction(0x1014, "bl", "#0x9000"),
            instruction(0x1018, "ldr", "x16, [x27, #0x20]"),
            instruction(0x101c, "stur", "w16, [x0, #0xf]"),
            instruction(0x1020, "bl", "#0x8000"),
            instruction(0x1024, "ret", ""),
        ]);
        assert!(interpolations(&statements).is_empty(), "{statements:?}");
    }

    #[test]
    fn interpolation_follows_x64_derived_element_pointer() {
        // x64 `formatPrice`: the array is spilled, reloaded, element 1 is
        // stored through `lea r13, [rdx + 0x13]`, and the array is passed
        // at [rsp].
        let instructions = [
            instruction(0x1000, "push", "rbp"),
            instruction(0x1001, "mov", "rbp, rsp"),
            instruction(0x1004, "sub", "rsp, 0x18"),
            instruction(0x1008, "mov", "r10d, 4"),
            instruction(0x100e, "call", "0x9000"),
            instruction(0x1013, "mov", "qword ptr [rbp - 8], rax"),
            instruction(0x1017, "mov", "r11, qword ptr [r15 + 0x1f]"),
            instruction(0x101e, "mov", "dword ptr [rax + 0xf], r11d"),
            instruction(0x1022, "mov", "rax, qword ptr [r15 + 0x27]"),
            instruction(0x1029, "mov", "rdx, qword ptr [rbp - 8]"),
            instruction(0x102d, "lea", "r13, [rdx + 0x13]"),
            instruction(0x1031, "mov", "dword ptr [r13], eax"),
            instruction(0x1035, "mov", "r11, qword ptr [rbp - 8]"),
            instruction(0x1039, "mov", "qword ptr [rsp], r11"),
            instruction(0x103d, "call", "0x8000"),
            instruction(0x1042, "mov", "rsp, rbp"),
            instruction(0x1045, "pop", "rbp"),
            instruction(0x1046, "ret", ""),
        ];
        let mut pool = vec![String::new(); 8];
        pool[2] = "\"Added \"".to_owned();
        pool[3] = "\"world\"".to_owned();
        let mut symbols = BTreeMap::new();
        symbols.insert(
            0x8000,
            Symbol::new("_StringBase._interpolate".to_owned(), None, None),
        );
        let statements = lift_semantics(
            Abi::X86_64,
            None,
            &instructions,
            &BTreeSet::from([0x1000]),
            &symbols,
            Some(&pool),
        );
        assert_eq!(
            interpolations(&statements),
            vec![("'Added world'".to_owned(), EvidenceConfidence::High)],
            "{statements:?}"
        );
    }

    /// Lifts `body` after an ARM64 frame prologue, with `_interpolate` at
    /// 0x8000 and pool strings "Added " (`[x27, #0x20]`) and "world"
    /// (`[x27, #0x28]`).
    fn lift_interpolation(body: &[DecodedInstruction]) -> Vec<SemanticStatement> {
        let mut instructions = vec![
            instruction(0x1000, "stp", "x29, x30, [x15, #-0x10]!"),
            instruction(0x1004, "mov", "x29, x15"),
            instruction(0x1008, "sub", "x15, x15, #0x18"),
            instruction(0x100c, "nop", ""),
        ];
        instructions.extend_from_slice(body);
        let mut pool = vec![String::new(); 8];
        pool[2] = "\"Added \"".to_owned();
        pool[3] = "\"world\"".to_owned();
        let mut symbols = BTreeMap::new();
        symbols.insert(
            0x8000,
            Symbol::new("_StringBase._interpolate".to_owned(), None, None),
        );
        lift_semantics(
            Abi::Arm64V8a,
            None,
            &instructions,
            &BTreeSet::from([0x1000]),
            &symbols,
            Some(&pool),
        )
    }

    fn interpolations(statements: &[SemanticStatement]) -> Vec<(String, EvidenceConfidence)> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::StringInterpolation {
                    parts, confidence, ..
                } => Some((parts.join(""), *confidence)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn bounds_expression_growth_in_loops() {
        let mut instructions = vec![instruction(0x1000, "add", "x1, x1, x1")];
        for index in 1..8 {
            instructions.push(instruction(0x1000 + index * 4, "add", "x1, x1, x1"));
        }
        instructions.push(instruction(0x1020, "mov", "x0, x1"));
        instructions.push(instruction(0x1024, "ret", ""));
        let statements = lift_semantics(
            Abi::Arm64V8a,
            Some(1),
            &instructions,
            &BTreeSet::from([0x1000]),
            &BTreeMap::new(),
            None,
        );
        assert!(
            !statements
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::Return { .. }))
        );
    }
}

#[cfg(test)]
mod fusion_tests {
    use super::*;

    fn insn(address: u64, text: &str) -> DecodedInstruction {
        let mut parts = text.splitn(2, ' ');
        let mnemonic = parts.next().unwrap().to_owned();
        let operands = parts.next().unwrap_or("").trim().to_owned();
        let byte_len = 4u64;
        DecodedInstruction {
            address,
            next: address + byte_len,
            mnemonic,
            operands,
        }
    }

    #[test]
    fn worklist_budget_discards_partial_input_states() {
        let code = vec![insn(0x100, "mov x0, x1"), insn(0x104, "ret")];
        let blocks = LifterBlocks::build(&code, &BTreeSet::from([0x100]));
        let mut entry = FlowState::default();
        entry.registers.insert(
            "x1".to_owned(),
            Expression {
                text: "argument".to_owned(),
                confidence: EvidenceConfidence::High,
                complexity: 1,
                class_name: None,
                class_library_uri: None,
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            },
        );
        let symbols = BTreeMap::<u64, Symbol>::new();
        let pool_loads = BTreeMap::<u64, usize>::new();
        let (states, exhausted) = solve_block_states(
            &blocks,
            &code,
            Abi::Arm64V8a,
            &symbols,
            None,
            &pool_loads,
            None,
            &entry,
            &BTreeSet::new(),
            &[],
            0,
            LiftContext::bare(),
        );
        assert!(exhausted);
        assert!(states[0].registers.is_empty());
        let (states, exhausted) = solve_block_states(
            &blocks,
            &code,
            Abi::Arm64V8a,
            &symbols,
            None,
            &pool_loads,
            None,
            &entry,
            &BTreeSet::new(),
            &[],
            8,
            LiftContext::bare(),
        );
        assert!(!exhausted);
        assert_eq!(states[0].registers["x1"].text, "argument");
    }

    #[test]
    fn converged_diamond_does_not_spend_budget_on_duplicate_join_visits() {
        let code = vec![
            insn(0x100, "cbz x1, #0x110"),
            insn(0x104, "mov x0, #7"),
            insn(0x108, "b #0x118"),
            insn(0x110, "mov x0, #7"),
            insn(0x114, "b #0x118"),
            insn(0x118, "ret"),
        ];
        let blocks = LifterBlocks::build(&code, &BTreeSet::from([0x100, 0x104, 0x110, 0x118]));
        let (states, exhausted) = solve_block_states(
            &blocks,
            &code,
            Abi::Arm64V8a,
            &BTreeMap::new(),
            None,
            &BTreeMap::new(),
            None,
            &FlowState::default(),
            &BTreeSet::new(),
            &[],
            4,
            LiftContext::bare(),
        );
        assert!(!exhausted);
        assert_eq!(states[3].registers["x0"].text, "7");
    }

    #[test]
    fn separate_call_results_do_not_merge_by_display_name() {
        let result = |site| Expression {
            text: "load_result".to_owned(),
            confidence: EvidenceConfidence::Low,
            complexity: 1,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: Some(site),
            high_word: false,
            exact_class: false,
        };
        let mut left = FlowState::default();
        left.registers.insert("x0".to_owned(), result(0x100));
        let mut right = FlowState::default();
        right.registers.insert("x0".to_owned(), result(0x200));
        assert!(FlowState::meet(&left, &right).registers.get("x0").is_none());
        right.registers.insert("x0".to_owned(), result(0x100));
        assert!(FlowState::meet(&left, &right).registers.contains_key("x0"));
    }

    #[test]
    fn metadata_handler_entry_recovers_isolated_catch_body() {
        let code = vec![
            insn(0x100, "mov x0, #1"),
            insn(0x104, "ret"),
            insn(0x108, "mov x0, #2"),
            insn(0x10c, "ret"),
        ];
        let starts = BTreeSet::from([0x100, 0x108]);
        let symbols = BTreeMap::new();
        let lift = |handlers: &[u64]| {
            lift_semantics_with_names_outcome(
                Abi::Arm64V8a,
                &[],
                &code,
                &starts,
                &symbols,
                None,
                None,
                None,
                None,
                None,
                None,
                handlers,
                &[],
            )
            .statements
            .into_iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Return { expression, .. } => Some(expression),
                _ => None,
            })
            .collect::<Vec<_>>()
        };
        assert_eq!(lift(&[]), vec!["1"]);
        assert_eq!(lift(&[0x108]), vec!["1", "2"]);

        // The handler is a separate VM entry. It must not inherit a live
        // normal-entry argument until catch-entry moves are decoded.
        let unknown_handler = vec![
            insn(0x100, "mov x0, #1"),
            insn(0x104, "ret"),
            insn(0x108, "mov x0, x1"),
            insn(0x10c, "ret"),
        ];
        let outcome = lift_semantics_with_names_outcome(
            Abi::Arm64V8a,
            &[ParameterHint {
                name: "arg0".to_owned(),
                class_name: None,
                class_library_uri: None,
            }],
            &unknown_handler,
            &starts,
            &symbols,
            None,
            None,
            None,
            None,
            None,
            None,
            &[0x108],
            &[],
        );
        assert!(!outcome.statements.iter().any(|statement| {
            matches!(statement, SemanticStatement::Return { expression, address, .. }
                if address == "0x10c" && expression == "arg0")
        }));
    }

    #[test]
    fn probe_recursive_checksum() {
        let code: Vec<DecodedInstruction> = vec![
            insn(0x630, "stp x29, x30, [x15, #-0x10]!"),
            insn(0x634, "mov x29, x15"),
            insn(0x638, "sub x15, x15, #8"),
            insn(0x63c, "mov x0, x1"),
            insn(0x640, "stur x1, [x29, #-8]"),
            insn(0x644, "ldr x16, [x26, #0x48]"),
            insn(0x648, "cmp x15, x16"),
            insn(0x64c, "b.ls #0x684"),
            insn(0x650, "cmp x0, #1"),
            insn(0x654, "b.gt #0x664"),
            insn(0x658, "mov x15, x29"),
            insn(0x65c, "ldp x29, x30, [x15], #0x10"),
            insn(0x660, "ret"),
            insn(0x664, "sub x1, x0, #2"),
            insn(0x668, "bl #0x630"),
            insn(0x66c, "ldur x1, [x29, #-8]"),
            insn(0x670, "add x2, x1, x0"),
            insn(0x674, "mov x0, x2"),
            insn(0x678, "mov x15, x29"),
            insn(0x67c, "ldp x29, x30, [x15], #0x10"),
            insn(0x680, "ret"),
            insn(0x684, "bl #0x9999"),
            insn(0x688, "b #0x650"),
        ];
        let starts = std::collections::BTreeSet::from([0x630u64, 0x650, 0x664, 0x684]);
        let symbols = BTreeMap::new();
        let fused = fuse_machine_idioms(Abi::Arm64V8a, &code, &symbols);
        println!("FUSED {}:", fused.len());
        for i in &fused {
            println!("  {:x} {}", i.address, i.mnemonic);
        }
        let blocks = LifterBlocks::build(&fused, &starts);
        for (i, (s, r)) in blocks.starts.iter().zip(blocks.ranges.iter()).enumerate() {
            println!("block{i}: {s:x} {:?}", r);
        }
        for (i, s) in block_successors(&blocks, &fused).iter().enumerate() {
            println!("succ{i}: {s:?}");
        }
        let statements = lift_semantics(
            Abi::Arm64V8a,
            Some(1),
            &code,
            &starts,
            &BTreeMap::new(),
            None,
        );
        println!("PROBE STATEMENTS: {}", statements.len());
        for s in &statements {
            println!("  {s:?}");
        }
    }

    #[test]
    fn recovers_smi_xor_with_args() {
        // NativeEntryPoints.retainedStaticEntrypoint: left ^ right
        let code = vec![
            insn(0x100, "ldr x2, [x15, #8]"),
            insn(0x104, "sbfx x3, x2, #1, #0x1f"),
            insn(0x108, "tbz w2, #0, #0x110"),
            insn(0x10c, "ldur x3, [x2, #7]"),
            insn(0x110, "ldr x2, [x15]"),
            insn(0x114, "sbfx x4, x2, #1, #0x1f"),
            insn(0x118, "tbz w2, #0, #0x120"),
            insn(0x11c, "ldur x4, [x2, #7]"),
            insn(0x120, "eor x2, x3, x4"),
            insn(0x124, "sbfiz x0, x2, #1, #0x1f"),
            insn(0x128, "cmp x2, x0, asr #1"),
            insn(0x12c, "b.eq #0x140"),
            insn(0x130, "stp x29, x30, [x15, #-0x10]!"),
            insn(0x134, "mov x29, x15"),
            insn(0x138, "bl #0x9999"),
            insn(0x13c, "stur x2, [x0, #7]"),
            insn(0x140, "ret"),
        ];
        let starts = std::collections::BTreeSet::from([0x100]);
        let statements = lift_semantics(
            Abi::Arm64V8a,
            Some(2),
            &code,
            &starts,
            &BTreeMap::new(),
            None,
        );
        let rendered = statements
            .iter()
            .map(|statement| format!("{statement:?}"))
            .collect::<Vec<_>>();
        assert!(
            rendered.iter().any(|text| text.contains("arg0 ^ arg1")),
            "expected xor of args, got {rendered:?}"
        );
        assert!(
            rendered.iter().any(|text| text.contains("Return")),
            "expected a recovered return, got {rendered:?}"
        );
    }

    #[test]
    fn recovers_negated_unboxed_double_comparison() {
        let code = vec![
            insn(0x100, "fmov d0, #10.00000000"),
            insn(0x104, "ldr x1, [x15]"),
            insn(0x108, "ldur d1, [x1, #0xf]"),
            insn(0x10c, "fcmp d1, d0"),
            insn(0x110, "add x16, x22, #0x20"),
            insn(0x114, "add x17, x22, #0x30"),
            insn(0x118, "csel x1, x16, x17, ge"),
            insn(0x11c, "eor x0, x1, #0x10"),
            insn(0x120, "ret"),
        ];
        let statements = lift_semantics_with_names(
            Abi::Arm64V8a,
            &[ParameterHint {
                name: "arg0".to_owned(),
                class_name: Some("Product".to_owned()),
                class_library_uri: Some("package:simple_app/models.dart".to_owned()),
            }],
            &code,
            &std::collections::BTreeSet::from([0x100]),
            &BTreeMap::new(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. }
                    if expression == "!(arg0._slot_10 >= 10.0)"
            )),
            "expected the source predicate, got {statements:?}"
        );
    }

    #[test]
    fn recovers_x64_negated_unboxed_double_comparison() {
        let code = vec![
            insn(0x100, "movsd xmm0, qword ptr [r15 + 0xac7]"),
            insn(0x104, "mov rcx, qword ptr [rsp + 8]"),
            insn(0x108, "movsd xmm1, qword ptr [rcx + 0xf]"),
            insn(0x10c, "comisd xmm1, xmm0"),
            insn(0x110, "jp 0x118"),
            insn(0x114, "jae 0x120"),
            insn(0x118, "mov rcx, qword ptr [r14 + 0xa0]"),
            insn(0x11c, "jmp 0x124"),
            insn(0x120, "mov rcx, qword ptr [r14 + 0x98]"),
            insn(0x124, "xor rcx, 0x10"),
            insn(0x128, "mov rax, rcx"),
            insn(0x12c, "ret"),
        ];
        let mut pool = vec![String::new(); 344];
        pool[343] = "4621819117588971520".to_owned();
        let hints = [ParameterHint {
            name: "arg0".to_owned(),
            class_name: Some("Product".to_owned()),
            class_library_uri: Some("package:simple_app/models.dart".to_owned()),
        }];
        let statements = lift_semantics_with_names(
            Abi::X86_64,
            &hints,
            &code,
            &std::collections::BTreeSet::from([0x100, 0x118, 0x120, 0x124]),
            &BTreeMap::new(),
            Some(&pool),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. }
                    if expression == "!(arg0._slot_10 >= 10.0)"
            )),
            "expected the source predicate, got {statements:?}"
        );
    }

    #[test]
    fn decodes_arm32_vfp_immediate_and_comparison() {
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x04, 0x0b, 0xb2, 0xee]),
            Some(("vmovd".to_owned(), "d0, #10.0".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x40, 0x2b, 0xb4, 0xee]),
            Some(("vcmpd".to_owned(), "d2, d0".to_owned()))
        );
    }

    #[test]
    fn decodes_arm32_vfp_arithmetic_conversions_and_zero_compares() {
        // Words pinned from the obf-raw ARM32 corpus; encodings follow Dart's
        // assembler (EmitVFPddd callers in assembler_arm.cc).
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x00, 0x4b, 0x22, 0xee]),
            Some(("vmul.f64".to_owned(), "d4, d2, d0".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x02, 0x4b, 0x30, 0xee]),
            Some(("vadd.f64".to_owned(), "d4, d0, d2".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x42, 0x4b, 0x30, 0xee]),
            Some(("vsub.f64".to_owned(), "d4, d0, d2".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x0e, 0xeb, 0x80, 0xee]),
            Some(("vdiv.f64".to_owned(), "d14, d0, d14".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x40, 0x2b, 0xb5, 0xee]),
            Some(("vcmpdz".to_owned(), "d2, d0".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x42, 0xab, 0x38, 0xee]),
            Some(("vsub.f64".to_owned(), "d10, d8, d2".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0xc0, 0xcb, 0xb7, 0xee]),
            Some(("vcvt.ds".to_owned(), "d12, d0".to_owned()))
        );
    }

    #[test]
    fn recovers_arm32_negated_unboxed_double_comparison() {
        let code = vec![
            insn(0x100, "vmovd d0, #10.0"),
            insn(0x104, "ldr r1, [sp]"),
            insn(0x108, "add ip, r1, #3"),
            insn(0x10c, "vldr d2, [ip, #8]"),
            insn(0x110, "vcmpd d2, d0"),
            insn(0x114, "vmrs apsr_nzcv, fpscr"),
            insn(0x118, "ldrge r1, [sl, #0x48]"),
            insn(0x11c, "ldrlt r1, [sl, #0x4c]"),
            insn(0x120, "eor r0, r1, #8"),
            insn(0x124, "bx lr"),
        ];
        let hints = [ParameterHint {
            name: "arg0".to_owned(),
            class_name: Some("Product".to_owned()),
            class_library_uri: Some("package:simple_app/models.dart".to_owned()),
        }];
        let statements = lift_semantics_with_names(
            Abi::ArmeabiV7a,
            &hints,
            &code,
            &std::collections::BTreeSet::from([0x100]),
            &BTreeMap::new(),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. }
                    if expression == "!(arg0._slot_c >= 10.0)"
            )),
            "expected the source predicate, got {statements:?}"
        );
    }

    #[test]
    fn lifts_arm32_vfp_double_arithmetic_to_source_expressions() {
        let code = vec![
            insn(0x100, "vmovd d0, #10.0"),
            insn(0x104, "vadd.f64 d3, d0, d0"),
            insn(0x108, "ldr r1, [sp]"),
            insn(0x10c, "add ip, r1, #3"),
            insn(0x110, "vldr d2, [ip, #8]"),
            insn(0x114, "vcmpd d2, d3"),
            insn(0x118, "vmrs apsr_nzcv, fpscr"),
            insn(0x11c, "ldrge r1, [sl, #0x48]"),
            insn(0x120, "ldrlt r1, [sl, #0x4c]"),
            insn(0x124, "eor r0, r1, #8"),
            insn(0x128, "bx lr"),
        ];
        let hints = [ParameterHint {
            name: "arg0".to_owned(),
            class_name: Some("Product".to_owned()),
            class_library_uri: Some("package:simple_app/models.dart".to_owned()),
        }];
        let statements = lift_semantics_with_names(
            Abi::ArmeabiV7a,
            &hints,
            &code,
            &std::collections::BTreeSet::from([0x100]),
            &BTreeMap::new(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. }
                    if expression.contains("(10.0 + 10.0)")
            )),
            "expected VADD to surface as Dart addition inside the returned predicate, got {statements:?}"
        );
    }

    #[test]
    fn removes_boxed_double_allocation_control_flow() {
        let code = vec![
            insn(0x100, "fmov d0, #10.00000000"),
            insn(0x104, "ldp x1, x2, [x26, #0x60]"),
            insn(0x108, "add x1, x1, #0x10"),
            insn(0x10c, "cmp x2, x1"),
            insn(0x110, "b.ls #0x140"),
            insn(0x114, "str x1, [x26, #0x60]"),
            insn(0x118, "sub x1, x1, #0xf"),
            insn(0x11c, "mov x2, #0xe19c"),
            insn(0x120, "movk x2, #3, lsl #16"),
            insn(0x124, "stur x2, [x1, #-1]"),
            insn(0x128, "dmb ishst"),
            insn(0x12c, "stur d0, [x1, #7]"),
            insn(0x130, "mov x0, x1"),
            insn(0x134, "ret"),
            insn(0x140, "str q0, [x15, #-0x10]!"),
            insn(0x144, "bl #0x9999"),
            insn(0x148, "mov x1, x0"),
            insn(0x14c, "ldr q0, [x15], #0x10"),
            insn(0x150, "b #0x12c"),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &code,
            &std::collections::BTreeSet::from([0x100, 0x114, 0x130]),
            &BTreeMap::from([(
                0x9999,
                Symbol::new("stub AllocateDouble".to_owned(), None, None),
            )]),
            None,
        );

        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. } if expression == "10.0"
            )),
            "expected allocation-neutral return, got {statements:?}"
        );
        assert!(
            statements
                .iter()
                .all(|statement| !matches!(statement, SemanticStatement::Condition { .. })),
            "allocation control flow leaked into source: {statements:?}"
        );
    }

    #[test]
    fn does_not_fuse_non_double_allocation() {
        let code = vec![
            insn(0x100, "fmov d0, #10.00000000"),
            insn(0x104, "ldp x1, x2, [x26, #0x60]"),
            insn(0x108, "add x1, x1, #0x10"),
            insn(0x10c, "cmp x2, x1"),
            insn(0x110, "b.ls #0x140"),
            insn(0x114, "str x1, [x26, #0x60]"),
            insn(0x118, "sub x1, x1, #0xf"),
            // CID 1000, with the same 16-byte size tag and GC bits as Double.
            insn(0x11c, "mov x2, #0x819c"),
            insn(0x120, "movk x2, #0x3e, lsl #16"),
            insn(0x124, "stur x2, [x1, #-1]"),
            insn(0x128, "dmb ishst"),
            insn(0x12c, "stur d0, [x1, #7]"),
            insn(0x130, "mov x0, x1"),
            insn(0x134, "ret"),
            insn(0x140, "str q0, [x15, #-0x10]!"),
            insn(0x144, "bl #0x9999"),
            insn(0x148, "mov x1, x0"),
            insn(0x14c, "ldr q0, [x15], #0x10"),
            insn(0x150, "b #0x12c"),
        ];

        let fused = fuse_machine_idioms(Abi::Arm64V8a, &code, &BTreeMap::new());

        assert!(
            fused
                .iter()
                .any(|instruction| instruction.mnemonic == "b.ls"),
            "a non-Double allocation was incorrectly erased: {fused:?}"
        );
    }

    /// The x64 `formatPrice` box of `value` for `toStringAsFixed`.
    fn x64_double_box(header: &str) -> Vec<DecodedInstruction> {
        vec![
            insn(0x100, "movsd xmm0, qword ptr [rbp - 0x10]"),
            insn(0x104, "mov rdi, qword ptr [r14 + 0x60]"),
            insn(0x108, "add rdi, 0x10"),
            insn(0x10c, "cmp rdi, qword ptr [r14 + 0x68]"),
            insn(0x110, "jae 0x140"),
            insn(0x114, "mov qword ptr [r14 + 0x60], rdi"),
            insn(0x118, "sub rdi, 0xf"),
            insn(0x11c, &format!("mov qword ptr [rdi - 1], {header}")),
            insn(0x120, "movsd qword ptr [rdi + 7], xmm0"),
            insn(0x124, "mov esi, 2"),
            insn(0x128, "call 0x9000"),
            insn(0x12c, "ret"),
            insn(0x140, "sub rsp, 0x10"),
            insn(0x144, "movups xmmword ptr [rsp], xmm0"),
            insn(0x148, "push rax"),
            insn(0x14c, "call 0x9999"),
            insn(0x150, "mov rdi, rax"),
            insn(0x154, "pop rax"),
            insn(0x158, "movups xmm0, xmmword ptr [rsp]"),
            insn(0x15c, "add rsp, 0x10"),
            insn(0x160, "jmp 0x120"),
        ]
    }

    #[test]
    fn fuses_x64_double_box() {
        let stubs = BTreeMap::from([(
            0x9999,
            Symbol::new("stub AllocateDouble".to_owned(), None, None),
        )]);
        let fused = fuse_machine_idioms(Abi::X86_64, &x64_double_box("0x3e19c"), &stubs);
        let mnemonics = fused
            .iter()
            .map(|instruction| instruction.mnemonic.as_str())
            .collect::<Vec<_>>();
        assert!(
            !mnemonics.contains(&"jae") && !mnemonics.contains(&"jmp"),
            "{fused:?}"
        );
        assert!(
            fused
                .iter()
                .any(|instruction| instruction.mnemonic == "fmov"
                    && instruction.operands == "rdi, xmm0"),
            "{fused:?}"
        );
        // CID 1000 with a Double-sized header is not a box.
        let fused = fuse_machine_idioms(Abi::X86_64, &x64_double_box("0x3e819c"), &stubs);
        assert!(
            fused
                .iter()
                .any(|instruction| instruction.mnemonic == "jae"),
            "{fused:?}"
        );
    }

    #[test]
    fn fuses_arm32_double_box() {
        // ARM32 aligns the VFP store address and uses an 8-byte-unit size tag.
        let code = vec![
            insn(0x100, "vldr d0, [fp, #-0xc]"),
            insn(0x104, "ldr r1, [sl, #0x2c]"),
            insn(0x108, "add r1, r1, #0x10"),
            insn(0x10c, "ldr ip, [sl, #0x30]"),
            insn(0x110, "cmp ip, r1"),
            insn(0x114, "bls #0x140"),
            insn(0x118, "str r1, [sl, #0x2c]"),
            insn(0x11c, "sub r1, r1, #0xf"),
            insn(0x120, "movw r2, #0xe29c"),
            insn(0x124, "movt r2, #3"),
            insn(0x128, "str r2, [r1, #-1]"),
            insn(0x12c, "dmb ishst"),
            insn(0x130, "add ip, r1, #3"),
            insn(0x134, "vstr d0, [ip, #4]"),
            insn(0x138, "bl #0x9000"),
            insn(0x13c, "pop {fp, pc}"),
            insn(0x140, "vpush {d0, d1}"),
            insn(0x144, "stmdb sp!, {r0}"),
            insn(0x148, "bl #0x9999"),
            insn(0x14c, "mov r1, r0"),
            insn(0x150, "ldm sp!, {r0}"),
            insn(0x154, "vpop {d0, d1}"),
            insn(0x158, "b #0x130"),
        ];
        let stubs = BTreeMap::from([(
            0x9999,
            Symbol::new("stub AllocateDouble".to_owned(), None, None),
        )]);
        let fused = fuse_machine_idioms(Abi::ArmeabiV7a, &code, &stubs);
        assert!(
            fused
                .iter()
                .all(|instruction| instruction.mnemonic != "bls" && instruction.mnemonic != "vstr"),
            "{fused:?}"
        );
        assert!(
            fused
                .iter()
                .any(|instruction| instruction.mnemonic == "fmov"
                    && instruction.operands == "r1, d0"),
            "{fused:?}"
        );
    }

    #[test]
    fn removes_x64_and_arm32_write_barriers() {
        // Only the element store and the join survive.
        let x64 = vec![
            insn(0x100, "mov dword ptr [r13], eax"),
            insn(0x104, "test al, 1"),
            insn(0x108, "je 0x124"),
            insn(0x10c, "mov r11b, byte ptr [rdx - 1]"),
            insn(0x110, "shr r11d, 2"),
            insn(0x114, "and r11d, dword ptr [r14 + 0x50]"),
            insn(0x118, "test byte ptr [rax - 1], r11b"),
            insn(0x11c, "je 0x124"),
            insn(0x120, "call 0x9000"),
            insn(0x124, "ret"),
        ];
        let arm32 = vec![
            insn(0x100, "str r0, [sb]"),
            insn(0x104, "tst r0, #1"),
            insn(0x108, "beq #0x124"),
            insn(0x10c, "ldrb ip, [r1, #-1]"),
            insn(0x110, "ldrb lr, [r0, #-1]"),
            insn(0x114, "and ip, lr, ip, lsr #2"),
            insn(0x118, "ldr lr, [sl, #0x28]"),
            insn(0x11c, "tst ip, lr"),
            insn(0x120, "blne #0x9000"),
            insn(0x124, "bx lr"),
        ];
        for (abi, code) in [(Abi::X86_64, x64), (Abi::ArmeabiV7a, arm32)] {
            let unknown = fuse_machine_idioms(abi, &code, &BTreeMap::new());
            assert!(
                unknown
                    .iter()
                    .any(|instruction| instruction.address == 0x120),
                "unnamed call was erased for {abi:?}"
            );
            let stubs = BTreeMap::from([(
                0x9000,
                Symbol::new("stub ArrayWriteBarrier".to_owned(), None, None),
            )]);
            let fused = fuse_machine_idioms(abi, &code, &stubs);
            assert_eq!(fused.len(), 2, "{abi:?}: {fused:?}");
        }
    }

    fn hint(name: &str) -> ParameterHint {
        ParameterHint {
            name: name.to_owned(),
            class_name: None,
            class_library_uri: None,
        }
    }

    fn lift_with_convention(
        abi: Abi,
        hints: &[ParameterHint],
        code: &[DecodedInstruction],
        symbols: &BTreeMap<u64, Symbol>,
        convention: Option<&ConventionInput>,
    ) -> Vec<SemanticStatement> {
        let starts = code
            .first()
            .map(|instruction| BTreeSet::from([instruction.address]))
            .unwrap_or_default();
        lift_semantics_with_names(
            abi, hints, code, &starts, symbols, None, None, None, None, None, convention,
        )
    }

    fn returned(statements: &[SemanticStatement]) -> Vec<String> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Return { expression, .. } => Some(expression.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn decompresses_x64_reference_through_heap_base() {
        // `mov eax, dword ptr [rdi + 0xb]; add rax, qword ptr [r14 + 0x58]`
        // reads a compressed reference field of `this`.
        let code = vec![
            insn(0x100, "mov eax, dword ptr [rdi + 0xb]"),
            insn(0x104, "add rax, qword ptr [r14 + 0x58]"),
            insn(0x108, "ret"),
        ];
        let statements =
            lift_with_convention(Abi::X86_64, &[hint("this")], &code, &BTreeMap::new(), None);
        assert_eq!(
            returned(&statements),
            vec!["this._slot_c".to_owned()],
            "{statements:?}"
        );
        // Any other thread slot is not the heap base.
        let mut code = code;
        code[1] = insn(0x104, "add rax, qword ptr [r14 + 0x60]");
        let statements =
            lift_with_convention(Abi::X86_64, &[hint("this")], &code, &BTreeMap::new(), None);
        assert!(returned(&statements).is_empty(), "{statements:?}");
    }

    #[test]
    fn catch_entry_moves_populate_handler_frame_slots() {
        use super::ExceptionalEdge;
        use crate::model::{CatchEntryMove, CatchMoveSource};
        let code = vec![
            insn(0x100, "stp x29, x30, [x15, #-0x10]!"),
            insn(0x104, "mov x29, x15"),
            insn(0x108, "stur x1, [x29, #-0x10]"),
            insn(0x10c, "bl #0x2000"),
            insn(0x110, "mov x0, #1"),
            insn(0x114, "ret"),
            insn(0x118, "ldur x0, [x29, #-8]"),
            insn(0x11c, "ret"),
        ];
        let starts = BTreeSet::from([0x100, 0x118]);
        let lift = |moves: Vec<CatchEntryMove>| {
            lift_semantics_with_names_outcome(
                Abi::Arm64V8a,
                &[hint("arg0")],
                &code,
                &starts,
                &BTreeMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
                &[0x118],
                &[ExceptionalEdge {
                    throw_return: 0x110,
                    handler: 0x118,
                    moves,
                }],
            )
            .statements
            .into_iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Return { expression, address, .. } if address == "0x11c" => {
                    Some(expression)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
        };
        // Slot 1 (fp - 16) holds arg0; the move copies it to slot 0 (fp - 8),
        // which the handler reads.
        let moved = lift(vec![CatchEntryMove {
            kind: CatchMoveSource::Tagged,
            source: 1,
            source_high: None,
            destination: 0,
        }]);
        assert_eq!(moved, vec!["arg0".to_owned()]);
        // Without the move, slot 0 was never written: nothing is claimed.
        assert!(lift(Vec::new()).iter().all(|value| value != "arg0"));
    }

    #[test]
    fn handlers_see_the_exception_register() {
        use super::ExceptionalEdge;
        let code = vec![
            insn(0x100, "bl #0x2000"),
            insn(0x104, "ret"),
            insn(0x108, "ret"),
        ];
        let statements = lift_semantics_with_names_outcome(
            Abi::Arm64V8a,
            &[],
            &code,
            &BTreeSet::from([0x100, 0x108]),
            &BTreeMap::new(),
            None,
            None,
            None,
            None,
            None,
            None,
            &[0x108],
            &[ExceptionalEdge {
                throw_return: 0x104,
                handler: 0x108,
                moves: Vec::new(),
            }],
        )
        .statements;
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, address, .. }
                if address == "0x108" && expression == "e"
        )));
    }

    #[test]
    fn optional_positional_defaults_come_from_the_missing_argument_path() {
        // (this, [name = null]): the prologue reads the argument count and
        // loads `name` only when it was supplied.
        let code = vec![
            insn(0x100, "ldur w1, [x4, #0x13]"),
            insn(0x104, "sub x2, x1, #2"),
            insn(0x108, "add x1, x29, w2, sxtw #2"),
            insn(0x10c, "ldr x1, [x1, #0x10]"),
            insn(0x110, "cmp w2, #2"),
            insn(0x114, "b.lt #0x124"),
            insn(0x118, "add x3, x29, w2, sxtw #2"),
            insn(0x11c, "ldr x0, [x3, #8]"),
            insn(0x120, "b #0x128"),
            insn(0x124, "mov x0, x22"),
            insn(0x128, "ret"),
        ];
        let outcome = lift_semantics_with_names_outcome(
            Abi::Arm64V8a,
            &[hint("this"), hint("name")],
            &code,
            &BTreeSet::from([0x100, 0x118, 0x124, 0x128]),
            &BTreeMap::new(),
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            &[],
        );
        assert_eq!(
            outcome.parameter_defaults,
            BTreeMap::from([(1, "null".to_owned())])
        );
        assert_eq!(returned(&outcome.statements), vec!["name".to_owned()]);
        assert!(!outcome.statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Condition { .. } | SemanticStatement::Assign { .. }
        )));
    }

    #[test]
    fn frameless_barrier_into_the_wrappers_stub_fuses() {
        let code = vec![
            insn(0x100, "stur w0, [x1, #0x13]"),
            insn(0x104, "ldurb w16, [x1, #-1]"),
            insn(0x108, "ldurb w17, [x0, #-1]"),
            insn(0x10c, "and x16, x17, x16, lsr #2"),
            insn(0x110, "tst x16, x28, lsr #32"),
            insn(0x114, "b.eq #0x124"),
            insn(0x118, "str x30, [x15, #-8]!"),
            insn(0x11c, "bl #0x5040"),
            insn(0x120, "ldr x30, [x15], #8"),
            insn(0x124, "ret"),
        ];
        // The call lands 0x40 bytes into the root-proven wrappers stub.
        let symbols = BTreeMap::from([(
            0x5000,
            Symbol::new("stub WriteBarrierWrappers".to_owned(), None, None),
        )]);
        let fused = fuse_machine_idioms(Abi::Arm64V8a, &code, &symbols);
        let addresses = fused.iter().map(|i| i.address).collect::<Vec<_>>();
        assert_eq!(addresses, vec![0x100, 0x104, 0x108, 0x10c, 0x124]);
        // An unrelated preceding symbol does not make the call a stub.
        let other = BTreeMap::from([(0x5000, Symbol::new("Foo.bar".to_owned(), None, None))]);
        assert_eq!(fuse_machine_idioms(Abi::Arm64V8a, &code, &other).len(), code.len());
    }

    #[test]
    fn x64_suspend_epilogue_is_not_a_function_exit() {
        use super::skip_x64_suspend_epilogues;
        let mut code = vec![
            insn(0x100, "call 0x2000"),
            insn(0x105, "mov rsp, rbp"),
            insn(0x108, "pop rbp"),
            insn(0x109, "ret"),
            insn(0x10a, "mov rax, rbx"),
            insn(0x10d, "ret"),
        ];
        skip_x64_suspend_epilogues(&mut code, &BTreeSet::from([0x100]));
        let mnemonics = code.iter().map(|i| i.address).collect::<Vec<_>>();
        assert_eq!(mnemonics, vec![0x100, 0x10a, 0x10d]);
        // Only a proven suspension call loses its epilogue.
        let mut plain = vec![
            insn(0x100, "call 0x2000"),
            insn(0x105, "mov rsp, rbp"),
            insn(0x108, "pop rbp"),
            insn(0x109, "ret"),
        ];
        skip_x64_suspend_epilogues(&mut plain, &BTreeSet::new());
        assert_eq!(plain.len(), 4);
    }

    #[test]
    fn decompresses_arm64_reference_through_heap_bits() {
        // `ldur w0, [x1, #0xb]; add x0, x0, x28, lsl #32`: Capstone prints
        // the shift as a separate operand.
        let code = vec![
            insn(0x100, "ldur w0, [x1, #0xb]"),
            insn(0x104, "add x0, x0, x28, lsl #32"),
            insn(0x108, "ret"),
        ];
        let statements =
            lift_with_convention(Abi::Arm64V8a, &[hint("this")], &code, &BTreeMap::new(), None);
        assert_eq!(
            returned(&statements),
            vec!["this._slot_c".to_owned()],
            "{statements:?}"
        );
        let mut code = code;
        code[1] = insn(0x104, "add x0, x0, x28, lsl #16");
        let statements =
            lift_with_convention(Abi::Arm64V8a, &[hint("this")], &code, &BTreeMap::new(), None);
        assert!(returned(&statements).is_empty(), "{statements:?}");
    }

    #[test]
    fn places_mixed_int_and_double_parameters_by_representation() {
        // (this, int a, double b, Object c) on ARM64: b travels in d0 and
        // does not consume a CPU register, so c arrives in x3.
        let code = vec![insn(0x100, "mov x0, x3"), insn(0x104, "ret")];
        let convention = ConventionInput {
            declared: Some(vec![
                DeclaredRepresentation::Tagged,
                DeclaredRepresentation::MaybeUnboxedInt,
                DeclaredRepresentation::MaybeUnboxedDouble,
                DeclaredRepresentation::Tagged,
            ]),
            window: Some(RegisterWindow { min: 0, max: 4 }),
            entry_offset: 0,
        };
        let statements = lift_with_convention(
            Abi::Arm64V8a,
            &[hint("this"), hint("a"), hint("b"), hint("c")],
            &code,
            &BTreeMap::new(),
            Some(&convention),
        );
        assert_eq!(returned(&statements), vec!["c".to_owned()]);
    }

    #[test]
    fn x64_fourth_cpu_argument_is_rbx_not_rcx() {
        let code = vec![insn(0x100, "mov rax, rbx"), insn(0x104, "ret")];
        let hints = ["a", "b", "c", "d"].map(hint);
        let statements = lift_with_convention(Abi::X86_64, &hints, &code, &BTreeMap::new(), None);
        assert_eq!(returned(&statements), vec!["d".to_owned()]);
        let from_rcx = vec![insn(0x100, "mov rax, rcx"), insn(0x104, "ret")];
        let statements =
            lift_with_convention(Abi::X86_64, &hints, &from_rcx, &BTreeMap::new(), None);
        assert!(
            returned(&statements)
                .iter()
                .all(|expression| expression != "d"),
            "rcx is not a Dart argument register: {statements:?}"
        );
    }

    #[test]
    fn arm32_stack_parameters_use_four_byte_slots() {
        // Closures use the stack convention: (closureContext, p) with p
        // nearest the entry stack pointer.
        let code = vec![insn(0x100, "ldr r0, [sp, #4]"), insn(0x104, "bx lr")];
        let convention = ConventionInput {
            declared: Some(vec![DeclaredRepresentation::Tagged; 2]),
            window: Some(RegisterWindow { min: 0, max: 0 }),
            entry_offset: 0,
        };
        let statements = lift_with_convention(
            Abi::ArmeabiV7a,
            &[hint("closureContext"), hint("p")],
            &code,
            &BTreeMap::new(),
            Some(&convention),
        );
        assert_eq!(returned(&statements), vec!["closureContext".to_owned()]);
    }

    #[test]
    fn frame_locals_and_outgoing_slots_are_not_parameters() {
        let convention = ConventionInput {
            declared: Some(vec![DeclaredRepresentation::Tagged]),
            window: Some(RegisterWindow { min: 0, max: 0 }),
            entry_offset: 0,
        };
        let prologue = [
            "stp x29, x30, [x15, #-0x10]!",
            "mov x29, x15",
            "sub x15, x15, #0x10",
        ];
        let mut code = prologue
            .iter()
            .enumerate()
            .map(|(index, text)| insn(0x100 + 4 * index as u64, text))
            .collect::<Vec<_>>();
        // After the frame exists, [x15] is the outgoing area, not the
        // incoming closure context the entry stack pointer addressed.
        code.push(insn(0x10c, "ldr x0, [x15]"));
        code.push(insn(0x110, "ret"));
        let statements = lift_with_convention(
            Abi::Arm64V8a,
            &[hint("closureContext")],
            &code,
            &BTreeMap::new(),
            Some(&convention),
        );
        assert!(
            returned(&statements)
                .iter()
                .all(|expression| expression != "closureContext"),
            "{statements:?}"
        );
        // The same slot through the frame pointer is the parameter.
        let mut code = code[..3].to_vec();
        code.push(insn(0x10c, "ldr x0, [x29, #0x10]"));
        code.push(insn(0x110, "ret"));
        let statements = lift_with_convention(
            Abi::Arm64V8a,
            &[hint("closureContext")],
            &code,
            &BTreeMap::new(),
            Some(&convention),
        );
        assert_eq!(returned(&statements), vec!["closureContext".to_owned()]);
    }

    #[test]
    fn values_do_not_survive_dart_calls_in_allocatable_registers() {
        let code = vec![
            insn(0x100, "mov x19, x1"),
            insn(0x104, "bl #0x200"),
            insn(0x108, "mov x0, x19"),
            insn(0x10c, "ret"),
        ];
        let statements = lift_with_convention(
            Abi::Arm64V8a,
            &[hint("this")],
            &code,
            &BTreeMap::new(),
            None,
        );
        assert!(
            returned(&statements)
                .iter()
                .all(|expression| expression != "this"),
            "x19 is allocatable and clobbered by Dart calls: {statements:?}"
        );
    }

    #[test]
    fn callee_convention_orders_register_and_stack_arguments() {
        let mut target = Symbol::new("helper".to_owned(), None, None);
        target.parameters = Some(std::sync::Arc::from(vec![
            ArgumentLocation::Register("x1"),
            ArgumentLocation::FpuRegister("d0"),
            ArgumentLocation::Stack { word: 0, words: 1 },
        ]));
        target.returns_fpu = true;
        let symbols = BTreeMap::from([(0x200, target)]);
        let code = vec![
            insn(0x100, "stp x29, x30, [x15, #-0x10]!"),
            insn(0x104, "mov x29, x15"),
            insn(0x108, "sub x15, x15, #0x10"),
            insn(0x10c, "fmov d0, #2.50000000"),
            insn(0x110, "str x2, [x15]"),
            insn(0x114, "bl #0x200"),
            insn(0x118, "fmov d1, d0"),
            insn(0x11c, "mov x15, x29"),
            insn(0x120, "ldp x29, x30, [x15], #0x10"),
            insn(0x124, "ret"),
        ];
        let statements = lift_with_convention(
            Abi::Arm64V8a,
            &[hint("this"), hint("value")],
            &code,
            &symbols,
            None,
        );
        let call = statements
            .iter()
            .find_map(|statement| match statement {
                SemanticStatement::ResolvedCall { arguments, .. } => Some(arguments.clone()),
                _ => None,
            })
            .expect("call");
        // x1 is forwarded unchanged from the caller's own receiver.
        assert_eq!(call, vec!["this", "2.5", "value"]);
    }

    #[test]
    fn retained_arity_bounds_only_register_only_unknown_calls() {
        let layout = TargetLayout::of(Abi::Arm64V8a);
        let mut target = Symbol::new("_Double.toStringAsFixed".to_owned(), None, None);
        target.value_argument_count = Some(2);
        let value = |text: &str| Expression {
            text: text.to_owned(),
            confidence: EvidenceConfidence::High,
            complexity: 1,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        };
        let registers = BTreeMap::from([
            ("x1".to_owned(), value("receiver")),
            ("x2".to_owned(), value("2")),
            ("x3".to_owned(), value("stale")),
        ]);
        let written = (1 << 0) | (1 << 1) | (1 << 2);
        assert_eq!(
            collect_call_arguments(
                Abi::Arm64V8a,
                &registers,
                &BTreeMap::new(),
                written,
                Some(&target)
            ),
            vec!["receiver", "2"]
        );
        let outgoing = BTreeMap::from([(-layout.word_size, value("stack"))]);
        assert_eq!(
            collect_call_arguments(Abi::Arm64V8a, &registers, &outgoing, written, Some(&target)),
            vec!["receiver", "2", "stale", "stack"]
        );
    }

    struct DispatchFixture {
        targets: Vec<Option<String>>,
        class_ids: Vec<usize>,
        names: BTreeMap<String, Vec<usize>>,
        qualified: BTreeMap<(Option<String>, String), Vec<usize>>,
    }

    impl DispatchFixture {
        /// Selector row at offset 100: class 10 (`Circle`) implements it as
        /// `Circle.area`, class 11 (`Square`) as `Square.area`.
        fn new() -> Self {
            let mut targets = vec![None; 200];
            targets[110] = Some("Circle.area".to_owned());
            targets[111] = Some("Square.area".to_owned());
            let mut names = BTreeMap::new();
            names.insert("Circle".to_owned(), vec![10]);
            names.insert("Square".to_owned(), vec![11]);
            let mut qualified = BTreeMap::new();
            qualified.insert(
                (
                    Some("package:app/shapes.dart".to_owned()),
                    "Circle".to_owned(),
                ),
                vec![10],
            );
            qualified.insert(
                (
                    Some("package:app/shapes.dart".to_owned()),
                    "Square".to_owned(),
                ),
                vec![11],
            );
            Self {
                targets,
                class_ids: vec![10, 11],
                names,
                qualified,
            }
        }

        fn analysis(&self, origin_element: usize) -> DispatchTableAnalysis<'_> {
            DispatchTableAnalysis {
                origin_element,
                targets: &self.targets,
                class_ids: &self.class_ids,
                cid_to_name: None,
                name_to_cids: Some(&self.names),
                qualified_to_cids: Some(&self.qualified),
                super_cids: None,
                target_owner_cids: &[],
                subtype_cids: None,
                label_results: None,
            }
        }
    }

    fn allocation_symbol(class: &str) -> Symbol {
        let mut symbol = Symbol::new(
            format!("package:app/shapes.dart.{class}"),
            Some("package:app/shapes.dart".to_owned()),
            None,
        )
        .with_result_class(Some(class.to_owned()));
        symbol.parameters = Some(std::sync::Arc::from(Vec::new()));
        symbol.allocation_stub = true;
        symbol
    }

    fn resolved_targets(statements: &[SemanticStatement]) -> Vec<(String, EvidenceConfidence)> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::ResolvedCall {
                    target, confidence, ..
                } => Some((target.clone(), *confidence)),
                _ => None,
            })
            .collect()
    }

    /// ARM64 dispatch through the class id of `receiver`, selector row 100
    /// with origin 4096: `sub x30, x0, #3996`.
    fn arm64_dispatch_on(receiver: &str, first: u64) -> Vec<DecodedInstruction> {
        vec![
            insn(first, &format!("ldur x0, [{receiver}, #-1]")),
            insn(first + 4, "ubfx x0, x0, #0xc, #0x14"),
            insn(first + 8, "sub x30, x0, #0xf9c"),
            insn(first + 12, "ldr x30, [x21, x30, lsl #3]"),
            insn(first + 16, "blr x30"),
        ]
    }

    fn lift_dispatch(
        abi: Abi,
        code: &[DecodedInstruction],
        symbols: &BTreeMap<u64, Symbol>,
        hints: &[ParameterHint],
        table: &DispatchTableAnalysis<'_>,
    ) -> Vec<SemanticStatement> {
        let calls = recover_dispatch_calls(abi, code, table);
        let starts = BTreeSet::from([code[0].address]);
        lift_semantics_with_names(
            abi,
            hints,
            code,
            &starts,
            symbols,
            None,
            None,
            None,
            Some(table),
            Some(&calls),
            None,
        )
    }

    #[test]
    fn dispatch_follows_the_class_id_register_not_other_live_values() {
        let fixture = DispatchFixture::new();
        let table = fixture.analysis(4096);
        let symbols = BTreeMap::from([
            (0x800, allocation_symbol("Circle")),
            (0x900, allocation_symbol("Square")),
        ]);
        // A Circle is allocated and parked in x19 while a Square becomes the
        // receiver whose class id indexes the table.
        let mut code = vec![
            insn(0x100, "bl #0x800"),
            insn(0x104, "mov x19, x0"),
            insn(0x108, "bl #0x900"),
            insn(0x10c, "mov x2, x0"),
            insn(0x110, "mov x3, x19"),
        ];
        code.extend(arm64_dispatch_on("x2", 0x114));
        let statements = lift_dispatch(Abi::Arm64V8a, &code, &symbols, &[], &table);
        let resolved = resolved_targets(&statements);
        assert!(
            resolved.contains(&("Square.area".to_owned(), EvidenceConfidence::High)),
            "{resolved:?}"
        );
        assert!(!resolved.iter().any(|(target, _)| target == "Circle.area"));
    }

    #[test]
    fn declared_receiver_types_do_not_prove_the_runtime_class() {
        let fixture = DispatchFixture::new();
        let table = fixture.analysis(4096);
        // A parameter declared as Circle may hold any subclass at runtime.
        let hints = [ParameterHint {
            name: "shape".to_owned(),
            class_name: Some("Circle".to_owned()),
            class_library_uri: Some("package:app/shapes.dart".to_owned()),
        }];
        let code = arm64_dispatch_on("x1", 0x100);
        let statements = lift_dispatch(Abi::Arm64V8a, &code, &BTreeMap::new(), &hints, &table);
        assert!(
            !resolved_targets(&statements)
                .iter()
                .any(|(target, _)| target.ends_with(".area")),
            "{statements:?}"
        );
    }

    #[test]
    fn exact_receiver_without_its_own_slot_stays_unresolved() {
        let mut fixture = DispatchFixture::new();
        // Square's slot is a hole in this packed row; the neighbouring
        // Circle slot must not stand in for it.
        fixture.targets[111] = None;
        let table = fixture.analysis(4096);
        let symbols = BTreeMap::from([(0x900, allocation_symbol("Square"))]);
        let mut code = vec![insn(0x100, "bl #0x900"), insn(0x104, "mov x2, x0")];
        code.extend(arm64_dispatch_on("x2", 0x108));
        let statements = lift_dispatch(Abi::Arm64V8a, &code, &symbols, &[], &table);
        assert!(
            resolved_targets(&statements)
                .iter()
                .all(|(target, _)| !target.ends_with(".area"))
        );
    }

    #[test]
    fn decodes_x64_and_split_arm32_dispatch_calls() {
        let fixture = DispatchFixture::new();
        // x64 origin 16: selector 100 is displacement (100 - 16) * 8.
        let x64 = vec![
            insn(0x100, "mov ecx, dword ptr [rdi - 1]"),
            insn(0x104, "shr ecx, 0xc"),
            insn(0x108, "mov rax, qword ptr [r14 + 0x70]"),
            insn(0x10c, "call qword ptr [rax + rcx*8 + 0x2a0]"),
        ];
        let calls = recover_dispatch_calls(Abi::X86_64, &x64, &fixture.analysis(16));
        assert_eq!(
            calls.get(&0x10c).map(|call| call.selector_offset),
            Some(100)
        );
        // ARM32 origin 1023: selector 5000 is (5000 - 1023) * 4 = 0x3e24,
        // split into `add lr, lr, #0x3000` and a 0xe24 load displacement.
        let arm32 = vec![
            insn(0x100, "add lr, r7, r0, lsl #2"),
            insn(0x104, "add lr, lr, #0x3000"),
            insn(0x108, "ldr lr, [lr, #0xe24]"),
            insn(0x10c, "blx lr"),
        ];
        let calls = recover_dispatch_calls(Abi::ArmeabiV7a, &arm32, &fixture.analysis(1023));
        assert_eq!(
            calls.get(&0x10c).map(|call| call.selector_offset),
            Some(5000)
        );
    }

    /// ARM32 `LoadTaggedClassIdMayBeSmi` of `r1` into `r0` followed by a
    /// dispatch through selector row 100 (origin 1023, displacement
    /// (100 - 1023) * 4 = -0xe6c).
    fn arm32_dispatch_on_r1(first: u64) -> Vec<DecodedInstruction> {
        vec![
            insn(first, "tst r1, #1"),
            insn(first + 4, "ldrne r0, [r1, #-1]"),
            insn(first + 8, "ubfxne r0, r0, #0xc, #0x14"),
            insn(first + 12, "moveq r0, #0x3c"),
            insn(first + 16, "add lr, r7, r0, lsl #2"),
            insn(first + 20, "ldr lr, [lr, #-0xe6c]"),
            insn(first + 24, "blx lr"),
        ]
    }

    #[test]
    fn arm32_predicated_class_id_load_keeps_the_exact_receiver() {
        let fixture = DispatchFixture::new();
        let table = fixture.analysis(1023);
        let symbols = BTreeMap::from([(0x900, allocation_symbol("Square"))]);
        let mut code = vec![insn(0x100, "bl #0x900"), insn(0x104, "mov r1, r0")];
        code.extend(arm32_dispatch_on_r1(0x108));
        let statements = lift_dispatch(Abi::ArmeabiV7a, &code, &symbols, &[], &table);
        assert!(
            resolved_targets(&statements)
                .contains(&("Square.area".to_owned(), EvidenceConfidence::High)),
            "{statements:?}"
        );
    }

    fn shape_hint() -> [ParameterHint; 1] {
        [ParameterHint {
            name: "shape".to_owned(),
            class_name: Some("Shape".to_owned()),
            class_library_uri: Some("package:app/shapes.dart".to_owned()),
        }]
    }

    /// `Shape` (class 9, abstract) has the concrete subtypes Circle (10) and
    /// Square (11); the ARM64 receiver arrives in x1.
    fn with_shape_hierarchy(fixture: &mut DispatchFixture) -> BTreeMap<usize, Vec<usize>> {
        fixture.names.insert("Shape".to_owned(), vec![9]);
        BTreeMap::from([(9, vec![10, 11]), (10, vec![10]), (11, vec![11])])
    }

    #[test]
    fn static_receiver_resolves_when_all_concrete_subtypes_share_one_body() {
        let mut fixture = DispatchFixture::new();
        fixture.targets[110] = Some("Shape.area".to_owned());
        fixture.targets[111] = Some("Shape.area".to_owned());
        let subtypes = with_shape_hierarchy(&mut fixture);
        let mut table = fixture.analysis(4096);
        table.subtype_cids = Some(&subtypes);
        let code = arm64_dispatch_on("x1", 0x100);
        let statements =
            lift_dispatch(Abi::Arm64V8a, &code, &BTreeMap::new(), &shape_hint(), &table);
        assert!(
            resolved_targets(&statements)
                .contains(&("Shape.area".to_owned(), EvidenceConfidence::High)),
            "{statements:?}"
        );
    }

    #[test]
    fn static_receiver_with_overriding_subtypes_stays_unresolved() {
        let mut fixture = DispatchFixture::new();
        let subtypes = with_shape_hierarchy(&mut fixture);
        let mut table = fixture.analysis(4096);
        table.subtype_cids = Some(&subtypes);
        let code = arm64_dispatch_on("x1", 0x100);
        let statements =
            lift_dispatch(Abi::Arm64V8a, &code, &BTreeMap::new(), &shape_hint(), &table);
        assert!(
            !resolved_targets(&statements)
                .iter()
                .any(|(target, _)| target.ends_with(".area")),
            "{statements:?}"
        );
    }

    #[test]
    fn static_receiver_rejects_a_slot_owned_by_an_unrelated_class() {
        let mut fixture = DispatchFixture::new();
        fixture.targets[110] = Some("Shape.area".to_owned());
        fixture.targets[111] = Some("Shape.area".to_owned());
        let subtypes = with_shape_hierarchy(&mut fixture);
        // Both slots hold one body owned by class 50, which is not a
        // superclass of Circle or Square: the slots belong to another row.
        let mut owners = vec![None; 200];
        owners[110] = Some(50);
        owners[111] = Some(50);
        let supers = BTreeMap::from([(10, 9), (11, 9)]);
        let mut table = fixture.analysis(4096);
        table.subtype_cids = Some(&subtypes);
        table.super_cids = Some(&supers);
        table.target_owner_cids = &owners;
        let code = arm64_dispatch_on("x1", 0x100);
        let statements =
            lift_dispatch(Abi::Arm64V8a, &code, &BTreeMap::new(), &shape_hint(), &table);
        assert!(
            !resolved_targets(&statements)
                .iter()
                .any(|(target, _)| target.ends_with(".area")),
            "{statements:?}"
        );
    }

    #[test]
    fn late_field_init_stubs_read_the_field() {
        let args = |values: &[&str]| {
            values
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            late_field_read("stub InitLateFinalInstanceField", &args(&["this", "Cart.total"])),
            Some("this.total".to_owned())
        );
        assert_eq!(
            late_field_read("stub InitLateStaticField", &args(&["r1", "Config.instance"])),
            Some("Config.instance".to_owned())
        );
        // The field operand must be a retained Field label, not a value.
        assert_eq!(
            late_field_read("stub InitLateStaticField", &args(&["sub_1234_result"])),
            None
        );
        assert_eq!(
            late_field_read("stub InitLateFinalInstanceField", &args(&["this", "r2"])),
            None
        );
        assert_eq!(late_field_read("stub AllocateArray", &args(&["Cart.total"])), None);
    }

    fn decoded(address: u64, mnemonic: &str, operands: &str) -> DecodedInstruction {
        DecodedInstruction {
            address,
            next: address + 4,
            mnemonic: mnemonic.to_owned(),
            operands: operands.to_owned(),
        }
    }

    fn static_layout() -> RecoveredFieldLayout {
        let mut layout = RecoveredFieldLayout::default();
        for (id, owner, name) in [
            (307, "PlatformDispatcher", "_instance"),
            (212, "Zone", "_current"),
        ] {
            layout.insert_static(
                false,
                id * 8,
                StaticFieldIdentity {
                    name: format!("{owner}.{name}"),
                    owner: Some(owner.to_owned()),
                    value_class: Some(owner.to_owned()),
                    value_library_uri: None,
                },
            );
        }
        layout.with_exact_thread_layout(Abi::Arm64V8a, Some("dart-3.12.2"))
    }

    fn lift_static(
        instructions: &[DecodedInstruction],
        blocks: &[u64],
        symbols: &BTreeMap<u64, Symbol>,
        layout: &RecoveredFieldLayout,
    ) -> Vec<SemanticStatement> {
        lift_semantics_with_names(
            Abi::Arm64V8a,
            &[],
            instructions,
            &blocks.iter().copied().collect(),
            symbols,
            None,
            Some(layout),
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn lazy_static_initialization_keeps_the_field_value() {
        let instructions = [
            decoded(0x1000, "ldr", "x0, [x26, #0x78]"),
            decoded(0x1004, "ldr", "x0, [x0, #0x998]"),
            decoded(0x1008, "ldr", "x16, [x26, #0x90]"),
            decoded(0x100c, "cmp", "w0, w16"),
            decoded(0x1010, "b.ne", "#0x101c"),
            decoded(0x1014, "ldr", "x2, [x27, #0x290]"),
            decoded(0x1018, "bl", "#0x5000"),
            decoded(0x101c, "ret", ""),
        ];
        let symbols = BTreeMap::from([(
            0x5000,
            Symbol::new("stub InitLateFinalStaticField".to_owned(), None, None),
        )]);
        let statements = lift_static(
            &instructions,
            &[0x1000, 0x1014, 0x101c],
            &symbols,
            &static_layout(),
        );
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::StaticFieldRead { field, field_id: 307, shared: false, .. }
                if field == "PlatformDispatcher._instance"
        )));
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. }
                if expression == "PlatformDispatcher._instance"
        )));
        assert!(!statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::ResolvedCall { .. } | SemanticStatement::Condition { .. }
        )));

        // An unnamed call target could be ordinary Dart code: keep it.
        let unnamed = lift_static(
            &instructions,
            &[0x1000, 0x1014, 0x101c],
            &BTreeMap::new(),
            &static_layout(),
        );
        assert!(
            unnamed
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::ResolvedCall { .. }))
        );
    }

    #[test]
    fn static_field_writes_retire_earlier_reads_of_the_field() {
        let instructions = [
            decoded(0x1000, "ldr", "x0, [x26, #0x78]"),
            decoded(0x1004, "ldr", "x1, [x0, #0x6a0]"),
            decoded(0x1008, "ldr", "x0, [x26, #0x78]"),
            decoded(0x100c, "str", "x2, [x0, #0x6a0]"),
            decoded(0x1010, "mov", "x0, x1"),
            decoded(0x1014, "ret", ""),
        ];
        let statements = lift_static(&instructions, &[0x1000], &BTreeMap::new(), &static_layout());
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::StaticFieldWrite { field, field_id: 212, .. }
                if field == "Zone._current"
        )));
        // `x1` holds the value from before the store, not the current field.
        assert!(!statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "Zone._current"
        )));
    }

    #[test]
    fn static_slots_need_an_exact_thread_layout_and_stay_anonymous_without_a_field() {
        let instructions = [
            decoded(0x1000, "ldr", "x0, [x26, #0x78]"),
            decoded(0x1004, "ldr", "x0, [x0, #0x28]"),
            decoded(0x1008, "ret", ""),
        ];
        let statements = lift_static(&instructions, &[0x1000], &BTreeMap::new(), &static_layout());
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::StaticFieldRead { field, field_id: 5, confidence: EvidenceConfidence::Low, .. }
                if field == "aot.staticField(5)"
        )));

        let unknown_profile = static_layout().with_exact_thread_layout(Abi::Arm64V8a, None);
        let statements = lift_static(&instructions, &[0x1000], &BTreeMap::new(), &unknown_profile);
        assert!(
            !statements
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::StaticFieldRead { .. }))
        );
    }

    #[test]
    fn conflicting_static_field_ids_are_refused() {
        let mut layout = static_layout();
        layout.insert_static(
            false,
            212 * 8,
            StaticFieldIdentity {
                name: "Other._current".to_owned(),
                owner: Some("Other".to_owned()),
                value_class: None,
                value_library_uri: None,
            },
        );
        assert!(layout.static_field(false, 212 * 8).is_none());
        assert!(layout.static_field(false, 307 * 8).is_some());
        assert!(layout.static_field(false, 307 * 8 + 4).is_none());
    }

    fn model_layout() -> RecoveredFieldLayout {
        let library = Some("package:app/model.dart".to_owned());
        let mut layout = RecoveredFieldLayout::default();
        // Base declares `id` at 8; Sub adds `label` at 12. Sub's bitmap
        // knows both slots but not their names.
        layout.insert(
            library.clone(),
            "Base".to_owned(),
            8,
            "id".to_owned(),
            None,
            None,
        );
        layout.insert_anonymous(library.clone(), "Sub".to_owned(), 8);
        layout.insert_anonymous(library.clone(), "Sub".to_owned(), 12);
        layout.superclasses.insert(
            (library.clone(), "Sub".to_owned()),
            (library.clone(), "Base".to_owned()),
        );
        layout
    }

    #[test]
    fn inherited_fields_resolve_through_the_superclass() {
        let layout = model_layout();
        let (offset, identity) = layout
            .field("Sub", Some("package:app/model.dart"), 7)
            .expect("inherited field");
        assert_eq!((offset, identity.name.as_str()), (8, "id"));
        assert!(!identity.synthesized_slot);
        // Sub's own unnamed slot stays anonymous and synthesized.
        let (_, slot) = layout
            .field("Sub", Some("package:app/model.dart"), 11)
            .expect("slot");
        assert!(slot.synthesized_slot);
        assert_eq!(slot.name, "_slot_c");
    }

    #[test]
    fn exact_fields_replace_anonymous_slots_and_are_never_duplicated() {
        let library = Some("package:app/model.dart".to_owned());
        let mut layout = RecoveredFieldLayout::default();
        layout.insert_anonymous(library.clone(), "Point".to_owned(), 8);
        layout.insert(
            library.clone(),
            "Point".to_owned(),
            8,
            "x".to_owned(),
            None,
            None,
        );
        let (_, identity) = layout.field("Point", library.as_deref(), 8).expect("field");
        assert_eq!(identity.name, "x");
        assert!(!identity.synthesized_slot);
        // A second placement of the same name in the same class is rejected.
        layout.insert(
            library.clone(),
            "Point".to_owned(),
            16,
            "x".to_owned(),
            None,
            None,
        );
        assert!(layout.field("Point", library.as_deref(), 16).is_none());
        // And an anonymous slot never replaces an exact field.
        layout.insert_anonymous(library.clone(), "Point".to_owned(), 8);
        assert_eq!(
            layout.field("Point", library.as_deref(), 8).unwrap().1.name,
            "x"
        );
    }
}

#[cfg(test)]
mod pair_and_stub_tests {
    use super::*;

    fn insn(address: u64, text: &str) -> DecodedInstruction {
        let mut parts = text.splitn(2, ' ');
        let mnemonic = parts.next().unwrap().to_owned();
        let operands = parts.next().unwrap_or("").trim().to_owned();
        DecodedInstruction {
            address,
            next: address + 4,
            mnemonic,
            operands,
        }
    }

    fn stub(label: &str) -> Symbol {
        Symbol::new(format!("stub {label}"), None, None)
    }

    /// Lifts ARM32 `code` with `parameters` untyped arguments in r1, r2, ….
    fn lift_arm32(
        code: &[DecodedInstruction],
        blocks: &[u64],
        parameters: usize,
        symbols: &BTreeMap<u64, Symbol>,
        pool: &[&str],
        layout: Option<&RecoveredFieldLayout>,
    ) -> Vec<SemanticStatement> {
        let hints = (0..parameters)
            .map(|index| ParameterHint {
                name: format!("arg{index}"),
                class_name: None,
                class_library_uri: None,
            })
            .collect::<Vec<_>>();
        let pool = pool
            .iter()
            .map(|label| (*label).to_owned())
            .collect::<Vec<_>>();
        lift_semantics_with_names(
            Abi::ArmeabiV7a,
            &hints,
            code,
            &blocks.iter().copied().collect(),
            symbols,
            Some(&pool),
            layout,
            None,
            None,
            None,
            None,
        )
    }

    fn calls(statements: &[SemanticStatement]) -> Vec<(String, Vec<String>)> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::ResolvedCall {
                    target, arguments, ..
                } => Some((target.clone(), arguments.clone())),
                _ => None,
            })
            .collect()
    }

    fn returned(statements: &[SemanticStatement]) -> Vec<String> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Return { expression, .. } => Some(expression.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn reads_arm64_array_elements_through_decompression() {
        // `list[i]` with an untagged index: the element pointer register is
        // reused as the loaded register before decompression.
        let code = [
            insn(0x1000, "add x16, x1, x2, lsl #2"),
            insn(0x1004, "ldur w16, [x16, #0xf]"),
            insn(0x1008, "add x16, x16, x28, lsl #32"),
            insn(0x100c, "mov x0, x16"),
            insn(0x1010, "ret"),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            Some(2),
            &code,
            &BTreeSet::from([0x1000]),
            &BTreeMap::new(),
            None,
        );
        assert_eq!(returned(&statements), vec!["arg0[arg1]".to_owned()]);
    }

    #[test]
    fn reads_and_writes_arm32_array_elements_with_smi_indices() {
        // `a[i - 1]` through a register the load overwrites, then
        // `a[i] = 7` through a fresh pointer.
        let code = [
            insn(0x1000, "lsl r3, r2, #1"),
            insn(0x1004, "add ip, r1, r3, lsl #1"),
            insn(0x1008, "ldr ip, [ip, #7]"),
            insn(0x100c, "add r0, r1, r2, lsl #2"),
            insn(0x1010, "mov r4, #7"),
            insn(0x1014, "str r4, [r0, #0xb]"),
            insn(0x1018, "mov r0, ip"),
            insn(0x101c, "bx lr"),
        ];
        let statements = lift_arm32(&code, &[0x1000], 2, &BTreeMap::new(), &[], None);
        assert_eq!(returned(&statements), vec!["arg0[arg1 - 1]".to_owned()]);
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::FieldWrite { receiver, field, value, .. }
                if receiver == "arg0" && field == "[arg1]" && value == "7"
        )));
    }

    #[test]
    fn reads_x64_scaled_index_elements() {
        let code = [
            insn(0x1000, "mov edi, dword ptr [rdi + rsi*4 + 0x13]"),
            insn(0x1004, "add rdi, qword ptr [r14 + 0x58]"),
            insn(0x1008, "mov rax, rdi"),
            insn(0x100c, "ret"),
        ];
        let statements = lift_semantics(
            Abi::X86_64,
            Some(2),
            &code,
            &BTreeSet::from([0x1000]),
            &BTreeMap::new(),
            None,
        );
        assert_eq!(returned(&statements), vec!["arg0[arg1 + 1]".to_owned()]);
    }

    #[test]
    fn untags_smi_index_texts() {
        let index = |text: &str| Expression {
            text: text.to_owned(),
            confidence: EvidenceConfidence::Low,
            complexity: 1,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        };
        assert_eq!(untagged_index(index("(i << 1)")).text, "i");
        assert_eq!(untagged_index(index("((i << 1) + 8)")).text, "(i + 4)");
        assert_eq!(untagged_index(index("((a + b) << 1)")).text, "(a + b)");
        assert_eq!(untagged_index(index("(a + b << 1)")).text, "(a + b << 1)");
        assert_eq!(untagged_index(index("merged1")).text, "merged1");
    }

    #[test]
    fn untracked_spills_overwrite_stale_frame_slots() {
        // `clz` is not modeled, so the second spill stores an unknown value;
        // the reload must not resurrect the first one.
        let code = [
            insn(0x1000, "mov r4, #6"),
            insn(0x1004, "str r4, [fp, #-8]"),
            insn(0x1008, "clz r4, r1"),
            insn(0x100c, "str r4, [fp, #-8]"),
            insn(0x1010, "ldr r0, [fp, #-8]"),
            insn(0x1014, "bx lr"),
        ];
        let statements = lift_arm32(&code, &[0x1000], 1, &BTreeMap::new(), &[], None);
        assert!(!statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "6"
        )));
    }

    #[test]
    fn fuses_x64_box_int64_into_the_boxed_value() {
        let code = [
            insn(0x1000, "mov rcx, rdi"),
            insn(0x1004, "lea rax, [rcx + rcx]"),
            insn(0x1008, "mov rbx, rcx"),
            insn(0x100c, "sar rbx, 0x1e"),
            insn(0x1010, "add rbx, 1"),
            insn(0x1014, "cmp rbx, 2"),
            insn(0x1018, "jb 0x1024"),
            insn(0x101c, "call 0x6000"),
            insn(0x1020, "mov qword ptr [rax + 7], rcx"),
            insn(0x1024, "ret"),
        ];
        let symbols = BTreeMap::from([(0x6000, stub("AllocateMintSharedWithoutFPURegs"))]);
        let statements = lift_semantics(
            Abi::X86_64,
            Some(1),
            &code,
            &BTreeSet::from([0x1000, 0x101c, 0x1024]),
            &symbols,
            None,
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::Return { expression, .. } if expression == "arg0"
            )),
            "{statements:#?}"
        );
    }

    #[test]
    fn string_constants_cannot_forge_ic_selectors() {
        let expression = |text: &str| Expression {
            text: text.to_owned(),
            confidence: EvidenceConfidence::High,
            complexity: 1,
            class_name: None,
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        };
        // A string label whose text only mentions `dynamicCall(`.
        let forged = BTreeMap::from([(
            "x2".to_owned(),
            expression("\"dynamicCall('delete')\""),
        )]);
        assert_eq!(find_ic_selector(&forged), None);
        let genuine = BTreeMap::from([(
            "x5".to_owned(),
            expression("dynamicCall(\"dyn:[]\", typeArgs=0, count=2, positional=2)"),
        )]);
        assert_eq!(find_ic_selector(&genuine).as_deref(), Some("dyn:[]"));
    }

    #[test]
    fn comparisons_split_on_a_separator_strings_cannot_contain() {
        let pending = pending_comparison("\"a ? b\"", "x0");
        assert_eq!(
            pending.split_once(COMPARISON_SEPARATOR),
            Some(("\"a ? b\"", "x0"))
        );
        assert_eq!(
            comparison_text("a == b", "!=", "c == d"),
            "(a == b) != (c == d)"
        );
        assert_eq!(comparison_text("(a == b)", "!=", "c"), "(a == b) != c");
        assert_eq!(comparison_text("x + 1", "<", "y"), "x + 1 < y");
    }

    #[test]
    fn pool_values_drop_nested_string_evidence() {
        assert_eq!(
            pool_value_text("snapshotRef(7) nestedStrings[\"'); evil(); ('\"]"),
            "snapshotRef(7)"
        );
        assert_eq!(pool_value_text("\"plain\""), "\"plain\"");
    }

    #[test]
    fn application_classes_named_context_keep_their_slots() {
        let application = Expression {
            text: "contextResult".to_owned(),
            confidence: EvidenceConfidence::Low,
            complexity: 1,
            class_name: Some("Context".to_owned()),
            class_library_uri: None,
            raw: false,
            definition_site: None,
            high_word: false,
            exact_class: false,
        };
        let (_, field) = recovered_field_or_slot(None, &application, 0xb, Abi::ArmeabiV7a).unwrap();
        assert_eq!(field.name, "_slot_c");
        let vm_context = Expression {
            class_library_uri: Some("dart:core".to_owned()),
            ..application
        };
        let (_, field) = recovered_field_or_slot(None, &vm_context, 0xb, Abi::ArmeabiV7a).unwrap();
        assert_eq!(field.name, "captured0");
    }

    #[test]
    fn names_closure_context_slots_from_the_allocation_stub() {
        let code = [
            insn(0x1000, "mov r1, #1"),
            insn(0x1004, "bl #0x6000"),
            insn(0x1008, "mov r2, #6"),
            insn(0x100c, "str r2, [r0, #0xb]"),
            insn(0x1010, "ldr r3, [r0, #7]"),
            insn(0x1014, "str r3, [r0, #0xf]"),
            insn(0x1018, "bx lr"),
        ];
        let symbols = BTreeMap::from([(0x6000, stub("AllocateContext"))]);
        let statements = lift_arm32(&code, &[0x1000], 0, &symbols, &[], None);

        assert_eq!(
            calls(&statements),
            vec![("stub AllocateContext".to_owned(), vec!["1".to_owned()])]
        );
        let writes = statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::FieldWrite {
                    receiver,
                    field,
                    value,
                    ..
                } => Some((receiver.as_str(), field.as_str(), value.as_str())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            writes,
            vec![
                ("stub_AllocateContext_1004_result", "captured0", "6"),
                (
                    "stub_AllocateContext_1004_result",
                    "captured1",
                    "stub_AllocateContext_1004_result.parent"
                ),
            ]
        );
    }

    #[test]
    fn calls_pool_stubs_through_their_code_entry_point() {
        // `InstanceOf` lives in the VM snapshot, so AOT code loads its Code
        // object from the pool and calls `Code::entry_point_`.
        let code = [
            insn(0x1000, "mov r0, r1"),
            insn(0x1004, "ldr r8, [r5, #0xf]"),
            insn(0x1008, "ldr lr, [r5, #0xb]"),
            insn(0x100c, "ldr lr, [lr, #3]"),
            insn(0x1010, "blx lr"),
            insn(0x1014, "bx lr"),
        ];
        let statements = lift_arm32(
            &code,
            &[0x1000],
            1,
            &BTreeMap::new(),
            &["null", "stub InstanceOf", "String"],
            None,
        );

        assert_eq!(
            calls(&statements),
            vec![(
                "stub InstanceOf".to_owned(),
                vec!["arg0".to_owned(), "String".to_owned()]
            )]
        );
        assert!(!statements
            .iter()
            .any(|statement| matches!(statement, SemanticStatement::FieldRead { .. })));
    }

    #[test]
    fn keeps_unresolved_dispatch_calls_with_their_receiver_and_result() {
        // `r3` holds a value computed for later use, not an argument: only
        // registers the call's block writes after the receiver count.
        let code = [
            insn(0x1000, "ldr r3, [r1, #7]"),
            insn(0x1004, "b #0x1008"),
            insn(0x1008, "ldr r0, [r2, #-1]"),
            insn(0x100c, "ubfx r0, r0, #0xc, #0x14"),
            insn(0x1010, "mov r1, r2"),
            insn(0x1014, "add lr, r7, r0, lsl #2"),
            insn(0x1018, "ldr lr, [lr, #-0x10]"),
            insn(0x101c, "blx lr"),
            insn(0x1020, "bx lr"),
        ];
        let hints = [0, 1].map(|index| ParameterHint {
            name: format!("arg{index}"),
            class_name: None,
            class_library_uri: None,
        });
        let dispatch = BTreeMap::from([(
            0x101c,
            DispatchCallEvidence {
                selector_offset: 1019,
                selector_name: None,
                candidate_targets: Vec::new(),
                candidate_count: 0,
                raw_slot_target_count: 0,
            },
        )]);
        let statements = lift_semantics_with_names(
            Abi::ArmeabiV7a,
            &hints,
            &code,
            &BTreeSet::from([0x1000, 0x1008]),
            &BTreeMap::new(),
            None,
            None,
            None,
            None,
            Some(&dispatch),
            None,
        );

        assert_eq!(
            calls(&statements),
            vec![("dispatch 1019".to_owned(), vec!["arg1".to_owned()])]
        );
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Return { expression, .. } if expression == "dispatch_1019_result"
        )));
    }

    fn returns(statements: &[SemanticStatement]) -> Vec<String> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Return { expression, .. } => Some(expression.clone()),
                _ => None,
            })
            .collect()
    }

    fn conditions(statements: &[SemanticStatement]) -> Vec<String> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                SemanticStatement::Condition { expression, .. } => Some(expression.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn int64_unboxing_passes_one_value_for_a_register_pair() {
        // Smi-or-Mint unboxing into r4:r3, then the pair passed in r2:r3.
        let code = [
            insn(0x100, "ldr r2, [r1, #7]"),
            insn(0x104, "asr r3, r2, #0x1f"),
            insn(0x108, "asrs r4, r2, #1"),
            insn(0x10c, "blo #0x118"),
            insn(0x110, "ldr r4, [r2, #7]"),
            insn(0x114, "ldr r3, [r2, #0xb]"),
            insn(0x118, "mov r2, r4"),
            insn(0x11c, "bl #0x5000"),
            insn(0x120, "bx lr"),
        ];
        let statements = lift_arm32(
            &code,
            &[0x100, 0x110, 0x118, 0x120],
            1,
            &BTreeMap::new(),
            &[],
            None,
        );
        assert_eq!(
            calls(&statements),
            [("sub_5000".to_owned(), vec!["arg0._slot_8".to_owned()])]
        );
        assert!(conditions(&statements).is_empty(), "{statements:?}");
    }

    #[test]
    fn int64_arithmetic_and_boxing_keep_the_source_value() {
        let code = [
            insn(0x100, "asr r3, r1, #0x1f"),
            insn(0x104, "asrs r2, r1, #1"),
            insn(0x108, "adds r4, r2, #1"),
            insn(0x10c, "adc r6, r3, #0"),
            insn(0x110, "lsl r0, r4, #1"),
            insn(0x114, "cmp r4, r0, asr #1"),
            insn(0x118, "cmpeq r6, r0, asr #31"),
            insn(0x11c, "beq #0x12c"),
            insn(0x120, "bl #0x6000"),
            insn(0x124, "str r4, [r0, #7]"),
            insn(0x128, "str r6, [r0, #0xb]"),
            insn(0x12c, "bx lr"),
        ];
        let symbols = BTreeMap::from([(0x6000, stub("AllocateMintSharedWithoutFPURegs"))]);
        let statements = lift_arm32(&code, &[0x100, 0x120, 0x12c], 1, &symbols, &[], None);
        assert_eq!(returns(&statements), ["(arg0 + 1)"]);
        assert!(conditions(&statements).is_empty(), "{statements:?}");
        assert!(
            !statements
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::FieldWrite { .. }))
        );
    }

    #[test]
    fn int64_comparison_towers_become_one_signed_comparison() {
        let code = [
            insn(0x100, "asr r3, r1, #0x1f"),
            insn(0x104, "asr r4, r2, #0x1f"),
            insn(0x108, "cmp r3, r4"),
            insn(0x10c, "blt #0x11c"),
            insn(0x110, "bgt #0x128"),
            insn(0x114, "cmp r1, r2"),
            insn(0x118, "bhs #0x128"),
            insn(0x11c, "mov r0, #1"),
            insn(0x120, "bx lr"),
            insn(0x128, "mov r0, #0"),
            insn(0x12c, "bx lr"),
        ];
        let fused = fuse_machine_idioms(Abi::ArmeabiV7a, &code, &BTreeMap::new());
        let branches = fused
            .iter()
            .filter(|instruction| branch_kind(&instruction.mnemonic) == Some(true))
            .map(|instruction| (instruction.mnemonic.as_str(), instruction.operands.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(branches, [("bge", "#0x128")]);
        let statements = lift_arm32(
            &code,
            &[0x100, 0x110, 0x114, 0x11c, 0x128],
            2,
            &BTreeMap::new(),
            &[],
            None,
        );
        assert_eq!(conditions(&statements), ["arg0 >= arg1"]);

        // A tower whose branches disagree with a signed comparison stays.
        let mut inconsistent = code.to_vec();
        inconsistent[6] = insn(0x118, "blo #0x128");
        let fused = fuse_machine_idioms(Abi::ArmeabiV7a, &inconsistent, &BTreeMap::new());
        assert_eq!(
            fused
                .iter()
                .filter(|instruction| branch_kind(&instruction.mnemonic) == Some(true))
                .count(),
            3
        );
    }

    #[test]
    fn throw_and_closure_stubs_lift_to_their_dart_meaning() {
        let code = [
            insn(0x100, "mov r2, r1"),
            insn(0x104, "ldr r1, [r5, #0x13]"),
            insn(0x108, "bl #0x7000"),
            insn(0x10c, "mov r1, r0"),
            insn(0x110, "bl #0x8000"),
            insn(0x114, "ldr r0, [r5, #0x17]"),
            insn(0x118, "bl #0x9000"),
            insn(0x11c, "bkpt #0"),
        ];
        let symbols = BTreeMap::from([(0x7000, stub("AllocateClosure")), (0x9000, stub("Throw"))]);
        let pool = ["", "", "", "Foo.<anonymous closure>", "Bar.error"];
        let statements = lift_arm32(&code, &[0x100], 1, &symbols, &pool, None);
        assert_eq!(
            calls(&statements),
            [(
                "sub_8000".to_owned(),
                vec!["aot.closure('Foo.<anonymous closure>', arg0)".to_owned()]
            )]
        );
        assert!(statements.iter().any(|statement| matches!(
            statement,
            SemanticStatement::Throw { expression, stack_trace: None, .. }
                if expression == "Bar.error"
        )));
    }

    #[test]
    fn a_new_closure_reads_back_its_captured_context() {
        let code = [
            insn(0x100, "mov r2, r1"),
            insn(0x104, "ldr r1, [r5, #0x13]"),
            insn(0x108, "bl #0x7000"),
            insn(0x10c, "ldr r1, [r0, #0x13]"),
            insn(0x110, "bl #0x8000"),
            insn(0x114, "bx lr"),
        ];
        let symbols = BTreeMap::from([(0x7000, stub("AllocateClosure"))]);
        let pool = ["", "", "", "Foo.<anonymous closure>"];
        let statements = lift_arm32(&code, &[0x100], 1, &symbols, &pool, None);
        assert_eq!(
            calls(&statements),
            [("sub_8000".to_owned(), vec!["arg0".to_owned()])]
        );
    }

    #[test]
    fn out_of_line_box_stubs_hold_the_stored_payload() {
        let code = [
            insn(0x100, "bl #0x6000"),
            insn(0x104, "add ip, r0, #3"),
            insn(0x108, "vstr d0, [ip, #4]"),
            insn(0x10c, "bx lr"),
        ];
        let symbols = BTreeMap::from([(0x6000, stub("AllocateDouble"))]);
        let mut hints = BTreeMap::new();
        hints.insert(
            "d0".to_owned(),
            Expression {
                text: "ratio".to_owned(),
                confidence: EvidenceConfidence::High,
                complexity: 1,
                class_name: Some("double".to_owned()),
                class_library_uri: None,
                raw: false,
                definition_site: None,
                high_word: false,
                exact_class: false,
            },
        );
        let fused = fuse_machine_idioms(Abi::ArmeabiV7a, &code, &symbols);
        let blocks = LifterBlocks::build(&fused, &BTreeSet::from([0x100]));
        let mut state = FlowState {
            registers: hints,
            ..FlowState::default()
        };
        let mut statements = Vec::new();
        simulate_range(
            &mut state,
            &fused,
            Some(blocks.instruction_start(0)..blocks.instruction_end(0)),
            Abi::ArmeabiV7a,
            &symbols,
            None,
            &BTreeMap::new(),
            None,
            Some(&mut statements),
            LiftContext::bare(),
            false,
        );
        assert_eq!(returns(&statements), ["ratio"]);
        assert!(calls(&statements).is_empty());
    }

    #[test]
    fn late_field_init_types_its_receiver_for_every_field_access() {
        // `arg0` has no declared type; the late field's Field names its
        // class, which then names another field read through `arg0`.
        let code = [
            insn(0xfc, "str r1, [fp, #-4]"),
            insn(0x100, "ldr r0, [r1, #0xc7]"),
            insn(0x104, "ldr ip, [r5, #0x1f]"),
            insn(0x108, "cmp r0, ip"),
            insn(0x10c, "bne #0x118"),
            insn(0x110, "ldr r2, [r5, #0x13]"),
            insn(0x114, "bl #0x5000"),
            insn(0x118, "ldr r1, [fp, #-4]"),
            insn(0x11c, "ldr r0, [r1, #0xcb]"),
            insn(0x120, "bx lr"),
        ];
        let symbols = BTreeMap::from([(0x5000, stub("InitLateFinalInstanceField"))]);
        let pool = ["", "", "", "KZ.RSc", "", "", "uninitializedSentinel"];
        let mut layout = RecoveredFieldLayout::default();
        layout.insert(None, "KZ".to_owned(), 0xc8, "RSc".to_owned(), None, None);
        layout.insert(None, "KZ".to_owned(), 0xcc, "count".to_owned(), None, None);
        let statements = lift_arm32(
            &code,
            &[0xfc, 0x110, 0x118],
            1,
            &symbols,
            &pool,
            Some(&layout),
        );
        assert_eq!(returns(&statements), ["arg0.count"], "{statements:#?}");
        assert!(conditions(&statements).is_empty(), "{statements:?}");

        // Without the Field label nothing proves the receiver's class.
        let unnamed = lift_arm32(
            &code,
            &[0xfc, 0x110, 0x118],
            1,
            &symbols,
            &["", "", "", "", "", "", "uninitializedSentinel"],
            Some(&layout),
        );
        assert_eq!(returns(&unnamed), ["arg0._slot_cc"]);
    }

    #[test]
    fn interprocedural_classes_need_agreeing_direct_callers() {
        let foo = || Some(("Foo".to_owned(), None));
        let caller = |arguments: Vec<(u64, usize, Option<(String, Option<String>)>)>| LiftFacts {
            arguments,
            ..LiftFacts::default()
        };
        let facts = vec![
            (
                0x10,
                false,
                caller(vec![(0x100, 0, foo()), (0x200, 0, foo())]),
            ),
            (
                0x20,
                false,
                caller(vec![(0x100, 0, foo()), (0x200, 0, None), (0x300, 0, foo())]),
            ),
            (
                0x30,
                false,
                caller(vec![
                    (0x200, 0, None),
                    (0x300, 0, Some(("Bar".to_owned(), None))),
                ]),
            ),
            (
                0x100,
                false,
                LiftFacts {
                    returns: vec![foo(), Some(("Null".to_owned(), None))],
                    ..LiftFacts::default()
                },
            ),
            (0x200, false, LiftFacts::default()),
            (
                0x300,
                false,
                LiftFacts {
                    returns: vec![foo(), None],
                    ..LiftFacts::default()
                },
            ),
            (0x400, true, LiftFacts::default()),
        ];
        let inferred = infer_interprocedural_classes(&facts);
        // 0x100: two agreeing typed callers. 0x200: more untyped than typed.
        // 0x300: conflicting classes.
        assert_eq!(
            inferred.parameters.keys().copied().collect::<Vec<_>>(),
            [0x100]
        );
        assert_eq!(
            inferred.results.keys().copied().collect::<Vec<_>>(),
            [0x100]
        );

        // A body also reachable indirectly keeps its parameters untyped.
        let indirect = vec![
            (0x10, false, caller(vec![(0x400, 0, foo())])),
            (0x400, true, LiftFacts::default()),
        ];
        assert!(
            infer_interprocedural_classes(&indirect)
                .parameters
                .is_empty()
        );
    }

    #[test]
    fn dispatch_results_take_the_target_declared_result_class() {
        let results = BTreeMap::from([(
            "Foo.build".to_owned(),
            (
                "Widget".to_owned(),
                Some("package:flutter/widgets.dart".to_owned()),
            ),
        )]);
        let table = DispatchTableAnalysis {
            origin_element: 0,
            targets: &[],
            class_ids: &[],
            cid_to_name: None,
            name_to_cids: None,
            qualified_to_cids: None,
            super_cids: None,
            target_owner_cids: &[],
            subtype_cids: None,
            label_results: Some(&results),
        };
        assert_eq!(
            resolved_result_class(Some(&table), "Foo.build"),
            (
                Some("Widget".to_owned()),
                Some("package:flutter/widgets.dart".to_owned())
            )
        );
        assert_eq!(
            resolved_result_class(Some(&table), "Foo.other"),
            (None, None)
        );
    }
}
