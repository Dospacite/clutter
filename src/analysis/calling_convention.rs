//! Dart AOT calling conventions.
//!
//! Dart 3.4 introduced register-based argument passing for AOT code
//! (`DartCallingConvention` in `runtime/vm/constants_<arch>.h`, allocated by
//! `ComputeCallingConvention` in
//! `runtime/vm/compiler/backend/dart_calling_conventions.cc`). Every
//! supported Dart release (3.4 through 3.12) uses the same register lists:
//!
//! | ABI   | CPU argument registers      | FPU argument registers |
//! | ----- | --------------------------- | ---------------------- |
//! | ARM64 | `x1 x2 x3 x5 x6 x7`         | `v0`..`v5`             |
//! | ARM32 | `r1 r2 r3 r8`               | `q0`..`q3` (`d0 d2 d4 d6`) |
//! | x64   | `rdi rsi rdx rbx r8 r9`     | `xmm1`..`xmm6`         |
//!
//! Tagged values and unboxed integers take CPU registers (a register pair
//! for unboxed `int` on 32-bit targets), unboxed doubles take the FPU
//! sequence, and only the first `Function::MaxNumberOfParametersInRegisters`
//! parameters are eligible at all. Everything else lives on the caller's
//! stack, allocated from the last parameter backwards so the last stacked
//! parameter sits nearest the frame.
//!
//! The runtime snapshot does not record the compiler's per-function
//! decision (unboxing metadata and `must_use_stack_calling_convention` are
//! kernel-only), so [`resolve_parameters`] combines the rules that are
//! derivable from the function kind with constraints observed in the callee
//! body and reports which locations the evidence actually pins down.

use crate::model::{Abi, RecoveredFunction, RecoveredFunctionKind};

/// Target facts the lifter needs to place arguments, frames and returns.
#[derive(Debug)]
pub(crate) struct TargetLayout {
    pub word_size: i64,
    pub cpu_arguments: &'static [&'static str],
    /// FPU argument registers spelled the way the lifter keys scalar double
    /// values (`d0` on ARM, `xmm1` on x64).
    pub fpu_arguments: &'static [&'static str],
    pub stack_pointer: &'static str,
    pub frame_pointer: &'static str,
    pub return_register: &'static str,
    pub fpu_return_register: &'static str,
    /// Byte offset of the monomorphic-checked entry's fall-through into the
    /// normal entry (`Instructions::kPolymorphicEntryOffsetAOT`).
    pub polymorphic_entry_offset: u64,
    /// CPU registers the Dart register allocator never assigns and Dart
    /// calls therefore preserve (`kReservedCpuRegisters` minus the
    /// assembler temporaries and link register, which calls clobber).
    preserved_registers: &'static [&'static str],
}

static ARM64: TargetLayout = TargetLayout {
    word_size: 8,
    cpu_arguments: &["x1", "x2", "x3", "x5", "x6", "x7"],
    fpu_arguments: &["d0", "d1", "d2", "d3", "d4", "d5"],
    stack_pointer: "x15",
    frame_pointer: "x29",
    return_register: "x0",
    fpu_return_register: "d0",
    polymorphic_entry_offset: 24,
    // SP(x15), FP(x29), PP(x27), THR(x26), HEAP_BITS(x28), NULL(x22),
    // platform x18, DISPATCH_TABLE(x21).
    preserved_registers: &["x15", "x29", "x27", "x26", "x28", "x22", "x18", "x21"],
};

static ARM32: TargetLayout = TargetLayout {
    word_size: 4,
    cpu_arguments: &["r1", "r2", "r3", "r8"],
    fpu_arguments: &["d0", "d2", "d4", "d6"],
    stack_pointer: "r13",
    frame_pointer: "r11",
    return_register: "r0",
    fpu_return_register: "d0",
    polymorphic_entry_offset: 16,
    // SP, FP, PP(r5), THR(r10), PC, DISPATCH_TABLE/NOTFP(r7).
    preserved_registers: &["r13", "r11", "r5", "r10", "r15", "r7"],
};

static X64: TargetLayout = TargetLayout {
    word_size: 8,
    cpu_arguments: &["rdi", "rsi", "rdx", "rbx", "r8", "r9"],
    fpu_arguments: &["xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6"],
    stack_pointer: "rsp",
    frame_pointer: "rbp",
    return_register: "rax",
    fpu_return_register: "xmm0",
    polymorphic_entry_offset: 22,
    // SP, FP, PP(r15), THR(r14).
    preserved_registers: &["rsp", "rbp", "r15", "r14"],
};

impl TargetLayout {
    pub(crate) fn of(abi: Abi) -> &'static TargetLayout {
        match abi {
            Abi::Arm64V8a => &ARM64,
            Abi::ArmeabiV7a => &ARM32,
            Abi::X86_64 => &X64,
        }
    }

    /// Byte offset from the established frame pointer to the last stacked
    /// parameter: `(param_end_from_fp + 1)` words, i.e. past the saved FP
    /// and return address on every supported architecture.
    pub(crate) fn incoming_frame_offset(&self) -> i64 {
        2 * self.word_size
    }

    /// Byte offset from the entry stack pointer (before any prologue) to the
    /// last stacked parameter. x64 `call` pushed the return address; ARM
    /// keeps it in the link register.
    pub(crate) fn incoming_entry_sp_offset(&self) -> i64 {
        if self.stack_pointer == "rsp" {
            self.word_size
        } else {
            0
        }
    }

    fn int64_stack_words(&self) -> usize {
        if self.word_size == 4 { 2 } else { 1 }
    }

    fn double_stack_words(&self) -> usize {
        (8 / self.word_size) as usize
    }

    /// Whether a Dart-to-Dart call may leave `register` holding a different
    /// value. Dart code has no callee-saved allocatable registers: the
    /// register allocator blocks every allocatable CPU and FPU register at a
    /// call, so only the VM's reserved registers survive. `register` uses
    /// the lifter's normalized spelling.
    pub(crate) fn is_clobbered_by_call(&self, register: &str) -> bool {
        !self.preserved_registers.contains(&register)
    }

    /// Index of `register` in the CPU argument window.
    pub(crate) fn cpu_argument_index(&self, register: &str) -> Option<usize> {
        self.cpu_arguments
            .iter()
            .position(|candidate| *candidate == register)
    }

    /// Index of `register` in the FPU argument window.
    pub(crate) fn fpu_argument_index(&self, register: &str) -> Option<usize> {
        self.fpu_arguments
            .iter()
            .position(|candidate| *candidate == register)
    }
}

/// How one argument value travels between caller and callee.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum Representation {
    Tagged,
    UnboxedInt64,
    UnboxedDouble,
}

/// Where one argument travels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentLocation {
    Register(&'static str),
    /// Low and high halves of an unboxed `int` on a 32-bit target.
    RegisterPair(&'static str, &'static str),
    FpuRegister(&'static str),
    /// `word` counts from the last stacked parameter: word 0 sits at
    /// `FP + incoming_frame_offset` in the callee and at `[SP]` of the
    /// caller's outgoing area. `words` is the slot width.
    Stack {
        word: usize,
        words: usize,
    },
}

impl ArgumentLocation {
    /// Byte displacement of a stack location from the callee's frame
    /// pointer, or from the caller's stack pointer at the call.
    pub(crate) fn frame_displacement(self, layout: &TargetLayout) -> Option<i64> {
        match self {
            Self::Stack { word, .. } => {
                Some(layout.incoming_frame_offset() + word as i64 * layout.word_size)
            }
            _ => None,
        }
    }

    pub(crate) fn entry_sp_displacement(self, layout: &TargetLayout) -> Option<i64> {
        match self {
            Self::Stack { word, .. } => {
                Some(layout.incoming_entry_sp_offset() + word as i64 * layout.word_size)
            }
            _ => None,
        }
    }
}

/// Mirrors `ComputeCallingConvention`: the first `max_in_registers`
/// parameters take registers from their representation's sequence until it
/// runs out; the rest are stacked from the last parameter backwards.
pub(crate) fn assign_locations(
    layout: &TargetLayout,
    max_in_registers: usize,
    representations: &[Representation],
) -> Vec<ArgumentLocation> {
    let mut next_cpu = 0usize;
    let mut next_fpu = 0usize;
    let mut locations = vec![None; representations.len()];
    for (index, representation) in representations.iter().enumerate() {
        if index >= max_in_registers {
            continue;
        }
        locations[index] = match representation {
            Representation::Tagged => layout.cpu_arguments.get(next_cpu).map(|register| {
                next_cpu += 1;
                ArgumentLocation::Register(register)
            }),
            Representation::UnboxedInt64 if layout.word_size == 4 => {
                (next_cpu + 2 <= layout.cpu_arguments.len()).then(|| {
                    let pair = ArgumentLocation::RegisterPair(
                        layout.cpu_arguments[next_cpu],
                        layout.cpu_arguments[next_cpu + 1],
                    );
                    next_cpu += 2;
                    pair
                })
            }
            Representation::UnboxedInt64 => layout.cpu_arguments.get(next_cpu).map(|register| {
                next_cpu += 1;
                ArgumentLocation::Register(register)
            }),
            Representation::UnboxedDouble => layout.fpu_arguments.get(next_fpu).map(|register| {
                next_fpu += 1;
                ArgumentLocation::FpuRegister(register)
            }),
        };
    }
    let mut word = 0usize;
    for index in (0..representations.len()).rev() {
        if locations[index].is_some() {
            continue;
        }
        let words = match representations[index] {
            Representation::Tagged => 1,
            Representation::UnboxedInt64 => layout.int64_stack_words(),
            Representation::UnboxedDouble => layout.double_stack_words(),
        };
        locations[index] = Some(ArgumentLocation::Stack { word, words });
        word += words;
    }
    locations
        .into_iter()
        .map(|location| location.expect("assigned"))
        .collect()
}

/// What a parameter's declared type allows its runtime representation to
/// be. TFA may unbox `int` and `double` parameters; every other type is
/// passed tagged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeclaredRepresentation {
    Tagged,
    MaybeUnboxedInt,
    MaybeUnboxedDouble,
    /// No declared type survived.
    Unknown,
}

impl DeclaredRepresentation {
    pub(crate) fn from_type_name(display_name: Option<&str>) -> Self {
        match display_name.map(str::trim) {
            None => Self::Unknown,
            // Nullable numbers stay boxed: the unboxed representations have
            // no null value.
            Some("int") => Self::MaybeUnboxedInt,
            Some("double") => Self::MaybeUnboxedDouble,
            Some(_) => Self::Tagged,
        }
    }

    /// Representations worth distinguishing on `layout`, most likely
    /// first. TFA unboxes a non-nullable `int`/`double` parameter unless the
    /// member is reachable from native code, dynamically overridden, or
    /// otherwise forced to the boxed convention. An unboxed `int` occupies
    /// the same single CPU register or stack word as a tagged value on
    /// 64-bit targets, so only 32-bit targets enumerate it.
    fn choices(self, layout: &TargetLayout) -> &'static [Representation] {
        let wide = layout.word_size == 8;
        match self {
            Self::Tagged => &[Representation::Tagged],
            Self::MaybeUnboxedInt if wide => &[Representation::Tagged],
            Self::MaybeUnboxedInt => &[Representation::UnboxedInt64, Representation::Tagged],
            Self::MaybeUnboxedDouble => &[Representation::UnboxedDouble, Representation::Tagged],
            Self::Unknown if wide => &[Representation::Tagged, Representation::UnboxedDouble],
            Self::Unknown => &[
                Representation::Tagged,
                Representation::UnboxedDouble,
                Representation::UnboxedInt64,
            ],
        }
    }
}

/// Bounds on `Function::MaxNumberOfParametersInRegisters` derivable from the
/// runtime Function object. The unboxing metadata that can force the stack
/// convention (entry points, natives, dynamically overridden members) or
/// shrink the window for instance members is kernel-only, so the lower
/// bound usually stays open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegisterWindow {
    pub min: usize,
    pub max: usize,
}

impl RegisterWindow {
    pub(crate) fn for_function(
        kind: Option<RecoveredFunctionKind>,
        is_generic: Option<bool>,
        fixed_parameters: Option<usize>,
        layout: &TargetLayout,
    ) -> Self {
        use RecoveredFunctionKind as Kind;
        let stack_only = matches!(
            kind,
            Some(
                Kind::Closure
                    | Kind::ImplicitClosure
                    | Kind::NoSuchMethodDispatcher
                    | Kind::InvokeFieldDispatcher
                    | Kind::DynamicInvocationForwarder
                    | Kind::MethodExtractor
                    | Kind::FfiTrampoline
                    | Kind::FieldInitializer
                    | Kind::Irregexp
            )
        ) || is_generic == Some(true);
        if stack_only {
            return Self { min: 0, max: 0 };
        }
        // With no surviving signature the window is bounded only by the
        // register files themselves.
        let max =
            fixed_parameters.unwrap_or(layout.cpu_arguments.len() + layout.fpu_arguments.len());
        Self { min: 0, max }
    }
}

/// Parameter-location constraints read from a callee body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BodyEvidence {
    /// CPU argument registers read before any write on some path from the
    /// normal entry, by window index.
    pub live_cpu: Vec<bool>,
    /// FPU argument registers read before any write, by window index.
    pub live_fpu: Vec<bool>,
    /// Incoming stack words read, counted from the last stacked parameter.
    pub stack_words: std::collections::BTreeSet<usize>,
}

impl BodyEvidence {
    fn is_empty(&self) -> bool {
        !self.live_cpu.iter().any(|live| *live)
            && !self.live_fpu.iter().any(|live| *live)
            && self.stack_words.is_empty()
    }
}

/// How strongly the evidence pins a parameter's location.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocationProof {
    /// Every hypothesis consistent with the body and the kind rules places
    /// the parameter here.
    Proven,
    /// Several consistent hypotheses disagree; this is the most likely one
    /// (the compiler's default when no unboxing or stack override applies).
    Assumed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedParameter {
    pub location: ArgumentLocation,
    pub representation: Representation,
    pub proof: LocationProof,
}

/// Parameter locations resolved for one function body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ResolvedParameters {
    pub parameters: Vec<ResolvedParameter>,
    /// Observed incoming locations no parameter accounts for (unknown
    /// parameter count). Stack words count from the last parameter.
    pub unexplained_cpu: Vec<usize>,
    pub unexplained_fpu: Vec<usize>,
    pub unexplained_stack_words: Vec<usize>,
}

/// Upper bound on hypotheses enumerated per function before falling back to
/// the default assignment.
const MAX_HYPOTHESES: usize = 4096;

/// Chooses one location and representation per parameter.
///
/// `declared` has one entry per parameter including implicit ones (the
/// receiver or closure context in slot zero). The only parameters eligible
/// for registers are the fixed ones, so `window.max` never exceeds the
/// fixed-parameter count supplied by the caller. Each hypothesis (register
/// window size × representation of every maybe-unboxed parameter) is scored
/// against the body: an observed read at a location no parameter occupies
/// is a contradiction. The locations shared by every least-contradicted
/// hypothesis are proven; the rest follow the compiler default (largest
/// window, declared non-nullable numbers unboxed unless the body proves
/// otherwise).
pub(crate) fn resolve_parameters(
    layout: &TargetLayout,
    window: RegisterWindow,
    declared: &[DeclaredRepresentation],
    evidence: &BodyEvidence,
) -> ResolvedParameters {
    let window = RegisterWindow {
        min: window.min.min(declared.len()),
        max: window.max.min(declared.len()),
    };
    let mut hypotheses = Vec::new();
    let variable = declared
        .iter()
        .enumerate()
        .filter(|(_, declared)| declared.choices(layout).len() > 1)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let combinations = variable
        .iter()
        .try_fold(1usize, |product, index| {
            product.checked_mul(declared[*index].choices(layout).len())
        })
        .unwrap_or(usize::MAX);
    let window_sizes = window.max.saturating_sub(window.min) + 1;
    let enumerate = combinations
        .checked_mul(window_sizes)
        .is_some_and(|total| total <= MAX_HYPOTHESES);
    for max_in_registers in (window.min..=window.max).rev() {
        if !enumerate {
            break;
        }
        for combination in 0..combinations {
            let mut remainder = combination;
            let representations = declared
                .iter()
                .map(|declared| {
                    let choices = declared.choices(layout);
                    if choices.len() == 1 {
                        return choices[0];
                    }
                    let choice = choices[remainder % choices.len()];
                    remainder /= choices.len();
                    choice
                })
                .collect::<Vec<_>>();
            // The stack convention is the fully boxed one: generic and
            // closure-like kinds never unbox, and TFA's `setFullyBoxed`
            // accompanies every forced stack convention.
            if max_in_registers == 0
                && representations
                    .iter()
                    .any(|representation| *representation != Representation::Tagged)
            {
                continue;
            }
            let locations = assign_locations(layout, max_in_registers, &representations);
            let (contradictions, explained) = score(layout, &locations, evidence);
            hypotheses.push(Hypothesis {
                max_in_registers,
                representations,
                locations,
                contradictions,
                explained,
                // Default preference: largest window, then each parameter's
                // most likely representation (combination zero).
                preference: combination,
            });
        }
    }
    if hypotheses.is_empty() {
        let representations = vec![Representation::Tagged; declared.len()];
        let locations = assign_locations(layout, window.max, &representations);
        return finish(layout, &locations, &representations, None, evidence);
    }
    let best_contradictions = hypotheses
        .iter()
        .map(|hypothesis| hypothesis.contradictions)
        .min()
        .unwrap_or_default();
    let best = hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.contradictions == best_contradictions)
        .map(|hypothesis| hypothesis.explained)
        .max()
        .unwrap_or_default();
    let consistent = hypotheses
        .iter()
        .filter(|hypothesis| {
            hypothesis.contradictions == best_contradictions && hypothesis.explained == best
        })
        .collect::<Vec<_>>();
    // Among equally consistent hypotheses prefer the compiler default: the
    // largest register window with every parameter in its most likely
    // representation.
    let chosen = consistent
        .iter()
        .min_by_key(|hypothesis| {
            (
                std::cmp::Reverse(hypothesis.max_in_registers),
                hypothesis.preference,
            )
        })
        .copied()
        .expect("at least one hypothesis");
    let agreement = (0..declared.len())
        .map(|index| {
            !evidence.is_empty()
                && consistent
                    .iter()
                    .all(|hypothesis| hypothesis.locations[index] == chosen.locations[index])
        })
        .collect::<Vec<_>>();
    finish(
        layout,
        &chosen.locations,
        &chosen.representations,
        Some(&agreement),
        evidence,
    )
}

struct Hypothesis {
    max_in_registers: usize,
    representations: Vec<Representation>,
    locations: Vec<ArgumentLocation>,
    contradictions: usize,
    explained: usize,
    preference: usize,
}

fn occupied(
    layout: &TargetLayout,
    locations: &[ArgumentLocation],
) -> (Vec<bool>, Vec<bool>, std::collections::BTreeSet<usize>) {
    let mut cpu = vec![false; layout.cpu_arguments.len()];
    let mut fpu = vec![false; layout.fpu_arguments.len()];
    let mut stack = std::collections::BTreeSet::new();
    for location in locations {
        match *location {
            ArgumentLocation::Register(register) => {
                if let Some(index) = layout.cpu_argument_index(register) {
                    cpu[index] = true;
                }
            }
            ArgumentLocation::RegisterPair(low, high) => {
                for register in [low, high] {
                    if let Some(index) = layout.cpu_argument_index(register) {
                        cpu[index] = true;
                    }
                }
            }
            ArgumentLocation::FpuRegister(register) => {
                if let Some(index) = layout.fpu_argument_index(register) {
                    fpu[index] = true;
                }
            }
            ArgumentLocation::Stack { word, words } => {
                stack.extend(word..word + words);
            }
        }
    }
    (cpu, fpu, stack)
}

fn score(
    layout: &TargetLayout,
    locations: &[ArgumentLocation],
    evidence: &BodyEvidence,
) -> (usize, usize) {
    let (cpu, fpu, stack) = occupied(layout, locations);
    let mut contradictions = 0usize;
    let mut explained = 0usize;
    for (index, live) in evidence.live_cpu.iter().enumerate() {
        if *live {
            if cpu.get(index).copied().unwrap_or(false) {
                explained += 1;
            } else {
                contradictions += 1;
            }
        }
    }
    for (index, live) in evidence.live_fpu.iter().enumerate() {
        if *live {
            if fpu.get(index).copied().unwrap_or(false) {
                explained += 1;
            } else {
                contradictions += 1;
            }
        }
    }
    for word in &evidence.stack_words {
        if stack.contains(word) {
            explained += 1;
        } else {
            contradictions += 1;
        }
    }
    (contradictions, explained)
}

fn finish(
    layout: &TargetLayout,
    locations: &[ArgumentLocation],
    representations: &[Representation],
    agreement: Option<&[bool]>,
    evidence: &BodyEvidence,
) -> ResolvedParameters {
    let (cpu, fpu, stack) = occupied(layout, locations);
    ResolvedParameters {
        parameters: locations
            .iter()
            .zip(representations)
            .enumerate()
            .map(|(index, (location, representation))| ResolvedParameter {
                location: *location,
                representation: *representation,
                proof: if agreement.is_some_and(|agreement| agreement[index]) {
                    LocationProof::Proven
                } else {
                    LocationProof::Assumed
                },
            })
            .collect(),
        unexplained_cpu: evidence
            .live_cpu
            .iter()
            .enumerate()
            .filter(|(index, live)| **live && !cpu.get(*index).copied().unwrap_or(false))
            .map(|(index, _)| index)
            .collect(),
        unexplained_fpu: evidence
            .live_fpu
            .iter()
            .enumerate()
            .filter(|(index, live)| **live && !fpu.get(*index).copied().unwrap_or(false))
            .map(|(index, _)| index)
            .collect(),
        unexplained_stack_words: evidence
            .stack_words
            .iter()
            .copied()
            .filter(|word| !stack.contains(word))
            .collect(),
    }
}

/// Declared representation of every parameter of `function`, implicit ones
/// first. Returns `None` when the parameter count did not survive.
pub(crate) fn declared_parameters(
    function: &RecoveredFunction,
) -> Option<Vec<DeclaredRepresentation>> {
    let signature = function.signature.as_ref();
    let implicit = signature
        .map(|signature| signature.implicit_parameter_count)
        .or_else(|| {
            function
                .vm_evidence
                .as_ref()
                .and_then(|evidence| evidence.implicit_parameter_count)
        })?;
    let visible = signature
        .map(|signature| {
            signature
                .fixed_parameter_count
                .saturating_add(signature.optional_parameter_count)
        })
        .or(function.parameter_count)?;
    let resolved = signature.and_then(|signature| signature.resolved.as_ref());
    let vm_parameters = function
        .vm_evidence
        .as_ref()
        .map(|evidence| evidence.parameters.as_slice())
        .unwrap_or_default();
    let mut declared = vec![DeclaredRepresentation::Tagged; implicit];
    for index in 0..visible {
        let name = resolved
            .and_then(|resolved| resolved.parameters.get(index))
            .and_then(|parameter| parameter.declared_type.as_ref())
            .map(|type_| type_.display_name.as_str())
            .or_else(|| {
                vm_parameters
                    .iter()
                    .filter(|parameter| !parameter.is_implicit)
                    .nth(index)
                    .and_then(|parameter| parameter.declared_type.as_ref())
                    .map(|type_| type_.display_name.as_str())
            });
        declared.push(DeclaredRepresentation::from_type_name(name));
    }
    Some(declared)
}

/// Number of fixed parameters including implicit ones, when known.
pub(crate) fn fixed_parameter_count(function: &RecoveredFunction) -> Option<usize> {
    if let Some(signature) = function.signature.as_ref() {
        return Some(
            signature
                .implicit_parameter_count
                .saturating_add(signature.fixed_parameter_count),
        );
    }
    let evidence = function.vm_evidence.as_ref()?;
    Some(
        evidence
            .implicit_parameter_count?
            .saturating_add(evidence.fixed_parameter_count?),
    )
}

/// Whether `function` is generic, when known. Generic functions always use
/// the stack convention.
pub(crate) fn is_generic(function: &RecoveredFunction) -> Option<bool> {
    if let Some(evidence) = function.vm_evidence.as_ref()
        && (evidence.fixed_parameter_count.is_some() || !evidence.type_parameters.is_empty())
    {
        return Some(!evidence.type_parameters.is_empty());
    }
    let signature = function.signature.as_ref()?;
    if let Some(resolved) = signature.resolved.as_ref()
        && !resolved.type_parameters.is_empty()
    {
        return Some(true);
    }
    // `FunctionType.packed_type_parameter_counts` keeps the number of the
    // function's own type parameters in its upper half.
    Some(signature.packed_type_parameter_counts >> 16 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tagged(count: usize) -> Vec<Representation> {
        vec![Representation::Tagged; count]
    }

    #[test]
    fn uses_the_real_dart_register_windows() {
        let arm64 = assign_locations(TargetLayout::of(Abi::Arm64V8a), 6, &tagged(6));
        assert_eq!(
            arm64,
            ["x1", "x2", "x3", "x5", "x6", "x7"]
                .map(ArgumentLocation::Register)
                .to_vec()
        );
        let x64 = assign_locations(TargetLayout::of(Abi::X86_64), 6, &tagged(6));
        assert_eq!(x64[3], ArgumentLocation::Register("rbx"));
        let arm32 = assign_locations(TargetLayout::of(Abi::ArmeabiV7a), 4, &tagged(4));
        assert_eq!(arm32[3], ArgumentLocation::Register("r8"));
    }

    #[test]
    fn overflows_to_stack_from_the_last_parameter() {
        let layout = TargetLayout::of(Abi::ArmeabiV7a);
        let locations = assign_locations(layout, 6, &tagged(6));
        assert_eq!(locations[4], ArgumentLocation::Stack { word: 1, words: 1 });
        assert_eq!(locations[5], ArgumentLocation::Stack { word: 0, words: 1 });
        // ARM32 words are four bytes wide.
        assert_eq!(locations[4].frame_displacement(layout), Some(12));
        assert_eq!(locations[5].frame_displacement(layout), Some(8));
    }

    #[test]
    fn doubles_use_the_fpu_sequence_without_consuming_cpu_registers() {
        let layout = TargetLayout::of(Abi::Arm64V8a);
        let locations = assign_locations(
            layout,
            3,
            &[
                Representation::Tagged,
                Representation::UnboxedDouble,
                Representation::Tagged,
            ],
        );
        assert_eq!(
            locations,
            vec![
                ArgumentLocation::Register("x1"),
                ArgumentLocation::FpuRegister("d0"),
                ArgumentLocation::Register("x2"),
            ]
        );
    }

    #[test]
    fn arm32_int64_takes_a_register_pair_or_two_stack_words() {
        let layout = TargetLayout::of(Abi::ArmeabiV7a);
        let locations = assign_locations(
            layout,
            3,
            &[
                Representation::Tagged,
                Representation::UnboxedInt64,
                Representation::UnboxedInt64,
            ],
        );
        assert_eq!(locations[1], ArgumentLocation::RegisterPair("r2", "r3"));
        // Only r8 remains: a pair cannot fit, so the value is stacked.
        assert_eq!(locations[2], ArgumentLocation::Stack { word: 0, words: 2 });
    }

    #[test]
    fn stack_only_kinds_and_generics_have_an_empty_window() {
        let layout = TargetLayout::of(Abi::Arm64V8a);
        for kind in [
            RecoveredFunctionKind::Closure,
            RecoveredFunctionKind::ImplicitClosure,
            RecoveredFunctionKind::DynamicInvocationForwarder,
        ] {
            assert_eq!(
                RegisterWindow::for_function(Some(kind), Some(false), Some(3), layout),
                RegisterWindow { min: 0, max: 0 }
            );
        }
        assert_eq!(
            RegisterWindow::for_function(
                Some(RecoveredFunctionKind::Regular),
                Some(true),
                Some(3),
                layout
            ),
            RegisterWindow { min: 0, max: 0 }
        );
    }

    #[test]
    fn body_reads_prove_an_unboxed_double_parameter() {
        let layout = TargetLayout::of(Abi::Arm64V8a);
        // formatPrice(double value) after signature shaking: the body reads
        // d0 before writing it.
        let evidence = BodyEvidence {
            live_cpu: vec![false; 6],
            live_fpu: vec![true, false, false, false, false, false],
            stack_words: Default::default(),
        };
        let resolved = resolve_parameters(
            layout,
            RegisterWindow { min: 0, max: 1 },
            &[DeclaredRepresentation::MaybeUnboxedDouble],
            &evidence,
        );
        assert_eq!(
            resolved.parameters[0].location,
            ArgumentLocation::FpuRegister("d0")
        );
        assert_eq!(resolved.parameters[0].proof, LocationProof::Proven);
    }

    #[test]
    fn stack_reads_prove_the_stack_convention() {
        let layout = TargetLayout::of(Abi::Arm64V8a);
        let evidence = BodyEvidence {
            live_cpu: vec![false; 6],
            live_fpu: vec![false; 6],
            stack_words: [0usize, 1].into_iter().collect(),
        };
        let resolved = resolve_parameters(
            layout,
            RegisterWindow { min: 0, max: 2 },
            &[
                DeclaredRepresentation::Tagged,
                DeclaredRepresentation::Tagged,
            ],
            &evidence,
        );
        assert_eq!(
            resolved.parameters[0].location,
            ArgumentLocation::Stack { word: 1, words: 1 }
        );
        assert_eq!(resolved.parameters[0].proof, LocationProof::Proven);
    }

    #[test]
    fn unobserved_locations_stay_assumed() {
        let layout = TargetLayout::of(Abi::X86_64);
        let resolved = resolve_parameters(
            layout,
            RegisterWindow { min: 0, max: 2 },
            &[
                DeclaredRepresentation::Tagged,
                DeclaredRepresentation::Tagged,
            ],
            &BodyEvidence::default(),
        );
        assert_eq!(
            resolved.parameters[1].location,
            ArgumentLocation::Register("rsi")
        );
        assert!(
            resolved
                .parameters
                .iter()
                .all(|parameter| parameter.proof == LocationProof::Assumed)
        );
    }

    #[test]
    fn dart_calls_clobber_everything_except_reserved_registers() {
        let layout = TargetLayout::of(Abi::Arm64V8a);
        assert!(layout.is_clobbered_by_call("x19"));
        assert!(layout.is_clobbered_by_call("d8"));
        assert!(!layout.is_clobbered_by_call("x22"));
        assert!(!layout.is_clobbered_by_call("x27"));
    }
}
