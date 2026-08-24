# Dart VM snapshot oracle

Clutter can augment its static snapshot parser with Dart VM evidence. The
oracle is a patched build of Dart's own `analyze_snapshot` executable. It asks
the exact product VM deserializer to load `libapp.so`, emits a semantic object
index, then shuts the isolate down. It never looks up or invokes `main`.

This provides evidence that is difficult to reconstruct safely from bytes
alone:

- the authoritative root library and import graph, including obfuscated URIs;
- live Library, Class, Field, Function, every AbstractType subtype, Code, and
  object-pool identity;
- class hierarchy, generic bounds/defaults, interfaces, modifiers, field
  offsets, instance layout slots, static storage/value identity, and declared
  field types;
- retained internal and user-visible function signatures, result/parameter
  types, required named parameters, and generic bounds;
- closure-to-parent Function links and a type-reference library dependency
  graph, even when the product import arrays are empty;
- static, async, generator, constructor, and synthetic flags;
- exact isolate-instruction offsets, code sizes, named VM stubs, and
  unattributed AOT instruction-table boundaries;
- typed object-pool references for strings, Functions, Code owners, Fields,
  Classes, Types, Libraries, and bounded instance/array contents;
- class ownership for code whose `Function` object was discarded by full AOT.

Clutter joins this VM graph to its independently decoded snapshot and machine
code by exact isolate-instruction offset. Logical aliases of the same optimized
body are assigned one-to-one using independently recovered owner, library,
kind, signature, parameter, and return-type evidence. Equal-score ties retain
their alternative parent Functions in the report. Code addresses remain fully
linked even when a shared body cannot safely inherit one semantic target name.

Clutter validates the analyzer's snapshot hash, target architecture, word size,
compressed-pointer layout, root `main`, and code offsets before trusting it.

## Build an analyzer

Use a Dart SDK source checkout whose snapshot hash matches the target APK. The
two APKs currently in `test-apk/` use snapshot hash
`78da37fed6bf1489361a312568249f3f`, which is accepted by the Dart 3.11.4
checkout used during development.

For the Dart 3.12.2 accuracy fixtures in `test/`, use the reproducible builder:

```bash
vm-oracle/build-exact-analyzers.sh \
  ~/Documents/Projects/dart-sdk \
  ~/Android/Sdk/ndk/28.2.13676358 \
  target/vm-oracle-sdk \
  d684a576a6aa954ae107a03b2b4e1d61c3bebe93 \
  x64c
```

This creates a disposable shared clone below the work directory, obtains
`depot_tools`, syncs the exact revision's DEPS and CIPD tools before patching,
and builds only in that clone. It verifies that the source SDK's commit and
worktree status are unchanged. Repeating the command reuses the synced tools
and build output.

After syncing the Dart source dependencies with `gclient sync`, run:

```bash
vm-oracle/build-analyzers.sh /path/to/dart-sdk /path/to/android-ndk arm arm64c
```

The patch adds schema version 5 and enables the analyzer on ARMv7. Dart
upstream normally gates it to 64-bit targets. On ARMv7 DWARF-mode snapshots,
`Function::CurrentCode()` can deserialize to a non-Code object even though the
corresponding `Code.owner` still points to the Function. The patched analyzer
pre-indexes that authoritative reverse relation, which recovers the code link
without dereferencing the invalid forward link.

Schema 5 additionally emits payload-wide semantic evidence that the per-object
graph cannot express:

- `static_calls`: every Code target reachable from the global object pool.
  Precompiled snapshots drop each Code's own `static_calls_target_table`
  (`NOT_IN_PRECOMPILED` in the VM source), so the pool is the authoritative
  static-call evidence; entries carry pool index, target offset, size, owner
  id, and — when the owner is a Function — its name, staticness, and
  parameter count (argument-descriptor seeds for the signature solver);
- `dispatch_metadata`: the materialized dispatch table as a selector-index →
  Code array with target offsets and owners, plus the architecture's
  `kOriginElement` and global pool length in metadata;
- `class_ranges`: populated `[start, end]` CID runs from the live class
  table (obfuscation renames classes but cannot compact CIDs);
- `unboxed_field_bitmap` per Class: the raw unboxed-field bitmap, one bit per
  instance word slot;
- `pc_descriptors` per linked Function: an entry count plus a kind histogram
  (`deopt`, `ic_call`, `unopt_static_call`, `runtime_call`, `osr_entry`,
  `rewind`, `other`);
- `argument_descriptor` per Function: fixed/optional/positional/named/implicit
  counts plus every surviving parameter name.

Architecture names map to APK ABIs as follows:

| Dart build arch | APK ABI | Pointer mode |
| --- | --- | --- |
| `arm` | `armeabi-v7a` | 32-bit |
| `arm64c` | `arm64-v8a` | compressed pointers |
| `x64c` | `x86_64` | compressed pointers |

The script applies `dart-sdk-3.11.4.patch`, links the selected Android NDK into
the Dart checkout layout, and builds product `analyze_snapshot` binaries.

## Generate and consume evidence

For the ARMv7-only obfuscated TechPos APK:

```bash
clutter vm-oracle test-apk/TechPos-26.10.1+1670.apk \
  --analyzer /path/to/dart-sdk/out/ProductAndroidARM/analyze_snapshot \
  --adb auto \
  --out techpos.vm.json

clutter decompile test-apk/TechPos-26.10.1+1670.apk \
  --abi armeabi-v7a \
  --vm-oracle techpos.vm.json \
  --out recovered-techpos
```

For the multi-ABI test APK:

```bash
clutter vm-oracle test-apk/app-release.apk \
  --abi arm64-v8a \
  --analyzer /path/to/dart-sdk/out/ProductAndroidARM64C/analyze_snapshot \
  --adb auto \
  --out app-release.arm64.vm.json

clutter decompile test-apk/app-release.apk \
  --abi arm64-v8a \
  --vm-oracle app-release.arm64.vm.json \
  --out recovered-app-release
```

`--adb auto` uses the single connected device selected by `adb`. Supplying an
explicit device serial is safer when multiple devices are attached. Without
`--adb`, Clutter runs the analyzer as a local executable.

Generation also writes `<oracle>.binding.json`. This manifest binds the raw
oracle bytes to the complete APK/AAB digest, selected archive member and
`libapp.so` digest, ABI, all four snapshot regions, pointer layout, analyzer
binary digest, analyzer schema, and exact Dart commit. Decompilation rejects a
raw analyzer document without this binding, or when any bound field differs.
Schema 4 omits process-randomized snapshot base addresses, so repeated runs of
the same analyzer against the same payload produce byte-identical JSON.

The generated project contains:

- `metadata/vm_oracle.json`: compact validation and match statistics;
- `metadata/vm_snapshot_analyzer.json`: the complete VM semantic object index;
- per-function VM links in `reports/functions.json`;
- VM-linked classes, fields, and code-less functions in
  `reports/declarations.json`;
- literal VM imports and inferred type-reference dependencies in
  `reports/libraries.json`;
- VM coverage counters in `reports/coverage.json`;
- VM-verified types, modifiers, signatures, ownership, async/static state, and
  code-offset annotations in generated Dart.

## Validation corpus

The schema-3 analyzers and `--scope all` pipeline were exercised against both
APKs in `test-apk/`. These are useful regression targets rather than claims
that erased source text was recovered:

| APK | VM objects | Linked function entries | Exact code ranges | Code-resolved direct calls | VM-linked declarations | Type-reference edges | Dart parse failures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `TechPos-26.10.1+1670.apk` (ARMv7, obfuscated) | 128,110 | 33,392 | 32,074 | 135,710 / 135,710 | 15,052 | 4,403 | 0 |
| `app-release.apk` (ARM64) | 106,526 | 19,334 | 18,648 | 89,477 / 89,477 | 22,720 | 2,211 | 0 |

The parse check is:

```bash
dart format --output=none recovered/lib
```

## Limits

The oracle recovers surviving runtime semantics, not erased source text. It
cannot recreate identifiers removed by obfuscation or recover source package
boundaries that were replaced with opaque URI tokens. Full AOT also discards
some `Function` objects; in those cases the VM still recovers the owning class
and library from the `Code` owner, while Clutter retains an address-stable
`unknownFunction` name.

In DWARF stack-trace mode the serialized instructions table can contain exact
code boundaries with no surviving semantic owner. Clutter retains these as
`AotCodeBoundary` evidence and does not invent a function name, signature, or
class. Positional parameter names and default argument expressions are also
generally absent from product snapshots.

Numeric instance offsets are never interpreted globally. Clutter assigns a
field name only when register provenance identifies a receiver class and that
class's VM layout contains the offset. Low-level Array/runtime stores therefore
cannot borrow an unrelated application field name merely because their byte
offset is equal.

Never reuse oracle JSON across APKs or ABIs. Clutter deliberately rejects
evidence when the root entry point, feature layout, or instruction offsets do
not match.
