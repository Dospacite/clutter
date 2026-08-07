use std::collections::BTreeMap;

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
            let mnemonic = instruction.mnemonic().unwrap_or("unknown");
            let operands = instruction.op_str().unwrap_or("");
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
                if let Some((pool_index, target)) =
                    pool_registers.get(&normalize_register(operands)).cloned()
                    && is_named_pool_target(&target)
                {
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
    parameter_names: &[String],
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
        parameter_names,
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
}

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
        let displacement = parts
            .get(1)
            .map_or(Some(0), |operand| signed_immediate(operand))?;
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

fn infer_dispatch_selector(raw_targets: &[String]) -> (Option<String>, Vec<String>, usize) {
    let mut groups = BTreeMap::<String, (usize, std::collections::BTreeSet<String>)>::new();
    for target in raw_targets {
        let selector = target
            .rsplit_once('.')
            .map_or(target.as_str(), |(_, name)| name);
        let group = groups.entry(selector.to_owned()).or_default();
        group.0 += 1;
        group.1.insert(target.clone());
    }
    let mut ranked = groups.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.0.cmp(&left.1.0).then_with(|| left.0.cmp(&right.0)));
    let Some((selector, (frequency, targets))) = ranked.first() else {
        return (None, Vec::new(), 0);
    };
    let runner_up = ranked.get(1).map_or(0, |(_, group)| group.0);
    // A heavily obfuscated table can contain thousands of synthetic `sub_*`
    // labels plus a handful of surviving operators. Winning 5-to-1 among
    // individually unique synthetic names is not meaningful dominance. Keep
    // the small-table exact case, but require a broader quorum otherwise.
    let dominant = (*frequency == raw_targets.len() && !raw_targets.is_empty())
        || (targets.len() >= 8 && *frequency >= runner_up.saturating_mul(2));
    if !dominant {
        return (None, Vec::new(), 0);
    }
    (
        Some(selector.clone()),
        targets.iter().take(16).cloned().collect(),
        targets.len(),
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
    let parameter_names = (0..parameter_count.unwrap_or_default())
        .map(|index| format!("arg{index}"))
        .collect::<Vec<_>>();
    lift_semantics_with_names(
        abi,
        &parameter_names,
        instructions,
        block_starts,
        symbols,
        object_pool,
        None,
        None,
    )
}

fn lift_semantics_with_names(
    abi: Abi,
    parameter_names: &[String],
    instructions: &[DecodedInstruction],
    block_starts: &std::collections::BTreeSet<u64>,
    symbols: &BTreeMap<u64, Symbol>,
    object_pool: Option<&[String]>,
    field_layout: Option<&RecoveredFieldLayout>,
    receiver_class: Option<(&str, Option<&str>)>,
) -> Vec<SemanticStatement> {
    if abi == Abi::X86_64 {
        return Vec::new();
    }
    let pool_loads = recover_object_pool_loads(abi, instructions);
    let return_register = abi_return_register(abi);
    let argument_registers = abi_argument_registers(abi);
    let mut registers = BTreeMap::<String, Expression>::new();
    let mut stack = BTreeMap::<String, Expression>::new();
    for (name, register) in parameter_names.iter().zip(argument_registers) {
        registers.insert(
            (*register).to_owned(),
            Expression {
                text: name.clone(),
                confidence: EvidenceConfidence::High,
                complexity: 1,
                class_name: (name == "this")
                    .then(|| receiver_class.map(|(name, _)| name.to_owned()))
                    .flatten(),
                class_library_uri: (name == "this")
                    .then(|| {
                        receiver_class.and_then(|(_, library_uri)| library_uri.map(str::to_owned))
                    })
                    .flatten(),
            },
        );
    }
    let mut statements = Vec::new();
    let mut last_comparison = None::<Expression>;
    let entry = instructions.first().map(|instruction| instruction.address);
    for instruction in instructions {
        if Some(instruction.address) != entry && block_starts.contains(&instruction.address) {
            // Do not let a value recovered on one predecessor leak into a
            // different basic block without a real phi/join proof.
            registers.clear();
            stack.clear();
            last_comparison = None;
        }
        let operands = split_operands(&instruction.operands);
        match instruction.mnemonic.as_str() {
            "mov" | "movz" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                if let Some(value) = resolve_expression(&operands[1], &registers, object_pool) {
                    registers.insert(target, value);
                } else {
                    registers.remove(&target);
                }
            }
            "movk" if operands.len() >= 2 => {
                registers.remove(&normalize_register(&operands[0]));
            }
            "add" | "sub" | "mul" | "and" | "orr" | "eor" if operands.len() >= 3 => {
                let target = normalize_register(&operands[0]);
                let left = resolve_expression(&operands[1], &registers, object_pool);
                let right = resolve_expression(&operands[2], &registers, object_pool);
                if let (Some(left), Some(right)) = (left, right) {
                    let operator = match instruction.mnemonic.as_str() {
                        "add" => "+",
                        "sub" => "-",
                        "mul" => "*",
                        "and" => "&",
                        "orr" => "|",
                        _ => "^",
                    };
                    if let Some(expression) = binary_expression(left, operator, right) {
                        registers.insert(target.clone(), expression);
                    } else {
                        registers.remove(&target);
                    }
                } else {
                    registers.remove(&target);
                }
            }
            "asr" | "lsr" | "lsl" if operands.len() >= 3 => {
                let target = normalize_register(&operands[0]);
                if let Some(value) = resolve_expression(&operands[1], &registers, object_pool)
                    && let Some(shift) = immediate_text(&operands[2])
                {
                    let operator = if instruction.mnemonic == "lsl" {
                        "<<"
                    } else {
                        ">>"
                    };
                    if value.complexity < 32 {
                        registers.insert(
                            target.clone(),
                            Expression {
                                text: format!("({} {operator} {shift})", value.text),
                                confidence: value.confidence,
                                complexity: value.complexity + 1,
                                class_name: None,
                                class_library_uri: None,
                            },
                        );
                    } else {
                        registers.remove(&target);
                    }
                } else {
                    registers.remove(&target);
                }
            }
            "cmp" | "cmn" if operands.len() >= 2 => {
                let left = resolve_expression(&operands[0], &registers, object_pool);
                let right = resolve_expression(&operands[1], &registers, object_pool);
                last_comparison = match (left, right) {
                    (Some(left), Some(right)) => Some(Expression {
                        text: format!("{} ? {}", left.text, right.text),
                        confidence: weaker(left.confidence, right.confidence),
                        complexity: left.complexity.saturating_add(right.complexity),
                        class_name: None,
                        class_library_uri: None,
                    }),
                    _ => None,
                };
            }
            "tst" if operands.len() >= 2 => {
                let left = resolve_expression(&operands[0], &registers, object_pool);
                let right = resolve_expression(&operands[1], &registers, object_pool);
                last_comparison = match (left, right) {
                    (Some(left), Some(right)) => Some(Expression {
                        text: format!("({} & {}) ? 0", left.text, right.text),
                        confidence: weaker(left.confidence, right.confidence),
                        complexity: left.complexity.saturating_add(right.complexity),
                        class_name: None,
                        class_library_uri: None,
                    }),
                    _ => None,
                };
            }
            "stur" | "str" if operands.len() >= 2 => {
                if is_frame_slot(&operands[1]) {
                    if let Some(value) = resolve_expression(&operands[0], &registers, object_pool) {
                        stack.insert(normalize_slot(&operands[1]), value);
                    }
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1])
                    && let Some(receiver) = registers.get(&base).cloned()
                    && let Some((field_offset, field)) =
                        recovered_field(field_layout, &receiver, displacement)
                    && let Some(value) = resolve_expression(&operands[0], &registers, object_pool)
                {
                    statements.push(SemanticStatement::FieldWrite {
                        receiver: receiver.text,
                        field: field.name.clone(),
                        offset: field_offset,
                        value: value.text,
                        confidence: weaker(receiver.confidence, value.confidence),
                        address: format!("0x{:x}", instruction.address),
                    });
                }
            }
            "ldur" | "ldr" if operands.len() >= 2 => {
                let target = normalize_register(&operands[0]);
                if is_frame_slot(&operands[1]) {
                    if let Some(value) = stack.get(&normalize_slot(&operands[1])).cloned() {
                        registers.insert(target.clone(), value);
                    } else {
                        registers.remove(&target);
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
                        },
                    );
                } else if let Some((base, displacement)) = arm_memory_address(&operands[1])
                    && let Some(receiver) = registers.get(&base).cloned()
                    && let Some((field_offset, field)) =
                        recovered_field(field_layout, &receiver, displacement)
                {
                    let expression = field_expression(&receiver.text, &field.name);
                    let confidence = receiver.confidence;
                    registers.insert(
                        target,
                        Expression {
                            text: expression.clone(),
                            confidence,
                            complexity: receiver.complexity.saturating_add(1),
                            class_name: field.value_class.clone(),
                            class_library_uri: field.value_library_uri.clone(),
                        },
                    );
                    statements.push(SemanticStatement::FieldRead {
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
                let arguments = call_arguments(abi, &registers);
                statements.push(SemanticStatement::ResolvedCall {
                    target: target.clone(),
                    arguments: arguments.clone(),
                    confidence: EvidenceConfidence::Medium,
                    address: format!("0x{:x}", instruction.address),
                });
                last_comparison = None;
                kill_caller_saved(abi, &mut registers);
                registers.insert(
                    return_register.to_owned(),
                    Expression {
                        text: format!("{}_result", sanitize_semantic_name(&target)),
                        confidence: EvidenceConfidence::Low,
                        complexity: 1,
                        class_name: symbol.and_then(|symbol| symbol.result_class.clone()),
                        class_library_uri: symbol.and_then(|symbol| symbol.library_uri.clone()),
                    },
                );
            }
            mnemonic if is_call(mnemonic) => {
                let target = operands
                    .first()
                    .and_then(|operand| resolve_expression(operand, &registers, object_pool))
                    .filter(|target| is_named_pool_target(&target.text));
                let arguments = call_arguments(abi, &registers);
                kill_caller_saved(abi, &mut registers);
                last_comparison = None;
                if let Some(target) = target {
                    statements.push(SemanticStatement::ResolvedCall {
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
                        },
                    );
                }
            }
            mnemonic if is_return(mnemonic, &instruction.operands) => {
                if let Some(value) = registers.get(return_register)
                    && !value.text.starts_with("pool[")
                {
                    statements.push(SemanticStatement::Return {
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
                    &registers,
                    last_comparison.as_ref(),
                    object_pool,
                );
                if let Some(condition) = condition {
                    let target = branch_target(&instruction.operands);
                    statements.push(SemanticStatement::Condition {
                        expression: condition.text,
                        true_target: target.map(|target| format!("0x{target:x}")),
                        false_target: Some(format!("0x{:x}", instruction.next)),
                        confidence: condition.confidence,
                        address: format!("0x{:x}", instruction.address),
                    });
                }
                last_comparison = None;
            }
            mnemonic if writes_first_operand(mnemonic) => {
                if let Some(target) = operands.first() {
                    registers.remove(&normalize_register(target));
                }
            }
            _ => {}
        }
    }
    statements
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

fn abi_argument_registers(abi: Abi) -> &'static [&'static str] {
    match abi {
        Abi::Arm64V8a => &["x1", "x2", "x3", "x4", "x5", "x6", "x7"],
        Abi::ArmeabiV7a => &["r1", "r2", "r3"],
        Abi::X86_64 => &[],
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
            })
        })
}

fn normalize_register(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
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

fn is_frame_slot(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("[x29") || value.contains("[fp")
}

fn normalize_slot(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn call_arguments(abi: Abi, registers: &BTreeMap<String, Expression>) -> Vec<String> {
    abi_argument_registers(abi)
        .iter()
        .map_while(|register| registers.get(*register))
        .map(|value| value.text.clone())
        .collect()
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
    let (first_entry_offset, word_size) = match abi {
        Abi::ArmeabiV7a => (7i64, 4i64),
        Abi::Arm64V8a | Abi::X86_64 => (16i64, 8i64),
    };
    let relative = offset.checked_sub(first_entry_offset)?;
    (relative >= 0 && relative % word_size == 0)
        .then(|| usize::try_from(relative / word_size).ok())
        .flatten()
}

fn is_pool_load(abi: Abi, mnemonic: &str, operands: &str) -> bool {
    match abi {
        Abi::ArmeabiV7a | Abi::Arm64V8a => mnemonic.starts_with("ldr"),
        Abi::X86_64 => {
            matches!(mnemonic, "mov" | "movq")
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
    let operands = split_operands(inner);
    let base = normalize_register(operands.first()?);
    let displacement = operands
        .get(1)
        .map_or(Some(0), |operand| signed_immediate(operand))?;
    Some((base, displacement))
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
        });
    }
    if matches!(mnemonic, "tbz" | "tbnz") {
        let value = resolve_expression(operands.first()?, registers, object_pool)?;
        let bit = operands.get(1).and_then(|value| immediate_text(value))?;
        let operator = if mnemonic == "tbz" { "==" } else { "!=" };
        return Some(Expression {
            text: format!("({} & (1 << {bit})) {operator} 0", value.text),
            confidence: value.confidence,
            complexity: value.complexity.saturating_add(2),
            class_name: None,
            class_library_uri: None,
        });
    }
    let comparison = comparison?;
    let (left, right) = comparison.text.split_once(" ? ")?;
    let condition = mnemonic
        .trim_start_matches("b.")
        .trim_start_matches('b')
        .trim_end_matches(".w");
    let operator = match condition {
        "eq" | "z" => "==",
        "ne" | "nz" => "!=",
        "lt" | "lo" | "cc" => "<",
        "le" | "ls" => "<=",
        "gt" | "hi" => ">",
        "ge" | "hs" | "cs" => ">=",
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

    use crate::model::{Abi, ControlFlowEdgeKind, PseudoStatement, SemanticStatement};

    use super::{
        DecodedInstruction, Disassembler, DispatchTableAnalysis, MAX_RENDERED_INSTRUCTIONS,
        RecoveredFieldLayout, branch_kind, build_control_flow, direct_call_target,
        infer_dispatch_selector, lift_semantics, lift_semantics_with_names, object_pool_index,
        reachable_block_count, recover_dispatch_calls, recover_object_pool_loads,
    };

    fn instruction(address: u64, mnemonic: &str, operands: &str) -> DecodedInstruction {
        DecodedInstruction {
            address,
            next: address + 4,
            mnemonic: mnemonic.to_owned(),
            operands: operands.to_owned(),
        }
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
    fn continues_after_embedded_arm_data() {
        // Capstone does not recognize the first VFP word in this mode, but it
        // should resume at the following `bx lr` instead of losing the tail.
        let bytes = [0x00, 0x0b, 0xb0, 0xee, 0x1e, 0xff, 0x2f, 0xe1];
        let disassembly = Disassembler::new(Abi::ArmeabiV7a)
            .unwrap()
            .analyze(0x1000, &bytes, &BTreeMap::new(), None, None, None)
            .unwrap();

        assert_eq!(disassembly.evidence.unknown_bytes, 4);
        assert_eq!(disassembly.evidence.decoded_bytes, 4);
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
        assert_eq!(object_pool_index(Abi::X86_64, "rax, [r15 + 0x28]"), Some(3));
        assert_eq!(object_pool_index(Abi::Arm64V8a, "x0, [x29, #0x20]"), None);
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
            &["this".to_owned()],
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
            &["this".to_owned()],
            &instructions,
            &blocks,
            &BTreeMap::new(),
            None,
            Some(&layout),
            Some(("Unrelated", Some("package:app/model.dart"))),
        );
        assert!(!unrelated.iter().any(|statement| matches!(
            statement,
            SemanticStatement::FieldRead { .. } | SemanticStatement::FieldWrite { .. }
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

        let (selector, candidates, count) = infer_dispatch_selector(&targets);
        assert_eq!(selector, None);
        assert!(candidates.is_empty());
        assert_eq!(count, 0);
    }

    #[test]
    fn kills_caller_saved_and_cross_block_values() {
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

        let branch_sequence = [
            instruction(0x1000, "mov", "x0, #1"),
            instruction(0x1004, "b", "#0x1008"),
            instruction(0x1008, "ret", ""),
        ];
        let statements = lift_semantics(
            Abi::Arm64V8a,
            None,
            &branch_sequence,
            &BTreeSet::from([0x1000, 0x1008]),
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
