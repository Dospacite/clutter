# Clutter AOT recovery

This directory contains conservative pseudocode recovered from `.tmp-probe/probe/build/app/outputs/flutter-apk/app-release.apk`.

- Dart snapshot: `ace654289f5abc240509fc941453ebc5`
- ABI: `arm64-v8a`
- Application package: `edge_probe`
- VM-resolved root library: `unavailable`
- Recovered libraries: 2
- Recovered identifiers: 8677
- Recovered strings: 10820
- Logical function entries: 39 across 39 unique AOT code ranges
- Source-view functions: 39; evidence-only runtime/boundary functions: 0
- Dart VM-linked function entries: 0
- Dart VM-linked declarations: 0
- Function declarations: 35 with standalone code, 3 without standalone code
- Resolved snapshot types: 13 return types, 12 parameter types, 0 named parameter names
- Typed declarations: 11 classes with metadata, 1 fields with declared types
- Direct call sites: 300 (296 code targets, 150 semantic targets); indirect call sites: 10

Global identifiers and strings are stored in `metadata/symbols.json` instead of being mixed into the generated Dart source. `reports/functions.json` contains function, CFG, semantic, and VM metadata; `reports/libraries.json` separates literal VM imports from inferred type-reference dependencies; `reports/declarations.json` includes declaration-only evidence; `reports/call_graph.json` separates direct, dynamic, and object-pool calls; and `reports/assembly.s` contains the complete annotated instruction stream. Typed snapshot totals are in `metadata/snapshot_evidence.json`; compressed class-dispatch runs are in `reports/dispatch_table.json` when snapshot recovery succeeds. Surviving FunctionType, Class, and Field graphs drive the rendered types and relationships. When supplied, the complete no-main-execution VM index is preserved in `metadata/vm_snapshot_analyzer.json` and its validated match summary is in `metadata/vm_oracle.json`. Full AOT omits positional parameter names and optional default expressions, so those remain labeled placeholders rather than guesses. VM/runtime helper calls are summarized in the source view and preserved exactly in the reports. Dynamic class-dispatch sites retain selector families and bounded candidate sets, while field names require receiver-class layout proof.

The `.dart` files are not original source and are not promised to build into an equivalent application. A recovered return is emitted only for branch-free, fully decoded machine code with one return and one high-confidence data-flow expression. Release AOT compilation removes debug information, tree-shakes unreachable code, folds constants, and may inline functions. A declaration without standalone code does not by itself distinguish inlining from folding, deferral, or tree shaking. Unknown behavior is intentionally left explicit instead of being guessed.
