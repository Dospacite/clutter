# Clutter AOT recovery

This directory contains conservative pseudocode recovered from `test/simple_app/dist/simple_app.apk`.

- Dart snapshot: `ace654289f5abc240509fc941453ebc5`
- ABI: `arm64-v8a`
- Application package: `simple_app`
- VM-resolved root library: `unavailable`
- Recovered libraries: 2
- Recovered identifiers: 11748
- Recovered strings: 14234
- Logical function entries: 30 across 30 unique AOT code ranges
- Source-view functions: 30; evidence-only runtime/boundary functions: 0
- Dart VM-linked function entries: 0
- Dart VM-linked declarations: 0
- Function declarations: 27 with standalone code, 3 without standalone code
- Resolved snapshot types: 12 return types, 5 parameter types, 0 named parameter names
- Typed declarations: 7 classes with metadata, 0 fields with declared types
- Direct call sites: 146 (139 code targets, 72 semantic targets); indirect call sites: 9

Global identifiers and strings are stored in `metadata/symbols.json` instead of being mixed into the generated Dart source. `reports/functions.json` contains function, CFG, semantic, and VM metadata; `reports/libraries.json` separates literal VM imports from inferred type-reference dependencies; `reports/declarations.json` includes declaration-only evidence; `reports/call_graph.json` separates direct, dynamic, and object-pool calls; and `reports/assembly.s` contains the complete annotated instruction stream. Typed snapshot totals are in `metadata/snapshot_evidence.json`; compressed class-dispatch runs are in `reports/dispatch_table.json` when snapshot recovery succeeds. Surviving FunctionType, Class, and Field graphs drive the rendered types and relationships. When supplied, the complete no-main-execution VM index is preserved in `metadata/vm_snapshot_analyzer.json` and its validated match summary is in `metadata/vm_oracle.json`. Full AOT omits positional parameter names and optional default expressions, so those remain labeled placeholders rather than guesses. VM/runtime helper calls are summarized in the source view and preserved exactly in the reports. Dynamic class-dispatch sites retain selector families and bounded candidate sets, while field names require receiver-class layout proof.

The `.dart` files are not original source and are not promised to build into an equivalent application. A recovered return is emitted only for branch-free, fully decoded machine code with one return and one high-confidence data-flow expression. Release AOT compilation removes debug information, tree-shakes unreachable code, folds constants, and may inline functions. A declaration without standalone code does not by itself distinguish inlining from folding, deferral, or tree shaking. Unknown behavior is intentionally left explicit instead of being guessed.
