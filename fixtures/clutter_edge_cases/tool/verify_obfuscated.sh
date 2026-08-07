#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
fixture_root="$repository_root/fixtures/clutter_edge_cases"
apk="$fixture_root/build/app/outputs/flutter-apk/app-release.apk"
symbols_dir="$fixture_root/build/obfuscated-symbols"
results_root="${1:-$fixture_root/build/obfuscated-recovery}"
expectations="$fixture_root/tool/obfuscated_recovery_expectations.json"

"$repository_root/target/release/clutter" version >/dev/null

for abi in armeabi-v7a arm64-v8a x86_64; do
  output="$results_root/$abi"
  "$repository_root/target/release/clutter" decompile "$apk" \
    --abi "$abi" \
    --scope app \
    --symbols "$symbols_dir" \
    --emit-ir \
    --no-assets \
    --replace \
    --out "$output"
  python3 "$fixture_root/tool/evaluate_recovery.py" \
    "$output" \
    --expectations "$expectations"
done
