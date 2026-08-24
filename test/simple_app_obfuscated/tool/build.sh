#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist="$root/dist"
mkdir -p "$dist/symbols"

cd "$root"
flutter build apk \
  --release \
  --obfuscate \
  --split-debug-info="$dist/symbols" \
  --extra-gen-snapshot-options="--save-obfuscation-map=$dist/obfuscation-map.json" \
  --target-platform android-arm,android-arm64,android-x64

cp "$root/build/app/outputs/flutter-apk/app-release.apk" "$dist/simple_app-obfuscated.apk"

echo "APK:              $dist/simple_app-obfuscated.apk"
echo "Split debug info: $dist/symbols"
echo "Obfuscation map:  $dist/obfuscation-map.json"
