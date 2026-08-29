#!/usr/bin/env bash
# Build a Linux/ARM analyze_snapshot that accepts ANDROID arm32 snapshots.
#
# The snapshot feature string embeds the target OS name and must match the
# APK byte-for-byte, so a stock Linux build is rejected with:
#
#   the snapshot requires '... arm android ...'
#   but the VM has     '... arm linux   ...'
#
# `dart_target_os_name_override="android"` (added by the Clutter analyzer
# patch) sets DART_TARGET_OS_ANDROID for the snapshot-compatibility label and
# the OS-conditional VM layout while the binary itself stays glibc/Linux, so
# it runs under qemu-arm with no Android device or bionic sysroot involved.
#
# usage: build-arm32-analyzer.sh DART_SDK_SOURCE WORK_ROOT DART_COMMIT
set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "usage: $0 DART_SDK_SOURCE WORK_ROOT DART_COMMIT" >&2
  exit 2
fi

dart_sdk_source=$(realpath "$1")
work_root_input=$2
dart_commit=$3
script_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

mkdir -p "$work_root_input"
work_root=$(realpath "$work_root_input")
sdk_root="$work_root/sdk"

if [[ ! -d "$sdk_root/.git" ]]; then
  echo "no synced SDK at $sdk_root" >&2
  echo "run build-exact-analyzers.sh first: it clones, syncs DEPS and patches" >&2
  exit 2
fi

current=$(git -C "$sdk_root" rev-parse HEAD)
expected=$(git -C "$dart_sdk_source" rev-parse "$dart_commit^{commit}")
if [[ "$current" != "$expected" ]]; then
  echo "SDK work checkout is at $current, expected $expected" >&2
  exit 3
fi

(
  cd "$sdk_root"
  python3 tools/build.py \
    --mode=product \
    --arch=arm \
    --os=linux \
    --no-rbe \
    --gn-args 'dart_target_os_name_override="android"' \
    analyze_snapshot
)

analyzer="$sdk_root/out/ProductXARM/analyze_snapshot"
if [[ ! -x "$analyzer" ]]; then
  echo "build did not produce $analyzer" >&2
  exit 4
fi

echo
echo "Linux/ARM analyzer (android snapshot target):"
printf '  %s  %s\n' "$(sha256sum "$analyzer" | cut -d' ' -f1)" "$analyzer"
echo
echo "export CLUTTER_ARM_ANALYZER=$analyzer"
