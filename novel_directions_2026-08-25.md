# Novel accuracy/recovery directions — research notes

Date: 2026-08-25. Companion to `further_improvements.md` (which remains the
canonical roadmap); this document adds what a fresh pass over the artifacts,
both SDK checkouts, and the prior-art record surfaced. Every measured claim
below was re-derived today from files in `target/`, the Dart SDK checkout at
287a20d2325, and the Flutter checkout at 4ebf37fe7df.

Constraint adopted for everything here: **no debug info is available**. No
DWARF, no `--split-debug-info` sidecars, no symbol maps. Ideas that only pay
off with those inputs are excluded from the ranked list.

---

## 1. New measured facts (this pass)

1. **Call-cache objects are tree-shaken in product AOT.** Mapping the
   `cluster_cids` histogram of `target/accuracy-final/obf-map/metadata/
   snapshot_evidence.json` through `class_id.h`: ICData = 0,
   MegamorphicCache = 0, SingleTargetCache = 0, MonomorphicSmiableCall = 0.
   The precompiler only keeps UnlinkedCall (7), SubtypeTestCache (115),
   PcDescriptors (156), ExceptionHandlers (190), ClosureData (1549).
   Any proposal premised on reading receiver CIDs out of IC caches is dead;
   dynamic-call naming must come from dispatch-table mining (P1 remainder)
   and selector inference.
2. **The schema-5 oracle already emits generator/async flags per Function**
   (`is_async`, `is_sync_generator`, `is_async_generator`; parsed into
   `VmFunctionEvidence` at `vm_oracle.rs:891`) — but nothing consumes them.
   The obf fixture has 9 such functions (e.g. `_loadLibrary`). This is the
   exact seed P4's residual async site needs, sitting unused.
3. **Field initializers survive as first-class functions.** 250
   `FieldInitializer`-kind Functions exist in the obf oracle doc; each Field
   carries `initializer_function` (parsed at `vm_oracle.rs:296`,
   `:1152`). Static-field bodies like edge-case E14's `bump()` semantics are
   recoverable by following that edge to Code — no consumer today.
4. **Closure-family functions are the signature majority under obfuscation.**
   Of 2,271 oracle Functions, 929 are ClosureFunction + 620
   ImplicitClosureFunction + 80 DynamicInvocationForwarder; 2,118 carry a
   fully typed `user_visible_signature` (93%). The static path's closure
   walk (P5) currently recovers a fraction of this; every anonymous-closure
   signature that renders comes from the oracle join, not the graph.
5. **Yield-index descriptor rows are present and unused.** Edge-probe IR:
   3 functions carry rows with `yield_index != -1` (8 rows) — exactly the
   sync*/async* machine seed EC-3 asked for; still not wired into any
   detector.
6. **Try-index descriptor joins are narrow and cheap.** Only 2 probe
   functions have try_index rows (23 rows total), confirming P3's gate:
   bracketing on descriptor spans touches almost nothing except real try
   bodies.
7. **`static_calls_target_table` is NOT_IN_PRECOMPILED** (`raw_object.h:2077`)
   — it cannot appear in AOT snapshots, closing the question of whether a
   deeper pool scan could find app-level entries: there are none to find.

## 2. Prior art — what exists, and where clutter already differs

Static snapshot parsers (no VM): Doldrums (Dart 2.10–2.12 only, class dumps),
darter (one snapshot version). Blutter embeds the Dart runtime of the *exact*
SDK version to deserialize, then annotates disassembly (arm64-only; loses
function identity entirely under `--obfuscate` because it leans on
CodeSourceMaps — issue #77 confirms). unflutter (2026) does static cluster
parsing + call graphs + behavioral signals and feeds Ghidra/IDA; explicitly
"no source reconstruction". JEB has a Dart snapshot plugin (metadata +
decompilation assist). reFlutter patches libflutter.so for traffic
interception — orthogonal.

Academic structuring line: Phoenix (condition-aware schema matching, 2013),
DREAM (pattern-independent structuring + duplication, 2015), Comb (duplication
to remove gotos, 2020), SAILR (USENIX 2024: decompilation quality should be
measured as closeness to original source; spurious gotos come from
compiler-introduced transformations, which the decompiler should invert).
LLM decompilation (LLM4Decompile, Decompile-Bench/NeurIPS 2025,
control-flow-augmented LLM decompilers): best reported re-executability is
still ~50–60% end-to-end on C; all of it targets native-C binaries, none
targets managed VM snapshots.

Clutter's differentiators hold: source-language-native output (not lifted C),
exact-revision bound evidence with tier discipline, cross-ABI consensus, and
a nine-variant executable gate. None of the public tools attempt Dart-source
reconstruction at all; SAILR's "closeness to original source" metric matches
what the accuracy evaluator already measures — worth citing, and its core
lesson (invert compiler lowering instead of fighting its output) is directly
actionable below.

## 3. Novel proposals (ranked; all debug-info-free)

Ranking = expected recovered-code gain × breadth ÷ risk, consistent with the
tier rules (loaders append facts; tiers never upgrade; projection may
discard).

### N1 — Compiler-aware machine-lowering inversion ("SAILR for Dart AOT")

The single biggest structural idea this research pass produces. SAILR proved
that most decompiler garbage is compiler-lowering residue, and the fix is to
recognize and invert specific known lowerings. Dart AOT is an unusually good
target because the "compiler" is one deterministic program whose lowerings
are readable in the same checkout:

| Lowering (source in SDK checkout) | Clutter symptom today | Inversion |
| --- | --- | --- |
| awaiter state machine (`StateMachineAssembler`, await stubs, `:await_jump_var`) | fabricated loops / misread dispatch cycles (E10) | render `await e;` at suspend points keyed by pc-descriptor yield positions + frame-slot constants |
| sync*/async* machines (`_SyncStreamIterator` etc., yield descriptors) | `while (x4 < arg0)` nonsense (E11) | suppress loop structuring over the dispatch cycle; render `yield e;` at yield-indexed pcs |
| null-safe implicit `!?` checks (`NullAwareEntry` lowering) | six nested empty low-confidence ifs (E05) | fold check+throw diamonds into `?.`/`??` syntax |
| subtype-test cache cid compares | dead `x4 == 2046` branches (E17) | recognize TTS shape; fold into `is`/`as` evidence comment |
| boxed-double fast/slow paths | already partially done (decision `boxed-double`) | extend to remaining allocation twin sites |

Each inversion is provable from instruction shapes + descriptor rows, enters
at the lowest tier it can prove, and deletes a fabrication class rather than
adding heuristics. The async half is exactly P4's remainder, now with the
oracle flag seed noted above; framing it as one coherent "lowering-inversion"
program (with a lab case per lowering, P9-style) is what makes it shippable.

### N2 — Closure-graph recovery from the static snapshot (deepened P5)

New specifics beyond the original P5 text, from reading the fill specs:

- `ClosureData.parent_function` survives AOT as a WSR field
  (`raw_object.h:1652`); 1,549 ClosureData objects exist vs 82 Closures.
  Following Function→ClosureData→parent_function reconstructs lexical
  nesting for every local function even when no Closure instance remains —
  strictly stronger than the current DWARF/source-line heuristic (and
  debug-info-free by construction).
- `DynamicInvocationForwarder` Functions retain their forwarding target via
  WSR (`DropFunctions` only severs it when the target itself would be
  dropped). Where the target survived, forwarders give free
  "dynamic-call → concrete target" edges — a devirtualization channel that
  needs no dispatch table.
- Implicit-closure twins (EC-6) get their true one-line semantics
  (`f_tearOff = f`) from parent links instead of lifter noise.

### N3 — Consume the generator/initializer evidence already parsed

Small, pure-consumption work on facts already inside the process boundary:

- `is_sync_generator`/`is_async_generator`/`is_async` from
  VmFunctionEvidence → seed N1 detectors and silence machine-region
  structuring (fixes P4's residual site without new decoding).
- `Field.initializer_function` → render `field = <recovered initializer
  body>` for static fields whose Field object survived (250 candidates in
  the obf fixture), recovering E14-class failures.
- Yield-index descriptor rows (already decoded) → sync* detector per EC-3.

### N4 — Dispatch-table stream mining (the live half of P1)

Unchanged in substance by this pass (ICData absence strengthens its
necessity): mine the serialized DispatchTable stream clutter already locates
(`dispatch.rs::find_table`) plus the code-index map to name selector→Code
rows authoritatively. With caches gone, this is the only remaining
authoritative channel for dynamic-call names. Pair with P8's named-argument
descriptors while in the seam.

### N5 — Cross-ABI consensus as a semantic filter (new application)

Existing consensus machinery currently guards identities. Extend it: a
statement/predicate that appears in one ABI's lift but provably contradicts
the consensus of the other two (same function, same offset-normalized
position) gets demoted to a comment rather than rendered as code. Targets the
"plausible but wrong" output class (E19's duplicated args, E24's inverted
guards). Needs the normalized position key the body-graph already computes;
risk is moderate (consensus can be wrong 2-vs-1 when one ABI mislifts), so
gate on disagreement magnitude, keep verbatim rendering behind a flag.

### N6 — LLM-assisted renaming/speculation (kept Speculative, per §4 rules)

The 2024–2025 literature (LLM4Decompile-End/Exe, Decompile-Bench,
sc²dec/FAE) reports meaningful gains for *native* decompilation, but all of
it presumes x86/ARM C output and none of it respects a soundness/tier
discipline. For clutter the honest positioning stays: optional post-pass,
Speculative tier, never promoted into proven text, plausibly useful for
renaming `_slot_10`-style placeholders using surrounding recovered context
(class names, string literals, call graph). Nothing in the measured pipeline
depends on it; treat as an experiment, not a roadmap item.

### Rejected / deprioritized by this pass

- ICData/MegamorphicCache/SimpleTargetCache receiver-CID mining — objects do
  not survive product AOT (§1.1). Dead.
- Anything DWARF/split-debug dependent (incl. blutter-style CodeSourceMap
  reliance under obfuscation) — excluded by constraint; note blutter breaks
  exactly there, which is a market position for clutter, not a feature gap.
- Extending `static_calls` mining — target table is JIT-only (§1.7).

## 4. Suggested sequencing (relative to existing roadmap)

1. N3 immediately (days): pure consumption of parsed evidence; measurable on
   both fixtures; unblocks P4's residual and E14/E11 scoring.
2. N2 next (small): ClosureData walk + forwarder targets; attacks EC-6/EC-8
   and adds dynamic-call edges without touching dispatch tables.
3. N4 = existing P1 remainder (dispatch stream), unchanged priority.
4. N1 as the umbrella for P3-remainder/P4/P9 lab expansion; land lowering
   inversions one lab case at a time (awaiter → sync* → null-flow folds →
   TTS folds).
5. N5 after the body-graph position key lands; N6 whenever idle, always
   Speculative.

Everything here preserves the standing invariant: evidence enters at the
lowest tier that proves it, joins happen on exact identities, and the
nine-variant evaluator decides whether recovered Dart actually improved.
