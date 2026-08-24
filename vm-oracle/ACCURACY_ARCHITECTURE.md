# Accuracy architecture

## Decision

Clutter will recover into immutable, source-native evidence and project to
`RecoveredProgram` only after explicit resolution. Phase one is intentionally
smaller than that final architecture:

1. bind an oracle to the exact archive, payload, ABI, snapshot regions, Dart
   revision, and analyzer binary;
2. model physical code bodies separately from logical Dart function
   occurrences;
3. preserve all exact-offset candidates and every range disagreement;
4. adapt the resolved view into the current renderer;
5. prove the change against the existing nine-variant accuracy matrix.

This keeps decompilation static. The optional VM analyzer deserializes a
snapshot offline and never invokes the application or `main`. Decompilation
without an oracle remains supported.

## Why this is the next lever

The current parsers already recover more than the output model can retain.
`ParseResult` contains the static object graph, reverse references, object
pools, and code objects, but that graph is discarded after function recovery.
The VM analyzer emits another rich graph, which is projected into a smaller
`LoadedVmOracle`. Both sources then mutate `RecoveredProgram`, which currently
acts as evidence store, conflict resolver, semantic IR, and render model.

That shape causes concrete accuracy loss:

- a physical code body can overwrite or erase other logical closures at the
  same entry offset;
- VM and static code-size claims are neither reconciled nor reported;
- typed object-pool identities become strings before semantic lifting;
- only the first recovered object pool is used;
- per-field provenance and disagreement disappear during mutation;
- cross-ABI maps collapse duplicate function occurrences;
- ABI-specific instruction and argument-recovery artifacts can look like
  semantic consensus.

More naming and instruction heuristics on top of this model would amplify
those losses. Exact VM evidence and retained ambiguity can improve both
unobfuscated and obfuscated recovery without inventing erased source text.

## Caller view

The target call path keeps the current CLI and renderer stable while moving
evidence ownership behind a narrow API.

```rust
let artifact = Artifact::open(&args.input)?;
let selected = artifact.select_payload(&args.module, args.abi)?;

let subject = ArtifactSubject::observe(&artifact, &selected)?;
let static_evidence = StaticEvidenceAdapter::recover(&subject, &selected)?;

let oracle = match args.vm_oracle.as_deref() {
    Some(path) => Some(
        OracleEnvelope::load(path)?
            .bind_exact(&subject)?
            .into_evidence()?,
    ),
    None => None,
};

let body_graph = BodyResolver::resolve(&static_evidence, oracle.as_ref())?;
let report = body_graph.report();
let program = LegacyProjection::project(body_graph, args.scope)?;
output::write(&program, &report, &args.out)?;
```

An unbound oracle cannot contribute facts. Binding failure happens before any
library, declaration, function, pool, or code claim reaches resolution.

## Identities and claims

Offsets are not interchangeable integers. The implementation should use
coordinate-specific newtypes and source-scoped object IDs.

```rust
struct ArchiveOffset(u64);
struct PayloadOffset(u64);
struct IsolateInstructionOffset(u64);

struct EvidenceSourceId(u32);
struct PhysicalBodyId(u32);
struct FunctionOccurrenceId(u32);
struct BodyBindingId(u32);

enum SnapshotPartition {
    Vm,
    Isolate,
}

struct StaticObjectId {
    source: EvidenceSourceId,
    partition: SnapshotPartition,
    reference: i32,
}

struct OracleObjectId {
    source: EvidenceSourceId,
    object: u64,
}
```

`PhysicalBodyId` is derived from the subject, ABI, and authoritative static
entry offset. Size is a claim about a body, never part of its identity. A
larger oracle size must not widen the bytes selected for disassembly.

```rust
struct PhysicalBody {
    id: PhysicalBodyId,
    entry: IsolateInstructionOffset,
    static_extent: ByteExtent,
    static_bytes_sha256: Sha256,
}

struct FunctionOccurrence {
    id: FunctionOccurrenceId,
    source_object: EvidenceObjectId,
    owner: Vec<Claim<OwnerId>>,
    name: Vec<Claim<String>>,
    signature: Vec<Claim<FunctionSignature>>,
}

struct BodyBinding {
    id: BodyBindingId,
    body: PhysicalBodyId,
    function: FunctionOccurrenceId,
    relation: RangeRelation,
    evidence: Vec<EvidenceRef>,
}
```

The body-to-function relationship is many-to-many. Shared optimized bodies,
closures with duplicate display names, and unattributed code all remain
representable.

## Exact oracle subject

The oracle envelope uses a non-circular outer format. `document_sha256` hashes
the raw analyzer document bytes stored beside the manifest. The manifest does
not hash an envelope containing itself.

```rust
struct ArtifactSubject {
    archive_sha256: Sha256,
    archive_size: u64,
    archive_format: ArchiveFormat,
    module: String,
    abi: Abi,
    payload_member: String,
    payload_sha256: Sha256,
    payload_size: u64,
    vm_data: RegionIdentity,
    vm_instructions: RegionIdentity,
    isolate_data: RegionIdentity,
    isolate_instructions: RegionIdentity,
    snapshot_hash: String,
    pointer_layout: PointerLayout,
}

struct OracleManifest {
    schema: u32,
    subject: ArtifactSubject,
    document_sha256: Sha256,
    analyzer_sha256: Sha256,
    analyzer_schema: u32,
    dart_version: String,
    dart_commit: String,
    target_arch: String,
}
```

The binder compares every field and returns a field-by-field mismatch. There
is no fuzzy or compatible mode. Moving byte-identical files is allowed because
filesystem paths are not identity fields.

Oracle generation is convergent. It uses a disposable checkout at the exact
Dart commit, syncs dependencies before patching, and applies all patches and
NDK links only there. An existing output with the same subject and analyzer
digest is reused or reproduced byte-for-byte. A different subject requires a
different output path or explicit replacement.

## Range reconciliation

An exact start address is useful physical evidence, but it is not sufficient
for semantic promotion when extents disagree.

```rust
enum RangeRelation {
    Exact,
    SameStartDifferentEnd {
        static_end: IsolateInstructionOffset,
        oracle_end: IsolateInstructionOffset,
    },
    OracleContainedByStatic,
    StaticContainedByOracle,
    Overlap,
    Disjoint,
}
```

An oracle binding can support semantic promotion only when:

- its subject is bound exactly;
- entry offsets match in the same coordinate system;
- ranges are `Exact`;
- body-byte hashes match when both sources provide bytes;
- the logical occurrence is unique or ambiguity is retained explicitly.

`SameStartDifferentEnd` still creates an audited physical candidate. It never
silently changes the static decode range or selects a semantic owner.

## Cross-ABI rules

Cross-ABI analysis consumes resolved occurrence multisets. Names can narrow a
candidate partition, but they are not unique map keys. Matching uses proven
owner, signature, parent occurrence, lexical position, and a deliberately
small ABI-neutral semantic fingerprint. Equal optima remain ambiguous.

Raw instruction counts, call counts, branch counts, register names, stack
slots, and unproven argument vectors are excluded from consensus. For example,
the fixture's `formatPrice` argument vectors differ across ARM64, ARM32, and
x86_64. That disagreement must be reported, not voted into generated Dart.

```rust
enum ArityEvidence {
    Proven { positional: u16, named: Vec<String> },
    Bounded { minimum: u16, maximum: u16 },
    Conflicted(Vec<ArityClaim>),
    Unknown,
}
```

## Module migration

Phase one should add only the following ownership boundaries:

```text
src/evidence/
  subject.rs       exact archive, payload, region, analyzer identities
  oracle.rs        envelope loading and UnboundOracle -> BoundOracle
  body.rs          physical bodies, occurrences, bindings, range relations
src/resolution/
  body.rs          exact joins, extent conflicts, borrowed ResolutionReport
src/projection/
  legacy.rs        temporary adapter to RecoveredProgram
```

The current snapshot and oracle parsers get adapters at their lossless seams.
The generic provenance graph, semantic derivation ledger, typed pool graph,
and full renderer migration are later phases. They should not be built before
the first vertical slice improves the real corpus.

## Acceptance contract

The release gate is the existing evaluator without `--allow-partial`:

```bash
python3 test/tool/evaluate_accuracy.py \
  --expectations test/accuracy_expectations.json \
  --run-root target/accuracy-runs
```

It must cover plain, obfuscated-with-map, and obfuscated-raw inputs on
arm64-v8a, armeabi-v7a, and x86_64. The evidence checker must additionally
prove:

- every oracle function with code is bound or explicitly unbound;
- shared-body function occurrence counts never shrink during projection;
- every size disagreement appears in the resolution report;
- static-only results remain available and do not regress;
- accepted ABI-neutral semantics agree across aligned occurrences;
- disputed argument vectors, including `formatPrice`, remain unresolved.

Negative tests must reject the oracle before import when swapping ABIs,
swapping plain and obfuscated artifacts, modifying only the outer archive,
modifying the selected payload or a snapshot region, or changing analyzer
schema and target metadata.

## Later accuracy work

Once phase one is proven, the retained evidence unlocks higher-value recovery:

1. retain the full static and VM object graphs through output;
2. attach every object pool to its owning Code object and preserve typed pool
   targets;
3. derive fields only from receiver-class provenance and VM layout evidence;
4. recover closure parentage and call arity from authoritative Function and
   FunctionType objects;
5. use VM code sizes as conflict evidence and measurement data, not automatic
   disassembly authority;
6. align occurrence multisets across ABIs and accept only portable semantic
   facts;
7. analyze deferred loading units as separate payload subjects;
8. add versioned analyzer patches and manifests keyed by snapshot hash and
   exact Dart commit;
9. emit unresolved alternatives and provenance in IR and reports so future
   rules can be evaluated without reparsing the APK.

The governing rule is that loaders append facts, resolvers retain rejected
claims, semantics derive from typed identities, and only projection may throw
information away.

## Implementation status (this wave)

Phase one is implemented and the following modules exist:

| Module | Responsibility |
| --- | --- |
| `src/evidence/body.rs` | physical bodies, occurrences, many-to-many bindings; `build()` runs on every decompile and its report lands in `reports/body_graph.json` |
| `src/evidence/consensus.rs` | ABI-neutral occurrence alignment over constants/call topology/pool identities; only unanimous agreement promotes to `CrossAbiCorroborated`; disputes are serialized verbatim in the IR and `reports/cross_abi_consensus.json` |
| `src/evidence/signature_solver.rs` | constraint solver over argument descriptors, call-site edges, receivers; authoritative descriptors are `Proven`, agreement is `Inferred`, conflicts stay `Bounded`, erasure stays `Unknown`/`Speculative` |
| `src/evidence/runtime_trace.rs` | dynamic-evidence schema (`clutter.runtime-trace/v1`) with snapshot-hash gating; refinements never touch static facts — surfaced via `clutter trace <file>` |
| `src/evidence/tier.rs` | explicit strength ordering so merging can never upgrade a claim; LLM-assisted naming is pinned to `Speculative` (`RecoveredNameSource::LlmAssisted`) |

Analyzer schema v5 (`vm-oracle/dart-sdk-3.11.4.patch`, verified against the
pinned Dart checkout) emits static-call targets from the global object pool,
dispatch-table metadata with selector→Code entries, populated CID runs,
per-class unboxed-field bitmaps, per-function PC-descriptor histograms, and
full argument descriptors.

`test/tool/compiler_lab.sh` is the differential compiler laboratory: it
generates one-construct Dart cases, builds them across ABIs × obfuscation
modes × installed SDK versions, decompiles every artifact, mines cross-variant
lowering templates into `templates/<case>.json`, and pins stable recovery
facts as CI fixtures under `fixtures/regressions.json`.
