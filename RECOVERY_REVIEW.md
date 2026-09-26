# Clutter recovery review

Reviewed on 2026-09-24. The best immediate opportunities are correcting argument and receiver provenance, decoding metadata already present in the binary, and preserving object identity through the analysis. A typed intermediate representation with merge values and memory effects is the larger change most likely to improve body recovery across many constructs.

This review combines source inspection with fresh runs of the existing plain application fixture. Proposed gains are qualitative. No proposed recovery algorithm was implemented or benchmarked for improvement during this review.

The source revisions inspected were:

| Repository | Revision | Scope |
| --- | --- | --- |
| Clutter | `6e58131bb444bf53752a2017d2088a731ac18a72` | Parsing, lifting, rendering, evidence integration, oracle and tests |
| Adjacent Dart SDK | `287a20d232578873d28bb8d78eadb4add46711c8` | Dart 3.14 development tree, frontend transforms, AOT compiler, serializer and runtime |
| Adjacent Flutter | `4ebf37fe7df0a130ba5bee17315b98f905c10b34` | Kernel compilation, AOT build options, debug information and deferred packaging |
| Dart fixture compiler source | `d684a576a6aa954ae107a03b2b4e1d61c3bebe93` | Additional checks against Dart 3.12.2 calling conventions and signature shaking |

The adjacent SDK is newer than Clutter's supported range. Format work should use the exact supported revision. In particular, the calling-convention errors below were checked against the 3.12.2 revision as well as the adjacent SDK.

The Flutter path I followed was `KernelSnapshot.build` → `KernelCompiler.compile` → frontend global transforms → `AOTSnapshotter.build` → `gen_snapshot` → `Precompiler::DoCompileAll` → clustered snapshot and instruction serialization. The relevant entry points are [Flutter common targets](../flutter/packages/flutter_tools/lib/src/build_system/targets/common.dart), [Flutter compiler invocation](../flutter/packages/flutter_tools/lib/src/compile.dart), [Flutter AOT builder](../flutter/packages/flutter_tools/lib/src/base/build.dart), [Dart frontend](../dart-sdk/pkg/vm/lib/kernel_front_end.dart), [precompiler](../dart-sdk/runtime/vm/compiler/aot/precompiler.cc), and [snapshot serializer](../dart-sdk/runtime/vm/app_snapshot.cc).

This path matters for interpreting evidence. Flutter enables AOT and TFA; TFA shakes signatures and fields, devirtualizes calls, and records unboxing information. Native compilation adds inlining, register allocation, allocation sinking and instruction deduplication. The precompiler then drops runtime objects, obfuscates retained identifiers and serializes the remaining graph. A fact recovered after one of these transformations does not automatically describe the original source declaration.

Fresh verification used `cargo test --lib`, `cargo build`, and application-scope decompilation of `test/simple_app/dist/simple_app.apk` on all three ABIs, with `--emit-ir`. All 155 library tests passed. The existing accuracy scorer passed for these three variants with `--allow-partial`. That is a limited baseline, not a complete semantic accuracy assessment.

| Fresh plain-fixture result | ARM64 | ARM32 | x64 |
| --- | ---: | ---: | ---: |
| Recovered functions | 30 | 30 | 30 |
| Functions with signatures | 12 | 12 | 12 |
| Recovered application Field declarations | 0 | 0 | 0 |
| Functions with internal source maps | 26 | 0 | 26 |
| Internal source-map events | 338 | 0 | 339 |
| Decoded stack-map entries | 141 | 145 | 142 |
| Detected dispatch-table call sites | 6 | 4 | 0 |
| Semantic statements | 300 | 299 | 179 |
| Signature-solver entries | 20 | 20 | 20 |

Each ABI produced ten `Unknown` signature solutions and ten `Bounded` solutions whose minimum and maximum positional counts were both zero. A separate `--cross-abi` run reported 22 aligned occurrences, four corroborated and 18 disputed, despite each input containing 30 recovered functions. Statement totals above measure output volume, not correctness.

The generated evidence is under `target/recovery-review-20260924/`. Those outputs are local, ignored build artifacts. The source-level findings below remain useful without them.

| Priority | Work | Expected benefit | Relative effort |
| --- | --- | --- | --- |
| P0 | Correct Dart argument locations and representations | Correct parameters, calls, returns and receiver types across all ABIs | Medium |
| P0 | Track the actual dispatch receiver | Remove falsely exact call targets; enable safe narrowing | Medium |
| P0 | Repair descriptor decoding and solver integration | Recover real call shapes and stop name-based collisions | Small to medium |
| P0 | Fix layout assumptions and constant/interpolation claims | Prevent incorrect fields, constructors and returns | Small to medium |
| P1 | Decode read-only metadata on ARM32 | More source locations, inline evidence and exception metadata | Small to medium |
| P1 | Preserve typed snapshot values and decode roots | Recover constants, static state and runtime identities | Medium |
| P1 | Make identity/evidence graphs operational | Consistent joins and usable provenance across analysis passes | Medium |
| P1 | Add typed SSA, memory effects and exceptional edges | Recover loop variables, merged values and handler bodies | Large |
| P1 | Analyze deferred loading units | Recover physically packaged bodies currently omitted | Medium to large |
| P1 | Repair compiler laboratory and semantic scoring | Measure gains and prevent plausible but wrong output | Medium |
| P2 | Optional defaults, async continuations and inline fragments | Recover specific constructs currently summarized or unresolved | Medium to large |
| P2 | Compiler sidecars and Kernel inputs | Much more information when matching build artifacts exist | Medium |
| P2 | Cross-ABI, runtime and Flutter-specific enrichment | Corroboration and additional context after identity fixes | Medium to large |

1. **Model Dart calling conventions per function and call kind.**

   This is a confirmed implementation defect. [Clutter's argument helpers](src/analysis/disassembly.rs), especially `abi_first_argument_register`, `abi_rest_argument_registers`, `entry_flow_state` and `collect_call_arguments`, treat the ABI as one fixed register window. The actual Dart register lists are:

   | ABI | Clutter's CPU argument window | Dart 3.12.2 CPU argument window |
   | --- | --- | --- |
   | ARM64 | `x1,x2,x3,x4,x5,x6,x7` | `x1,x2,x3,x5,x6,x7` |
   | ARM32 | `r1,r2,r3` | `r1,r2,r3,r8` |
   | x64 | `rdi,rsi,rdx,rcx,r8,r9` | `rdi,rsi,rdx,rbx,r8,r9` |

   See [ARM64 constants](../dart-sdk/runtime/vm/constants_arm64.h), [ARM constants](../dart-sdk/runtime/vm/constants_arm.h), [x64 constants](../dart-sdk/runtime/vm/constants_x64.h), and `ComputeCallingConvention` in [dart_calling_conventions.cc](../dart-sdk/runtime/vm/compiler/backend/dart_calling_conventions.cc). The [3.12.2 compiler implementation](https://github.com/dart-lang/sdk/blob/d684a576a6aa954ae107a03b2b4e1d61c3bebe93/runtime/vm/compiler/backend/dart_calling_conventions.cc) confirms the representation-dependent allocation.

   Correcting those arrays is only the first step. Dart allocates tagged values and unboxed integers to CPU registers, unboxed doubles to a separate FPU register sequence, and 64-bit integers to register pairs on 32-bit targets. `Function::MaxNumberOfParametersInRegisters` in [object.cc](../dart-sdk/runtime/vm/object.cc) selects stack calling conventions for generic functions and several synthetic kinds, including closures and dynamic invocation forwarders. Optional parameters can remain on the stack. The runtime snapshot does not preserve every compiler decision needed to reconstruct this directly, so unknown cases need constraints from the caller, callee prologue and entry kind.

   Clutter also seeds incoming stack parameters at `slot * 8` for every ABI, seeds parameters in both registers and stack slots, and treats all nonnegative SP stores as outgoing arguments. These assumptions can name frame locals as arguments. Introduce a target-layout object and a `CallConvention` object, track SP changes relative to frame entry, and assign one location and representation to each established argument. Model Dart calls, VM stubs and FFI calls separately. Include return representations and actual clobber sets; the current ARM64/ARM32 clobber helpers leave FPU state untouched.

   Validate with mixed integer/double arguments, four or more CPU arguments, generic calls, closures, named parameters, overflow into stack slots, ARM32 unboxed int64 pairs, and values that stay live across a call. Check recovered operand identity and order, not only argument count.

2. **Resolve dispatch through the class-ID expression used by the call.**

   [Clutter's `resolve_dispatch_via_receiver`](src/analysis/disassembly.rs) scans many live registers for any value carrying a class name and returns the first singleton table target. That class is not necessarily the receiver whose CID indexed this call. A declared type also does not establish an exact runtime class when subclasses are possible. The resolved call can then receive high confidence.

   The compiler exposes a precise reverse route. `EmitDispatchTableCall` in [flow_graph_compiler_arm64.cc](../dart-sdk/runtime/vm/compiler/backend/flow_graph_compiler_arm64.cc) uses `DispatchTableNullErrorABI::kClassIdReg` plus a selector displacement. Follow the defining CID load back to its receiver, or follow a constant/range-proven CID. Preserve exact-class, subtype-set and unknown-type facts separately. Only a proven singleton target over the applicable receiver set permits an exact semantic target.

   Do not interpret an arbitrary populated slot or a parent-class fallback as selector membership. [Dispatch table generation](../dart-sdk/runtime/vm/compiler/aot/dispatch_table_generator.cc) packs rows with holes. The existing family inference already acknowledges that complication, but the exact-target path needs the same discipline.

   Add the missing x64 decoder using `EmitDispatchTableCall` in [flow_graph_compiler_x64.cc](../dart-sdk/runtime/vm/compiler/backend/flow_graph_compiler_x64.cc). Clutter currently returns an empty dispatch-call map for x64. The fresh fixture has a recovered dispatch table but zero identified x64 dispatch sites.

   Validate with two unrelated typed objects live at one call, an interface parameter whose runtime receiver is a subclass, and rows with shared unused slots. Assert that unrelated registers cannot change the selected target.

3. **Decode actual argument-descriptor elements, then connect the signature solver.**

   This is another confirmed defect. `unlinked_call_label` in [instructions.rs](src/snapshot/cluster/instructions.rs) reads descriptor values from `scalars_of(array)`. The [array fill decoder](src/snapshot/cluster/fill_skip.rs) stores only array length in that scalar list; element values are references after the type-arguments reference. The unit test constructs a different in-memory shape, putting all descriptor values in the scalar list, and therefore misses the production bug.

   Follow the element references to integer/string objects, resolving VM base objects as well as isolate objects. [ArgumentsDescriptor](../dart-sdk/runtime/vm/dart_entry.h) defines `Count()` as excluding the type-argument vector already. Subtracting `TypeArgsLen()` is incorrect, and that length describes type arguments, not a number of ordinary value arguments to remove. Preserve positional count, total count, named names and their positions, and the hidden type-vector slot as different facts.

   The current [signature solver](src/evidence/signature_solver.rs) then has three integration gaps. `accumulate_call_sites` records `positional: 0` for every call. It finds the first matching leaf name, ignoring owner and library. Finally, [CLI integration](src/cli.rs) stores `signature_solutions` but the lifter and renderer never consume those solutions. Descriptor collection also precedes the later oracle attachment pass and uses names instead of the established object/code links.

   Feed typed call-site observations into occurrence-keyed constraints, attach authoritative metadata first, distinguish unknown count from zero, and use solved facts in a second lifting pass. Use exact entry/body identity plus logical alternatives for shared code. Preserve all logical closures rather than collapsing identical display names. Agreement across several call sites does not establish that optional parameters are absent; solve minimum required count, observed supplied count and total callable capacity separately.

   Validation should use a serialized descriptor fixture, including a generic invocation and reordered named arguments, then check that real recovered calls improve the callee and generated code. Add two same-named methods in different libraries and multiple anonymous closures. The fresh 30-to-20 collapse is a useful regression case.

4. **Distinguish a compiled signature from an original source signature.**

   This is a necessary accuracy rule even after fixing the solver. [SignatureShaker](../dart-sdk/pkg/vm/lib/transformations/type_flow/signature_shaking.dart) removes unused, never-supplied and constant parameters. It can turn always-supplied named parameters into required positional parameters and rewrite callers. The same behavior is documented in the [3.12.2 implementation](https://github.com/dart-lang/sdk/blob/d684a576a6aa954ae107a03b2b4e1d61c3bebe93/pkg/vm/lib/transformations/type_flow/signature_shaking.dart).

   Record separate provenance for original-source evidence, retained runtime signatures and inferred machine interfaces. A runtime FunctionType is authoritative for the retained callable shape, but can describe the transformed program. When source parameters were eliminated, a direct-call prologue cannot prove their original number, names or optionality. Related tear-offs and forwarders can supply constraints, but need explicit transformation-aware correspondence.

   Validate with a function whose named parameter is always passed and a function whose parameter is always a constant. Keep ordinary optimized fixtures alongside deliberately retained compiler-lab functions so the test set measures both erasure and recovery.

5. **Stop synthesizing field identities from incomplete declaration lists.**

   `RecoveredFieldLayout::synthesize_offsets` in [disassembly.rs](src/analysis/disassembly.rs) starts every class at a fixed header, treats a declared `double` as an aligned unboxed field, and uses eight-byte references for all x64 objects. [Type recovery](src/snapshot/cluster/type_recovery.rs) also hardcodes x64 slot width to eight bytes. The fixture's x64 snapshot explicitly has compressed pointers.

   `Class::CalculateFieldOffsets` in [object.cc](../dart-sdk/runtime/vm/object.cc) starts subclass fields after the superclass layout, reuses inherited type-argument storage, and sizes fields according to the unboxed flag and guarded CID. It can fall back to boxing when bitmap capacity is exhausted. Declared Dart type alone is insufficient. Surviving Field objects may also be only a subset of the original layout.

   Preserve exact offsets and class slot bitmaps, derive widths from the snapshot features, and keep uncertain locations as anonymous slots keyed by class identity and byte offset. Promote a slot to a field name only through exact field metadata or a verified accessor/layout constraint. In particular, an inferred offset should not enter `insert` with the same `synthesized_slot: false` state as an authoritative named field. Even when an exact field offset exists, the current synthesis pass can add another offset for the same name.

   This also enables useful recovery when all Field objects were dropped. Cluster loads and stores by concrete receiver CID, offset and representation, and emit anonymous recovered fields with observed value constraints. Recovering `_field_10` with a proven storage type is more useful than repeating unrelated low-level slot expressions. Original names and declared nullability still require additional evidence.

   Validate inheritance, shared generic type-argument slots, boxed and unboxed doubles, integer fields, dropped intervening fields and x64 compressed-pointer snapshots. Include negative tests that forbid invented field names.

6. **Decode read-only source maps and PC descriptors.**

   This is a particularly focused increase in information recovery. [Allocation parsing](src/snapshot/cluster/alloc.rs) already records the read-only offsets for PC descriptors, CodeSourceMaps and compressed stack maps. [Fill parsing](src/snapshot/cluster/fill_spec.rs) intentionally does nothing for those objects when pointers are uncompressed. [rodata.rs](src/snapshot/cluster/rodata.rs) then extracts only strings. Consequently, `decode_code_source_map` cannot find a metadata object payload for those references.

   The VM's `RODataSerializationCluster` and `RODataDeserializationCluster` in [app_snapshot.cc](../dart-sdk/runtime/vm/app_snapshot.cc) provide the exact delta-offset and object-image mechanism. Add per-profile readers for the metadata object headers and payloads, then feed the bytes to Clutter's existing decoders. Keep the already decoded instruction-table stack-map path; the baseline proves that it works independently.

   The plain ARM32 fixture has `no-dwarf_stack_traces_mode` but reports zero source-map events, while ARM64 and x64 report hundreds. The code omission is confirmed; the exact number recoverable from that ARM32 binary needs the proposed decoder to be measured. Validate the recovered payload bytes and PC ranges against an exact VM analyzer before claiming parity.

7. **Keep typed snapshot values through rendering and enforce interpolation completeness.**

   The parser retains arrays, instances, records, scalars and references, but [pool labeling](src/snapshot/cluster/instructions.rs) compresses much of this into strings such as `snapshotRef(...)`, `snapshotInstance(Class)` or a set of nested strings. `Recovery` in [cluster/mod.rs](src/snapshot/cluster/mod.rs) does not expose the complete parsed value graph. Pool string lookup is isolate-only even though the type resolver already supports VM and isolate lookup. That loses readily available base-object string references.

   Preserve a typed value graph with IDs, exact integer/double values, string encoding, array order, map key/value pairs, record shape, instance slots, canonicality and reference cycles. Resolve constants and pool references through a common VM/isolate namespace. The [Map](../dart-sdk/runtime/vm/app_snapshot.cc), Record and Instance serialization clusters specify their retained content. This can recover const catalogs, route/configuration maps, enum values and widget argument values without reconstructing machine instructions for every constant.

   A confirmed renderer error is `prettify_snapshot_instance` in [dart.rs](src/render/dart.rs). It turns a canonical instance into `const ClassName()` without evidence of a zero-argument constructor or its arguments. The fixture's `Product` constructor takes four arguments. Keep an explicit object value or recovered field representation until constructor equivalence is established.

   A second confirmed error appears in the fresh x64 `formatPrice`. Its recovered high-confidence interpolation and return contain only `'${snapshotRef(371)}'`; the formatted numeric result is absent. `take_interpolation_literal` chooses the buffer with the most populated slots instead of following the actual `_interpolate` argument. Buffers do not retain allocation length, so a missing trailing element can disappear entirely, and the result receives high confidence unconditionally.

   Tie the interpolation to the actual array argument, store its allocated length and element provenance, retain unknown trailing slots, and propagate the weakest input evidence. Validate two live arrays, a missing final store, branch-dependent elements, escaped strings, repeated references and a nontrivial const constructor.

8. **Parse snapshot roots to recover static storage and runtime identities.**

   [parse_snapshot](src/snapshot/cluster/mod.rs) finishes cluster fill and scans the tail for a dispatch table; it does not decode the complete root sequence. The serializer's `WriteRoots`/`ReadRoots` in [app_snapshot.cc](../dart-sdk/runtime/vm/app_snapshot.cc) include object-store roots and initial field tables. These provide authoritative relationships that cannot be reconstructed from unrelated graph nodes alone.

   Generate the root layout for an exact SDK profile, and recover the root library, global pool, static and shared field tables, loading units and retained stub roots. Then recognize thread → field table → slot loads and stores, including lazy-initializer sentinels. Join retained Field IDs where available; otherwise preserve anonymous static slots and their actual initial values. This improves globals and initializers even when names were removed.

   Root/stub identity also helps classify allocation, type-test, error and suspension helpers without names. The current `is_runtime_stub_call` accepts an unnamed target as a possible runtime stub. Missing semantic names should not itself authorize removal of a call's effects. Use root identity, named oracle evidence or a validated exact-version code pattern.

   Validate both retained and dropped Field objects, lazy and late statics, shared field storage, and root counts across supported versions. Root decoding is version-sensitive; a guessed root position must not become a proven identity.

9. **Make the physical-body and logical-occurrence graph the common identity system.**

   The existing graph is a good starting point, but [CLI integration](src/cli.rs) retains only its counter report. [body.rs](src/evidence/body.rs) ignores `instructions_region`, stores empty byte hashes, and labels absolute function addresses as isolate-relative offsets. It compares those values directly with oracle `code_offset`, which is relative to the isolate instruction image. The separate [oracle attachment path](src/vm_oracle.rs) correctly subtracts the image base before matching, so this finding concerns the body graph rather than all oracle joins.

   The graph also inserts oracle bodies and exact bindings without consistently deriving their range relation to the static body. The output named `body_graph.json` contains counters rather than the full nodes, bindings and evidence. The name-keyed signature solver and cross-ABI alignment subsequently lose identities the graph was designed to preserve.

   Use a subject ID bound to payload hash, ABI, loading unit and instruction region. Convert addresses through explicit types. Identify logical occurrences by snapshot or oracle object identity, preserving closure parents and shared-body alternatives. Keep the graph in `RecoveredProgram`, export it, and make call resolution, signatures, source maps and cross-ABI matching use it. Add claim-level provenance to semantic values; current semantic statements carry `High/Medium/Low` confidence separately from the `EvidenceTier` infrastructure.

   Validate with a nonzero ELF image base, two logical functions sharing a body, duplicate closure names, unequal range claims and two loading units with equal local offsets.

10. **Introduce typed SSA and memory effects incrementally.**

    [FlowState](src/analysis/disassembly.rs) retains a value at a join only when predecessor expressions match. That is useful for identical provenance, but deliberately loses branch-selected values and changing loop variables. Expressions are rendered strings, and call results are named from the target, so separate calls can have equal textual identities. The worklist also silently stops after 4096 visits. Capstone detail is enabled, but most lifting operates on mnemonic/operand strings.

    Add an architecture-neutral machine IR with typed operands, widths, register definitions, flags, memory operations and explicit unknown effects. Use stable value IDs, phi values at joins and memory versions for loads/stores. Track representation independently of source type: tagged object, compressed reference, Smi, unboxed int64, double and raw address. Keep a bounded set/range of possible CIDs where exact identity is unavailable. Split decoded machine IR, Dart lowering recognition and source rendering into separate passes.

    Build this in stages. First represent call results and stack locations with IDs. Then add phis for branch merges and loops, followed by heap effects and compiler-specific patterns. Reuse existing metadata readers, CFG structuring and proven idiom rules. A budget limit should produce a diagnostic and conservative unknown states instead of being reported as convergence.

    The compiler counterparts are [calling-convention allocation](../dart-sdk/runtime/vm/compiler/backend/dart_calling_conventions.cc), [IL operations](../dart-sdk/runtime/vm/compiler/backend/il.cc), [architecture lowering](../dart-sdk/runtime/vm/compiler/backend/il_arm64.cc) and [flow-graph compilation](../dart-sdk/runtime/vm/compiler/backend/flow_graph_compiler.cc). Reversing individual operations is feasible; recovering the compiler's original SSA graph exactly is not required.

    Validate loop accumulators, conditional assignments, repeated calls to the same callee, aliasing field stores, floating-point predicates including NaN, and unsupported instructions with live outputs. Measure correct recovered expressions and complete exit paths.

11. **Add exceptional and suspension edges before declaring blocks unreachable.**

    Clutter already decodes exception handler rows and derives try regions. However, [semantic reachability](src/analysis/disassembly.rs) uses ordinary branch successors and begins at the normal entry. Catch entries are not ordinary branch targets. This can discard handler statements before the renderer gets a chance to structure them. `catch_entry_reference` is retained but its move maps are not interpreted.

    `FlowGraphCompiler::RecordCatchEntryMoves` in [flow_graph_compiler.cc](../dart-sdk/runtime/vm/compiler/backend/flow_graph_compiler.cc) serializes how values at a throwing PC become catch-entry variables. [CatchEntryMovesMapReader](../dart-sdk/runtime/vm/exceptions.h) and [exceptions.cc](../dart-sdk/runtime/vm/exceptions.cc) provide the decoder and execution semantics. Join PC descriptors to handler indices, add exceptional successors, seed exception/stack-trace registers, and apply the move maps, including unboxed representations. Preserve generated finally/async handlers as distinct cases.

    For async/generators, [SuspendInstr::EmitNativeCode](../dart-sdk/runtime/vm/compiler/backend/il.cc) records yield metadata and uses distinct await/yield/suspend stubs. On x86 it emits a return epilogue that resume skips. Treating that machine return as the only semantic exit can hide the continuation. Build explicit suspend/resume edges using verified stub identity, frame layout and yield metadata. This offers a stronger route than target-name substring matching for `await` and generator detection.

    Validate nested try/finally, a handler reading a local set before a call, stack-trace capture, two awaits, await in a loop and both generator kinds on all ABIs. Metadata can establish continuation mechanics without proving the original source syntax in every optimized case.

12. **Recover optional defaults from prologue semantics when they survive.**

    Clutter correctly avoids obtaining defaults from FunctionType, which does not store their source expressions. There is another route. [PrologueBuilder](../dart-sdk/runtime/vm/compiler/frontend/prologue_builder.cc) compares the supplied descriptor with optional parameters and loads `DefaultParameterValueAt(...)` on the missing-argument path. This compiles into machine constants and stores.

    Recognize the descriptor comparison, supplied/missing branches and shared parameter destination. If the missing path assigns a proven constant and the join establishes that parameter, emit a recovered runtime default value with code provenance. Apply the typed constant graph to const instances, collections and records. Defaults eliminated by signature shaking or propagated into callers remain unavailable as declaration defaults. A constant seen at one call is not enough to recover a default.

    Validate optional positional and named arguments, explicit null versus omitted arguments, nontrivial const defaults, generic functions and a case where shaking removes the optional parameter.

13. **Turn retained inline metadata into attributed semantic fragments.**

    [instructions.rs](src/snapshot/cluster/instructions.rs) already records inline transitions and ranges, and [debug recovery](src/analysis/debug_recovery.rs) imports DWARF inline information. Preserve this evidence as a tree of occurrences rather than only callee lists, comments or source bands. Attribute lifted operations to inline intervals, retaining parentage and discontiguous ranges. Report a recovered fragment when there is no standalone function body.

    The compiler's [CodeSourceMapBuilder](../dart-sdk/runtime/vm/code_descriptors.cc) explains the interval and inline-stack semantics. Its null-check operation contains another focused recovery route. `DoThrowNullError` in [runtime_entry.cc](../dart-sdk/runtime/vm/runtime_entry.cc) resolves the map's name index through the object pool. Clutter currently records the opcode/argument but does not use this relationship to name the associated check. Recover that name at its PC, keeping obfuscated tokens as such.

    Do not assemble a supposedly complete original body merely by concatenating different callers' inline fragments. Constant propagation and specialization can give different semantics and missing paths at each site. Validate nested and partially inlined functions, repeated inline occurrences and null checks with retained pool names.

14. **Deserialize deferred units as children of the root snapshot.**

    [Archive handling](src/archive/mod.rs) recognizes `libapp.so-N.part.so`, and [analysis](src/analysis/mod.rs) indexes deferred payloads, but body recovery covers only the root. This is actual packaged code that can be recovered, subject to the parent snapshot being present.

    `UnitSerializationRoots` and `UnitDeserializationRoots` in [app_snapshot.cc](../dart-sdk/runtime/vm/app_snapshot.cc) describe the required process. A child imports its parent's object references, supplies instructions/source maps for deferred Code objects, patches global-pool entries and refreshes dispatch entries. Parsing each child as an independent root would produce incorrect references.

    Retain parent reference tables, load units in dependency order, use unit-qualified body addresses, apply pool updates, and rerun call/type resolution. [Flutter Android targets](../flutter/packages/flutter_tools/lib/src/build_system/targets/android.dart) request a loading-unit manifest and package the parts. Support collecting the needed modules from an AAB or a set of split APKs. Report missing units as missing input, rather than tree-shaken code.

    Validate a deferred application with a child constant, a method, a closure and a cross-unit call. Compare recovery with a monolithic build while allowing different optimizer decisions. An oracle can also load units without invoking main, but static parent/child support remains useful independently.

15. **Repair cross-ABI matching before using it to strengthen evidence.**

    [consensus.rs](src/evidence/consensus.rs) assigns lexical indices from encounter order grouped by owner name without the library. Duplicate names reuse the first index, `parent_lexical` is always absent, and map insertion can overwrite distinct occurrences. Snapshot/code order is not a source lexical-order guarantee across architectures. The fresh run reduces 30 functions per input to 22 aligned occurrences.

    The fingerprint also needs correction. `fold_hashed` stops after 16 items, so its comment about incorporating the true total count is not implemented. XOR cancels repeated equal elements. Constants come partly from machine immediates, which include offsets and masks. Opaque call addresses and snapshot reference labels differ between ABIs. `parse_pool_double` accepts any parenthesized numeric label, including non-double object references. Equal empty observations provide no meaningful semantic corroboration.

    Align occurrences using restored identity, source/inline ranges, closure parents, signature constraints and graph neighborhoods, keeping ambiguous matches explicit. Compare normalized typed facts independently: a literal value, target identity or return expression. Use complete canonical encodings or proper multiset digests and preserve differences beyond any display cap. Agreement produced by the same incorrect rule on two architectures is still not proof of source semantics.

    Validate duplicate closures, function reordering, absent/inlined bodies, equal reference numbers for unequal objects, differing constants after position 16, and two empty observations. Require full occurrence accounting before measuring corroboration rate.

16. **Accept matching compiler sidecars and Kernel artifacts.**

    This is a separate input capability with a high information ceiling when users possess build outputs. [Flutter Android targets](../flutter/packages/flutter_tools/lib/src/build_system/targets/android.dart) already request `--write-v8-snapshot-profile-to` and `--trace-precompiler-to` during code-size analysis. The [precompiler tracer](../dart-sdk/runtime/vm/compiler/aot/precompiler_tracer.cc) records function/class/field entities, library URLs and selector IDs before the later dropping/obfuscation stages. The [V8 snapshot writer](../dart-sdk/runtime/vm/v8_snapshot_writer.h) records object/reference relationships, including dropped-reference metadata. These files can preserve identities absent from the final AOT graph.

    Add importers for matching profile/trace files, retained-reasons output, object-layout output and instruction-size information where the exact SDK provides them. Profile IDs, trace entity IDs, table selector IDs and snapshot reference IDs are different namespaces. Join them only through documented offsets, graph relationships or a capture manifest, not by numeric equality. Such sidecars are not assumed to be present in an ordinary APK.

    Flutter's [KernelSnapshot](../flutter/packages/flutter_tools/lib/src/build_system/targets/common.dart) creates `app.dill`; debug bundling copies it as `kernel_blob.bin`. [Archive.open](src/archive/mod.rs) currently requires `libapp.so`. A Kernel input path could recover structured code from matching intermediates or debug bundles. Use an exact-version Dart helper built on `package:kernel` rather than writing a second Kernel grammar in Rust. Record whether the component is before or after global transforms, and whether source text is embedded. Release AOT input may already have lost signatures, fields and unreachable code.

    Flutter also couples split-debug output to `--dwarf-stack-traces`. `DwarfStackTracesHandler` in [object.cc](../dart-sdk/runtime/vm/object.cc) disables general Function/Code retention in product mode, and the serializer omits CodeSourceMaps in this mode. Obfuscation, DWARF mode and stripping therefore need separate experiment dimensions. The adjacent Flutter revision delegates Android stripping to AGP; inspect the actual package and sidecar instead of assuming one stripping path across all versions. Widget creation tracking is explicitly disabled for release in `KernelSnapshot.build`, and Flutter removes selected framework `toString` bodies through frontend options. Those are limits on available information.

    Validate wrong-build sidecar rejection, named identity recovery from an analysis-size trace, and both debug Kernel and transformed release Kernel inputs. For controlled builds, write an evidence manifest containing the input/output digests, compiler revision and full flags. Retention flags cannot reverse earlier TFA deletion.

17. **Generate version profiles and turn the compiler laboratory into an independent accuracy test.**

    [profiles.rs](src/snapshot/profiles.rs) recognizes snapshot hashes, but [CID selection](src/snapshot/cluster/cid.rs) largely groups layouts by minor version. Other ABI/layout facts remain hardcoded in the lifter. [Class fill](src/snapshot/cluster/fill.rs) can scan around an unexpected position to resynchronize. That may salvage input, but recovered facts need explicit degraded provenance until independent invariants validate the result.

    Extract a profile from the exact SDK revision and build configuration: CIDs, serialized fields, object headers, compressed/native widths, function flags, root layouts, dispatch constants, thread/stub offsets and calling conventions. Use generated C++ probes for target layout constants where necessary, plus serializer fixtures and oracle comparisons. Separate exact-format support from experimental compatible parsing. Extend 3.13/3.14 only with the corresponding format evidence, not by widening the accepted version range.

    [compiler_lab.sh](test/tool/compiler_lab.sh) claims a multi-SDK matrix but uses one `flutter` from PATH. Its template miner reads low-level `statements`, compares an entire-program common prefix, and checks `DirectCall`/`ObjectPoolCall` while the JSON uses `direct_call`/`object_pool_call`. Literal-only example programs invite constant folding and inlining, which can erase the construct intended for examination. Skipped builds can leave an incomplete matrix without a complete required-variant gate.

    Build an explicit pinned matrix with runtime-dependent inputs and marked target functions. In controlled training builds, capture Kernel/IL before and after key passes, layout output, trace/profile sidecars and final machine code. Use never-inline or entry-point pragmas for selected training cases, then validate the rules separately on normal production-style optimization. Match target occurrences and semantic dataflow rather than whole-program statement prefixes. Write actual regression expectations for held-out programs.

    Extend [the accuracy scorer](test/tool/evaluate_accuracy.py) beyond its current small semantic sample. Measure retained-body coverage, correct call arguments, fields and constants, exception/continuation recovery, false high-confidence claims and complete behavior for a safe supported subset. Track original-source recoverability separately from recovery of the optimized program. The malformed descriptor test and incorrect high-confidence x64 interpolation both pass current tests, so negative assertions are essential.

18. **Integrate dynamic and Flutter-specific evidence after the static identity fixes.**

    [Runtime trace support](src/evidence/runtime_trace.rs) currently parses observations and [the CLI](src/cli.rs) prints refinement counts. It is not an input to decompilation. The schema binds only to a snapshot hash and ABI, and the CLI's hash comparison is optional. [make_version.py](../dart-sdk/tools/make_version.py) shows that the snapshot hash derives from SDK source files; different applications built by the same SDK share it. Reuse the existing full payload identity from [subject.rs](src/evidence/subject.rs), add unit/region identity, and validate PC/CID ranges before attaching trace facts.

    Preserve observed target and argument distributions rather than reducing each selector to its hottest target and each callee to its maximum positional count. An observed receiver establishes that an execution occurred, not that all other receiver types are impossible. Connect traces to occurrence IDs, and provide a collector only as an explicit execution-based mode. This can improve exploration and ambiguity ranking without overstating static proof.

    Flutter-specific work can then use framework structure. Decode constant widget instances and connect them to calls in recovered `build` methods; join proven asset-path constants to the extracted asset index; link proven MethodChannel calls to channel/method strings and optionally native-side handlers. A versioned framework matcher could use SDK identities, class hierarchy, constants and normalized call graphs to label known framework fragments. These are proposals requiring validation. Optimizer-dependent byte similarity alone should stay heuristic, and a widget summary should distinguish retained constant configuration from runtime UI state.

I would implement the first tranche in this order: correct ABI locations and dispatch receiver provenance; repair real descriptor decoding and occurrence-keyed solver feedback; fix field/constant/interpolation promotion; then add read-only metadata and root/value graph decoding. Establish the stronger regression suite alongside those changes. Use the resulting typed facts to introduce SSA and exceptional/continuation edges, followed by deferred-unit reconstruction and optional sidecar importers.

The main ceiling remains compiler erasure. Fully eliminated code, original names absent from all supplied artifacts, removed source parameters and original formatting have no reliable inverse in the release binary. Several findings above concern information that still exists but Clutter currently skips, flattens, misattributes or never feeds back into recovery. Those are the best places to invest first.

## Implementation progress

Updated 2026-09-24. Nothing has been committed. All changes are in the working tree. `cargo test --lib` passes 230 tests; the baseline was 155. The accuracy scorer passes on all nine variants (49 checks). Status is listed per numbered item above.

| Item | Status |
| --- | --- |
| 1. Calling conventions | Done |
| 2. Dispatch receiver provenance | Done |
| 3. Descriptor decoding and solver | Done |
| 4. Compiled vs. source signatures | Done for provenance and rendering; no compiler-lab fixtures yet |
| 5. Field identities | Done |
| 6. Read-only source maps (ARM32) | Done for the plain fixture; exact VM-analyzer byte comparison remains |
| 7. Typed values and interpolation | Done; uniquely matched fixed-arity declarations now bound opaque register-only calls, while ambiguous and stack calls still need work |
| 8. Snapshot roots and static storage | Done for 3.12.2: roots, stub identities, static field loads/stores, lazy initialization and initial values; other versions and deferred-unit roots remain |
| 9. Body/occurrence graph | Partial: correct offsets, byte hashes, range claims and full export |
| 10. SSA and memory effects | Partial: merged values (phis) at joins and loops, static-field write effects, scalar double lifting; heap memory versions and typed operands remain |
| 11. Exceptional and suspension flow | Mostly done: catch-entry moves, exceptional edges, exception registers, handler rendering, yield-proven awaits and the x64 suspend epilogue; generated finally handlers and async* await/yield disambiguation remain |
| 12. Optional defaults | Done for retained signatures: positional and named defaults from prologue merges |
| 13. Inline fragments and null checks | Done: null-check names, an inline occurrence tree with in-place attribution, and a fragment report |
| 14. Deferred units | Not started |
| 15. Cross-ABI matching | Partial: collision and fingerprint defects fixed; semantic identity still needs stronger evidence |
| 16. Compiler sidecars and Kernel input | Not started |
| 17. Compiler laboratory and scoring | Partial: required matrix gate and interpolation accuracy assertions |
| 18. Runtime and Flutter evidence | Partial: trace refinement preserves complete observed target and call-shape distributions; artifact binding and semantic integration remain open |

### 1. Calling conventions — done

- New `src/analysis/calling_convention.rs`:
  - `TargetLayout` holds the real per-ABI CPU and FPU argument registers, preserved registers, word size and polymorphic entry offset (ARM64 `x1,x2,x3,x5,x6,x7`; ARM32 `r1,r2,r3,r8`; x64 `rdi,rsi,rdx,rbx,r8,r9`).
  - `assign_locations` mirrors `ComputeCallingConvention`. Tagged values and unboxed ints go to CPU registers, doubles to the FPU sequence, int64 to register pairs on ARM32, and the rest to stack slots counted backwards from the last parameter.
  - `RegisterWindow::for_function` follows `MaxNumberOfParametersInRegisters`. Closures, forwarders, dispatchers, extractors, FFI trampolines and generic functions are stack-only.
- Where the declared representation is unknown, `resolve_parameters` enumerates the possible assignments and scores them against liveness and stack-read evidence from the body. A location is marked `Proven` only when every consistent assignment agrees; otherwise it is `Assumed`.
- The lifter tracks SP and FP relative to the entry SP (`FrameDeltas`). Stack slots are keyed relative to the entry SP, and each parameter gets exactly one location.
- Outgoing arguments:
  - Only stores below the entry SP count, using word-sized strides.
  - With a known callee convention, arguments are read from the callee's locations.
  - Otherwise only registers written since the previous call are used.
- A call clobbers everything except the preserved registers, including FPU state. FPU returns are modelled.
- Each function gets a `machine_interface` (location, representation, proof) in the IR and a rendered note.
- Allocation stubs are recognised and no longer receive stale arguments.

### 2. Dispatch receiver provenance — done

- Dispatch-table calls resolve only through the class-ID register actually used by the call (`LoadClassId` provenance).
- Resolution needs an exact receiver class: a canonical constant or an allocation result. A declared type is not enough.
- There is no superclass or neighbouring-slot fallback, so packed-row holes are no longer treated as selector members.
- IC call narrowing needs an exact receiver and a unique matching symbol.
- Added the x64 dispatch decoder and handling for the split ARM32 `add lr` form.
- Fixture dispatch sites: x64 went from 0 to 6, ARM32 from 4 to 6.

### 3. Descriptor decoding and solver — done

- New `src/evidence/call_shape.rs`. Descriptor elements are followed through VM and isolate references, and structural invariants are validated.
  - `Count()` is no longer reduced by the type-argument count.
  - Named argument names and positions are preserved.
- VM base objects are versioned (`BaseObjects` for 3.4 / 3.5 / 3.9+), so cached descriptors and base strings resolve.
- `signature_solver.rs` is rewritten and keyed by code address (one entry per occurrence):
  - Shared bodies keep their logical alternatives.
  - Unknown is kept separate from zero.
  - Call sites contribute supplied count, required-positional upper bound, capacity lower bound, accepted names and type-argument counts.
- The solver runs before semantic enrichment, and the renderer prints its facts.
- Fixture solver entries went from 20 to 30. Generic calls are detected (for example `firstWhereOrNull`, typeArgs=1).
- `propagate_matching_signatures` is now kind-aware (no sharing between anonymous closures; implicit counts are handled per kind).

### 4. Compiled vs. source signatures — done for provenance and rendering

- `RecoveredSignatureSource` documents that retained signatures describe the program after the TFA transformation.
- The solver separates retained-runtime-signature authority from VM-oracle authority.
- Rendered functions state when parameters follow the retained AOT signature, which may differ from the source.
- Compiler-lab fixtures for signature shaking are not added yet (see item 17).

### 5. Field identities — done

- `synthesize_offsets` is removed. Field offsets now come from exact Field metadata:
  - Instance fields: compressed words × slot width.
  - Static fields: field-table id × native word.
- Object layout (header size, compressed and native word) comes from the snapshot profile, so the x64 compressed-pointer fixture uses 4-byte slots.
- Consecutive unboxed words are paired into one 8-byte slot.
- Field lookup walks the superclass chain.
- Anonymous slots stay `_slot_<offset>`. Inferred entries can never override an exact named entry or duplicate a name.
- On the fixture, 352 to 354 of about 545 fields per ABI decode an offset, with 0 mismatches against implicit setters.

### 6. Read-only source maps (ARM32) — implemented

- `rodata.rs` now follows the allocation cluster's delta offsets into the read-only data image for PC descriptors, CodeSourceMaps and compressed stack maps. It checks each object header's CID and payload bounds before attaching bytes and header values to the parsed object graph. The existing string extraction remains in the same pass.
- The existing metadata decoders now receive ARM32 CodeSourceMap and PC descriptor bytes. On the plain ARM32 fixture, 26 application functions have internal source maps with 338 events, up from zero functions and zero events. This matches the plain ARM64 fixture's counts. One application function has a decoded PC descriptor.
- Two focused tests check the ARM32 header layout, CID rejection, truncated payload rejection and compressed stack-map flags. `cargo test --lib` passes 199 tests, and the `plain_arm32` accuracy scorer passes.
- Payload bytes and PC ranges have not yet been compared with an analyzer built from the exact Dart SDK revision. The fixture count establishes coverage, not byte-for-byte correctness.

### 7. Typed values and interpolation — done

Constant graph:

- New `src/snapshot/cluster/constants.rs` builds a typed constant graph from pool roots and exposes it as `RecoveredProgram.constants`. It covers null, bool, int, double, string, list, map, set, record, instance (slots plus unboxed bits, enum names), type and closure. Reference cycles are handled.
- Pool strings resolve through both VM and isolate objects.
- Instance labels carry their reference (`snapshotInstance(Class@N)`).
- The invented `const ClassName()` rendering is removed. Constants now render from the graph:
  - Enums render as `Class.value`.
  - Collections render as `const <T>[...]` and `const <K, V>{...}`.
  - Instances render as `aot.constObject('Class', {...})`.
  - Anything not in the graph stays `aot.snapshotRef(N)`.
  - The fixture's `catalog` list renders all five `Product` values with their strings, prices and `Category` enum values.
- Constant slots are named from exact Field objects of every scope, including inherited and framework fields (`ConstantSlot.field`, `field_type`). An unboxed slot whose Field declares `double` or `int` renders by that type. On the fixture, `Color`, `MaterialColor` and `IconData` now render with real field names and double channels, and raw `aot.unboxedBits` output went from 33 to 0 per ABI.
- `Product` slots stay `_slot_<offset>`. None of its Field objects survived in this snapshot, so there is no name to prove. On x64, `Color.a` stays `_slot_8` for the same reason, while ARM64 retained that Field.

Interpolation:

- Element buffers are keyed by allocation site. Registers, frame slots and outgoing argument slots refer to a buffer through aliases, so spills, reloads and derived element pointers no longer fork its contents.
- `_interpolate` consumes the buffer passed as its argument (`[SP]` or the first argument register). A filled array that never reaches the argument is ignored.
- The allocated length comes from the `AllocateArray` Smi length register (`x2`/`r2`/`r10`). Unstored trailing slots stay explicit gaps.
- Confidence is the weakest part's. Gaps make the result Low, and so do call-result placeholders such as `formatPrice_result`. All seven fixture interpolations are now Low; they were all High.
- The x64 `formatPrice` case is fixed. All three ABIs produce `toStringAsFixed(arg0, 2)` and `'\$${_Double_toStringAsFixed_result}'`. x64 interpolations previously lost every part after the first store.
- Negative tests cover two live arrays, a missing trailing store, an array never passed as the argument, and the x64 `lea` element pointer.

x64 and ARM32 lifting fixes found along the way:

- `mov [rsp], reg` was treated as a stack-pointer write in both `moves_stack_pointer` and `FrameDeltas`, and `mov [rsp], imm` as ARM post-index writeback. This dropped every x64 outgoing stack argument. `firstWhereOrNull`, `ThemeData` and `Map._fromLiteral` calls now show their arguments.
- x64 two-operand arithmetic (`add`/`sub`/`imul`/`and`/`or`) is lifted as three-operand. This also enables the existing heap-base decompression (`add reg, [r14 + 0x58]`, checked against `AOT_Thread_heap_base_offset`), which previously never ran.
- The anonymous-slot fallback assumed uncompressed x64 slots and a 64-bit header on ARM32. It now uses compressed 4-byte slots with an 8-byte header on ARM64 and x64, and a 4-byte header on ARM32.
- Inline boxed-double allocation is fused on all ABIs (previously ARM64 only), including the ARM32 `movw`/`movt` header and split `add ip; vstr` value store. The header check derives the size tag from the target's object alignment.
- Write-barrier fusion covers x64 `call` and ARM32 `blne`, including the ARM32 thread mask load and the Smi test before the first barrier branch.
- FPU spills (`vstr`, x64 `movsd [mem], xmm`) are recorded, and FPU reloads read frame slots. Previously an x64 `movsd` store wrote into a register named after the memory operand.
- The snapshot recovered declarations only for the output scope, so framework Field and Class declarations never reached the field layout or signature evidence. `Recovery.evidence_declarations` now holds every scope; output declarations are still scoped. For example, `_enumToString` now reads `this._name`.

Fixture effect, x64 before → after: `field_read` 3 → 69, `field_write` 28 → 74. ARM32 `field_read` 71 → 86. ARM64 statements are unchanged apart from the `_enumToString` field name.

Scorer: the ARM64 `boxed_double_receiver` expectation now expects `arg0`, not `local10`. The inline box is folded to the double reloaded from the slot where `arg0` was spilled, so `arg0` is the source receiver `value`; `local10` was only the old slot name.

Still open:

- Uniquely matched retained Function declarations with no optional parameters now bound register-only calls even when the callee body is absent. `Product.toString` calls `toStringAsFixed` with exactly its receiver and `2` on all three ABIs. This removes the ARM32 int64 high word and ARM64 stale `x3` from that call. New accuracy expectations cover both errors.
- Calls without a unique declaration, calls with optional parameters, and calls with outgoing stack writes still report the argument registers written since the previous call. A retained signature bounds their arity but does not establish register representations or the actual supplied optional count. These cases need call-site descriptor and callee convention evidence before arguments can be removed safely.

### 8. Snapshot roots — partial

- The exact Dart 3.12.2 object-store root list is generated from SDK commit `d684a576a6aa954ae107a03b2b4e1d61c3bebe93`. For that snapshot hash only, Clutter reads 244 named roots, both initial field tables and the dispatch table at the resulting offset. It rejects a mismatched layout after checking the root library, global pool, allocation stub CIDs and dispatch table.
- The plain fixture on each ABI exposes 76 named stub roots, 612 initial field references and 15 shared initial field references. The typed constant graph also includes values reached through these tables. Root-proven calls such as `stub AllocateArray` receive their actual helper name.
- Fusion no longer treats an unnamed call as a runtime stub. The negative regression keeps an unknown call intact; tests with named stub evidence still fuse allocation and barrier patterns.
- Static field accesses are lifted for the exact 3.12.2 profile. The lifter recognises `ldr t, [THR, #field_table_values]` followed by a full-width load or store at `[t, #id * word]`, using the `AOT_Thread_(shared_)field_table_values_offset` values from that revision's `runtime_offsets_extracted.h` (ARM64 and x64 `0x78`/`0x80`, ARM32 `0x38`/`0x3c`). Other profiles keep these as raw loads.
  - New `static_field_read` and `static_field_write` statements carry the field-table id and table. A retained static Field names the slot as `Owner.name` with High confidence. Ids without a Field render as `aot.staticField(id)` / `aot.setStaticField(id, value)` with Low confidence. An id claimed by two different Fields is refused. Field metadata now records `is_shared` (kind bit 15), so shared-table ids do not collide with ordinary ones.
  - The lazy-initialization guard (`cmp result, sentinel; b.ne done; load Field; call init stub`) is fused only when the call target is a root-proven `Init*StaticField` stub. The field value then survives the join, so `_getLocaleClosure` recovers `PlatformDispatcher._instance` as the closure context on all three ABIs.
  - A store to a static field drops live register and stack values that name that field, because they hold the old value. A Dart call (not a VM stub) downgrades spilled static-field values to Low. Before this, `_rootRun`'s `Zone._leave(old)` restore rendered as `Zone._current = Zone._current`.
  - Whole-program plain fixture: ARM64 837 reads and 100 writes (611 named), ARM32 876 and 98, x64 872 and 93. The application scope has no mutable statics, so the plain accuracy results do not change.
- Stub bodies are now named from their unique root (`stub InitLateFinalStaticField` and so on) instead of `sub_<offset>`. With `--scope all`, `relink_calls` used to replace root stub labels with the body's synthetic name. That disabled every stub-dependent fusion, including 9042 unfused ARM64 stack-overflow guards, down to 13 now. The whole-program ARM64 run now exhausts the worklist budget in 3 functions instead of 1, because values that the unnamed calls used to kill now survive.
- Static declarations show their initial field-table value. A trivial initializer (evaluated at compile time) renders as ` = value`; any other decoded value is a comment (`snapshot value ...`). The lazy-initialization sentinel is not a constant and is never shown. Field metadata records the table id (`static_field_id`).
- The kernel loader marks every static field with an initializer `late`, so rendered declarations no longer add `late` for those; the bit is shown only where it is source evidence.
- Exact layouts for other SDK versions and deferred loading-unit roots remain open.

### 9. Body graph — partial

- Body entries use offsets relative to the isolate instruction image and hash the exact byte range. The graph records payload/module/region identity, retains oracle range disagreements on oracle bindings and leaves oracle occurrences without a static body unbound.
- `reports/body_graph.json` and `ir/program.json` now contain the graph's bodies, occurrences and bindings. The ARM64 plain fixture has 30 bodies and 30 occurrences with byte hashes.
- Snapshot Function occurrence IDs, claim-level provenance and integration into call resolution and cross-ABI alignment remain open.

### 10. Typed SSA and memory effects — partial

- The semantic worklist now reports when it reaches its 4096-visit budget. It discards partial input states, withholds semantic statements for that body, sets `machine_code.semantic_worklist_exhausted`, and adds a program warning naming affected functions. The machine listing remains available. A focused test forces exhaustion with a zero-visit budget; the plain ARM64 fixture does not exhaust the budget and still passes the accuracy scorer.
- Each direct, dispatch and IC call result now carries its defining instruction address through register moves and spills. Results from two distinct call sites no longer merge solely because both print `load_result`. A focused merge test covers this.
- Merged values. At a join where every predecessor defines a register or stack slot but with different values, the solver creates `phi_<join>_<location>` instead of dropping it. The emission pass writes an `assign` statement at the end of each predecessor, ordered so one merged value reading another is assigned first. The renderer declares each as `var mergedN;`.
  - Termination: once a location merges at a join it stays merged, and its confidence and class facts only weaken. Without this, joins in a cycle flipped between merged and plain values; the whole-program ARM64 run now exhausts the worklist in 0 functions (it was 1 before merging and 10 with naive merging).
  - A merged value whose assignments all carry one value is replaced by that value (trivial-phi removal), and assignments nothing reads are dropped.
  - A merged register is not an outgoing argument of an unknown callee. This removed a loop accumulator that was being passed to `moveNext`.
  - `Cart.itemCount` and `Cart.subtotal` now recover their accumulators on all three ABIs (`merged1 = 0; ... merged1 = merged1 + line.quantity`, and `0.0` with `price * quantity`). New accuracy expectations require both starting values and forbid the old `subtotal` return of the loop condition.
- Structuring: an `if` arm now stops at the branch's merge block. A fall-through arm used to absorb the join, which nested `build()`'s whole Column/Scaffold tail inside one arm and left `isEmpty`'s `else` without a return. A branch whose true edge goes straight to the merge is negated so the guarded arm is the `then` body.
- Fixes found while doing this:
  - ARM64 compressed-pointer decompression never fired: the check read the `lsl #32` shift from the `x28` operand instead of the next one. ARM64 reference field reads now survive decompression (application statements 336 to 435).
  - x64 64-bit field loads (unboxed ints) were dropped; they now resolve like ARM loads. `xor r, r` is zero.
  - Scalar doubles: ARM64 `fadd`/`fsub`/`fmul`/`fdiv`/`scvtf`, x64 `addsd`/`subsd`/`mulsd`/`divsd`/`cvtsi2sd`, vector zeroing and register copies are lifted, and a body that returns an unboxed double returns `d0`/`xmm0`.
  - The ARM32 VFP fallback decoder dropped the `Vn` source of `vadd`/`vsub`/`vmul`/`vdiv`, so no ARM32 double arithmetic was ever lifted. It now prints all three registers. ARM32 `adds`/`subs` (the low word of an int64 pair) lift as the full operation.
  - ARM32 pool loads through `movw ip, #k; add ip, r5, ip` are recognised, and a `vldr` from the pool combines the two 32-bit words of a double constant.
  - Exact-profile thread constants (`null`, `true`, `false` slots) resolve on x64 and ARM32.
- Heap memory versions, typed operands and a separate machine IR remain open.

### 11. Exceptional and suspension flow — partial

- The final semantic lift now includes handler entry addresses only when a non-generated handler is joined to protected PC descriptors by `try_index`. Each handler starts with unknown machine state, so normal-entry parameters are not silently copied into a catch block. A focused test recovers a handler return that normal branch reachability misses and checks that an unresolved catch register does not become `arg0`.
- Catch-entry move maps are decoded (`src/snapshot/cluster/catch_moves.rs`) from each Code's `catch_entry` TypedData, following `CatchEntryMovesMapReader`: entry headers are target-`intptr_t` stream values, prefixes are stored back to front, and entries share suffixes. A `suffix_offset` of `-1` means no suffix; rejecting it at first hid every map. On the whole-program ARM64 fixture, 519 entries in 190 functions decode. All sit at try-range pc descriptors, 444 at exact `bl`/`blr` return addresses, and 15 carry moves.
- Exceptional edges. Every pc descriptor inside a try with a real handler is an edge from that call to the handler. The lifter splits the block after the call. The handler's input is the meet over its throwing calls of the frame slots at the throw, with the moves applied (slot `s` is `fp - (s + 1) * word` in AOT frames; constants come from the pool), plus the VM's reserved registers, `e` in the exception register and `stackTrace` in the stack-trace register (`x0`/`x1`, `r0`/`r1`, `rax`/`rdx`). Values that differ between throwing sites become merged values assigned at each call. A handler waits for its throwing calls to be simulated instead of starting empty; the empty start flowed around an enclosing loop and lost the null register there for good.
- Rendering. Handler entries are always block starts. Blocks only a handler reaches are no longer taken by the resume walk or pulled into an enclosing natural loop, and a handler whose entry block only restores the frame still renders. A catch clause after a terminal `return` still renders, and it binds `stackTrace` when the body reads it. On the whole-program ARM64 fixture, unrecovered catch bodies went from 11 to 1 of 26 clauses; for example `RenderObject._layoutWithoutResize` renders `catch (e, stackTrace) { _reportException('performLayout', e, stackTrace); }`, as in the Flutter source.
- Suspensions. `Function::ModifierBits` is decoded for the exact 3.12.2 layout (kind 5 bits, 9 recognized-method bits, then the modifier) as `async_modifier`. On the whole program it reports 75 `async` and 3 `sync*` functions, and every function with a yield descriptor is one of them. A call at a pc descriptor with a yield index (`SuspendInstr::EmitNativeCode`) is a suspension. In an `async` body its stub is `stub Await`, taking the awaited value in `SuspendStubABI::kArgumentReg`. On x64 the `LeaveFrame; ret` after the suspend call is removed: resume skips it, and treating it as the exit dropped everything after the first `await`. `_checkout` now renders `await future;` and its continuation on all three ABIs, and an accuracy expectation requires it. An await is a Dart call for static-field staleness.
- Remaining: generated finally handlers, telling await from yield in `async*` bodies, and precise try-range brackets in the renderer.

### 12. Optional defaults — done for retained signatures

- The lifter recognizes `PrologueBuilder`'s optional-parameter prologue on all three ABIs. The arguments descriptor (`x4`, `r4`, `r10`) is seeded at entry. Its count element becomes `aot.argumentCount`, `count - 2·fixed` a Smi, and a frame base scaled by it (`add C, fp, wB, sxtw #2`, `add C, fp, rB, lsl #1`, or an x64 `[rbp + rcx*4 + d]` operand) resolves parameter `j` at displacement `d` as `j = fixed + 1 - d / word`.
- Named parameters: the prologue compares descriptor name `k` with a pool string, then loads that entry's position. The compared string names the parameter the matching path loads.
- A merged value whose assignments are exactly one optional parameter and one constant is that parameter with its default: the missing-argument path stores `DefaultParameterValueAt`. The merged value is replaced by the parameter, the prologue's count tests are dropped, and the default is recorded in `parameter_defaults` (visible position). Only optional slots of the retained signature qualify, and interpolated strings are not constants.
- Whole-program ARM64: 35 functions recover defaults (17 with named parameters), for example `OSError([String arg0 = "", int arg1 = -1])` (`noErrorCode` is -1), `ArgumentError.value(value, [name = null, message = null])`, `Utf8Codec.decode(..., {bool? allowMalformed = null})`, and `RenderObject.showOnScreen({..., Curve? curve = Cubic(0.25, 0.1, 0.25, 1.0)})`, which is `Curves.ease`. Defaults eliminated by signature shaking remain unavailable: the fixture's `Cart.add({int quantity = 1})` was turned into a constant.
- Also fixed here: the write-barrier call enters `WriteBarrierWrappers` at a per-register offset, and frameless ARM64 code saves `x30` around it. Both now fuse; unfused barrier checks on the whole program went from 51 rendered to 0.

### 13. Inline fragments and null checks — partial

- A `CodeSourceMap` null-check operation now carries `null_check_name` when its pool index resolves to a String through the validated exact-version `global_object_pool` root. Missing roots, non-reference entries, negative indexes and unresolved strings remain unnamed. This follows the VM's `Code::GetObjectPool` and `GetNullCheckNameIndexAt` path. A decoder test checks the index and rejects a negative one.
- The plain ARM64 application functions contain no retained null-check operations, so this path has no fixture-level coverage yet.
- Inline regions form a tree: each records its depth, its enclosing region, the caller's source line at the push, and an occurrence id. Ranges of one inlined call split by the scheduler share an occurrence (same callee, same parent occurrence, same call line).
- The renderer no longer re-prints each region's statements in a trailing "Statements of X (inlined ...)" block, which duplicated statements already in the body. It marks in place where statements enter an inlined call (`// inlined: get:values (called at line 52)`, nested calls joined with `>`) and where they return to the host.
- `reports/inline_fragments.json` lists every inlined call with its callee, host, depth, enclosing callees, call line, pc ranges, the statements whose innermost occurrence it is, and whether the callee also has a standalone body. Fragments of one callee from different hosts are not merged into a body. The plain ARM64 fixture has 15 fragments, 14 of them for callees with no standalone body.

### 15. Cross-ABI consensus — partial

- Qualified names and occurrence ordinals prevent map overwrites. Repeated names without a proven parent stay ABI-local instead of receiving accidental corroboration.
- Fingerprints include complete sorted multisets, so repeated values and differences after item 16 affect the digest. Unknown addresses, snapshot reference labels, machine immediates and unrelated parenthesized labels no longer count as source constants. Equal empty observations cannot corroborate.
- The three-ABI plain fixture reports 19 aligned occurrences, 12 corroborated observations, seven disputed and 33 unaligned. Stronger identity joins, normalized typed facts and proof that matching rules are independently correct remain open.

### 17. Compiler lab and accuracy scoring — partial

- The lab accepts an exact `--flutter-bin`, records its version and fails if any requested variant does not produce a matrix row. The miner compares uniquely named functions and intersects complete statement-shape counts using the actual snake-case JSON kinds. It still needs pinned multi-SDK orchestration and runtime-dependent cases.
- The accuracy scorer supports forbidden semantic statements and confidence/parts matching. A regression now requires the complete low-confidence `formatPrice` return on all three plain ABIs and forbids the old incomplete high-confidence return. The three current plain outputs pass.

### 18. Runtime and Flutter evidence — partial

- Runtime trace refinement now exports every observed dispatch target and every argument shape with its hit count. Repeated observations of one target are combined, while minority targets and differing named-argument lists remain visible. The old dominant-target and maximum-arity summaries remain for existing consumers.
- Traces still use the snapshot hash and ABI rather than a full payload identity. They are not yet attached to body occurrences or used during decompilation, and Flutter-specific enrichment remains open.
