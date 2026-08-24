# Decompilation edge cases — probe results and failure analysis

Date: 2026-08-24
Method: differential probing against a purpose-built Flutter app, not speculation.

## How these edge cases were produced

A probe app was built whose 25 case functions each isolate exactly one language
construct (`.tmp-probe/probe/lib/probe_code.dart`, anchored from
`main.dart`). All inputs are derived from `DateTime.now()` at the call site so
full-AOT cross-function constant propagation cannot specialize the bodies; what
survives in the snapshot is the general compiled body, so any recovery failure
belongs to Clutter rather than to a constant-folded stub. Two builds were made:

- arm64-v8a release (Flutter 3.44.9 / Dart 3.12.2), decompiled with
  `clutter decompile … --abi arm64-v8a --scope app --emit-ir --no-assets`
  into `.tmp-probe/out2` (the first, constant-folded run is `.tmp-probe/out`);
- armeabi-v7a release, decompiled into `.tmp-probe/out2-arm32`.

Everything below cites recovered output vs. source and names the code path
responsible. Reproduce with:

```sh
cd .tmp-probe/probe && flutter build apk --release --target-platform android-arm64
cd ../.. && ./target/release/clutter decompile \
  .tmp-probe/probe/build/app/outputs/flutter-apk/app-release.apk \
  --abi arm64-v8a --scope app --emit-ir --no-assets --out .tmp-probe/out2
```

## Scoreboard (arm64, general-body build)

| # | Construct (source) | Recovered as | Verdict |
| --- | --- | --- | --- |
| E01 | chained interpolation | full string rebuilt, 3 of 7 holes `aot.unresolvedValue('interpolated part')` | partial |
| E02 | cascade (`..`) | correct sequence of calls + spurious capacity branch | mostly ok |
| E03 | `switch` on strings | **body missing entirely** — `UnsupportedError('AOT body unavailable')`; statements only inside an inlined closure comment | FAIL |
| E04 | shifts/masks/`~/`/`remainder` | `return aot.unresolvedRegion(...)` after one mis-attributed branch | FAIL |
| E05 | `??`, `?.`, `...?`, `??=` | 6 nested empty if/else shells, all predicates low-confidence, wrong nesting vs source | FAIL |
| E06 | record destructure `(a, b)` for-in | loop with dead predicate, destructuring gone, `.current` selector unresolved | FAIL |
| E07 | F-bounded generics `<T extends num, U extends T>` | signature survives; call renders bare `map(...)` with 5 positional args, receiver lost | partial |
| E08 | labeled `break`/`continue outer` | body absent (inlined); inner-loop shape appears in caller with constant `0 < n` predicates | FAIL |
| E09 | try/on/rethrow/finally + 2 handlers | no `try`; handler rows decoded (`exception_handlers: [pc 404 generated, pc 168 real]`) but zero IR/rendering; fabricated `while (true)` inside catch path | FAIL |
| E10 | async loop, early return over awaits | state machine rendered as `while (x2 < arg0)` + duplicated `_asyncComplete` args; `await` boundaries unnamed; trailing unresolvedRegion | FAIL |
| E11 | `sync*` yield/yield* | machine loop misread: `while (x4 < arg0) {} return false;` + unreachable-statement dump | FAIL |
| E12 | tear-offs (`list.sort`, `Duration.new`, …) | only `get:first` recovered; all tear-off identities dropped | FAIL |
| E13 | `noSuchMethod` override + dynamic member access | interpolation shell kept; `inv.memberName` rendered as bare identifier `memberName;` without receiver; `d.unknownMember` unresolved register call | partial |
| E14 | static fields w/ initializers | `version`/`counter` Field objects tree-shaken; `bump()` = empty branch + unresolvedRegion; class mislabeled `abstract` | FAIL |
| E15 | operator family (`==`, unary-, `%`, compareTo) | `==` named `operator_equals` (not Dart syntax), unary- and `%` operators absent entirely, `compareTo` keeps only one of two symmetric operands | FAIL |
| E16 | SplayTreeSet w/ nullable comparator | good call sequence; comparator arg collapsed to `null` even when non-null path exists | mostly ok |
| E17 | JSON round-trip + `cast`/`whereType` | closure shell kept but body unresolved; 5 dead low-confidence cid-compare branches (`x4 <= 55 / ==2046 / ==2105`) leaked into output | FAIL |
| E18 | double edges NaN/truncate/BigInt/radix | reduced to `return 'nan';` — the two live branches' statements exist only as "recovered literals" comments | FAIL |
| E19 | Ackermann recursion w/ optional param | structure ok-ish but duplicated garbage arguments at every recursive call (`(local8, res, ref23, local8, res)`) | partial |
| E20 | mixin order + `super.greet()` | linear chain `_MixinApplication1…` reconstructed correctly; `greet` constant-folded to `'base+combo'` (correct) | ok |
| E21 | enhanced enum members | enum rendered `abstract class E21Mode extends _Enum`; `parse`→`EnumByName|byName` ok; `isBad` getter and severity field gone | partial |
| E22 | extension on nullable generic | `safeLen` getter absent (inlined+folded); extension itself unrecoverable | FAIL |
| E23 | `Function.apply` w/ Symbols | global `apply()` called with zero args; Symbol map invisible; toString candidates listed but not resolved | FAIL |
| E24 | knot CFG (do-while, infinite loop) | best-recovered control flow: both loops + returns present; loop exit conditions inverted/misnested (`while ((arg2 >> 1) > 4)` empty body) | partial |
| E25 | string intrinsics + `identical` | only `String.+` kept; `isEmpty`/`startsWith`/`codeUnitAt`/`identical` all erased; early `return false` unconditional | FAIL |

Cross-cutting counts for this build: 39 functions recovered,
`functions_with_signatures: 13/39`, `resolved_indirect_call_sites: 0 of 10`,
bare machine temporaries leaked into output Dart (`local18`, `x16`, …):
54 occurrences in `probe_code.dart` + 44 in `main.dart`.

---

## Edge-case failures, why they happen, and how to fix them

### EC-1 String-switch bodies vanish instead of degrading (E03, E08, partially E22)

Observed: `e03StringSwitch` and `e08LabeledLoops` render as
`throw UnsupportedError('AOT body unavailable');` while their actual statements
were folded into `ProbeApp.build`'s inlined closure — where they appear only as
a literal list (`'alpha'`, `'beta-or-gamma'`, `'other'`) and a bogus
`while (0 < (local20 % 5))` skeleton.

Why: the AOT compiler inlined these small callees; the standalone Code object
disappeared. Clutter's fallback for a retained declaration without its own Code
is a hard `UnsupportedError`, and the inline evidence is used only to emit a
comment listing callee names (`dart.rs:1273`). The information that the body is
recoverable *inside another function* exists (`inlined_functions` from the
code source map's `push_function` events, decoded at
`snapshot/cluster/instructions.rs:656`) but is never turned into a pointer to
where those statements actually are.

Fix: render "body inlined into X" with a cross-reference instead of
`UnsupportedError`, and when the inlining host is known, attribute the
statement range between that function's `push_function`/`pop_function` events
back to the inlinee (the source-map entries already carry `inline_depth` and
per-event `function_reference`; the data needed is decoded today and then
discarded). This converts three of the worst failures into ordinary partial
recoveries without any new decoding.

### EC-2 Try/catch/finally decodes but cannot be expressed (E09)

Observed: `e09TryRethrow` shows `recovered_exception_handlers: 2` in coverage;
IR contains `exception_handlers: [{handler_pc_offset:404, is_generated:true},
{handler_pc_offset:168, outer_try_index:0, is_generated:false}]` for this exact
function. The rendered Dart has neither `try` nor `catch`; worse, the catch
path's error-construction got wrapped in a fabricated
`/* Loop shape recovered without a provable predicate */ while (true) { ... }`.

Why: `SemanticStatement` has no try/catch node and the structurer has no region
for it (already documented as P3 in further_improvements.md §2.5). The new
detail this probe adds: with no handler bracketing, the catch body's blocks
fall through to the generic loop-structuring path, which fabricates a
`while (true)` — i.e., the missing P3 feature doesn't just lose fidelity here,
it actively produces false control flow.

Fix: land P3 keyed on `is_generated == false` rows plus the handler's cid
compare for guard types, as designed. Until then, a cheap guard: any block that
is the target of a decoded handler row must be excluded from loop discovery and
rendered under an explicit `// catch handler (type unresolved)` banner. That
one-line exclusion removes the fabrication class even before full try/catch IR
exists.

### EC-3 Async/generator machines still misread as plain loops (E10, E11)

Observed: `e10AsyncLoop` renders `while (x2 < arg0)` containing a duplicated
argument vector `_Future._asyncComplete(futureResult, local20, futureResult,
local20)`; the `acc > 100` early return becomes an empty if/else. `e11SyncGen`
renders `while (x4 < arg0) { }` followed by `return false;` and then a
"Statements recovered from unreached machine regions" dump containing the
recursive `yield*` call.

Why: `detected_async_style` (dart.rs:2379) recognizes await machinery via call
targets like `AwaitStub`, `_Future.await`, `Future.delayed`,
`StreamIterator.`. In this snapshot the async collaborators appear as pool refs
and `_Future._asyncComplete`/`_Future.immediate` — none match the patterns, so
`is_async_machine` is false and the unpredicated-cycle suppression at
`dart.rs:1904` never fires. The sync* lowering has no detector at all: its
dispatch cycle is indistinguishable from a counted loop to the current
heuristics because the loop head happens to compare the state variable against
the parameter.

Fix: extend `detected_async_style` with `_asyncComplete`, `_Future.immediate`,
`_FutureListener`, and the `:await_jump_var` frame-slot idiom (P4 remainder);
add a SyncStar detector keyed on `_SyncStreamIterator` /
`_GeneratedStreamImpl`-style collaborators or on pc-descriptor rows carrying
`yield_index != -1` (present in this very function's `pc_descriptors`:
`yield_index: 26/54` — again decoded and unused). For sync*, treat any loop
whose body contains a statement with a yield-indexed pc descriptor as machine
dispatch, not source loop.

### EC-4 Low-confidence register predicates leak as nested empty ifs (E05, E17)

Observed: `e05NullFlow` renders six levels of
`if (x2 != lookupResult) { /* low-confidence */ } else { }` with calls
stranded in alternating arms; `e17JsonRoundTrip` emits dead branches on
`x4 <= 55`, `x4 == 2046`, `x4 == 2105` — these are subtype-test cache cid
compares from the `as`/`is` checks in `whereType/cast`, not source conditions.

Why: when the lifter can't prove what a register holds it still emits a
Condition statement tagged low-confidence, and the structurer faithfully
structures it. The renderer has confidence notes but no policy for
low-confidence conditions whose both arms are empty or whose comparison operand
is an untyped register (`x*`, `local*`). The `x4 == 2046` family is provably
not user code: SubtypeTestCache arrays hold `[cid, type]` pairs (§2.2 of
further_improvements.md), and 2046/2105 are CIDs, not values a Dart program
compares against.

Fix: two rules, both local. (1) A Condition whose every reachable successor
block contributes no meaningful statement should render as a comment
(`// unresolved predicate over x2`), not as an `if`. (2) Conditions comparing
against values that resolve to ClassRanges/CID sets should be suppressed
entirely once P7's CID attribution lands; until then, recognize the
`ldr rX,[PP]; cmp; b.eq` shape feeding a call argument as a type check and fold
it into the call's evidence comment.

### EC-5 Machine temporaries and phantom identifiers make output non-compilable (all functions)

Observed: `local18`, `local70`, `x16`, bare `snapshotRef(458)` and
`snapshotInstance(E21Mode)` (without `aot.` prefix) appear directly in
expressions, e.g.
`return 'ok:${local70}';`, `if (x0 != x16)`, `e19Ackermann(local8, …)`.
None are declared anywhere in the file.

Why: `local{:x}` names are minted in `analysis/disassembly.rs:3233` whenever a
stack slot is read before being written in the tracked window; `xN` names come
from the ARM64 register printer (`disassembly.rs:4831`). The renderer's
`clean_condition`/`render_expression` only substitutes aliases; it never checks
that a rendered identifier has a binding. Bare `snapshotRef(` occurs when the
pool-label text passes through a path that doesn't go through the `aot.`
qualifier logic (three occurrences in main.dart).

Fix: add a final rendering pass per function that collects declared locals and
parameters and rewrites unknown free identifiers to
`aot.unresolvedValue('slot 0xNN')` / `aot.unresolvedRegister('x16')`. This is
mechanical, tier-safe (it only renames already-unresolved tokens), and makes
every emitted file pass `dart format`/analyzer parsing — which the fixture
verify scripts currently have to tolerate.

### EC-6 Implicit-closure twin rendered as an independent function with invented behavior (E19)

Observed: alongside `int e19Ackermann(int, [int])` there is
`int e19Ackermann_tearOff(int arg0, [int arg1])` whose body is a different,
wrong reconstruction (`e19Ackermann(2, aot.snapshotRef(23), 2)` — constants
that exist nowhere in the source). Both carry `kind: implicit_closure/regular`
correctly in IR, and `lexical_parent` links them, but the renderer emits the
twin as a sibling top-level function with its own (garbage) body.

Why: `function_kind_symbol_suffix` (dart.rs:3564) renames the implicit-closure
Code object to `<name>_tearOff` to avoid collision — reasonable — but then
renders whatever statements the lifter attributed to that tiny stub as if they
were the function's logic. An implicit closure's real semantics are exactly
"forward to the parent with the captured receiver"; anything else recovered
from its 132 bytes is lifter noise.

Fix: when `kind == ImplicitClosure` and a `lexical_parent` exists, render a
one-liner (`final e19AckermannTearOff = e19Ackermann; // implicit closure`) and
suppress body structuring entirely. The same treatment covers the `main_tearOff`
duplicate seen in main.dart. This deletes a whole category of plausible-looking
but wrong output — the most dangerous kind for a decompiler.

### EC-7 Unboxed-field slot lists don't match declared fields (E14/E15/E21)

Observed: `E15Vec` (source fields `dx, dy`) gets four slots
(+0x8,+0xc,+0x10,+0x14 on arm64; +0x4..+0x10 on arm32); `E21Mode` shows gaps
(+0x8,+0xc,+0x14,+0x18) mixing what are really reference slots into an
unboxed-only list; `E14Statics.stamp` is the sole surviving Field object while
`version`/`counter` are gone, yet `bump()` references counter semantics
through an unresolvedRegion. On arm32 the renderer even prints
`this._slot_4` — an offset that aliases the header word layout differently
than arm64's `+0x8`, so cross-ABI readers see inconsistent identities for the
same field.

Why: instance-slot derivation (`type_recovery.rs:186`) iterates bitmap bits
over `header_words..64` regardless of the cluster's `next_field_words`, so bits
beyond the class's true field count (bitmap padding or neighboring-class bits)
become phantom fields, and reference slots are simply absent (they're not in
the unboxed bitmap), leaving holes like E21Mode's missing +0x10. Nothing
normalizes offsets across ABIs.

Fix: bound the bit walk by `next_field_words - header_words` (the value is
already read at `fill_skip.rs:198` and currently only used for scalar counts),
mark reference slots explicitly as `slot_type: "reference"` when the field
count says a slot exists but the bitmap bit is clear, and key rendered slot
names by ordinal (`field0`, `field1`) rather than raw offset so arm64/arm32/x64
outputs agree. Combined with P7's schema-5 bitmap join, this turns `_slot_8`
into `dx` wherever the oracle exists and into stable typed placeholders where
it doesn't.

### EC-8 Operator and tear-off identity loss (E12, E15, E22, E23)

Observed: E15Vec's unary `-` and `%` operators produce no trace at all (their
bodies were folded into callers, and no inline record names them);
`operator ==` renders as `dynamic operator_equals(List<dynamic> args)` — VM
selector syntax, not Dart. E12's seven tear-offs reduce to one `get_first`;
E23's `Function.apply` renders a global `apply()` with no arguments.

Why: (a) selectors like `operator ==` are stored as VM names; the renderer maps
some (`get:` prefix stripping at dart.rs:1288) but not the `operator` family —
`operator_equals` needs a selector-to-source-name table
(`==`, `~`, `%`, `-`, unary-). (b) Tear-off Code objects are implicit closures
(EC-6) whose identity lives in the parent link plus the Closure→Function edge
(P5) — currently only DWARF/source-line containment feeds closure nesting, and
plain snapshots have neither. (c) `Function.apply` reaches the UnlinkedCall
path (§2.1): the selector survives in CallSiteData but no consumer labels it.

Fix: add the operator-selector table to `clean_symbol` (mechanical, ~20 rows);
land P5's Closure.reference walk so tear-offs recover their FunctionType and
parent; land P2's UnlinkedCall labeling so `apply` sites show
`dynamicCall("apply", arity=…)`. All three are already-designed proposals —
this probe confirms they are load-bearing for everyday Dart, not exotic code.

### EC-9 Constant-folding across call sites silently changes callee signatures (build 1 lesson)

Observed (`.tmp-probe/out`, first build): with literal arguments at the only
call site, `e04BitTwiddle` decompiled as `int e04BitTwiddle() { return 68; }`
— parameter list erased, body replaced by the folded result. `e25Intrinsics`
became `return false;` unconditionally. These are *faithful* recoveries of the
specialized code, but a consumer diffing recovered output against source sees
phantom signature loss and unconditional returns.

Why: full AOT clones/specializes by call site when profiling-free heuristics
prove a single-call-site constant. Clutter attributes the specialized Code to
the original Function name, so N specializations collapse onto one name (last
writer wins) and the general body may not survive anywhere.

Fix: when multiple Code objects map to one Function name (or one Code's
constant-pool fingerprint shows specialization), suffix variants
(`e04BitTwiddle@site1`) and note the specialization. Cross-ABI/cross-build
consensus (already computed in `cross_abi`) can pick the general variant when
one exists. At minimum, document in the header that single-site folding
occurred, so downstream consumers don't conclude the source had no parameters.

### EC-10 Stale scored artifacts make the stock evaluator lie (process finding)

Observed: running `test/tool/evaluate_accuracy.py` over the nine stored
`target/accuracy-final/*` outputs fails `negated_expensive_predicate` on all
nine; regenerating just `plain` with the current binary passes everything. The
stored IR predates the Aug 24 slot-renaming (`arg0.slot0f` → `arg0._slot_10`),
so the checked expression string no longer matches — a tooling-state bug, not
a decompiler regression.

Fix: regenerate the nine-variant matrix after any renderer change that alters
expression text (the README already implies this; a timestamp/schema-version
check inside the evaluator would make the staleness explicit instead of a
silent fail).

---

## What survived well (for calibration)

Not everything fails — worth recording so fixes preserve strengths:

- E20 mixin-application chain: `_MixinApplication1&E20Base&Counter` classes
  reconstructed with correct linearization and `super.greet()` folding.
- E21Mode.parse → `EnumByName|byName` dispatch recovered with the argument.
- E02/E16 collection call sequences (allocator + addAll + sort) are faithful
  modulo one spurious capacity-growth branch each.
- E24's knot CFG: both loops found, all three returns placed correctly even
  though loop guards are misattributed.
- E01's interpolation spine: all six separator literals positioned correctly
  in one template string.

## Priority order implied by this probe

1. EC-5 (free-identifier rewrite) — smallest change, makes every other fix
   verifiable in compilable output.
2. EC-2 guard (exclude handler-target blocks from loop structuring) — stops
   active fabrication now; full P3 later.
3. EC-1 (inlinee statement attribution from push_function ranges) — recovers
   whole bodies already decoded.
4. EC-3 (async/sync-star detector extensions incl. `yield_index` descriptors).
5. EC-6 (implicit-closure one-liners) — deletes wrong-output class.
6. EC-4 (empty-arm low-confidence condition demotion).
7. EC-7 (bounded slot walk + ordinal naming) — unlocks cross-ABI stability.
8. EC-8 (operator table + P5/P2 landings), EC-9 (specialization suffixing),
   EC-10 (evaluator freshness check).

Items 1–6 are implementable without the VM oracle; 7–8 benefit from the
schema-5 joins already planned (P1/P2/P5/P7).
