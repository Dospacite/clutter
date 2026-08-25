# Further accuracy improvements — research notes and ranked proposals

Date: 2026-08-24 (continuation of the 2026-08-23 accuracy waves in `decisions.tsv`)

This document is grounded in measured evidence from the current artifacts, not
speculation. Every claim below was checked against:

- `target/accuracy-final/{plain,obf-map,obf-raw}*` (Dart 3.12.2 nine-variant runs)
- a fresh decompile of the richer `fixtures/clutter_edge_cases` fixture
  (`target/edge-probe`, arm64, `--emit-ir`) which exercises records, patterns,
  async-star, try/catch, factories, extensions, and mixins
- the Dart SDK checkout at `~/Documents/Projects/dart-sdk`
  (287a20d2325, snapshot-compatible with 3.12.2)
- the Flutter checkout at `~/Documents/Projects/flutter`

---

## 1. Where recovery actually stands (measured)

### 1.1 Simple-app plain build (arm64, readable symbols)

| Fact | Value | Meaning |
| --- | ---: | --- |
| functions_with_signatures | 12 / 30 | 18 bodies render `(List<dynamic> args)` |
| resolved_dispatch_table_call_sites | 0 of 6 sites, 23,261 table entries | selector→Code evidence unused at call sites |
| direct calls to unnamed code | 31 of 96 resolved calls target `sub_*` | widget-constructor bodies whose Function was dropped |
| field declarations | 0 | all instance fields render as `_slot_10`/`slot17` placeholders |
| named-parameter names | 0 | every optional-named call renders positionally |
| call_target_scopes/unknown | 77 of ~155 edges | most callee identities unproven |

### 1.2 Edge-case fixture (arm64, richer constructs)

Ground truth vs recovered source tokens: `await` 5 → **0**, `try` 8 → **0**,
`switch` (patterns) 1 → **0**, `factory` 1 → **0**, `extension` 1 → **0**.
`recovered_exception_handlers: 3` proves the handler tables decode, but no
`try`/`catch` statement exists anywhere in the output — there is not even an IR node for it — `SemanticStatement`
(`src/model.rs:928`) has Return / ResolvedCall / FieldRead / FieldWrite /
Condition / StringInterpolation only.

The async body renders as `Future.delayed(...)` plus a fabricated
`while (true) {}` with a "low-confidence predicate" comment — the await
state-machine dispatch is being structurally misread rather than recognized.

### 1.3 Obfuscated raw builds

*Resolved 2026-08-24:* the 1,960 `W_UNKNOWN_INSTRUCTION` warnings are gone
(→ **0**). The final residual families were register moves, `vneg`,
`vsqrt`, `vcvt.f64.f32`, and the condition-predicated twins of every
family — all in the VFP A2 space with opcode nibble bits[23:20] == `0xb`
(the discriminator against three-register forms such as
`vsub.f64 dD, d0, dM`). See the P6 status note below for validation. The
negated unboxed-double price predicate now proves on ARM32 directly.

### 1.4 Oracle pipeline status

`functions_linked_by_vm_oracle: 0` in every scored run: the analyzer binary,
exact binding, and schema-5 emitter are built and verified
(decisions 15–18), but no scored run consumes oracle JSON yet. Meanwhile the
schema already emits exactly what the gaps need (see §3.1).

---

## 2. What the Dart SDK proves survives AOT but Clutter ignores

These are verified against `runtime/vm/raw_object.h`, `object.h`, and
`compiler/aot/precompiler.cc` of the local SDK checkout.

### 2.1 UnlinkedCall survives and carries dynamic-call selectors

`UntaggedCallSiteData` (raw_object.h:2756) holds `target_name` +
`args_descriptor`. The AOT precompiler itself mines these objects for
selector coverage (`precompiler.cc`: "case kUnlinkedCallCid … *temp_selector =
call_site.target_name()"), proving they survive in product snapshots and are
the authoritative dynamic-call record. Clutter parses them
(`fill_spec.rs:108`, 7 objects in the simple-app snapshot) but **no consumer
exists**: `grep UnlinkedCall src/snapshot/cluster/instructions.rs` → 0 hits,
and static pool labeling (`object_pool_labels`, instructions.rs:943) has no
UnlinkedCall branch, so those pool entries fall through to generic labels.

Consequence today: dynamic-call renderings stop at "candidate set
unresolved", and switchable-call sites (`ldr xN,[PP,#unlinked]; blr stub`)
are counted as indirect calls with no name evidence.

### 2.2 SubtypeTestCache arrays carry instantiated type arguments

115 SubtypeTestCache objects in the simple-app snapshot (cid 38 in CIDS_392).
Each holds a cache array of `[cid, type]` pairs for a specific `is`/`as`
site. Clutter parses refs(1) (`fill_spec.rs:110`) but never walks the array,
so recovered type tests cannot be attributed to concrete instantiated types
even though the snapshot retains them.

### 2.3 Schema-5 oracle payloads are emitted but dropped

Verified consumption matrix across `src/`:

| Analyzer emission (patch) | Rust consumer | Status |
| --- | --- | --- |
| per-function `argument_descriptor` (counts + names + required flags) | `cli.rs:438` keys descriptors by (library, owner, name); `runtime_trace.rs` schema | parsed; names ignored by renderer |
| `StaticCalls` pseudo-object (pool_index → target offset + owner id/name/is_static/param count) | `vm_oracle.rs:1119` | reduced to a **count** (`static_call_targets`), rows discarded |
| `Dispatch` metadata (selector_index → Code owner_name/target_offset) | none | `"Dispatch"` never matched outside the patch |
| `pc_descriptors` histograms | count-only in evidence | not joined |
| `unboxed_field_bitmap` per class | field marked dead_code | never read |
| `ClassRanges` CID runs | stored as ranges in evidence | not used to attribute synthetic entries |

The single highest-value example: StaticCalls gives an exact
pool_index → Function-name map. Clutter's ARM64 lifter already propagates
pool indices (`x27` affine provenance), so joining that map would rename
every pool-seeded direct call — including the 31 `sub_*` calls in the plain
build — with zero new heuristics. Today that join does not happen because the
rows are thrown away at parse time.

### 2.4 Closure objects retain their Function (and thus signature)

83 Closure objects in the simple-app snapshot. `Closure::function()`
(object.h:2399+) links each closure instance to its full Function object with
FunctionType. The static recovery path iterates `isolate.named` Functions and
Code owners but never follows Closure→Function edges, so anonymous-closure
signatures that did survive are only picked up indirectly
(`signatures_from_related_functions: 2`).

### 2.5 Exception-handler tables decode but have no IR/rendering

`recovered_exception_handlers: 3` (edge probe) and decoded handler rows exist
in IR (`run`: `handler_pc_offset 3068, needs_stack_trace, has_catch_all`),
yet neither `SemanticStatement` nor the structurer can express try/catch, so
the information dies at rendering.

---

## 3. Ranked proposals

Ranking = expected accuracy gain × breadth ÷ risk, consistent with the
governing rules in `ACCURACY_ARCHITECTURE.md` (loaders append facts, tiers
never upgrade, projection may discard).

### P1 — Join schema-5 StaticCalls + Dispatch metadata into call resolution

*Status 2026-08-24 (evening): the exact-revision schema-5 analyzers now build for
all three ABIs (`vm-oracle/build-exact-analyzers.sh` at d684a576a6; the patch's
hunk 18 needed 3.12.2 field renames — `info_.snapshot_text` →
`info_.vm_isolate_instructions` — and the `snapshot_data` block context). The
full oracle path works end-to-end through `clutter vm-oracle --adb <serial>` on
an x86_64 emulator, including the cryptographic binding.

Measured effect of `--vm-oracle` on the obfuscated simple_app corpus (arm64):

| Metric | static-only | + schema-5 oracle |
| --- | ---: | ---: |
| resolved_direct_call_sites | 6,630 | **27,563** |
| functions_with_signatures | 2,269 | **2,746** |
| recovered_parameter_types | 2,079 | **2,646** |
| typed_field_declarations | 484 | **629** |
| classes_with_recovered_metadata | 2,526 | **3,311** |
| functions_linked_by_vm_oracle | 0 | **12,038** |

Two analyzer limitations discovered by measurement (they bound what P1 can
still deliver):

1. `static_calls` only captures pool entries whose payload is a *tagged Code*
   object — in fully-AOT x64/arm64 pools that is exactly the 133 VM-stub
   entries; application call targets use other encodings, so the section adds
   no app-level call names yet. The per-object graph (Code.owner → Function)
   is what actually delivers the resolution gains above.
2. `dispatch_metadata.code_entry_count` is always 0: the deserializer decodes
   the dispatch table into the raw `DispatchTable` (entry-point words), never
   into `object_store()->dispatch_table_code_entries()` (that array exists only
   during precompilation). Selector→Code naming therefore needs to mine the
   serialized table stream (which Clutter already locates independently in
   `cluster/dispatch.rs`) plus the code-index map, not the object store.

*(original proposal below)*

- Parse the per-row payload of `StaticCalls` and `Dispatch` into typed maps:
  `pool_index → (target_offset, size, owner_id, owner_name?, is_static?,
  param_count)` and `selector_index → (owner_name, target_offset)`.
- Feed `static_calls` into the existing pool-provenance path: when a call's
  provenance resolves to pool index *k* and the map has *k*, emit a
  `Proven`-tier resolved call instead of `sub_*`/`aot.invoke`.
- Feed dispatch rows into `dispatch_call_evidence`
  (`disassembly.rs:1035`): replace the frequency-quorum selector guess with
  the authoritative selector→owner mapping, keeping the quorum as fallback
  when the oracle is absent.
- Expected effect on the current corpus: the 31 `sub_*` plain-build calls and
  `resolved_dispatch_table_call_sites: 0` both become named/proven;
  `call_target_scopes/unknown` collapses.
- Guardrail: these facts enter through the bound-oracle envelope, so they
  inherit exact-subject validation automatically.

### P2 — Consume UnlinkedCall selectors + argument descriptors at dynamic sites

*Status 2026-08-24 (late): selector inference rebuilt on distinct
implementations; UnlinkedCall finding recorded.*

- `infer_dispatch_selector` previously grouped raw table slots by member name,
  so one widely-shared implementation occupying hundreds of displaced rows
  could outvote the true selector, and sparse readable names in an opaque table
  could pass a quorum measured against slot counts (not implementations). It
  now collapses slots to *distinct Code labels* first: one implementation is a
  proven selector; otherwise a name wins only with a strict 2:1 majority over
  the runner-up, ≥3 named implementations, and ≥25% coverage of the swept set.
  Non-synthetic implementations stay as bounded candidate evidence when no name
  is provable; purely synthetic sweeps stay silent. On the fixtures this turns
  every recognized dispatch site from "candidate set unresolved" into a bounded
  candidate list (plain: 6/6 sites; obfuscated: 3,584/3,584), while proving no
  names the evidence cannot support.
- **UnlinkedCall finding (measured):** the pool-label recognizer is correct but
  site-less on current fixtures. Instrumented scan of the simple_app global
  pool shows 11 entries referencing 7 UnlinkedCall objects and producing
  `dynamicCall(...)` labels, yet no app-scoped instruction loads those slots —
  full AOT devirtualized the switchable calls, leaving the objects reachable
  only from runtime miss paths. The probe fixture behaves identically. Naming
  dynamic calls therefore depends on dispatch-table recovery above, not on more
  switchable-call recognizers.

*(original proposal below)*

- Extend `object_pool_labels` (instructions.rs:943) with an UnlinkedCall
  branch: label = `dynamicCall("<target_name>", arity=…)`, restoring the
  obfuscation-map token when available.
- Teach the ARM64/ARM32/x64 indirect-call recognizers the three switchable
  shapes (SingleTarget/IC/Megamorphic stub sequences) so a `blr` whose
  scratch register was loaded from an UnlinkedCall-labeled slot renders as
  `receiver.<selector>(…)` evidence with descriptor arity.
- This is obfuscation-independent (names live in CallSiteData, not symbols)
  and directly attacks `indirect_call_sites`/megamorphic unknowns.
- Cross-check with `precompiler.cc` semantics: `DropImplicitCallPrefix`
  handling should be mirrored so getters/setters keep Dart property syntax.

### P3 — Try/catch recovery from exception-handler tables

*Status 2026-08-24 (late): landed, renderer-level with an evidence-safe gate.*

- `RecoveredExceptionHandler` now carries its row position as `try_index`
  (runtime/vm/exceptions.h: "The index into the ExceptionHandlers table
  corresponds to the try_index"), and `RecoveredCodeMetadata::try_regions()`
  joins each real handler row with the pc-descriptor rows carrying that index.
  Those descriptor rows are the only surviving record of which instructions a
  try block protected — the handler table stores just the catch entry. Note
  code_descriptors.cc:47: precompiled snapshots emit descriptor rows *only*
  for exceptions/relocations/yields, so a plain body has zero rows; arm32
  snapshots (uncompressed pointer layout) store no descriptor payload at all,
  so try recovery is arm64/x64-only by construction today.
- The renderer brackets a protected range only when a root-level structured
  child's statement span starts inside it and its handler decoded into
  statements (`try { // protected range …`). This keeps the `catch` clause
  from ever closing across a structurer-opened `if`/`while` brace — the naive
  statement-level bracketing first tried produced invalid nesting and was
  rejected. Handler bodies recovered normally render inside the catch;
  otherwise the clause closes with an explicit unresolved-region note.
- Verified on the rebuilt arm64 probe fixture: `e09TryRethrow` now renders
  real `try { … } catch (e) { … }` (previously zero `try` tokens plus a
  fabricated `while (true)` in the catch path), parses clean under
  `dart analyze` (0 errors), and `dart format` accepts it.

*Status 2026-08-25: the remaining catch-path `while (true)` fabrication is
fixed at the root. The generated handler row's target block
(`is_generated == true`, finally/async dispatch cleanup) was reachable only
through exception dispatch whose incoming edges are not decoded, so it kept
a full dominator set that poisoned every successor's set — its branch back
into the body then looked like a loop latch. `structure_body` now computes
entry reachability and requires a predecessor to be reachable before its
edge can vote in the loop-header test, in addition to the existing handler
exclusion; only real handlers render catch banners (`catch_banners`
parameter). Probe `e09TryRethrow`: both fabricated loops gone, statements
render as straight-line code; e24Knot's genuine do-while knot still
structures. Regression: `poisoned_dominator_edge_does_not_fabricate_loop`.*

*(original proposal below)*

- Add `TryCatch { try_region, catch_regions, exception_var, confidence }`
  to `SemanticStatement` and a corresponding structure node.
- Use decoded handler rows (already present: handler_pc_offset,
  needs_stack_trace, has_catch_all, outer_try_index) to bracket regions:
  handlers whose try_index chain nests give lexical nesting for free.
- Render `on X catch (e)` only when SubtypeTestCache/P1-type evidence proves
  the guard type; otherwise `catch (e)` with an explicit unresolved guard
  comment — never invent a type.
- The async machine's `has_async_exception_handler` flag then also anchors
  P4.

*Status 2026-08-24: designed but not implemented. Edge-probe measurement
refined the plan — descriptor-derived try ranges are unreliable inside
async bodies (the compiled form wraps catch-body code inside the protected
pc range), so bracketing should key on non-generated handler rows
(`is_generated == false`) plus the handler's cid compare
(`cmp x1, #EdgeFailure-cid`) as the guard-type signal. No IR/rendering
change landed yet; ground truth still `try 8 → 0`.*

### P4 — Async state-machine recognition instead of fabricated loops
*(fixes a correctness smell: today's `while (true) {}` with a low-confidence
note is worse than silence)*

- Recognize the awaiter lowering: SuspendState/await-stub call pattern plus
  `:await_jump_var`/`:await_ctx_var` frame slots (frame-index constants recur
  identically across ABIs; compiler_lab can pin them).
- When recognized, suppress branch/loop structuring inside the dispatch
  region and render `await expr;` at the suspension points identified by the
  Future-typed awaited operands; leave remaining fragments under the existing
  fragment comment.
- The VM oracle's `is_async` flag (VmFunctionEvidence) provides the seed
  classification without heuristics; the static-only fallback uses the
  Future-return heuristic that already exists (`detected_async_style`,
  dart.rs:2321).

*Status 2026-08-24: first half implemented. Unpredicated `While` nodes no
longer render as fabricated `while (true)` when the body's async style is
proven (`BodyEmitter.is_async_machine`); `detected_async_style` gained
named-runtime-collaborator signals (`Future.delayed`, `_StreamIterator`,
`_AsyncStarStreamController`, `_Future.await`) so plain snapshots without
stub symbols qualify. Edge-probe fabricated-loop sites dropped 3 → 1; the
residual (a FutureBuilder closure whose async machinery stayed unnamed)
needs the oracle `is_async` seed or split-debug stub names. `await expr;`
rendering at suspension points remains open.*

*Status 2026-08-25 (N3c of `novel_directions_2026-08-25.md`): the yield-index
fallback in `detected_async_style` no longer demands a resolved
Iterable-shaped return type. Precompiled snapshots emit pc-descriptor rows
only for exceptions, relocations, and yields (`code_descriptors.cc`), so any
body with a `yield_index >= 0` row is a suspension machine; bodies with named
async collaborators or VM flags are claimed earlier, so what reaches this
fallback is a generator whose flavor evidence was erased. The probe's
`e11SyncGen` (no surviving signature) now renders the generator banner and
its dispatch cycle is suppressed from loop structuring.*

### P5 — Closure→Function signature walk
*(cheap, bounded)*

- In function recovery, follow Closure.reference edges (cid 57, refs(6)) to
  their Function and register the occurrence with its FunctionType-derived
  signature, tiered `Inferred` (object identity is exact; the link
  closure-instance→occurrence is the soft part).
- Also lift `parent_function` from ClosureData for lexical nesting — this
  generalizes the existing DWARF/source-line containment heuristic and works
  without debug info.

### P6 — ARM32 VFP decoder completion
*(biggest single-ABI win; mechanical)*

- Extend `decode_arm32_vfp_fallback` beyond vmovd/vcmp to the four measured
  families: VADD/VSUB/VMUL/VDIV/NMUL (opc1 0x3/0x4 group), VCVT family
  (0x2/0xb groups incl. signed/unsigned fixed/int conversions), VFMA/VFMS
  (0x8 group), VSTR/VLDR (0xb with L bit), plus their conditional-prefix
  variants (9.4% of the unknown words).
- Wire results into the existing expression builder so double math renders
  (`a * b`, `x ~/ y` equivalents), reusing the ARM64 double-expression
  machinery; add compiler_lab cases (`double_pool` already exists) with
  ARM32 expectations.
- Risk is low because encoding masks come from Dart's own assembler macros
  (same source the current fallback cites).

*Status 2026-08-24: COMPLETE. The four measured families plus register
moves, `vneg`, `vsqrt`, `vcvt.f64.f32`, and all condition-predicated twins
landed in `decode_arm32_vfp_fallback` (gate: opcode nibble bits[23:20] ==
1011, validated word-by-word against binutils over every residual corpus
word and against all pre-existing test vectors, including the
three-register `vsub.f64 dD, d0, dM` collision). obf-raw-arm32
`W_UNKNOWN_INSTRUCTION`: 1,960 → 0; the nine-variant evaluator passes with
the negated double predicate proven on ARM32 for the first time.*

### P7 — Field-name recovery from unboxed-field bitmaps + ClassRanges
*(uses two more dropped schema-5 emissions)*

- `unboxed_field_bitmap` distinguishes pointer slots from unboxed doubles/
  ints per class; combined with `instance_slots` and allocation-site CID
  provenance (already tracked), slot placeholders become typed
  (`double _slot_10`) even when Field names were tree-shaken.
- `ClassRanges` populated runs let synthetic instruction-table entries be
  attributed to "code owned by some class in [start,end]" — weaker than a
  name but strictly better than address-only identity, and it feeds the
  consensus aligner a stable key under obfuscation.

*Status 2026-08-24: bitmap half landed with schema-5 parsing
(`VmOracleEvidence.class_id_ranges` populated; consumption deferred).
Feasibility probe for the second half: the static dispatch table is
selector-major (RowFitter allocates per-selector offsets), so slots are NOT
cid-indexable without the analyzer's selector map; CID sets themselves are
cross-ABI stable (identical across all three ABIs in matrix-wave2). A
fresh exact-bound oracle run over a current fixture is the prerequisite;
deferred until that exists.*

### P8 — Named-parameter names through the renderer
*(small change, user-visible)*

- `cli.rs:438` already collects descriptor parameter names; extend the
  solved-signature outcome consumed by the renderer (currently only tier
  counts land in coverage, `output/mod.rs`) so zipped call sites render
  `label: value` when the descriptor supplied the name, keeping
  positional-only rendering when names were erased.

### P9 — Compiler-lab expansion toward the gap list

Current lab cases cover 8 constructs. Add: try/catch/finally, pattern
switch, records access, async-star, factory constructors, extension members,
`late`/nullable flow, and string multi-interpolation depth. Each case should
pin its template on all three ABIs; the fixtures/regressions.json mechanism
then locks gains into CI. This is the vehicle that makes P1–P8 verifiable
rather than aspirational.

---

## 4. Explicitly rejected directions (with reasons)

- **LLM-assisted naming as anything other than speculative** — already pinned
  to `Speculative` by design (`tier.rs`); promoting it would violate the
  merging rule.
- **Fuzzy oracle binding** ("compatible" mode) — decisions 15–16 established
  byte-exact binding after observing silent-mismatch risk; loosening trades
  the strongest correctness property in the codebase for convenience.
- **Voting disputed cross-ABI facts into output** — the formatPrice
  disagreement is the canonical regression test; consensus stays verbatim.
- **Guessed default values or positional parameter names** — Full AOT
  deliberately erases them; inventing them converts a sound tool into an
  unsound one.

## 5. Emulator/oracle positioning (decided 2026-08-23)

An Android emulator is not an independent lever: the x64c analyzer already
runs natively (decision 18), and the oracle covers everything deserialization
can prove. Emulator work buys exactly two things: hosting ARM/ARM64 analyzer
builds (`--adb auto` path, already implemented) and the runtime-trace channel
(`clutter.runtime-trace/v1`) for megamorphic/shared-body disambiguation —
valuable, but strictly after P1–P5 make the static side consume what already
exists.

## 6. Suggested sequencing

1. ~~P1 (+P8 while touching the same seam) → rescore nine-variant matrix.~~
   **Done 2026-08-23/24.** P2 (UnlinkedCall labels), P8 (signature solver),
   P5 (closure parents), and the schema-5 bitmap half of P7 also landed in
   the same evidence wave; the emit-ir tuple-key regression it introduced
   was fixed via entry-record serialization.
2. ~~P6 (isolated decoder work) → compiler_lab `double_pool` ARM32 gate.~~
   **Done 2026-08-24** — obf-raw-arm32 unknowns 1,960 → 0.
3. P2 follow-through: the switchable-call *recognizers* (ARM32/x64 shapes)
   beyond the ARM64 selector path, watching indirect-call resolution.
4. P3+P4 remainder together (both touch the structurer): TryCatch IR node +
   `await expr;` suspension rendering; edge-case scorer extension
   (`try 8 → 0`, `await 5 → 0` ground truth still open).
5. Oracle run over a current fixture unlocks: P1's StaticCalls/Dispatch
   joins, P7 ClassRanges attribution, the async `is_async` seed for P4's
   residual site.
6. P9 lab expansion as each construct gains support.
7. Keep `test/tool/compiler_lab.sh` green after every step; any template drift
   pins a new regression before proceeding.

Everything above keeps the invariant that has made the recent waves stick:
new evidence enters at the lowest tier it can prove, joins happen on exact
identities, and the evaluator decides whether the generated Dart actually
improved.
