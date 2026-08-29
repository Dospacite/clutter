#!/usr/bin/env bash
# Run a Linux/ARM analyze_snapshot under QEMU user-mode emulation.
#
# Android dropped 32-bit ARM emulator support, so an Android-hosted arm32
# analyzer can no longer be executed anywhere: the modern emulator refuses the
# architecture, and old API-24/25 bionic deadlocks the Dart VM's thread
# start-up under QEMU. The workable combination is a *Linux*-hosted arm32
# analyzer (glibc, no bionic) built with
# `--gn-args 'dart_target_os_name_override="android"'` so its snapshot
# feature string still reads `android` and matches the APK.
#
# Configure with:
#   CLUTTER_QEMU_ARM   path to a qemu-arm binary (default: qemu-arm on PATH)
#   CLUTTER_ARM_SYSROOT path to an armhf glibc sysroot (default: /tmp/armhf-sysroot)
#   CLUTTER_ARM_ANALYZER path to the Linux/ARM analyze_snapshot
#
# All arguments are forwarded verbatim to the analyzer.
set -euo pipefail

qemu=${CLUTTER_QEMU_ARM:-qemu-arm}
sysroot=${CLUTTER_ARM_SYSROOT:-/tmp/armhf-sysroot}
analyzer=${CLUTTER_ARM_ANALYZER:-}

if [[ -z "$analyzer" ]]; then
  echo "set CLUTTER_ARM_ANALYZER to the Linux/ARM analyze_snapshot" >&2
  exit 2
fi
if ! command -v "$qemu" >/dev/null 2>&1 && [[ ! -x "$qemu" ]]; then
  echo "qemu-arm not found: $qemu (set CLUTTER_QEMU_ARM)" >&2
  exit 2
fi
if [[ ! -e "$sysroot/lib/ld-linux-armhf.so.3" ]]; then
  echo "armhf sysroot missing its loader: $sysroot (set CLUTTER_ARM_SYSROOT)" >&2
  exit 2
fi

exec "$qemu" -L "$sysroot" "$analyzer" "$@"
