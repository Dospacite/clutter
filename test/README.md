# Recovery test applications

Two minimal Flutter apps used as ground truth for measuring and improving
Clutter's decompilation. Both contain identical Dart code
(`lib/main.dart`, `lib/models.dart`) exercising constructs a decompiler must
recover: enums, const constructors, final/instance/map fields, getters,
methods with optional named parameters, generic functions with function-typed
parameters, closures, `async`/`await`, string interpolation, collection
operations, and exceptions.

## What current Clutter recovers in these apps

Beyond the declaration/type evidence documented in the fixture result files,
the semantic lifter and renderer now recover, per body:

- cross-block register/stack dataflow (fixpoint over the CFG), so call
  arguments, branch predicates, and returned expressions survive branches;
- structured `if`/`else` and `while` regions with explicit returns;
- string interpolations rebuilt from the array-allocation `_interpolate`
  lowering (for example `'Deal: ${…} at ${formatPrice(…)}'`);
- Dart-style call sites: property getters/setters, receiver-carrying instance
  calls, allocator+constructor pairs collapsed into one constructor, and
  named arguments zipped from surviving callee signatures;
- canonical `null`/`true`/`false` constants folded from VM register idioms,
  and `const ClassName()` rendering for canonical snapshot instances;
- closures nested as local functions inside their proven parent member
  (VM lexical-parent links or source-line containment), with stable
  disambiguation suffixes for same-line siblings;
- `async` members with explicit `await` boundaries when the async-machine
  stubs are named (split-debug or VM oracle), and an explanatory comment
  otherwise; generator identity is documented without inventing `yield`.

| Directory | Purpose |
| --- | --- |
| `simple_app/` | Plain release build (readable symbols). |
| `simple_app_obfuscated/` | Same code built with `--obfuscate`, producing split-debug-info ELFs and a gen_snapshot obfuscation map. |

## Build

Requires Flutter (stable channel) and the Android SDK. Each app builds all
three supported ABIs into one fat release APK:

```sh
test/simple_app/tool/build.sh
test/simple_app_obfuscated/tool/build.sh
```

Artifacts land in each app's `dist/` (gitignored):

- `simple_app/dist/simple_app.apk`
- `simple_app_obfuscated/dist/simple_app-obfuscated.apk`
- `simple_app_obfuscated/dist/symbols/` — per-ABI split-debug ELFs
- `simple_app_obfuscated/dist/obfuscation-map.json`

## Running Clutter

```sh
cargo build --release

# Plain
./target/release/clutter decompile test/simple_app/dist/simple_app.apk \
  --abi arm64-v8a --scope app --out out/plain

# Obfuscated, raw (no auxiliary files)
./target/release/clutter decompile \
  test/simple_app_obfuscated/dist/simple_app-obfuscated.apk \
  --abi arm64-v8a --scope app --out out/obf-raw

# Obfuscated, enriched with split-debug info and the obfuscation map
./target/release/clutter decompile \
  test/simple_app_obfuscated/dist/simple_app-obfuscated.apk \
  --abi arm64-v8a --scope app \
  --symbols test/simple_app_obfuscated/dist/symbols \
  --obfuscation-map test/simple_app_obfuscated/dist/obfuscation-map.json \
  --out out/obf-full
```

Because both apps share identical source, plain vs. obfuscated recoveries can
be diffed directly against each other and against the true sources in `lib/`.

## Accuracy regression scorer

`tool/evaluate_accuracy.py` checks source-level IR facts instead of treating
symbol or metadata counts as recovered code. Generate outputs with `--emit-ir`,
then pass each output under the matching name from
`accuracy_expectations.json`:

```sh
python3 test/tool/evaluate_accuracy.py \
  --variant plain=out/plain \
  --variant obfuscated_map=out/obf-map \
  --variant obfuscated_raw=out/obf-raw \
  --variant plain_arm32=out/plain-arm32 \
  --variant obfuscated_map_arm32=out/obf-map-arm32 \
  --variant obfuscated_raw_arm32=out/obf-raw-arm32 \
  --variant plain_x64=out/plain-x64 \
  --variant obfuscated_map_x64=out/obf-map-x64 \
  --variant obfuscated_raw_x64=out/obf-raw-x64
```

The scorer requires every declared variant by default so a missing ABI cannot
silently pass. Use `--allow-partial` only for a focused local check. The
current checks prove application-package attribution, a coarse retained-body
safety floor, the exact negated unboxed-double price predicate on all three
ABIs, and preservation of the boxed-double receiver in `formatPrice` IR.

## Known baseline gaps (measurement targets)

Observed on the first Dart 3.12.2 recovery of `simple_app`:

- Instance/static Field objects for application classes are tree-shaken by
  full AOT in this app's code shape (`field_declarations: 0` at app scope).
  This matches expected AOT behavior — the Dart 3.11.4 edge-case fixture
  baseline also retained only one typed field — not a 3.12 regression.
  Cross-version parity was verified by rebuilding that fixture with Flutter
  3.44.9: signatures (21), return/parameter types (21/28), class graphs (14),
  named-parameter names (6), and direct/indirect calls (257/8) all match the
  recorded Dart 3.11.4 baseline exactly.
- Signatures survive for 12/30 app functions; callables whose exact signature
  was dropped render as `(List<dynamic> args)` returning `dynamic`.
- Calls whose exact signature was tree-shaken can still carry stale
  caller-saved values because no trustworthy arity remains. These stay
  explicit rather than being silently trimmed.
- Complex bodies still contain `aot.unresolvedRegion` when the CFG cannot be
  structured without inventing source control flow.
