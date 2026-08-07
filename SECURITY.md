# Security

## Threat model

Clutter treats every APK, AAB, manifest, ELF, snapshot, symbol, and asset name as
attacker-controlled. It does not execute Dart code, load native libraries from
the input, invoke Android tooling, or deserialize through a Flutter runtime
during `inspect` or the default static `decompile` workflow.

`vm-oracle` is an explicit exception: it invokes a separately built Dart
`analyze_snapshot` process and can transfer that process plus `libapp.so` to an
Android target through `adb`. The analyzer creates and shuts down an isolate
without resolving or invoking `main`, but its native VM deserializer still
processes attacker-controlled snapshot data. Use a disposable, non-privileged
emulator/device with no sensitive data. Clutter uses a unique directory below
`/data/local/tmp`, downloads the JSON result, and removes that directory after
a successful run.

The parser applies these controls:

- archive paths are rejected if absolute or traversing;
- native libraries and manifests have explicit read limits;
- snapshot counts, offsets, ranges, and variable-length integers are bounded;
- string transduction caps carrier size, decoded output, search depth, state
  count, and decompression output before accepting a candidate;
- ELF virtual ranges must be backed by file segments;
- generated source paths are sanitized;
- assets are streamed instead of buffered in memory;
- output is assembled in a staging directory and moved into place only after
  successful completion;
- `--replace` refuses any directory without a Clutter manifest and preserves the
  previous result as a backup.

Large valid archives can still consume substantial CPU, disk, and memory. Run
analysis with normal user privileges, in a workspace with a disk quota when
processing untrusted samples, and avoid opening recovered secrets in shared
systems.

## Reporting a vulnerability

Do not attach a proprietary APK or recovered secrets to a public report. Provide
the smallest synthetic reproducer possible, the Clutter version, target ABI,
Dart version or snapshot hash, and the observed impact.
