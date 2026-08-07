#!/usr/bin/env bash
set -euo pipefail

fixture_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
symbols_dir="${1:-$fixture_root/build/obfuscated-symbols}"

mkdir -p "$symbols_dir"
cd "$fixture_root"
flutter test
flutter build apk \
  --release \
  --obfuscate \
  --split-debug-info="$symbols_dir" \
  --target-platform android-arm,android-arm64,android-x64 \
  --no-pub

echo "APK: $fixture_root/build/app/outputs/flutter-apk/app-release.apk"
echo "Symbols: $symbols_dir"
