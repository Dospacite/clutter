# Obfuscated recovery results

The fixture was built with Flutter 3.41.6 / Dart 3.11.4 using `--obfuscate`,
`--split-debug-info`, and all three supported Android targets. Clutter selected
each symbol ELF from the output directory by architecture and GNU build ID.

| ABI | Functions | Source mapped | Inline ranges | Direct calls resolved |
| --- | ---: | ---: | ---: | ---: |
| armeabi-v7a | 38 | 38/38 | 26 | 195/195 (100%) |
| arm64-v8a | 38 | 38/38 | 26 | 249/257 (96.9%) |
| x86_64 | 38 | 38/38 | 26 | 214/222 (96.4%) |

All 17 expectations with retained executable evidence were recovered on every
ABI, including async and async-star bodies, nested closures, optional-argument
tear-offs, operators, getters, records, generics, recursion, and both
`@pragma('vm:entry-point')` functions. The unreferenced sentinel remained
tree-shaken. All three generated output trees pass the Dart parser; unresolved
imports, erased defaults, and deliberately explicit pseudocode boundaries are
expected to remain analyzer errors.

The allocation/fill type-graph join recovers 18 function return types, 24
parameter types, six named-parameter tokens with their `required` flags, all 14
surviving application Class graphs, and the retained typed Field on every ABI.
When obfuscation removes every application package URI, Clutter now derives the
package from the split-debug `main.dart` root and bridges debug functions to raw
snapshot declarations by exact address. One-to-one and uniquely remaining
per-library class mappings restore `EdgeCaseApp`, `EdgeCaseScreen`, and
`EdgeVector`; the synthetic DWARF mixin-application name restores `AuditTrail`.
Those mappings are propagated through signatures and class graphs while
ambiguous tokens remain visibly obfuscated.

DWARF recovery now includes declaration locations, source line spans, and
inlined-subroutine ranges with call-site lines. For example, the optimized
`EdgeCaseHarness.run` body links its inlined `EdgeVector` constructor and
`AuditTrail.recordEvent` call back to their source declarations. The reports
also distinguish eight ARM indirect calls from direct calls and record zero
undecoded bytes in recovered ARM64 function ranges.

Flutter's optional `--save-obfuscation-map` JSON is accepted as supplemental
identifier and library-ownership evidence; split-debug ELF is not required.
Using matched APK/map pairs without `--symbols` recovers 20 executable entries,
26 declarations, and all 14 application classes on every ABI in about 0.6
seconds. This includes declaration-only types whose names are absent from the
debug ELF.

With neither map nor debug ELF, Clutter deliberately broadens app scope because
the short obfuscated library tokens cannot be reversed. On the ARM64 fixture it
recovers 11,341 raw instruction-table entries covering 2,479,604 decoded AOT
bytes in about 16 seconds, rather than returning only the four unobfuscated
entry points. Names and ownership that the binary no longer associates remain
address-based.

The split-debug ELF remains the strongest optional source-level evidence: it
adds exact qualified symbols, source ownership, line ranges, inline
declarations, and many more named call targets. Flutter map JSON carries no
build ID or ABI, so a map from another build must not be reused.

`ArithmeticOperation`, `AddOperation`, and `MultiplyOperation` are deliberately
not counted as recoverable in the obfuscated profile. Their behavior was fully
inlined into `EdgeCaseHarness.run`, leaving no standalone text symbol, while
obfuscation removed their source identifiers from both the snapshot and split
debug ELF. Clutter retains the inlined machine and call evidence but does not
invent the erased type names.

Reproduce the build and measurements from the repository root:

```sh
fixtures/clutter_edge_cases/tool/build_obfuscated.sh
cargo build --release
fixtures/clutter_edge_cases/tool/verify_obfuscated.sh
```
