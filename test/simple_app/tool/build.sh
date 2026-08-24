#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist="$root/dist"
mkdir -p "$dist"

cd "$root"
flutter build apk \
  --release \
  --target-platform android-arm,android-arm64,android-x64

cp "$root/build/app/outputs/flutter-apk/app-release.apk" "$dist/simple_app.apk"

echo "APK: $dist/simple_app.apk"
