#!/usr/bin/env bash
# Provision the arm32 analyzer host environment (QEMU + armhf glibc sysroot).
#
# Why this exists: Flutter APKs that ship only `armeabi-v7a` need an arm32
# `analyze_snapshot` to produce VM-oracle evidence, but there is no longer any
# way to *run* an Android arm32 binary on a workstation --
#
#   * the modern Android emulator refuses `arm` outright
#     ("CPU Architecture 'arm' is not supported by the QEMU2 emulator");
#   * arm64 emulators translate arm64 only -- no /system/bin/linker, no
#     /system/lib, so 32-bit Android binaries cannot load;
#   * the newest arm32 system images are API 24/25 (2016) whose bionic both
#     rejects NDK r28's DT_FLAGS_1=DF_1_PIE and deadlocks the Dart VM's
#     thread start-up under QEMU (both threads park in futex_do_wait).
#
# The combination that does work: build the analyzer for **Linux/ARM** (glibc,
# no bionic) while forcing the snapshot-compatibility label back to `android`
# via `--gn-args 'dart_target_os_name_override="android"'`, then run it under
# qemu-arm against an armhf glibc sysroot. See build-arm32-analyzer.sh.
#
# Everything is installed under the given prefix; no root required.
set -euo pipefail

prefix=${1:-$HOME/.cache/clutter/arm32-host}
qemu_url=${CLUTTER_QEMU_DEB_URL:-https://deb.debian.org/debian/pool/main/q/qemu/qemu-user_11.1.0+ds-2_amd64.deb}
libc_url=${CLUTTER_LIBC_DEB_URL:-https://deb.debian.org/debian/pool/main/g/glibc/libc6_2.41-12+deb13u4_armhf.deb}

mkdir -p "$prefix"
prefix=$(cd "$prefix" && pwd)
qemu_dir="$prefix/qemu"
sysroot="$prefix/armhf-sysroot"

for tool in curl ar tar; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "required tool missing: $tool" >&2
    exit 2
  }
done

if [[ ! -x "$qemu_dir/usr/bin/qemu-arm" ]]; then
  echo "fetching qemu-user ..."
  rm -rf "$qemu_dir"
  mkdir -p "$qemu_dir"
  (
    cd "$qemu_dir"
    curl -fsSL -o qemu.deb "$qemu_url"
    ar x qemu.deb
    tar xf data.tar.xz
    rm -f qemu.deb data.tar.* control.tar.* debian-binary
  )
fi

if [[ ! -e "$sysroot/lib/ld-linux-armhf.so.3" ]]; then
  echo "fetching armhf glibc sysroot ..."
  rm -rf "$sysroot"
  mkdir -p "$sysroot"
  (
    cd "$sysroot"
    curl -fsSL -o libc6.deb "$libc_url"
    ar x libc6.deb
    tar xf data.tar.xz
    rm -f libc6.deb data.tar.* control.tar.* debian-binary
    # Debian ships the loader under /usr/lib; qemu -L looks for /lib.
    mkdir -p lib
    ln -sfn ../usr/lib/arm-linux-gnueabihf lib/arm-linux-gnueabihf
    ln -sfn ../usr/lib/ld-linux-armhf.so.3 lib/ld-linux-armhf.so.3
  )
fi

qemu_bin="$qemu_dir/usr/bin/qemu-arm"
"$qemu_bin" --version | head -1

cat <<EOF

arm32 analyzer host ready.

  CLUTTER_QEMU_ARM=$qemu_bin
  CLUTTER_ARM_SYSROOT=$sysroot

Build the analyzer with:
  vm-oracle/build-arm32-analyzer.sh <work-root> <dart-commit>

Then point clutter at the wrapper:
  clutter vm-oracle <apk> --abi armeabi-v7a \\
    --analyzer \$PWD/vm-oracle/run-arm-analyzer.sh --out <doc>.vm.json
EOF
