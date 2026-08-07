# Clutter Flutter recovery fixture

This small application is ground truth for exercising Clutter against both
ordinary and obfuscated Flutter release AOT snapshots. Its code is deliberately
split by role so recovered library and class ownership can be checked.

The fixture reaches records and pattern switches, generics, extensions,
operators, closures and tear-offs, optional parameters, recursion,
try/catch/finally, `async`, `async*`, inheritance, mixins, enhanced enums,
private names, and Unicode strings. Two uncalled functions use
`@pragma('vm:entry-point')`; one ordinary uncalled function is expected to be
tree-shaken.

From the repository root:

```sh
cd fixtures/clutter_edge_cases
flutter test
flutter build apk --release --target-platform android-arm64
cd ../..

cargo build --release
./target/release/clutter decompile \
  fixtures/clutter_edge_cases/build/app/outputs/flutter-apk/app-release.apk \
  --abi arm64-v8a --scope app --emit-ir --no-assets --out recovered-fixture

python3 fixtures/clutter_edge_cases/tool/evaluate_recovery.py recovered-fixture
```

Build one genuine obfuscated APK containing all three supported ABIs and the
matching Flutter split-debug ELFs:

```sh
fixtures/clutter_edge_cases/tool/build_obfuscated.sh
cargo build --release
fixtures/clutter_edge_cases/tool/verify_obfuscated.sh
```

The verification script passes the split-debug directory to Clutter, decompiles
ARM32, ARM64, and x64 independently, evaluates executable-name and tree-shaking
expectations, enforces source/name/call-resolution thresholds, and emits
generated Dart that can be checked with `dart format --output=none`. Analyzer
errors are expected where the AOT compiler erased imports, defaults, or type
names and Clutter leaves an explicit pseudocode boundary.

The evaluator checks recovered names, pragma retention, and tree shaking. It
also prints the quantitative coverage report. AOT inlining and specialization
mean a source declaration can survive as an identifier without retaining a
standalone function body.
