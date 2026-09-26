# Pavo Connect recovery review

Analyzed `pavo-test/pavo_connect.apk` on 2026-09-26. The fixes below improve
Clutter's recovery and reporting. They do not modify the APK.

The APK is version 2.5.10, contains only `armeabi-v7a`, and uses the exact known
Dart 3.9.2 snapshot hash `97ff04a728735e6b6b098bdf983faaba`. Its SHA-256 is
`6ff2936f5f0d289be21beb098a0356811b68b1b7c18dcc683d0a853bdd2ad5e2`.
The input has 41 Flutter assets and no deferred loading units. No debug ELF,
obfuscation map, or VM oracle was supplied. Analysis was static.

## Results

Both runs used a release build and the same command options:

```sh
clutter decompile ./pavo-test/pavo_connect.apk --out OUTPUT --emit-ir
```

| Measurement | Before | After |
| --- | ---: | ---: |
| Elapsed time | 324.13 s | 49.06 s |
| System CPU time | 272.23 s | 1.37 s |
| User CPU time | 83.32 s | 80.55 s |
| Peak resident memory | 2,171,248 KiB | 2,168,644 KiB |
| Logical functions | 16,515 | 16,515 |
| Unique code ranges | 16,120 | 16,120 |
| Decoded function bytes | 3,702,420 | 3,702,420 |
| Undecoded function bytes | 0 | 0 |
| Functions with decoded async modifiers | 0 | 3,123 |
| Functions proven async by modifiers | 0 | 24 |
| Modifier/descriptor-backed `stub Await` statements | 0 | 27 |
| VM stub bodies emitted as source functions | 76 | 0 |
| VM stub bodies preserved in reports | 76 | 76 |
| Manifest root library | unavailable | `KKf` |
| Bodies exhausting the dataflow budget | 2 | 2 |

Elapsed time improved by 6.6 times in this pair of local runs. This is not a
statistical benchmark. The baseline overlapped compilation and the final run
overlapped small fixture checks. The large drop in system CPU time directly
supports the report-writing diagnosis. Memory use remains about 2.1 GiB.

Complete outputs are in `target/pavo-before/` and `target/pavo-after/`.
Timing and stderr logs are alongside those directories. The final output is
about 555 MiB, including the optional IR and all assets. Generated source and
application data stay in ignored build directories.

## Fixed issues

1. JSON serialization wrote directly to an unbuffered `File`. A sample of the
   original run had already issued over 150 million writes before finishing.
   JSON reports and unresolved JSONL now use `BufWriter`. Explicit flushes
   propagate failures before the staged output is published. Regression tests
   cover a report larger than the buffer and a small write whose failure occurs
   only at flush.

2. Async modifier decoding was restricted to Dart 3.12.2. The matching SDK
   3.9.2 sources verify the same five Function-kind bits and nine recognizer
   bits before the two async bits. Exact snapshot hashes now select that
   verified layout. Unknown hashes keep their fallback behavior. The Pavo run
   identifies 24 async and 3,099 synchronous Functions and recovers 27 await
   boundaries at retained yield PC descriptors. The function report now exposes
   these modifiers. A synchronous modifier also prevents an async-callee
   heuristic from relabeling a synchronous caller.

3. Static snapshot roots were decoded but their root library was omitted from
   the manifest, and the root evidence was available only with `--emit-ir`.
   Recovery now resolves the validated root reference to `KKf` and writes
   `metadata/snapshot_roots.json`. This preserves the opaque library identity
   without claiming to know its original package name. Oracle enrichment can
   still replace it with authoritative evidence when supplied.

4. The source filter recognized VM stubs only through oracle metadata, despite
   static roots already proving their Code identities. The filter now uses
   those references, including code aliases. All 76 stub bodies remain in the
   reports and assembly but no longer appear as reconstructed Dart functions.
   Display names alone do not cause exclusion.

5. The dataflow queue could contain a join once per predecessor. A queued block
   already consumes each predecessor's latest output, so duplicate visits
   wasted the safety budget. Queue membership is now tracked for normal and
   exceptional edges. A regression checks that an agreeing branch diamond
   converges within four visits. This change did not eliminate Pavo's two
   exhaustion cases, and the 4,096-visit limit remains unchanged.

## Remaining shortcomings

- Obfuscation and AOT stripping leave 11,530 functions with synthetic names and
  777 of 789 library URIs opaque. The root token `KKf` identifies one library,
  but it cannot restore source package boundaries. Clutter correctly broadens
  the requested application scope and keeps the complete snapshot. A matching
  obfuscation map or split-debug ELF is needed for original-name enrichment.
- No internal source maps or source locations are recovered from this build.
  It uses `dwarf_stack_traces_mode`, and no matching debug ELF was supplied.
  PC descriptors, stack maps and exception-handler metadata do survive.
- Only 3,123 Function objects retain modifier information. The other 13,392
  logical entries remain unknown. There are 173 retained yield descriptors;
  27 become verified awaits in modifier-proven async bodies. The other
  descriptors are not automatically promoted to awaits.
- Only 25,236 of 49,640 direct calls have semantic targets, although all have
  resolved code targets. Only 48 of 5,721 dispatch-table sites resolve exactly.
  More physical address matches alone cannot prove the missing Dart identities.
- `sub_1d3650`, 476 bytes, and `sub_3616bc`, 17,836 bytes, still exhaust the
  dataflow budget. Their partial states are discarded and their complete
  instruction evidence remains available. The next solver work should trace
  changes in auxiliary state around loops and measure convergence, rather than
  raise the limit and accept partial facts.
- Peak analysis memory is still about 2.1 GiB. Shared evidence storage and
  streaming assembly emission are useful future targets, but are separate from
  the unbuffered JSON fix implemented here.

## Validation

- `cargo test`: 253 library tests and seven CLI tests passed.
- Fresh application-scope decompilations of the plain fixture passed
  `test/tool/evaluate_accuracy.py --allow-partial` on ARM64, ARM32 and x64.
- `test/tool/evaluate_pavo.py target/pavo-after`: 118 checks passed. These check
  the exact input, complete decoding, preserved function inventory, root
  evidence, stub exclusion, modifier counts and the exact PC descriptor for
  each recovered await.
- `git diff --check` passed. The workspace already had unrelated rustfmt
  differences; those existing edits were preserved.

To reproduce the Pavo checks:

```sh
cargo build --release
target/release/clutter decompile pavo-test/pavo_connect.apk \
  --out target/pavo-check --emit-ir
python3 test/tool/evaluate_pavo.py target/pavo-check
```

These checks establish retained metadata and evidence consistency. There is no
original Pavo source or matching oracle here to establish source equivalence.

## Rendered-code wave (2026-09-26, afternoon)

This pass targeted the rendered Dart instead of the reports. The same
static run was measured before and after:

```sh
clutter decompile ./pavo-test/pavo_connect.apk --out OUTPUT --no-assets
```

Counts cover every generated `.dart` file, including
`lib/recovered/unattributed.dart`, which holds most anonymous bodies.

| Placeholder or construct in `lib/` | Before | After |
| --- | ---: | ---: |
| `aot.unresolvedRegister(...)` | 6,889 | 4,713 |
| `aot.unresolvedValue(...)` | 9,113 | 5,508 |
| `'shared-code result'` | 2,386 | 251 |
| `'slot 0x…'` stack placeholders | 3,737 | 2,946 |
| `'stack argument'` | 1,061 | 105 |
| Unresolved predicates | 3,955 | 3,258 |
| `aot.unresolvedRegion(...)` | 3,711 | 3,436 |
| Bound `final` statements | 32,349 | 39,832 |

`reports/coverage.json` changes: resolved indirect calls 1,569 → 2,372 and
semantic statements 187,250 → 197,027. Exact dispatch resolutions went from
48 to 46. Both lost sites (`sub_4bd35c`, `sub_472928`) had resolved `==` on
a stale closure. The receiver is actually the result of the preceding
`listen`-style dispatch call, which the old lift had dropped.

### Fixes

1. **ARM32 `stm sp, {…}` was read as a stack-pointer write.** It set the
   frame delta to unknown for the rest of the block and discarded the
   stored outgoing arguments. Store-multiple now only writes memory, and
   `stm sp, {a, b}` records ascending outgoing slots. Loads from the
   outgoing area read the staged value back; switchable calls reload their
   receiver this way.
2. **VM stubs are named from the VM snapshot roots.** `VMSerializationRoots`
   writes one root per `VM_STUB_CODE_LIST` entry after the symbols. The
   per-release lists (3.4–3.12) are generated from the SDK tags in
   `snapshot/cluster/vm_stub_names.rs`. The tail is accepted only when every
   reference is a distinct VM `Code` object. Pavo names all 171, so pool
   entries such as `InstanceOf`, `Subtype*TestCache` and
   `InstantiateTypeArguments` get names. `ldr lr, [code, #entry_point]`
   keeps that name for the following `blx`, which removes the bogus `_slot_4`
   field reads.
3. **Native calls.** A `blx` through a reset pool entry while R9/X5/RBX holds
   a `nativePoolEntry` is a `NativeCallInstr`. Its stacked arguments and the
   `[SP]` result slot are modeled, and it renders as
   `aot.native('_Double._toString', <dynamic>[this])`. All 169 native entry
   loads lift.
4. **Allocation stubs bind values.** `AllocateContext`, `AllocateArray`,
   `AllocateGrowableArray`, typed-data arrays and small records read their
   ABI registers. They render as `aot.context(n)`,
   `List<dynamic>.filled(n, null)`, `<dynamic>[]`, `Uint8List(n)` and
   `(a, b)`. Allocations nothing reads afterwards, such as arrays consumed by
   an interpolation, are omitted.
5. **Closure contexts.** Context slots are named `parent` and `captured<i>`
   from the 3.9.2 layouts. The closure parameter is typed `_Closure`, and the
   retained `_Closure._context` field carries the Context type. The output
   now reads `closureContext._context.captured0` instead of `._slot_14._slot_c`.
6. **Unresolved dispatch-table calls are kept.** 5,665 sites that previously
   emitted nothing now bind a result. A named selector renders as the member
   invocation, for example `products[index]` or `list.length`. Otherwise the
   site renders as `aot.dispatch(offset, receiver, [...])`. Arguments are the
   contiguous registers after the receiver that the call's own block writes.
   When the receiver is also the first stacked slot, the call uses the
   all-stack convention. These sites do not count as resolved.
7. **Type tests and type arguments.** `InstanceOf` renders as `x is T` when
   the destination type has a Dart spelling. `InstantiateTypeArguments*`
   binds `aot.instantiateTypeArguments(...)`.
8. **All-stack callees.** A body that reads incoming stack words and no
   argument register gets stack parameter locations. Callers then pass their
   stacked values instead of stale registers, as in
   `Product.toString(product)`.
9. **Thread `empty_array`.** It follows `bool_false` in every supported
   release, so `Map._fromLiteral(const [])` no longer loses its argument.

### Validation

- `cargo test`: 261 library tests and seven CLI tests passed. New tests
  cover native calls, `stm` argument staging, context naming, pool-stub
  entry calls, unresolved dispatch arguments, VM stub roots and the
  renderer forms.
- All nine `test/tool/evaluate_accuracy.py` fixture variants passed. On the
  plain ARM32 fixture the `ListView.builder` item builder now reads
  `closureContext._context.captured1[arg1]`, `product.toString()` and
  `itemCount: products.length`, which matches `test/simple_app/lib/main.dart`.
- `test/tool/evaluate_pavo.py`: 125 checks passed. New checks cover
  native calls, `InstanceOf` naming, Code-entry reads, kept dispatch calls
  and rendered contexts.
- `pavo-test/base_launcher.apk` (also Dart 3.9.2) decompiles in 59 s.

### Remaining shortcomings

- 5,665 dispatch sites still lack a proven implementation. Row
  displacement mixes other selectors' slots into each candidate set, so
  neither the selector name nor the arity can be read from the table.
- The FFI lazy-lookup cache slots (`resetPoolEntry(N) == 0`) are not yet
  rendered as FFI resolution.
- Growable-list literals are not rebuilt: the backing array's element
  stores stay implicit, and `list._slot_c = array` shows the machine layout.
- Callees with unknown arity still receive extra caller-written registers.
