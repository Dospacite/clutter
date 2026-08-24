# Clutter

Clutter is a static-analysis CLI that recovers conservative, human-readable
Dart-like pseudocode from Flutter Android release artifacts, including
obfuscated builds with or without Flutter split-debug-info. It does not load the
snapshot into a Dart VM or execute application code during its default static
pass. An optional,
explicit Dart VM oracle can deserialize the snapshot without invoking `main`
and corroborate the static result. Alongside the source view it emits a
typed snapshot-object graph summary, VM metadata, complete annotated assembly,
control-flow graphs, call evidence, and optional cross-ABI consistency checks.

The result is not the original source. AOT compilation removes debug
information, tree-shakes unreachable code, inlines functions, and lowers Dart
semantics into machine instructions. Clutter therefore preserves recovered
library, class, function, address, and signature evidence. The primary `.dart`
view presents recovered Dart concepts; behavior it cannot prove is marked as
unresolved, while machine-level evidence stays in the reports.

## Supported inputs

- Android APK and AAB archives
- `arm64-v8a`, `armeabi-v7a`, and `x86_64` Flutter AOT payloads
- Dart 3.4 through 3.12 clustered snapshots
- Non-obfuscated builds directly from `libapp.so`
- Obfuscated builds directly as raw address/name evidence, optionally enriched
  by a matching obfuscation map and/or split-debug ELF

An APK or AAB must contain `libapp.so`. `libflutter.so` is used when present to
identify the precise Dart runtime version.

## Build

Rust 1.85 or newer and a C toolchain are required.

```sh
cargo build --release
./target/release/clutter version
```

The release artifact is a single `clutter` executable. Capstone is built into
the binary; no Dart or Flutter SDK is required for static analysis. The
optional VM oracle uses a separately built, ABI-compatible Dart
`analyze_snapshot` executable.

## Usage

Inspect an artifact without writing recovered files:

```sh
clutter inspect app-release.apk
clutter inspect app-release.apk --json
```

Recover application-owned libraries and Flutter assets:

```sh
clutter decompile app-release.apk --out recovered
```

Recover an obfuscated build without auxiliary files. If obfuscation hides the
library ownership needed by a narrow scope, Clutter preserves the whole raw
snapshot rather than silently dropping functions:

```sh
clutter decompile app-release.apk --out recovered
```

`--symbols` is optional enrichment. It accepts either the ABI-specific symbol
ELF or the directory Flutter wrote; directory mode selects the ABI and GNU build
ID match automatically:

```sh
clutter decompile app-release.apk \
  --symbols build/symbols \
  --out recovered
```

Flutter can optionally emit a JSON identifier map with
`--extra-gen-snapshot-options=--save-obfuscation-map=...`. Use it as
supplemental evidence:

```sh
clutter decompile app-release.apk \
  --symbols build/symbols \
  --obfuscation-map build/obfuscation-map.json \
  --out recovered
```

Without debug symbols, the JSON map reverses retained identifier and library
tokens before scope filtering, enabling focused application recovery. It cannot
name address-only instruction entries or provide source lines and inline
records. Flutter's JSON map has no ABI or build ID, so it must come from the
exact selected build and target; Clutter reports that identity cannot be
cryptographically validated.

Useful options:

```text
--abi auto|arm64-v8a|armeabi-v7a|x86_64
--module <name>                 AAB module; default: base
--scope app|packages|all        Default: app
--symbols <file-or-directory>   Matching Flutter split-debug-info
--obfuscation-map <json>        Optional gen_snapshot identifier map
--vm-oracle <json>              Optional no-main-execution Dart VM evidence
--no-assets                     Do not extract flutter_assets
--emit-ir                       Write ir/program.json
--cross-abi                     Compare logical functions across packaged ABIs
--jobs <count>                  Rendering worker count
--replace                       Preserve old output as a timestamped backup
```

`--cross-abi` aligns functions by restored library, owner, name, and kind, then
compares architecture-neutral control-flow/call fingerprints. For an obfuscated
multi-ABI artifact, supply the exact `--obfuscation-map`; Clutter will skip the
comparison rather than align opaque names speculatively.

`--replace` only replaces a directory containing Clutter's
`decompilation.json`; it refuses unrelated directories.

## Optional Dart VM oracle

Clutter can use Dart's own product snapshot deserializer as a semantic oracle.
The patched analyzer creates an isolate from `libapp.so`, records authoritative
library/class/field/type/function/code identity, and shuts the isolate down
before looking up or invoking `main`. It recovers the root library even when
its URI is an obfuscated token, restores class/library attribution for code
whose `Function` object was discarded by full AOT, and contributes generic
bounds, declared types, modifiers, required named parameters, async/static
state, lexical closure parents, class instance-slot layouts, field/static
storage offsets, object-pool identities, named VM stubs, and declaration-only
runtime objects. When product AOT has erased the literal import table, Clutter
also derives a separate library-reference graph from surviving VM type edges.

Generate evidence and then pass it to `decompile`:

```sh
clutter vm-oracle app-release.apk \
  --abi arm64-v8a \
  --analyzer /path/to/analyze_snapshot \
  --adb auto \
  --out app-release.vm.json

clutter decompile app-release.apk \
  --abi arm64-v8a \
  --vm-oracle app-release.vm.json \
  --out recovered
```

The analyzer must match the target ABI, snapshot hash, product feature set, and
pointer mode. Clutter validates those properties, the root `main`, and exact
instruction offsets before applying evidence. Exact offsets are joined with
the independently decoded static result. Shared optimized bodies are matched
one-to-one against logical Functions using owner, library, kind, signature,
parameter, and return-type evidence; ties remain explicitly marked with their
alternative lexical parents. A direct call to a body with several distinct
logical aliases receives exact code identity but no arbitrary semantic target.
ARMv7,
ARM64 compressed-pointer, and x64 compressed-pointer build instructions and
validation-corpus measurements are in
[`vm-oracle/README.md`](vm-oracle/README.md).

## Output

```text
recovered/
├── lib/                        one Dart-like file per recovered library
├── support/aot_intrinsics.dart explicit unresolved-operation helpers
├── resources/flutter_assets/  streamed assets, unless --no-assets
├── metadata/
│   ├── android.json
│   ├── resources.json
│   ├── symbols.json             identifiers and strings with provenance
│   ├── snapshot_evidence.json   typed object-graph and VM metadata totals
│   ├── vm_oracle.json           optional VM validation and match totals
│   ├── vm_snapshot_analyzer.json complete optional VM semantic index
│   └── deferred_units.json      when deferred AOT units are packaged
├── reports/
│   ├── coverage.json
│   ├── functions.json           function, CFG, semantic, and VM metadata
│   ├── declarations.json        class/field/function declaration evidence
│   ├── libraries.json           VM imports and type-reference dependencies
│   ├── call_graph.json          direct, dynamic, and object-pool call sites
│   ├── assembly.s               complete annotated machine instructions
│   ├── dispatch_table.json      compressed class-dispatch runs and targets
│   ├── cross_abi.json           only with a successful --cross-abi comparison
│   ├── cross_abi_consensus.json ABI-neutral consensus tiers and disputes
│   ├── body_graph.json          physical-body/logical-occurrence resolution
│   └── unresolved.jsonl
├── ir/program.json             only with --emit-ir
├── decompilation.json
└── README.md
```

Every generated Dart file states that it is pseudocode. Recovered methods are
grouped by class owner, private-key suffixes are hidden from display while raw
snapshot symbols remain in the reports, and surviving snapshot type graphs are
rendered as Dart return types, nested generic parameter types, bounds,
nullability, named-parameter names, and `required` markers. Positional names use
stable placeholders because the Full AOT serializer deliberately omits them.
Optional default expressions are likewise marked `default unavailable` rather
than guessed because FunctionType retains their types and shape, not their
source expressions.
Signature evidence copied from a related closure or tear-off is labeled
separately from an exact per-function signature.

The primary source view reconstructs recognizable fields, initializers,
constructors, method calls, return expressions, and source-line-grouped string
literals where the evidence supports them. Intermediate results receive
Dart-style local names. VM allocation, write-barrier, type-test, and failure
helpers are summarized as runtime evidence instead of being emitted as
misleading unconditional Dart statements. Class-dispatch sites are summarized
by recovered selector family and bounded candidate examples. Field names are
promoted only when receiver provenance proves the matching VM class layout;
an offset shared by unrelated classes is never treated as a global field ID.
Raw addresses, pool indexes, decoded
instructions, CFG edges, VM stack maps, PC descriptors, exception handlers,
CodeSourceMap events, and register/stack flow stay in `reports/assembly.s` and
the JSON reports.

Most bodies end in an explicit unresolved operation only when some reachable
statement or exit path remains unproven; branch-free bodies with one proven
machine return render a recovered `return` without a trailing region.
Snapshot and native-pool identities remain
explicit support intrinsics rather than being presented as reconstructed Dart
values. Calls that are structurally recovered but cannot be expressed as a
proven Dart invocation use the readable `aot.invoke` boundary; fuller semantic
and confidence evidence remains in the JSON reports.

With split debug info, reports include original-name provenance, the obfuscated
snapshot name when available, exact ELF symbol ranges, line/column spans, and
DWARF inline call ranges (including range-list DIEs). Without it, serialized
CodeSourceMaps can still recover source lines and inline-stack transitions when
the selected snapshot contains them.

## Snapshot type-graph reconstruction

Clutter reconstructs source-facing types by joining data that the Dart VM
serializes in different phases. The allocation pass assigns object references
and writes unboxed Mint values; the fill pass writes object edges and packed
flags. Retaining both makes it possible to decode required-named-parameter bit
vectors, class IDs, record shapes, and scalar pool entries that are otherwise
lost by a fill-only parser.

The type resolver walks `FunctionType`, `Type`, `TypeArguments`,
`TypeParameter`, `TypeParameters`, `RecordType`, and `FutureOr` objects with
cycle and depth guards. It then joins those types to Function, Class, and Field
declarations. The generated source can consequently recover class modifiers,
generic bounds, `extends`/`with`/`implements` relationships, field modifiers
and declared types, and detailed callable signatures. Synthetic mixin
application classes are recognized from the VM flag and collapsed into a
source-facing `with` clause when the relationship is provable.

This survives ordinary identifier obfuscation because graph topology, class
IDs, type arguments, parameter layout, and modifier bits are runtime metadata
rather than debug names. A matching obfuscation map is still needed to restore
an obfuscated type or parameter token itself. Types and signatures that the
precompiler determined were unnecessary can no longer be recovered; Clutter
falls back to `dynamic` and preserves raw references instead of inventing them.

When split debug info is available but package URIs were obfuscated, Clutter
locates the DWARF source root for `main`, infers a valid application package
from that root, and joins debug functions to snapshot Functions by exact code
address. Only one-to-one library and class-name mappings are promoted. Clutter
then closes uniquely determined per-library class matches and reads the source
mixin name embedded in synthetic DWARF mixin-application symbols. This bridge
carries Class and Field type graphs into source-scoped obfuscated output even
though the debug ELF itself contains no serialized Dart types. Ambiguous
mappings stay raw, and the original token remains in `snapshot_name`.

## Obfuscation-independent string transduction

In addition to ordinary ELF and Dart string objects, Clutter treats serialized
one-byte typed data as a potential string carrier. It explores a bounded
transduction graph over UTF-8, UTF-16, hex, Base64, gzip, zlib, and
single-byte-XOR representations, then emits only results with strong text or
structural evidence. This is independent of function, class, and library names,
so Flutter identifier obfuscation does not disable it.

Each result in `metadata/symbols.json` records its snapshot reference, transform
chain, and confidence. Direct typed-data decoding is distinguished from a
transduced value. The search is static: Clutter never invokes a recovered
decryptor or loads the APK. Per-object input and output are capped at 1 MiB,
transform depth at three, candidate states at twelve, and XOR input at 16 KiB.

This method recovers deterministic values whose representation and required
transform survive in the snapshot. It cannot recover a value that depends on a
server response, device keystore, user input, or an unsupported cipher/key
schedule. A medium-confidence XOR result remains heuristic evidence rather than
proof.

## Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | Success |
| 2 | Invalid CLI usage |
| 3 | Malformed APK, AAB, ELF, manifest, or snapshot |
| 4 | Unsupported ABI, module, or Dart snapshot |
| 5 | Analysis or serialization failure |
| 6 | I/O problem or protected output path |

## Security and responsible use

Treat APK and AAB files as untrusted input. Clutter's default path performs
static parsing, checks archive paths, bounds large entries, and writes through
a staging directory. The optional oracle loads attacker-controlled snapshot
data into a matching Dart VM process, but does not invoke the Dart entry point;
run it on a disposable Android emulator/device. See [SECURITY.md](SECURITY.md)
for the threat model.

Only analyze software you own or are authorized to inspect. Recovered code may
contain confidential strings, API endpoints, or other sensitive material.

## Accuracy notes

- Every recovered fact carries an explicit evidence tier — `proven`,
  `cross_abi_corroborated`, `inferred`, or `speculative` (see
  `src/evidence/tier.rs`). Merging never upgrades a claim, and LLM-assisted
  naming (`RecoveredNameSource::LlmAssisted`) is pinned to `speculative`.
- The physical-body / logical-occurrence graph keeps shared optimized bodies and
  same-address closures as distinct occurrences; nothing overwrites anything at
  the same entry offset. The resolution summary lands in
  `reports/body_graph.json`.
- Cross-ABI consensus (`--cross-abi`) aligns occurrences by owner, lexical
  order, and arity rather than display name. Constants, call topology, and pool
  identities must agree unanimously before a fact is marked
  `cross_abi_corroborated`; disagreements are retained verbatim in
  `reports/cross_abi_consensus.json`, never voted into generated Dart.
- The signature solver recovers parameter shapes from argument descriptors,
  call-site edges, receiver CIDs, and field traffic. Authoritative descriptors
  are proven, agreement is inferred, conflicts stay bounded, and genuinely
  erased signatures remain unknown instead of guessed.
- Dynamic evidence is separate: an instrumented emulator writes a
  `clutter.runtime-trace/v1` document (executed PCs, receiver CIDs, dispatch
  targets, argument descriptors) that `clutter trace <file>` validates against
  the exact snapshot hash. Trace facts rank plausibility in the inferred tier;
  they never promote static claims.
- Snapshot hashes select versioned CID/layout profiles. Compatible unknown patch
  hashes are labeled as such in the manifest.
- The clustered-snapshot parser retains object kinds, references, scalar
  fields, byte payloads, reverse references, arrays, records, contexts, type
  arguments, object pools, Code metadata, exception handlers, and instance
  layout bitmaps in a compact graph. Allocation-pass Mint values are retained
  as typed scalar objects instead of being skipped. `snapshot_evidence.json`
  summarizes this evidence; `--emit-ir` retains the public recovered program
  model.
- Bodies are lifted with a worklist fixpoint over the basic-block CFG: a
  register or stack value survives a branch join only when every predecessor
  carries identical provenance, so recovered call arguments, conditions, and
  returns remain sound while reaching far deeper into real functions than a
  per-block scan.
- The synthetic incoming-argument local (`final args = <dynamic>[…]`) is
  emitted only when some rendered statement actually references it; unused
  bundles are noise relative to the original source and are dropped.
- Statements stranded after a proven machine return inside a straight-line
  region are unreachable code and are not rendered as reachable Dart. Block
  emission stops at any child whose every path ends in a return, so dead
  optimizer tails no longer appear after `return`.
- Member staticness is recovered without debug information by calibrating the
  static bit of `Function::kind_tag_` per snapshot: the bit sits above
  version-dependent recognizer-kind bits, so its position is derived from
  definition-fixed constraints (implicit field accessors) and accepted only
  when exactly one candidate satisfies all of them. Proven instance members
  name their receiver `this`, and own-receiver calls render with Dart's
  implicit-`this` syntax (`setState(…)`, bare property reads) instead of
  passing the receiver as ordinary data.
- Before dataflow, the lifter fuses Dart AOT's machine-only idioms so they
  stop splitting values across branch joins: the stack-overflow guard,
  the Smi/Mint untag diamond (`sbfx`/`tbz`/Mint load — both arms hold the
  same integer), the re-tag overflow check with its Mint-allocation slow
  path, and compressed write-barrier tag tests ahead of runtime-stub calls.
  Untagged Smi payloads keep their source-level expression text, and only
  the lifting stream is filtered; complete decoded instructions stay in
  `reports/assembly.s`. Unreachable fused tails are not emitted as
  unreachable Dart.
- Dart's ARM64/ARM32 calling convention is modeled directly: argument zero
  arrives in a fixed register, remaining arguments are pushed right-to-left
  onto the stack (last at `[SP]`, addressed through FP after the prologue).
  Incoming slots are seeded even when the signature did not survive, and a
  call site reports exactly the register argument the caller itself wrote
  plus its outgoing stack stores. Functions whose signature was tree-shaken
  therefore still recover computed call arguments such as `f(value - 2)`.
- Recovered bodies are restructured into Dart-shaped control flow: `if`/`else`
  regions from branch diamonds, `while` loops from natural loops (with
  negated predicates where the loop body sits on the false edge), nested
  branches and loops re-nested recursively inside loop bodies instead of
  being flattened in address order, resumed walks that structure diamonds
  after a re-joining block stalls the linear pass, explicit `return`
  statements on proven machine returns, and a linear tail for anything the
  CFG cannot structure. Set `CLUTTER_DEBUG_STRUCTURE=1` to dump
  the region tree per function.
- Dart's string-interpolation lowering is recognized end-to-end: an array
  allocation stub, compressed element stores (including spilled arrays and
  derived element pointers), and the `_interpolate` call are rebuilt as a
  single interpolated literal whose unproven parts become explicit
  placeholders.
- Recovered arithmetic, comparison, and operator expressions render as real
  Dart: lifter-built expressions over named registers, numeric literals, and
  field reads are emitted verbatim (`value - 2`, `left ^ right`,
  `a ~/ b`), and recovered operator invocations such as `EdgeVector.+`
  render as infix `receiver + operand`. Without this evidence the
  `aot.unresolvedValue` boundary remains.
- Call sites render as Dart calls: getters/setters become property syntax,
  instance members keep their recovered receiver, allocator + constructor
  stub pairs collapse into one constructor invocation, and positional slots
  zip onto declared named parameters (leading/trailing null dummies dropped)
  when the callee's FunctionType survived tree-shaking. VM-internal helpers
  stay behind `aot.invoke`.
- Canonical tagged constants are folded: ARM64 `null` register values render
  as `null`, and the fixed true/false offsets render as booleans.
- Async machines are recovered on two tiers. Split-debug or VM-oracle names
  for the async stubs (`InitAsyncStub`, `AwaitStub`) become `async` members
  with explicit `await` boundaries; unnamed snapshots fall back to a
  Future-return-type heuristic plus an explanatory comment. Generator
  (`sync*`/`async*`) identity is reported as documentation because recovered
  bodies use `return`, not `yield`.
- Closures with a provable enclosing member — an authoritative VM
  lexical-parent link or containment inside the parent's source-line span —
  render as local functions nested in that parent; identically named
  siblings receive stable `_2`/`_3` suffixes. Unproven closures remain
  ordinary members.
- Canonical snapshot instances are labeled `snapshotInstance(Class)` from the
  class-id table, giving const-constructor rendering and receiver-class
  provenance for field reads through pool-loaded constants.
- Statements stranded in machine regions the structurer cannot reach (async
  state dispatch, table jumps) still surface under an explicit fragment
  comment instead of being dropped.
- Instance-field names still require receiver-class proof. Exact offsets come
  from the VM oracle or split-debug joins; surviving Field declarations fill
  remaining slots using Dart's deterministic layout rules (header, optional
  type-argument slot, declaration-order references, aligned unboxed doubles).
  Classes whose Field objects were tree-shaken fall back to slot-named
  placeholders rather than guessed names.
- FunctionType graphs recover return and parameter types, nested generics,
  nullability, type-parameter bounds, named names, and packed `required`
  flags. Class state bits and type edges recover modifiers, superclasses,
  interfaces, and mixin applications; Field edges recover declared types and
  `static`/`const`/`final`/`late` flags. The coverage report counts these gains
  explicitly.
- Function and library names come from serialized Dart objects. Dart library
  private-key suffixes such as `@123456` are retained as raw evidence but
  removed from human-facing names. When name evidence is missing, address-based
  names are used.
- Serialized typed-data and encoded string carriers are analyzed at their Dart
  object boundaries instead of as arbitrary byte windows. This retains
  snapshot-reference provenance and substantially reduces false candidates
  compared with blind whole-file decoder scans.
- For obfuscated builds, Flutter's split debug ELF restores original function
  names, source libraries and line spans, inline origins, exact function sizes,
  and far more direct-call targets. If package URIs were removed, the main
  library's DWARF source root recovers application scope; exact address joins
  then transfer unambiguous original library/class names onto the raw snapshot
  declaration graph. Clutter validates the ELF ABI and GNU build ID before use.
- Split debug info is not required for analysis. Without a map, Clutter detects
  opaque library tokens—including builds where no application `package:` URI
  survives—and broadens a requested narrow scope to preserve all raw
  instruction-table ranges. With a matching map, library ownership is restored
  before `--scope` is applied.
- A Flutter obfuscation map restores identifiers and declarations that remain
  serialized, including declaration-only classes. It cannot associate a name
  with code whose owner was discarded by AOT compilation.
- Without the VM oracle, `libraries.json` still derives a conservative import
  graph from resolved direct-call evidence: when a recovered function in
  library A calls a named function attributed to library B, B is listed among
  A's imports and type-reference dependencies. These are call-site edges, not
  proof of an exact `import` directive; the oracle's authoritative import
  lists take precedence when available.
- `coverage.json` distinguishes logical function entries from unique physical
  code ranges, so deduplicated tear-offs do not inflate recovered byte and call
  counts. It separately reports code-resolved direct calls and semantically
  named direct calls: an interior or shared-body address can be physically
  exact without proving one Dart declaration. It also reports direct versus
  indirect calls, resolved object-pool calls, call-target scope, CFG
  reachability, internal source maps, stack maps, exception handlers, pool
  loads, recovered fields/conditions, and semantic-lift confidence.
- Object-pool references are annotated using the ABI-specific Dart pool pointer
  (`x27`, `r5`, or `r15`). On ARM32 and ARM64, Clutter propagates affine pool
  aliases through the split-immediate `add derived, PP, #page; ldr value,
  [derived, #remainder]` form used by large object pools; ARM64 shifted
  immediates are included in the byte offset, while ARM32 accounts for the
  tagged `r5` pointer. Provenance is killed at calls, branches, and register
  overwrites. References to serialized `Code` objects are resolved through
  their owning Dart function where possible. Register expressions are
  complexity-bounded, killed at unproven block joins, and invalidated across
  caller-saved call boundaries to avoid stale-value guesses.
- A pool root that is a canonical instance, array, record, or context is
  traversed through value-container edges with strict depth, node, and string
  caps. Descendant strings are emitted as `nestedStrings` evidence while the
  root remains an explicit `snapshotRef`; Clutter does not misrepresent a
  widget/config object as a direct scalar string.
- Ownerless field-initializer Functions are joined back to their Field using
  the retained initializer reference, with an exact private-key-bearing
  `init:<field>` symbol as a unique fallback. This restores the Field's class
  and library before scope filtering without relying on readable identifiers.
- The bounded semantic lifter supports ARM64, ARM32, and x86_64 register
  conventions. ARM32 pool constants and calls reuse the same affine provenance
  evidence; x64 lifts its own idioms (thread-relative stack guards and write
  barriers, `sar` Smi untags, heap-base decompression through the thread,
  `setcc` boolean materialization) with arguments in RDI/RSI/RDX/RCX/R8/R9 and
  right-to-left stack overflow. VM instance layouts are indexed by
  `(library, class, offset)`, and register provenance carries
  constructor/instance class identity through moves and field reads. This
  prevents the same numeric offset in an Array, framework object, and
  application model from being assigned one false field name.
- When a receiver's class is proven but its Field declarations were
  tree-shaken, accesses still surface as explicit low-confidence slot
  placeholders (`slot0b`) instead of disappearing; no member name is invented.
  Per-class allocation stubs (whose Code owner is the Class itself) donate
  their class to freshly allocated receivers so constructor argument and
  field-store evidence chains together.
- Integer semantics are recovered at source level: `sdiv`+`msub` pairs render
  as Dart `%` (with the negative-remainder adjustment diamond fused away),
  truncating division renders `~/`, condition-set instructions materialize
  pending comparisons as `bool`, and Smi tag/untag steps never appear in
  expressions because untagged payloads *are* the Dart integers.
- Boolean values are recovered from Dart's canonical-boolean machine idioms.
  `EmitBoolTest` discriminates `true`/`false` by the object-alignment bit
  (`pointer_tagging.h`), so `tbz`/`tbnz` on that bit (and `test`+`je`/`jne`
  against the same mask on x64) render as the value itself with correct
  polarity instead of an opaque `(x & (1 << 4)) != 0`; canonical constants at
  the fixed null offsets map to `true`/`false`. Branch joins come from the
  immediate post-dominator, so diamonds whose arms re-enter each other no
  longer collapse into linear code. An `if (!moveNext()) {} else { rest }`
  exit test inside a `while (true)` loop is promoted into the loop predicate,
  recovering real `while` conditions for `for-in` loops. Decompressed
  reference loads keep slot-placeholder provenance when Field objects were
  tree-shaken, and comparisons over untracked registers keep their machine
  names so branch structure survives. The code source map's last inline-depth-
  0 line approximates a member's end line, letting closures nest inside their
  proven parent by source-line containment even without debug information.
- The code source map's inline-stack transitions resolve to named callees:
  bodies list the functions the optimizer folded into them, and named record
  types recover their field names from the snapshot graph when retained.
- The compressed class dispatch table is recovered from the snapshot root tail
  without depending on unstable VM root counts. The decoder validates the Code
  cluster identity, repeat/recent encoding, declared length, and exact snapshot
  boundary, then emits run-length target evidence. On ARM64, `x21` indexed-call
  sequences recover selector offsets and candidate selector families. On
  ARM32, affine propagation recognizes both direct `r7 + class_id * 4` loads
  and the large signed row-displacement form materialized as `add` plus `ldr`.
  Sparse surviving names in an otherwise opaque row do not count as a selector
  quorum. Dart's row-displacement table deliberately shares unused slots, so an
  inferred family is never promoted to an exact target unless receiver
  class-ID flow proves one concrete slot.
- CodeSourceMaps use the Dart VM's bounded signed encoding and `-1` initial line
  register. Compressed stack maps decode inline, canonical-table, and
  table-reference forms; PC descriptors and exception-handler tables remain
  attached to their physical code range.
- `declarations.json` includes declarations that survive without a standalone
  code range. That state does not prove whether the compiler inlined, folded,
  deferred, or tree-shook the body.
- A declaration that was fully inlined or tree-shaken has no AOT function range.
  If obfuscation also removed its identifier, neither `libapp.so` nor the split
  debug ELF contains enough evidence to restore that source-only name; a
  matching external obfuscation map can still restore a retained declaration
  token, but not a body that was removed.
- Generated files are designed to be readable and syntactically checkable.
  Evidence intrinsics compile as explicit throwing helpers, but the output is
  not promised to build into an equivalent application.
- Android deferred-component part libraries are statically parsed and indexed
  by ABI, SHA-256, build ID, snapshot symbols, text-symbol count, and instruction
  bytes. Logical function reconstruction currently covers the root
  `libapp.so`; affected outputs carry an explicit warning.
- Cross-ABI reports are corroboration, not a proof of source equivalence.
  Optimizer decisions may legitimately differ by architecture, and opaque
  obfuscated names cannot be aligned safely without the matching map.
- Indirect calls through megamorphic caches, FFI trampolines, runtime stubs, or
  dispatch sites whose receiver class remains dynamic may remain unresolved.
  Clutter reports selector/candidate evidence instead of assigning a
  speculative exact Dart target.
- Assets are copied byte-for-byte and indexed with SHA-256 hashes.

Clutter was independently implemented with reference to the Dart SDK app-snapshot serializer
and the BSD-licensed `unflutter` snapshot grammar. See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
