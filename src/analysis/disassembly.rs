use std::collections::{BTreeMap, BTreeSet};

use capstone::prelude::*;

use crate::diagnostic::{ClutterError, Result};
use crate::model::{
    Abi, CallTargetScope, ControlFlowEdge, ControlFlowEdgeKind, DirectCallResolution,
    EvidenceConfidence, MachineCodeEvidence, MachineInstruction, PseudoStatement,
    SemanticStatement,
};

const MAX_RENDERED_INSTRUCTIONS: usize = 80;

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
}

pub struct Disassembly {
    pub statements: Vec<PseudoStatement>,
    pub evidence: MachineCodeEvidence,
    pub instructions: Vec<MachineInstruction>,
    pub control_flow: Vec<ControlFlowEdge>,
    pub semantic_statements: Vec<SemanticStatement>,
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
            return Some((
                conditional_mnemonic(name, condition),
                format!("d{d}, d{m}"),
            ));
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
            "eq", "ne", "cs", "cc", "mi", "pl", "vs", "vc", "hi", "ls", "ge",
            "lt", "gt", "le", "al",
        ];
        match usize::try_from(condition).ok().and_then(|index| CONDITIONS.get(index)) {
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
            let pool_value = pool_index
                .and_then(|index| object_pool.and_then(|pool| pool.get(index)))
                .cloned();
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
                let symbol = symbols.get(&target_address);
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
                let called_entry =
                    pool_registers.get(&normalize_register(operands)).cloned();
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
                    Some((index, target)) if is_named_pool_target(&target) => {
                        Some((index, target))
                    }
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
        if let Some(dispatch_table) = dispatch_table {
            let dispatch_calls = recover_dispatch_calls(self.abi, &decoded, dispatch_table);
            evidence.dispatch_table_calls = dispatch_calls.len();
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
                let Some(call) = dispatch_calls.get(&call_address) else {
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
        }
        let semantic_statements = lift_semantics(
            self.abi,
            parameter_count,
            &decoded,
            &block_starts,
            symbols,
            object_pool,
        );
        evidence.semantic_statements = semantic_statements.len();
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
) -> Vec<SemanticStatement> {
    let decoded = function
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
        return function.semantic_statements.clone();
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
    lift_semantics_with_names(
        abi,
        parameter_hints,
        &decoded,
        &block_starts,
        symbols,
        object_pool.as_deref(),
        field_layout,
        receiver_class,
    )
}

#[derive(Clone, Debug)]
struct RecoveredFieldIdentity {
    name: String,
    value_class: Option<String>,
    value_library_uri: Option<String>,
    /// True when the field name was not recovered and the access is a
    /// slot-offset placeholder for a proven receiver class.
    synthesized_slot: bool,
}

/// Per-class declared fields in encounter order.
type ClassFields =
    BTreeMap<(Option<String>, String), Vec<(String, Option<crate::model::RecoveredType>)>>;

/// VM-verified instance layouts keyed by declaring class and byte offset.
///
/// Field offsets are only meaningful inside a particular class. Keeping them
/// globally keyed by offset can silently turn an Array store at `+0x10` into a
/// completely unrelated application field. This index deliberately requires a
/// receiver-class proof before assigning a field name.
#[derive(Clone, Debug, Default)]
pub(crate) struct RecoveredFieldLayout {
    fields: BTreeMap<(Option<String>, String, i64), RecoveredFieldIdentity>,
}

impl RecoveredFieldLayout {
    /// Builds the layout index from recovered Field declarations and class
    /// instance slots. Accepts declarations from every scope: field names are
    /// only promoted when the receiver class is proven, so out-of-scope
    /// Flutter/Dart SDK layouts are safe enrichment evidence.
    ///
    /// When neither the VM oracle nor split-debug evidence supplied exact
    /// offsets, remaining fields receive offsets synthesized from Dart's
    /// deterministic instance layout: header (+ optional type-arguments
    /// slot), then declaration-order fields sized by their declared type
    /// (references 4 bytes compressed, unboxed doubles 8-byte aligned).
    pub(crate) fn from_declarations(
        abi: Abi,
        declarations: &[crate::model::RecoveredDeclaration],
    ) -> Self {
        use crate::model::{RecoveredDeclarationKind, RecoveredFunctionKind};
        let mut layouts = Self::default();
        // Exact evidence first; synthesized offsets below never overwrite it.
        for declaration in declarations {
            match declaration.kind {
                RecoveredDeclarationKind::Field => {
                    let (Some(owner), Some(offset)) = (
                        declaration.owner.as_deref(),
                        declaration
                            .field_metadata
                            .as_ref()
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
                    layouts.insert(
                        declaration.library_uri.clone(),
                        crate::analysis::readable_snapshot_name(owner),
                        offset,
                        crate::analysis::readable_snapshot_name(&declaration.name),
                        value_class,
                        value_library_uri,
                    );
                }
                RecoveredDeclarationKind::Class => {
                    let Some(metadata) = declaration.class_metadata.as_ref() else {
                        continue;
                    };
                    let class_name = crate::analysis::readable_snapshot_name(&declaration.name);
                    for slot in &metadata.instance_slots {
                        if slot.slot_type == "type_arguments_field" {
                            continue;
                        }
                        let name = slot.field_name.clone().unwrap_or_else(|| {
                            format!("_slot_{:x}", u64::try_from(slot.offset).unwrap_or_default())
                        });
                        layouts.insert(
                            declaration.library_uri.clone(),
                            class_name.clone(),
                            slot.offset,
                            crate::analysis::readable_snapshot_name(&name),
                            None,
                            None,
                        );
                    }
                }
                RecoveredDeclarationKind::Function
                    if declaration.function_kind == Some(RecoveredFunctionKind::Constructor) =>
                {
                    // Constructors carry no layout evidence.
                }
                RecoveredDeclarationKind::Function => {}
            }
        }
        layouts.synthesize_offsets(abi, declarations);
        layouts
    }

    /// Fills missing instance-field offsets using Dart's deterministic
    /// layout rules for compressed-pointer targets: header, an optional
    /// type-arguments slot for generic classes, then declaration-order
    /// fields (4-byte compressed references; unboxed doubles 8-byte
    /// aligned). Only fills offsets that lack exact evidence.
    fn synthesize_offsets(
        &mut self,
        abi: Abi,
        declarations: &[crate::model::RecoveredDeclaration],
    ) {
        use crate::model::{RecoveredDeclarationKind, RecoveredType};

        let (header_bytes, reference_size) = match abi {
            Abi::Arm64V8a => (8i64, 4i64),
            // ARM32 keeps a 1-word header and 4-byte compressed references.
            Abi::ArmeabiV7a => (4, 4),
            Abi::X86_64 => (16, 8),
        };

        // Encounter-order class list with per-class instance fields.
        let mut classes: Vec<(Option<String>, String)> = Vec::new();
        let mut generic_classes = BTreeSet::<(Option<String>, String)>::new();
        let mut fields_by_class = ClassFields::default();
        for declaration in declarations {
            match declaration.kind {
                RecoveredDeclarationKind::Class => {
                    let Some(owner) = declaration.owner.as_deref() else {
                        continue;
                    };
                    if matches!(owner, "::" | "top_level") {
                        continue;
                    }
                    let key = (
                        declaration.library_uri.clone(),
                        crate::analysis::readable_snapshot_name(owner),
                    );
                    if !fields_by_class.contains_key(&key) {
                        classes.push(key.clone());
                    }
                    if declaration
                        .class_metadata
                        .as_ref()
                        .is_some_and(|metadata| !metadata.type_parameters.is_empty())
                    {
                        generic_classes.insert(key.clone());
                    }
                }
                RecoveredDeclarationKind::Field => {
                    let Some(owner) = declaration.owner.as_deref() else {
                        continue;
                    };
                    if matches!(owner, "::" | "top_level") {
                        continue;
                    }
                    let is_static = declaration
                        .field_metadata
                        .as_ref()
                        .is_some_and(|metadata| metadata.is_static);
                    if is_static {
                        continue;
                    }
                    let key = (
                        declaration.library_uri.clone(),
                        crate::analysis::readable_snapshot_name(owner),
                    );
                    let declared_type = declaration
                        .field_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.declared_type.clone());
                    let name = crate::analysis::readable_snapshot_name(&declaration.name);
                    fields_by_class
                        .entry(key)
                        .or_default()
                        .push((name, declared_type));
                }
                _ => {}
            }
        }

        fn unboxed_double_size(declared: Option<&RecoveredType>) -> bool {
            declared
                .map(|value| {
                    value
                        .display_name
                        .split('<')
                        .next()
                        .unwrap_or_default()
                        .trim()
                        == "double"
                })
                .unwrap_or(false)
        }

        for (library_uri, class_name) in classes {
            let fields = match fields_by_class.get(&(library_uri.clone(), class_name.clone())) {
                Some(fields) if !fields.is_empty() => fields,
                _ => continue,
            };
            let mut offset = header_bytes;
            if generic_classes.contains(&(library_uri.clone(), class_name.clone())) {
                offset += reference_size;
            }
            for (name, declared_type) in fields {
                let size = if unboxed_double_size(declared_type.as_ref()) {
                    let aligned = (offset + 7) / 8 * 8;
                    offset = aligned;
                    8
                } else {
                    reference_size
                };
                let value_class = declared_type
                    .as_ref()
                    .and_then(|value| simple_class_type(&value.display_name));
                let value_library_uri = declared_type
                    .as_ref()
                    .and_then(|value| value.library_uri.clone());
                self.insert(
                    library_uri.clone(),
                    class_name.clone(),
                    offset,
                    name.clone(),
                    value_class,
                    value_library_uri,
                );
                offset += size;
            }
        }
    }

    pub(crate) fn insert(
        &mut self,
        library_uri: Option<String>,
        class_name: String,
        offset: i64,
        field_name: String,
        value_class: Option<String>,
        value_library_uri: Option<String>,
    ) {
        self.fields
            .entry((library_uri, class_name, offset))
            .or_insert_with(|| RecoveredFieldIdentity {
                name: field_name,
                value_class,
                value_library_uri,
                synthesized_slot: false,
            });
    }

    fn field(
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
    if abi == Abi::ArmeabiV7a {
        return recover_arm32_dispatch_calls(instructions, table);
    }
    if abi != Abi::Arm64V8a {
        return BTreeMap::new();
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
                    let raw_targets = table
                        .class_ids
                        .iter()
                        .filter_map(|class_id| selector_offset.checked_add(*class_id))
                        .filter_map(|index| table.targets.get(index))
                        .flatten()
                        .cloned()
                        .collect::<Vec<_>>();
                    let (selector_name, candidate_targets, candidate_count) =
                        infer_dispatch_selector(&raw_targets);
                    recovered.insert(
                        instruction.address,
                        DispatchCallEvidence {
                            selector_offset,
                            selector_name,
                            candidate_targets,
                            candidate_count,
                            raw_slot_target_count: raw_targets.len(),
                        },
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
        .filter_map(|class_id| selector_offset.checked_add(*class_id))
        .filter_map(|index| table.targets.get(index))
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
                let member = target.rsplit_once('.').map_or(target.as_str(), |(_, name)| name);
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
}

impl PoolPointerProvenance {
    fn new(abi: Abi) -> Self {
        Self {
            abi,
            derived_offsets: BTreeMap::new(),
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
            return;
        }
        let operands = split_operands(operands);
        let Some(target) = operands.first().map(|operand| normalize_register(operand)) else {
            return;
        };
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
            if let (Some(source_offset), Some(mut delta)) =
                (source_offset, signed_immediate(&operands[2]))
            {
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
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.confidence == other.confidence
            && self.class_name == other.class_name
            && self.class_library_uri == other.class_library_uri
            && self.raw == other.raw
    }
}

/// An array being filled element-by-element immediately after an allocation
/// stub call. Dart AOT lowers string interpolation to exactly this shape:
/// allocate an array, store literal parts and value expressions into the
/// element slots, then call `String._interpolate`.
#[derive(Clone, Debug, Default, PartialEq)]
struct ElementBuffer {
    parts: Vec<Option<String>>,
}

fn reg_key(register: &str) -> String {
    format!("reg:{register}")
}

fn stack_key(slot: &str) -> String {
    format!("stk:{slot}")
}

/// Register/stack/buffer state carried across instructions. States meet at
/// control-flow joins by intersection: a value survives only when every
/// predecessor agrees on identical provenance.
#[derive(Clone, Debug, Default, PartialEq)]
struct FlowState {
    registers: BTreeMap<String, Expression>,
    stack: BTreeMap<String, Expression>,
    buffers: BTreeMap<String, ElementBuffer>,
    /// Derived pointers into a tracked buffer: register -> (buffer key,
    /// extra byte displacement). Produced by `add dst, arrayBase, #offset`.
    aliases: BTreeMap<String, (String, i64)>,
    /// Outgoing stack arguments for the next call: byte displacement from
    /// the stack pointer -> stored value. Dart AOT passes argument zero in
    /// a register and pushes the remaining arguments right-to-left so the
    /// last argument sits at `[SP]`.
    outgoing: BTreeMap<i64, Expression>,
    /// Bitmask of argument registers this body wrote since the last call
    /// (bit i = register i of the ABI argument window). Calls may only report
    /// those registers as arguments when the caller itself established them;
    /// the incoming seeds exist for callee-side reads, not for calls.
    written_argument_registers: u16,
}

impl FlowState {
    fn meet(left: &FlowState, right: &FlowState) -> FlowState {
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
        for (key, left_buffer) in &left.buffers {
            if let Some(right_buffer) = right.buffers.get(key) {
                let len = left_buffer.parts.len().max(right_buffer.parts.len());
                let mut parts = Vec::with_capacity(len);
                for index in 0..len {
                    let l = left_buffer.parts.get(index).cloned().flatten();
                    let r = right_buffer.parts.get(index).cloned().flatten();
                    parts.push(if l.is_some() && l == r { l } else { None });
                }
                buffers.insert(key.clone(), ElementBuffer { parts });
            }
        }
        FlowState {
            registers: merge_expressions(&left.registers, &right.registers),
            stack: merge_expressions(&left.stack, &right.stack),
            buffers,
            aliases: merge_equal(&left.aliases, &right.aliases),
            outgoing: merge_outgoing(&left.outgoing, &right.outgoing),
            written_argument_registers: left.written_argument_registers
                & right.written_argument_registers,
        }
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

/// Frame-pointer register for each supported ABI.
fn abi_frame_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x29",
        Abi::ArmeabiV7a => "r11",
        Abi::X86_64 => "rbp",
    }
}

/// Register holding the first Dart argument for each ABI. x64 follows
/// receiver-in-RDI with further arguments in RSI, RDX, RCX, R8, R9.
fn abi_first_argument_register(abi: Abi) -> &'static str {
    match abi {
        Abi::Arm64V8a => "x1",
        Abi::ArmeabiV7a => "r1",
        Abi::X86_64 => "rdi",
    }
}

/// Full argument-register window for an ABI: index 0 is the first Dart
/// argument slot, further entries carry subsequent arguments.
fn abi_argument_window(abi: Abi) -> Vec<&'static str> {
    let mut window = vec![abi_first_argument_register(abi)];
    window.extend_from_slice(abi_rest_argument_registers(abi));
    window
}

/// Argument registers beyond the first, in order.
fn abi_rest_argument_registers(abi: Abi) -> &'static [&'static str] {
    match abi {
        Abi::Arm64V8a => &["x2", "x3", "x4", "x5", "x6", "x7"],
        Abi::ArmeabiV7a => &["r2", "r3"],
        Abi::X86_64 => &["rsi", "rdx", "rcx", "r8", "r9"],
    }
}

/// Offset from the established frame pointer to the first incoming argument.
/// x64 accounts for both the pushed RBP and the return address.
fn abi_frame_header_size(abi: Abi) -> i64 {
    match abi {
        Abi::Arm64V8a | Abi::X86_64 => 16,
        Abi::ArmeabiV7a => 8,
    }
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

/// Replays the ARM64 immediate-building instructions that can initialize an
/// object header register. Returning `None` on any other write keeps the
/// allocation fusion conservative.
fn arm64_loaded_immediate(
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
            mnemonic if writes_first_operand(mnemonic) => constant = None,
            _ => {}
        }
    }
    constant
}

/// Dart predefined class IDs are stable across VM builds (`class_id.h`). The
/// object header stores the 20-bit CID at bit 12 and a four-bit size in units
/// of ARM64 object alignment at bit 8. A boxed Double is CID 62 and 16 bytes.
fn is_arm64_boxed_double_tag(tag: u64) -> bool {
    const CLASS_ID_SHIFT: u32 = 12;
    const CLASS_ID_MASK: u64 = (1 << 20) - 1;
    const SIZE_TAG_SHIFT: u32 = 8;
    const SIZE_TAG_MASK: u64 = 0xf;
    const DOUBLE_CID: u64 = 62;
    const DOUBLE_SIZE_IN_ALIGNMENT_UNITS: u64 = 1;

    ((tag >> CLASS_ID_SHIFT) & CLASS_ID_MASK) == DOUBLE_CID
        && ((tag >> SIZE_TAG_SHIFT) & SIZE_TAG_MASK) == DOUBLE_SIZE_IN_ALIGNMENT_UNITS
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
    // A runtime-stub call is either unnamed in the symbol table or carries a
    // known VM helper label. Calls to recovered Dart functions never fuse.
    let is_runtime_stub_call = |index: usize| -> bool {
        let Some(target) = call_target(index) else {
            return false;
        };
        match symbols.get(&target) {
            Some(symbol) => {
                !symbol.semantic_name
                    || ["StackOverflow", "WriteBarrier", "Barrier", "Deopt"]
                        .iter()
                        .any(|marker| symbol.label.contains(marker))
                    || symbol.label.starts_with("stub")
                    || symbol.label.starts_with("_iso_stub_")
            }
            None => true,
        }
    };

    if abi == Abi::ArmeabiV7a {
        // ARM32 adjusts a tagged object pointer into a VFP-aligned base before
        // loading an unboxed field. Fold the adjacent address calculation
        // into the VFP load so field recovery sees the original receiver.
        for index in 0..input.len().saturating_sub(1) {
            if input[index].mnemonic != "add" || input[index + 1].mnemonic != "vldr" {
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
                    mnemonic: "vldr".to_owned(),
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

    // Pass 4: inline ARM64 boxed-double allocation. The fast bump allocation
    // and the out-of-line allocation stub merge at the same value store. The
    // decoded VM header must prove CID 62 and a 16-byte object before this is
    // treated as a source-level box operation.
    if abi == Abi::Arm64V8a {
        for index in 0..input.len().saturating_sub(7) {
            if !keep[index] || input[index].mnemonic != "ldp" {
                continue;
            }
            let load = operands_of(index);
            if load.len() < 3 {
                continue;
            }
            let result = normalize_register(&load[0]);
            let limit = normalize_register(&load[1]);
            let Some((base, top_offset)) = arm_memory_address(&load[2]) else {
                continue;
            };
            if base != thread {
                continue;
            }
            let add = operands_of(index + 1);
            let compare = operands_of(index + 2);
            let branch = &input[index + 3];
            if input[index + 1].mnemonic != "add"
                || add.first().map(|value| normalize_register(value)) != Some(result.clone())
                || add.get(1).map(|value| normalize_register(value)) != Some(result.clone())
                || add.get(2).and_then(|value| signed_immediate(value)) != Some(16)
                || input[index + 2].mnemonic != "cmp"
                || compare.first().map(|value| normalize_register(value)) != Some(limit)
                || compare.get(1).map(|value| normalize_register(value)) != Some(result.clone())
                || branch.mnemonic != "b.ls"
            {
                continue;
            }
            let Some(slow_address) = branch_target(&branch.operands) else {
                continue;
            };
            let Some(slow_index) = index_of.get(&slow_address).copied() else {
                continue;
            };
            let top_store = operands_of(index + 4);
            let tagged_result = operands_of(index + 5);
            if input[index + 4].mnemonic != "str"
                || top_store.first().map(|value| normalize_register(value)) != Some(result.clone())
                || top_store.get(1).and_then(|value| arm_memory_address(value))
                    != Some((thread.to_owned(), top_offset))
                || input[index + 5].mnemonic != "sub"
                || tagged_result.first().map(|value| normalize_register(value))
                    != Some(result.clone())
                || tagged_result.get(1).map(|value| normalize_register(value))
                    != Some(result.clone())
                || tagged_result
                    .get(2)
                    .and_then(|value| signed_immediate(value))
                    != Some(15)
            {
                continue;
            }
            let value_store = (index + 6..slow_index).find(|candidate| {
                if !matches!(input[*candidate].mnemonic.as_str(), "str" | "stur") {
                    return false;
                }
                let operands = operands_of(*candidate);
                let Some(value) = operands.first() else {
                    return false;
                };
                let Some((store_base, displacement)) = operands
                    .get(1)
                    .and_then(|operand| arm_memory_address(operand))
                else {
                    return false;
                };
                value.trim().to_ascii_lowercase().starts_with('d')
                    && store_base == result
                    && displacement == 7
            });
            let Some(value_store) = value_store else {
                continue;
            };
            let header_store = (index + 6..value_store).find(|candidate| {
                if !matches!(input[*candidate].mnemonic.as_str(), "str" | "stur") {
                    return false;
                }
                let operands = operands_of(*candidate);
                let Some(header_register) = operands.first() else {
                    return false;
                };
                let Some((store_base, displacement)) = operands
                    .get(1)
                    .and_then(|operand| arm_memory_address(operand))
                else {
                    return false;
                };
                if store_base != result || displacement != -1 {
                    return false;
                }
                let header_register = normalize_register(header_register);
                arm64_loaded_immediate(input, index + 6, *candidate, &header_register)
                    .is_some_and(is_arm64_boxed_double_tag)
            });
            if header_store.is_none() {
                continue;
            }
            let store_operands = operands_of(value_store);
            let value = normalize_register(&store_operands[0]);
            let slow_end = (slow_index..input.len().min(slow_index + 16)).find(|candidate| {
                branch_kind(&input[*candidate].mnemonic) == Some(false)
                    && branch_target(&input[*candidate].operands)
                        == Some(input[value_store].address)
            });
            let Some(slow_end) = slow_end else {
                continue;
            };
            let has_allocation_call = (slow_index..slow_end).any(|candidate| {
                input[candidate].mnemonic == "bl" && is_runtime_stub_call(candidate)
            });
            let moves_result = (slow_index..slow_end).any(|candidate| {
                if input[candidate].mnemonic != "mov" {
                    return false;
                }
                let operands = operands_of(candidate);
                operands.first().map(|value| normalize_register(value)) == Some(result.clone())
                    && operands.get(1).map(|value| normalize_register(value))
                        == Some(abi_return_register(abi).to_owned())
            });
            if !has_allocation_call || !moves_result {
                continue;
            }
            for keep_slot in &mut keep[index..=value_store] {
                *keep_slot = false;
            }
            for keep_slot in &mut keep[slow_index..=slow_end] {
                *keep_slot = false;
            }
            replacements.insert(
                index,
                DecodedInstruction {
                    address: input[index].address,
                    next: input[index].next,
                    mnemonic: "fmov".to_owned(),
                    operands: format!("{result}, {value}"),
                },
            );
        }
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

    // Pass 6: compressed write-barrier tests ahead of runtime-stub calls.
    for call_index in 0..input.len() {
        if !keep[call_index] || input[call_index].mnemonic != "bl" {
            continue;
        }
        if !is_runtime_stub_call(call_index) {
            continue;
        }
        let mut start = None;
        let mut scan = call_index;
        // Walk back over the bounded barrier-test window.
        for _step in 0..6 {
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
                let matches_join =
                    branch_target(&input[index].operands) == Some(input[call_index].next);
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
                || ((mnemonic == "ldurb" || mnemonic == "ldur")
                    && input[index].operands.contains("#-1]"))
                || (mnemonic == "mov" && input[index].operands.contains("- 1]"))
                || matches!(mnemonic, "shr" | "sar");
            if !is_barrier_test {
                break;
            }
            scan = index;
        }
        let Some(start) = start else {
            continue;
        };
        for keep_slot in &mut keep[start..=call_index] {
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
        if is_return(&last.mnemonic, &last.operands) {
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
) -> Vec<SemanticStatement> {
    let fused = fuse_machine_idioms(abi, instructions, symbols);
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
    for instruction in &fused {
        let ends_block = branch_kind(&instruction.mnemonic).is_some()
            || is_return(&instruction.mnemonic, &instruction.operands);
        if ends_block && live_addresses.contains(&instruction.next) {
            fused_starts.insert(instruction.next);
        }
    }
    let blocks = LifterBlocks::build(&fused, &fused_starts);
    let entry_state = entry_flow_state(abi, parameter_hints, receiver_class);
    let converged = solve_block_states(
        &blocks,
        &fused,
        abi,
        symbols,
        object_pool,
        &pool_loads,
        field_layout,
        &entry_state,
    );
    // Only emit statements for blocks reachable from the entry. Unreachable
    // ranges are allocation slow paths or stub tails whose machine evidence
    // stays in the reports; emitting them would fabricate unreachable Dart.
    let successors = block_successors(&blocks, &fused);
    let mut reachable = vec![false; blocks.starts.len()];
    if !reachable.is_empty() {
        reachable[0] = true;
        let mut pending = vec![0usize];
        while let Some(index) = pending.pop() {
            for successor in &successors[index] {
                if !reachable[*successor] {
                    reachable[*successor] = true;
                    pending.push(*successor);
                }
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
        );
    }
    statements
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

fn entry_flow_state(
    abi: Abi,
    parameter_hints: &[ParameterHint],
    receiver_class: Option<(&str, Option<&str>)>,
) -> FlowState {
    let mut registers = BTreeMap::new();
    // Dart AOT passes argument zero in a fixed register; the remaining
    // arguments arrive on the stack (seeded below).
    let first = parameter_hints.first().cloned().unwrap_or(ParameterHint {
        name: "arg0".to_owned(),
        class_name: None,
        class_library_uri: None,
    });
    {
        let receiver = (first.name == "this").then_some(receiver_class).flatten();
        let (class_name, class_library_uri) = receiver.map_or(
            (first.class_name.clone(), first.class_library_uri.clone()),
            |(name, uri)| (Some(name.to_owned()), uri.map(str::to_owned)),
        );
        registers.insert(
            abi_first_argument_register(abi).to_owned(),
            Expression {
                text: first.name,
                confidence: EvidenceConfidence::High,
                complexity: 1,
                class_name,
                class_library_uri,
                raw: false,
            },
        );
    }
    // Remaining register arguments (x64 uses SysV-style register windows).
    for (index, register) in abi_rest_argument_registers(abi).iter().enumerate() {
        let hint = parameter_hints.get(index + 1);
        let expression = Expression {
            text: hint.map_or_else(|| format!("arg{}", index + 1), |hint| hint.name.clone()),
            confidence: EvidenceConfidence::High,
            complexity: 1,
            class_name: hint.and_then(|hint| hint.class_name.clone()),
            class_library_uri: hint.and_then(|hint| hint.class_library_uri.clone()),
            raw: false,
        };
        registers.insert((*register).to_owned(), expression);
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
            },
        );
    }
    // Incoming stack arguments: pushed right-to-left so parameter i of n sits
    // at `[entry SP + (n-1-i)*8]`, addressed through either the frame pointer
    // after the prologue or the stack pointer before it. When the signature
    // did not survive, seed a generous fixed window instead: any read of one
    // of these slots is by definition an incoming argument, so over-seeding
    // only names slots that are actually read.
    const MAX_INCOMING_STACK_SLOTS: usize = 16;
    let frame_register = abi_frame_register(abi);
    let stack_register = abi_stack_register(abi);
    let header = abi_frame_header_size(abi);
    let stack_header = if abi == Abi::X86_64 { 8 } else { 0 };
    let count = parameter_hints.len();
    let slot_count = if count == 0 {
        MAX_INCOMING_STACK_SLOTS
    } else {
        count
    };
    let mut stack = BTreeMap::new();
    for slot in 0..slot_count {
        let displacement = slot as i64 * 8;
        let hint = if count == 0 {
            ParameterHint {
                name: format!("arg{slot}"),
                class_name: None,
                class_library_uri: None,
            }
        } else {
            parameter_hints[count - 1 - slot].clone()
        };
        let name = hint.name.clone();
        let expression = Expression {
            text: name.clone(),
            confidence: EvidenceConfidence::High,
            complexity: 1,
            class_name: if name == "this" {
                receiver_class
                    .map(|(class, _)| class.to_owned())
                    .or(hint.class_name)
            } else {
                hint.class_name
            },
            class_library_uri: if name == "this" {
                receiver_class
                    .and_then(|(_, uri)| uri.map(str::to_owned))
                    .or(hint.class_library_uri)
            } else {
                hint.class_library_uri
            },
            raw: false,
        };
        for key in slot_keys(frame_register, displacement + header) {
            stack.insert(key, expression.clone());
        }
        for key in slot_keys(stack_register, displacement + stack_header) {
            stack.insert(key, expression.clone());
        }
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
) -> Vec<FlowState> {
    if blocks.starts.is_empty() {
        return Vec::new();
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
    inputs[0] = Some(entry_state.clone());
    let mut pending = std::collections::VecDeque::from([0usize]);
    let mut visits = 0usize;
    while let Some(index) = pending.pop_front() {
        visits += 1;
        if visits > 4096 {
            break;
        }
        let mut input = if index == 0 {
            entry_state.clone()
        } else {
            FlowState::default()
        };
        let predecessors = &predecessors[index];
        if index != 0 {
            let mut merged: Option<FlowState> = None;
            for predecessor in predecessors {
                let Some(output) = &outputs[*predecessor] else {
                    continue;
                };
                merged = Some(match merged {
                    Some(current) => FlowState::meet(&current, output),
                    None => (*output).clone(),
                });
            }
            match merged {
                Some(merged) => input = merged,
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
        );
        let changed = outputs[index].as_ref() != Some(&working);
        inputs[index] = Some(input);
        outputs[index] = Some(working);
        if changed {
            for successor in &successors[index] {
                pending.push_back(*successor);
            }
        }
    }
    inputs
        .into_iter()
        .zip(outputs)
        .map(|(input, _)| input.unwrap_or_default())
        .collect()
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
        if is_return(&last.mnemonic, &last.operands) {
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
) {
    let return_register = abi_return_register(abi);
    let begin = range.as_ref().map_or(0, |range| range.start);
    let end = range.as_ref().map_or(instructions.len(), |range| range.end);
    let mut last_comparison: Option<Expression> = None;
    // Pending integer division (quotient register, dividend, divisor) used to
    // pair a following multiply-subtract into Dart's `%`.
    let mut last_division: Option<(String, String, String)> = None;
    // 32-bit (compressed) reference loads awaiting decompression:
    // destination register -> (base register, load displacement).
    let mut compressed_loads = BTreeMap::<String, (String, i64)>::new();
    let stack_register = abi_stack_register(abi);
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
    for instruction in instructions[begin..end].iter() {
        let operands = split_operands(&instruction.operands);
        // Any write to the stack pointer retires the outgoing argument area
        // (prologue frame allocation or epilogue restore).
        let destination_register = operands
            .first()
            .filter(|_| writes_first_operand(&instruction.mnemonic))
            .map(|target| normalize_register(target));
        if let Some(destination) = destination_register.as_deref()
            && let Some(bit) = abi_argument_window(abi)
                .iter()
                .position(|reg| *reg == destination)
        {
            state.written_argument_registers |= 1 << bit;
        }
        if destination_register.as_deref() == Some(stack_register)
            || instruction.operands.contains("]!")
        {
            state.outgoing.clear();
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
            _ => None,
        } && base == stack_register
            && displacement >= 0
            && !instruction.operands.contains('!')
        {
            // Outgoing argument push for the next call.
            for (stored, operand) in stored_operands.iter().enumerate() {
                if let Some(value) = resolve_expression(operand, registers, object_pool) {
                    state
                        .outgoing
                        .insert(displacement + stored as i64 * 8, value);
                } else {
                    state.outgoing.remove(&(displacement + stored as i64 * 8));
                }
            }
            continue;
        }
        match instruction.mnemonic.as_str() {
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
            "movsd" | "vldr" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                let source = &operands[1];
                let value = if let Some(index) = pool_loads.get(&instruction.address).copied() {
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
                    if let Some(slot) = stack_slot_key(abi, &operands[0]) {
                        let source_register = normalize_register(&operands[1]);
                        // Spilling a tracked array keeps it alive across calls.
                        if let Some(buffer) = buffers.remove(&reg_key(&source_register)) {
                            buffers.insert(stack_key(&slot), buffer);
                            aliases.remove(&source_register);
                            aliases.insert(source_register.clone(), (stack_key(&slot), 0));
                        }
                        if let Some(value) =
                            resolve_expression(&operands[1], registers, object_pool)
                        {
                            stack.insert(slot, value);
                        }
                        continue;
                    }
                    let value_operand_index = 1usize;
                    if let Some((base, displacement)) =
                        operands.first().and_then(|value| arm_memory_address(value))
                        && let Some(value) = resolve_expression(
                            &operands[value_operand_index],
                            registers,
                            object_pool,
                        )
                    {
                        let direct_key = reg_key(&base);
                        let (buffer_key, effective_displacement) =
                            if buffers.contains_key(&direct_key) {
                                (direct_key.clone(), displacement)
                            } else if let Some((key, extra)) = aliases.get(&base).cloned() {
                                (key, extra.saturating_add(displacement))
                            } else {
                                (String::new(), displacement)
                            };
                        if !buffer_key.is_empty()
                            && let Some(buffer) = buffers.get_mut(&buffer_key)
                        {
                            match element_index(abi, effective_displacement) {
                                Some(index) => {
                                    if buffer.parts.len() <= index {
                                        buffer.parts.resize(index + 1, None);
                                    }
                                    buffer.parts[index] =
                                        (!value.text.is_empty()).then_some(value.text.clone());
                                }
                                None => {
                                    buffers.remove(&buffer_key);
                                }
                            }
                            continue;
                        }
                        if let Some(receiver) = registers.get(&base).cloned()
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
                    if let Some(slot) = stack_slot_key(abi, source_operand) {
                        if let Some(buffer) = buffers.get(&stack_key(&slot)).cloned() {
                            buffers.insert(reg_key(&target), buffer);
                            aliases.remove(&target);
                            aliases.insert(target.clone(), (stack_key(&slot), 0));
                        } else {
                            aliases.remove(&target);
                            buffers.remove(&reg_key(&target));
                        }
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
                                },
                            );
                        }
                        buffers.remove(&reg_key(&target));
                        aliases.remove(&target);
                        compressed_loads.remove(&target);
                        continue;
                    }
                    if let Some(index) = pool_loads.get(&instruction.address).copied()
                        && let Some(value) = object_pool.and_then(|pool| pool.get(index))
                    {
                        registers.insert(
                            target.clone(),
                            Expression {
                                text: value.clone(),
                                confidence: EvidenceConfidence::High,
                                complexity: 1,
                                class_name: snapshot_instance_class(value),
                                class_library_uri: None,
                                raw: false,
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
                    if is_32bit_destination
                        && !pool_loads.contains_key(&instruction.address)
                        && let Some((base, displacement)) = arm_memory_address(source_operand)
                    {
                        compressed_loads.insert(target.clone(), (base, displacement));
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
                // A register rename carries an in-flight element buffer.
                let source = normalize_register(&operands[1]);
                if source != target {
                    if let Some(buffer) = buffers.remove(&reg_key(&source)) {
                        buffers.insert(reg_key(&target), buffer);
                    } else if let Some((key, extra)) = aliases.remove(&source) {
                        aliases.insert(target.clone(), (key, extra));
                    } else {
                        buffers.remove(&reg_key(&target));
                    }
                    aliases.remove(&target);
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
                        },
                    );
                } else {
                    registers.remove(&target);
                }
            }
            "add" | "sub" | "mul" | "imul" | "and" | "orr" | "eor" | "or" | "xor"
                if operands.len() >= 3 =>
            {
                let target = normalize_register(&operands[0]);
                // x64 decompression: `mov eax, dword ptr [recv+off];
                // add rax, qword ptr [r14+0x58]` adds the heap base held in
                // the thread. Treat it exactly like the ARM shift idiom.
                if instruction.mnemonic == "add"
                    && operands.len() >= 3
                    && operands[2].contains('[')
                    && operands[2].contains("r14")
                {
                    let source = normalize_register(&operands[1]);
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
                    && operands[2].contains("x28")
                    && shift_amount(&operands[2]) == Some(32)
                {
                    let source = normalize_register(&operands[1]);
                    if let Some((base_register, displacement)) =
                        compressed_loads.get(&source).cloned()
                        && let Some(receiver) = registers.get(&base_register).cloned()
                    {
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
                        "add" => "+",
                        "sub" => "-",
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
                                    },
                                );
                            }
                            // Unknown null-relative arithmetic is not a
                            // recoverable Dart expression.
                            None => {
                                registers.remove(&target);
                            }
                        }
                    } else if let Some(expression) = binary_expression(left, operator, right) {
                        registers.insert(target.clone(), expression);
                    } else {
                        registers.remove(&target);
                    }
                } else {
                    registers.remove(&target);
                }
                // Derived element pointers: `add dst, arrBase, #offset`
                // extends buffer provenance with a byte displacement.
                aliases.remove(&target);
                if instruction.mnemonic == "add"
                    && let Some(delta) = signed_immediate(&operands[2])
                {
                    let source = normalize_register(&operands[1]);
                    if let Some((key, extra)) = aliases.get(&source).cloned() {
                        aliases.insert(target.clone(), (key, extra.saturating_add(delta)));
                    } else if buffers.contains_key(&reg_key(&source)) {
                        aliases.insert(target.clone(), (reg_key(&source), delta));
                    }
                }
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
                            },
                        );
                    }
                    _ => {
                        registers.remove(&target);
                    }
                }
            }
            "asr" | "lsr" | "lsl" | "sar" | "shr" | "shl" if operands.len() >= 3 => {
                let target = normalize_register(&operands[0]);
                if let Some(value) = resolve_expression(&operands[1], registers, object_pool)
                    && let Some(shift) = immediate_text(&operands[2])
                {
                    if instruction.mnemonic == "sar" && shift == "1" && !value.raw {
                        // x64 Smi untag: the payload IS the source value.
                        registers.insert(
                            target,
                            Expression {
                                raw: true,
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
                    if matches!(instruction.mnemonic.as_str(), "lsl" | "shl")
                        && shift == "1"
                        && int_derived
                    {
                        registers.insert(target, value);
                        continue;
                    }
                    let operator = if matches!(instruction.mnemonic.as_str(), "lsl" | "shl") {
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
                            .split_once(" ? ")
                            .map(|(left, right)| format!("{left} {operator} {right}"))
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
            "vadd.f64" | "vsub.f64" | "vmul.f64" | "vdiv.f64"
                if operands.len() >= 3 =>
            {
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
                        text: format!("{} ? {}", left.text, right.text),
                        confidence: weaker(left.confidence, right.confidence),
                        complexity: left.complexity.saturating_add(right.complexity),
                        class_name: None,
                        class_library_uri: None,
                        raw: false,
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
                            })
                    })
                };
                let left = resolve_or_named(&operands[0]);
                let right = resolve_or_named(&operands[1]);
                last_comparison = match (left, right) {
                    (Some(left), Some(right)) => {
                        let text = if matches!(instruction.mnemonic.as_str(), "tst" | "test") {
                            format!("({} & {}) ? 0", left.text, right.text)
                        } else {
                            format!("{} ? {}", left.text, right.text)
                        };
                        Some(Expression {
                            text,
                            confidence: weaker(left.confidence, right.confidence),
                            complexity: left.complexity.saturating_add(right.complexity),
                            class_name: None,
                            class_library_uri: None,
                            raw: false,
                        })
                    }
                    _ => None,
                };
            }
            "stur" | "str" if operands.len() >= 2 => {
                if let Some(slot) = stack_slot_key(abi, &operands[1]) {
                    let source = normalize_register(&operands[0]);
                    // Spilling a tracked array keeps it alive across calls.
                    if let Some(buffer) = buffers.remove(&reg_key(&source)) {
                        buffers.insert(stack_key(&slot), buffer);
                        aliases.remove(&source);
                        aliases.insert(source, (stack_key(&slot), 0));
                    }
                    if let Some(value) = resolve_expression(&operands[0], registers, object_pool) {
                        stack.insert(slot, value);
                    }
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1]) {
                    let value = resolve_expression(&operands[0], registers, object_pool);
                    // Resolve the buffer key: direct register or derived alias.
                    let direct_key = reg_key(&base);
                    let (buffer_key, effective_displacement) = if buffers.contains_key(&direct_key)
                    {
                        (direct_key.clone(), displacement)
                    } else if let Some((key, extra)) = aliases.get(&base).cloned() {
                        (key, extra.saturating_add(displacement))
                    } else {
                        (String::new(), displacement)
                    };
                    if !buffer_key.is_empty()
                        && let Some(buffer) = buffers.get_mut(&buffer_key)
                    {
                        match element_index(abi, effective_displacement) {
                            Some(index) => {
                                // Unknown values become explicit gaps rather
                                // than invalidating the literal pattern.
                                if buffer.parts.len() <= index {
                                    buffer.parts.resize(index + 1, None);
                                }
                                buffer.parts[index] =
                                    value.as_ref().map(|value| value.text.clone());
                            }
                            // A non-element store through the tracked
                            // register invalidates the allocation pattern.
                            None => {
                                buffers.remove(&buffer_key);
                            }
                        }
                    } else if let Some(receiver) = registers.get(&base).cloned()
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
            }
            "ldur" | "ldr" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                // Track 32-bit reference loads for the decompression idiom.
                if operands[0].trim().to_ascii_lowercase().starts_with('w') {
                    let trackable = stack_slot_key(abi, &operands[1]).is_none()
                        && !pool_loads.contains_key(&instruction.address)
                        && arm_memory_address(&operands[1]).is_some();
                    if trackable {
                        if let Some((base, displacement)) = arm_memory_address(&operands[1]) {
                            compressed_loads.insert(target.clone(), (base, displacement));
                        }
                    } else {
                        compressed_loads.remove(&target);
                    }
                } else {
                    compressed_loads.remove(&target);
                }
                if let Some(slot) = stack_slot_key(abi, &operands[1]) {
                    let slot_key = stack_key(&slot);
                    // Reloading a spilled array re-attaches buffer tracking.
                    if let Some(buffer) = buffers.get(&slot_key).cloned() {
                        buffers.insert(reg_key(&target), buffer);
                        aliases.remove(&target);
                        aliases.insert(target.clone(), (slot_key, 0));
                    } else {
                        aliases.remove(&target);
                        buffers.remove(&reg_key(&target));
                    }
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
                            },
                        );
                    }
                } else if let Some(index) = pool_loads.get(&instruction.address).copied()
                    && let Some(value) = object_pool.and_then(|pool| pool.get(index))
                {
                    registers.insert(
                        target,
                        Expression {
                            text: value.clone(),
                            confidence: EvidenceConfidence::High,
                            complexity: 1,
                            class_name: snapshot_instance_class(value),
                            class_library_uri: None,
                            raw: false,
                        },
                    );
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
                    && let Some(literal) = take_interpolation_literal(buffers)
                {
                    push_statement!(SemanticStatement::StringInterpolation {
                        parts: vec![literal.clone()],
                        confidence: EvidenceConfidence::High,
                        address: format!("0x{:x}", instruction.address),
                    });
                    last_comparison = None;
                    last_division = None;
                    kill_caller_saved(abi, registers);
                    state.outgoing.clear();
                    state.written_argument_registers = 0;
                    buffers.clear();
                    aliases.clear();
                    registers.insert(
                        return_register.to_owned(),
                        Expression {
                            text: literal,
                            confidence: EvidenceConfidence::High,
                            complexity: 1,
                            class_name: Some("String".to_owned()),
                            class_library_uri: Some("dart:core".to_owned()),
                            raw: false,
                        },
                    );
                    continue;
                }
                let arguments = collect_call_arguments(
                    abi,
                    registers,
                    &state.outgoing,
                    state.written_argument_registers,
                );
                push_statement!(SemanticStatement::ResolvedCall {
                    target: target.clone(),
                    arguments: arguments.clone(),
                    confidence: EvidenceConfidence::Medium,
                    address: format!("0x{:x}", instruction.address),
                });
                last_comparison = None;
                last_division = None;
                kill_caller_saved(abi, registers);
                state.outgoing.clear();
                state.written_argument_registers = 0;
                // Register-resident bookkeeping dies at the call; spilled
                // arrays survive on the stack.
                buffers.retain(|key, _| key.starts_with("stk:"));
                aliases.clear();
                if looks_like_allocation_stub(&target) {
                    // Tentative buffer; it survives only while subsequent
                    // stores match the compressed Array element layout.
                    buffers.insert(reg_key(return_register), ElementBuffer::default());
                }
                registers.insert(
                    return_register.to_owned(),
                    Expression {
                        text: format!("{}_result", sanitize_semantic_name(&target)),
                        confidence: EvidenceConfidence::Low,
                        complexity: 1,
                        class_name: symbol.and_then(|symbol| symbol.result_class.clone()),
                        class_library_uri: symbol.and_then(|symbol| symbol.library_uri.clone()),
                        raw: false,
                    },
                );
            }
            mnemonic if is_call(mnemonic) => {
                let target = operands
                    .first()
                    .and_then(|operand| resolve_expression(operand, registers, object_pool))
                    .filter(|target| is_named_pool_target(&target.text));
                let arguments = collect_call_arguments(
                    abi,
                    registers,
                    &state.outgoing,
                    state.written_argument_registers,
                );
                kill_caller_saved(abi, registers);
                state.outgoing.clear();
                buffers.retain(|key, _| key.starts_with("stk:"));
                aliases.clear();
                last_comparison = None;
                if let Some(target) = target {
                    push_statement!(SemanticStatement::ResolvedCall {
                        target: target.text.clone(),
                        arguments,
                        confidence: EvidenceConfidence::Medium,
                        address: format!("0x{:x}", instruction.address),
                    });
                    registers.insert(
                        return_register.to_owned(),
                        Expression {
                            text: format!("{}_result", sanitize_semantic_name(&target.text)),
                            confidence: EvidenceConfidence::Low,
                            complexity: 1,
                            class_name: result_class_from_target(&target.text),
                            class_library_uri: None,
                            raw: false,
                        },
                    );
                }
            }
            mnemonic if is_return(mnemonic, &instruction.operands) => {
                if let Some(value) = registers.get(return_register)
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
                aliases.remove(&target);
                buffers.remove(&reg_key(&target));
                compressed_loads.remove(&target);
                let Some((base, displacement)) =
                    operands.last().and_then(|value| arm_memory_address(value))
                else {
                    continue;
                };
                let direct_key = reg_key(&base);
                if let Some((key, extra)) = aliases.get(&base).cloned() {
                    aliases.insert(target, (key, extra.saturating_add(displacement)));
                } else if buffers.contains_key(&direct_key) {
                    aliases.insert(target, (direct_key, displacement));
                }
            }
            mnemonic if writes_first_operand(mnemonic) => {
                if let Some(target) = operands.first() {
                    let target = normalize_register(target);
                    registers.remove(&target);
                    buffers.remove(&reg_key(&target));
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

/// Consumes the most recently filled element buffer and renders it as a Dart
/// string literal. Literal parts are JSON-decoded pool strings; unresolved
/// interior parts become explicit placeholders instead of being guessed.
fn take_interpolation_literal(buffers: &mut BTreeMap<String, ElementBuffer>) -> Option<String> {
    let candidate = buffers
        .iter()
        .max_by_key(|(_, buffer)| buffer.parts.iter().flatten().count())
        .filter(|(_, buffer)| !buffer.parts.is_empty())
        .map(|(key, _)| key.clone())?;
    let buffer = buffers.remove(&candidate)?;
    if buffer.parts.iter().flatten().count() == 0 {
        return None;
    }
    let mut output = String::from("'");
    for part in &buffer.parts {
        match part {
            Some(text) => {
                if let Some(decoded) = decode_json_string(text) {
                    for character in decoded.chars() {
                        match character {
                            '$' => output.push_str("\\$"),
                            '\'' => output.push_str("\\'"),
                            '\\' => output.push_str("\\\\"),
                            '\n' => output.push_str("\\n"),
                            '\r' => output.push_str("\\r"),
                            other => output.push(other),
                        }
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
    Some(output)
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
        let name = if index == 0 && function.kind == Some(RecoveredFunctionKind::Closure) {
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
        names.push(ParameterHint {
            name,
            class_name: None,
            class_library_uri: None,
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

/// Builds the call-target symbol table from recovered functions, plus the
/// qualified-name → library candidates used to attribute recovered indirect
/// calls.
#[allow(clippy::type_complexity)]
pub(crate) fn build_function_symbols(
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
        let semantic = function.name_source != crate::model::RecoveredNameSource::Synthetic
            && !function.name.starts_with("sub_")
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
        let result_class = match function.kind {
            Some(crate::model::RecoveredFunctionKind::Constructor) => function
                .owner
                .clone()
                .map(|owner| crate::analysis::readable_snapshot_name(&owner)),
            _ => function
                .signature
                .as_ref()
                .and_then(|signature| signature.resolved.as_ref())
                .and_then(|resolved| resolved.return_type.as_ref())
                .and_then(|return_type| simple_class_type(&return_type.display_name)),
        };
        let symbol = if semantic {
            let label = match function.owner.as_deref() {
                Some(owner) if !matches!(owner, "::" | "top_level") => {
                    format!("{owner}.{}", function.name)
                }
                _ => function.name.clone(),
            };
            Symbol::new(label, function.library_uri.clone(), application_package)
                .with_code_identity(address, 0, crate::model::DirectCallResolution::ExactEntry)
                .with_result_class(result_class)
        } else {
            Symbol::code_boundary(address)
        };
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

fn kill_caller_saved(abi: Abi, registers: &mut BTreeMap<String, Expression>) {
    match abi {
        Abi::Arm64V8a => {
            for index in 0..=18 {
                registers.remove(&format!("x{index}"));
            }
        }
        Abi::ArmeabiV7a => {
            for register in ["r0", "r1", "r2", "r3", "ip", "r12", "lr"] {
                registers.remove(register);
            }
        }
        Abi::X86_64 => registers.clear(),
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
        })
        .or_else(|| {
            let index = operand
                .strip_prefix("pool[")?
                .strip_suffix(']')?
                .parse::<usize>()
                .ok()?;
            Some(Expression {
                text: object_pool?.get(index)?.clone(),
                confidence: EvidenceConfidence::Medium,
                complexity: 1,
                class_name: object_pool
                    .and_then(|pool| pool.get(index))
                    .and_then(|value| snapshot_instance_class(value)),
                class_library_uri: None,
                raw: false,
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
    })
}

/// Classifies a memory operand as a named stack slot: either frame-relative
/// (`[x29, #offset]`) or an incoming-argument slot relative to the stack
/// pointer (`[x15, #offset]`). Returns the normalized slot key.
fn stack_slot_key(abi: Abi, value: &str) -> Option<String> {
    let (base, displacement) = arm_memory_address(value)?;
    if base == abi_stack_register(abi) || base == abi_frame_register(abi) {
        slot_keys(&base, displacement).into_iter().next()
    } else {
        None
    }
}

/// Dart AOT call arguments: argument zero travels in a fixed register and
/// the remaining arguments sit on the stack, pushed right-to-left so the
/// last argument occupies `[SP]` and earlier arguments higher slots.
fn collect_call_arguments(
    abi: Abi,
    registers: &BTreeMap<String, Expression>,
    outgoing: &BTreeMap<i64, Expression>,
    written_argument_registers: u16,
) -> Vec<String> {
    let mut arguments = Vec::new();
    for (index, register) in abi_argument_window(abi).into_iter().enumerate() {
        // Holes are tolerated: static calls leave the receiver slot unused
        // while further arguments still travel in their own registers.
        if written_argument_registers & (1 << index) != 0
            && let Some(value) = registers.get(register)
        {
            arguments.push(value.text.clone());
        }
    }
    // Stack arguments were pushed right-to-left: the highest displacement is
    // the first stacked argument.
    let mut slots = outgoing.iter().collect::<Vec<_>>();
    slots.sort_by(|left, right| right.0.cmp(left.0));
    arguments.extend(slots.into_iter().map(|(_, value)| value.text.clone()));
    arguments
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
    })
}

fn snapshot_instance_class(value: &str) -> Option<String> {
    value
        .strip_prefix("snapshotInstance(")?
        .split_once(')')
        .map(|(class_name, _)| class_name.to_owned())
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
        return Some((offset, identity.clone()));
    }
    let class_name = receiver.class_name.as_deref()?;
    // Machine operands address a tagged heap pointer, so the displacement is
    // one byte below the VM's object-layout offset. Keep matching in machine
    // coordinates, but expose the untagged offset in semantic IR and names.
    let (first_field, stride) = match abi {
        Abi::Arm64V8a | Abi::ArmeabiV7a => (7i64, 4i64),
        Abi::X86_64 => (15i64, 8i64),
    };
    if displacement < first_field
        || (displacement - first_field) % stride != 0
        || displacement > 4096
    {
        return None;
    }
    if matches!(
        class_name,
        "Array" | "_GrowableList" | "_ImmutableList" | "String" | "Map" | "Set" | "Context"
    ) {
        // Container internals have their own meaning; never placeholder them.
        return None;
    }
    let offset = displacement.checked_add(1)?;
    Some((
        offset,
        RecoveredFieldIdentity {
            name: format!("_slot_{offset:x}"),
            value_class: None,
            value_library_uri: None,
            synthesized_slot: true,
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
    let (left, right) = comparison.text.split_once(" ? ")?;
    Some(format!("{left} {operator} {right}"))
}

fn branch_condition(
    mnemonic: &str,
    operands: &[String],
    registers: &BTreeMap<String, Expression>,
    comparison: Option<&Expression>,
    object_pool: Option<&[String]>,
) -> Option<Expression> {
    if matches!(mnemonic, "cbz" | "cbnz") {
        let value = resolve_expression(operands.first()?, registers, object_pool)?;
        let operator = if mnemonic == "cbz" { "==" } else { "!=" };
        return Some(Expression {
            text: format!("{} {operator} 0", value.text),
            confidence: value.confidence,
            complexity: value.complexity.saturating_add(1),
            class_name: None,
            class_library_uri: None,
            raw: false,
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
        });
    }
    let comparison = comparison?;
    let (left, right) = comparison.text.split_once(" ? ")?;
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
        text: format!("{left} {operator} {right}"),
        confidence: comparison.confidence,
        complexity: comparison.complexity.saturating_add(1),
        class_name: None,
        class_library_uri: None,
        raw: false,
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

fn is_named_pool_target(value: &str) -> bool {
    !value.starts_with("snapshotRef(")
        && !value.starts_with("snapshotClass(")
        && !value.starts_with("snapshotType(")
        && !value.starts_with("snapshotField(")
        && !value.starts_with("snapshotInstance(")
        && !value.starts_with("snapshotLibrary(")
        && !value.starts_with("nativePoolEntry(")
        && !value.starts_with("resetPoolEntry(")
        && !value.starts_with('"')
        && value.parse::<i64>().is_err()
}

fn writes_first_operand(mnemonic: &str) -> bool {
    !matches!(
        mnemonic,
        "cmp"
            | "cmn"
            | "tst"
            | "str"
            | "stur"
            | "stp"
            | "b"
            | "bl"
            | "blr"
            | "ret"
            | "cbz"
            | "cbnz"
            | "tbz"
            | "tbnz"
    ) && branch_kind(mnemonic).is_none()
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
    if label.starts_with("stub ")
        || label.starts_with("_iso_stub_")
        || label.contains("Stub") && library_uri.is_none()
    {
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
        object_pool_index, reachable_block_count, recover_dispatch_calls,
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
            text: "(w0 & 16) ? 0".to_owned(),
            confidence: EvidenceConfidence::Low,
            complexity: 2,
            class_name: None,
            class_library_uri: None,
            raw: false,
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
            text: "(w0 & 16) ? 0".to_owned(),
            confidence: EvidenceConfidence::Low,
            complexity: 2,
            class_name: None,
            class_library_uri: None,
            raw: false,
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
            .analyze(0x1000, &bytes, &BTreeMap::new(), None, None, None)
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
            .analyze(0x1000, &bytes, &BTreeMap::new(), None, None, None)
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
            0x70, 0x0b, 0x40, 0xf9, 0x71, 0x0f, 0x40, 0xf9, 0x20, 0x02, 0x3f,
            0xd6,
        ];
        let pool = vec![
            "dynamicCall(\"isEmpty\", arity=2)".to_owned(),
            // Unnamed stub Code renders as an opaque reference.
            "snapshotRef(41)".to_owned(),
        ];
        let disassembly = Disassembler::new(Abi::Arm64V8a)
            .unwrap()
            .analyze(0x1000, &bytes, &BTreeMap::new(), None, Some(&pool), None)
            .unwrap();

        let selector_call = disassembly
            .statements
            .iter()
            .find_map(|statement| match statement {
                PseudoStatement::ObjectPoolCall {
                    pool_index,
                    target,
                    ..
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
            .analyze(0x1000, &bytes, &BTreeMap::new(), None, None, None)
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
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.starts_with("sub_")));
        assert_eq!(count, 105);
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

        // A join whose predecessors disagree must not invent a value.
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
        assert!(
            !statements
                .iter()
                .any(|statement| matches!(statement, SemanticStatement::Return { .. }))
        );

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
        // bl _interpolate — the exact AOT lowering of 'Added ${...}'.
        let instructions = [
            instruction(0x1000, "mov", "x2, #4"),
            instruction(0x1004, "bl", "#0x9000"),
            instruction(0x1008, "ldr", "x16, [x27, #0x20]"),
            instruction(0x100c, "stur", "w16, [x0, #0xf]"),
            instruction(0x1010, "ldr", "x17, [x27, #0x28]"),
            instruction(0x1014, "add", "x1, x17, #0"),
            instruction(0x1018, "stur", "w1, [x0, #0x13]"),
            instruction(0x101c, "bl", "#0x8000"),
            instruction(0x1020, "ret", ""),
        ];
        let mut pool = vec![String::new(); 8];
        pool[2] = "\"Added \"".to_owned();
        pool[5] = "\"world\"".to_owned();
        let mut symbols = BTreeMap::new();
        symbols.insert(
            0x8000,
            Symbol::new("_StringBase._interpolate".to_owned(), None, None),
        );
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &instructions,
            &BTreeSet::from([0x1000]),
            &symbols,
            Some(&pool),
        );
        assert!(
            statements.iter().any(|statement| matches!(
                statement,
                SemanticStatement::StringInterpolation { parts, .. }
                    if parts.first().is_some_and(|part| part.contains("Added "))
            )),
            "expected interpolation recovery: {statements:?}"
        );
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
            Some(("vmul.f64".to_owned(), "d4, d0".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x02, 0x4b, 0x30, 0xee]),
            Some(("vadd.f64".to_owned(), "d4, d2".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x42, 0x4b, 0x30, 0xee]),
            Some(("vsub.f64".to_owned(), "d4, d2".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x0e, 0xeb, 0x80, 0xee]),
            Some(("vdiv.f64".to_owned(), "d14, d14".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x40, 0x2b, 0xb5, 0xee]),
            Some(("vcmpdz".to_owned(), "d2, d0".to_owned()))
        );
        assert_eq!(
            decode_arm32_vfp_fallback(&[0x42, 0xab, 0x38, 0xee]),
            Some(("vsub.f64".to_owned(), "d10, d2".to_owned()))
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
            &BTreeMap::new(),
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
}
