# Recovery experiment

The fixture was built non-obfuscated with Flutter 3.41.6 / Dart 3.11.4 for
all three supported Android ABIs, then decompiled with Clutter 0.1.0. The main
figures below use `arm64-v8a`.

- APK size: 16,050,088 bytes
- APK SHA-256:
  `49c4564e8b00607b8cdc4c6d6b21ad01424e1198866467d363041849d516ddac`
- Generated Dart: all files are accepted by the Dart parser

## Final recovery

| Evidence | Result |
| --- | ---: |
| Application libraries | 5 |
| Logical AOT functions | 43 |
| Unique machine-code ranges | 41 |
| AOT bytes assigned to unique ranges | 9,640 |
| Surviving declarations | 49 |
| Functions with snapshot signatures | 21 |
| Resolved return / parameter types | 21 / 28 |
| Named parameter names / class graphs / typed fields | 6 / 14 / 1 |
| Direct / indirect call sites | 257 / 8 |
| Direct calls resolved to names | 104 |
| Undecoded function bytes | 0 |
| Expected identities found | 20 / 20 |
| Pragma-retained, uncalled functions found | 2 / 2 |
| Ordinary unreachable functions absent | 1 / 1 |

Recovered function kinds include 23 regular functions, eight closures, six
implicit closures, one constructor, one getter, and one dynamic invocation
forwarder. Two logical entries share machine-code ranges; coverage counts their
physical bytes and call sites only once.

The tear-offs retain especially useful signature evidence:

- `optionalNamed`: `String` return type plus `required String label`,
  `int count`, and `bool enabled`.
- `optionalPositional`: `String` return type plus one `int` fixed parameter and
  optional `int` and `String` parameters. Their positional source names and
  default expressions are intentionally absent from Full AOT metadata.

The direct optimized bodies do not always retain a signature object. Clutter
can associate two such bodies with a matching closure or tear-off, but labels
that evidence as `related_function` rather than presenting it as exact.

## ABI comparison

Every expectation passed and every generated output parsed cleanly on all
three ABIs.

| ABI | Functions / ranges | Signatures | Direct / indirect calls | Resolved calls |
| --- | ---: | ---: | ---: | ---: |
| `arm64-v8a` | 43 / 41 | 21 | 257 / 8 | 104 |
| `armeabi-v7a` | 43 / 41 | 21 | 192 / 8 | 105 |
| `x86_64` | 43 / 41 | 21 | 222 / 43 | 104 |

The call-site count differs because instruction selection and direct versus
indirect dispatch differ by architecture. The recovered Dart function set and
snapshot signature evidence are identical.

## Changes driven by the baseline

The first decompilation of the same initial APK produced 229,558 bytes of Dart
source, mostly because `main.dart` embedded thousands of unrelated global
snapshot strings. It also stopped collecting calls after each function's first
80 machine instructions.

| Same initial APK | Baseline | Improved |
| --- | ---: | ---: |
| Generated Dart bytes | 229,558 | 62,808 |
| `main.dart` bytes | 172,303 | 2,951 |
| Direct call sites retained | 142 | 257 |
| Direct calls resolved to names | 59 | 104 |

Global identifiers and strings now live in `metadata/symbols.json`. Generated
Dart groups members by recovered class owner, uses readable operator/closure
names, resolves surviving return/parameter types, named names, generic bounds,
class relationships, and field modifiers, labels VM function kinds, and
summarizes machine-code evidence. Positional names and optional defaults that
Full AOT removes remain explicit placeholders. Separate declaration, function,
and call-graph reports preserve detail without flooding pseudocode. Assembly
comments remain bounded, but calls and branches are analyzed across the entire
decoded function.

## Recovery limits observed

- The sealed arithmetic classes survive as identities, but their tiny `apply`
  methods are inlined and have no standalone body.
- Records, patterns, generics, exceptions, and async lowering leave named
  functions and machine-code evidence; Clutter cannot reconstruct their exact
  original Dart statements.
- `async` and `async*` are compiler state machines in AOT. Their source names
  survive here, while original suspension structure and local names do not.
- Capturing closures survive as separate functions, but captured variable names
  do not.
- Tree-shaken code is absent and cannot be recovered from the release artifact.
- Indirect dispatch through registers is reported separately and is never
  claimed as a resolved direct call.
