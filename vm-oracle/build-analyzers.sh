#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 DART_SDK_ROOT ANDROID_NDK_ROOT [arm|arm64c|x64c ...]" >&2
  exit 2
fi

dart_sdk_root=$(realpath "$1")
android_ndk_root=$(realpath "$2")
shift 2
script_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
patch_path="$script_root/dart-sdk-3.11.4.patch"
# Revision-coupled patch selection, mirroring build-exact-analyzers.sh.
# `CLUTTER_ANALYZER_PATCH` pins a variant; otherwise probe every shipped
# patch against this checkout and use the one that applies.
if [[ -n "${CLUTTER_ANALYZER_PATCH:-}" ]]; then
  patch_path=$(realpath "$CLUTTER_ANALYZER_PATCH")
else
  for candidate in "$script_root"/dart-sdk-*.patch; do
    [[ -e "$candidate" ]] || continue
    if git -C "$dart_sdk_root" apply --check "$candidate" 2>/dev/null ||
      git -C "$dart_sdk_root" apply --check --reverse "$candidate" 2>/dev/null; then
      patch_path="$candidate"
      break
    fi
  done
fi
echo "using analyzer patch: $(basename "$patch_path")"

if [[ ! -f "$dart_sdk_root/runtime/vm/analyze_snapshot_api_impl.cc" ]]; then
  echo "not a Dart SDK source checkout: $dart_sdk_root" >&2
  exit 2
fi
if [[ ! -x "$android_ndk_root/toolchains/llvm/prebuilt/linux-x86_64/bin/clang++" ]]; then
  echo "Android NDK toolchain not found below: $android_ndk_root" >&2
  exit 2
fi

if git -C "$dart_sdk_root" apply --check --reverse "$patch_path" 2>/dev/null; then
  echo "Clutter VM-oracle patch is already applied"
elif git -C "$dart_sdk_root" apply --check "$patch_path"; then
  git -C "$dart_sdk_root" apply "$patch_path"
else
  echo "the VM-oracle patch does not apply cleanly to this Dart checkout" >&2
  exit 3
fi

mkdir -p "$dart_sdk_root/third_party/android_tools"
ndk_link="$dart_sdk_root/third_party/android_tools/ndk"
if [[ -L "$ndk_link" ]]; then
  if [[ $(realpath "$ndk_link") != "$android_ndk_root" ]]; then
    echo "existing NDK link points elsewhere: $ndk_link" >&2
    exit 3
  fi
elif [[ -e "$ndk_link" ]]; then
  echo "existing non-link NDK directory left unchanged: $ndk_link"
else
  ln -s "$android_ndk_root" "$ndk_link"
fi

architectures=("$@")
if [[ ${#architectures[@]} -eq 0 ]]; then
  architectures=(arm arm64c x64c)
fi

for architecture in "${architectures[@]}"; do
  case "$architecture" in
    arm | arm64c | x64c) ;;
    *)
      echo "unsupported analyzer architecture: $architecture" >&2
      exit 2
      ;;
  esac
  (
    cd "$dart_sdk_root"
    python3 tools/build.py \
      --mode=product \
      --arch="$architecture" \
      --os=android \
      --no-rbe \
      analyze_snapshot
  )
done

echo
echo "Built analyzers:"
for architecture in "${architectures[@]}"; do
  case "$architecture" in
    arm) output=ProductAndroidARM ;;
    arm64c) output=ProductAndroidARM64C ;;
    x64c) output=ProductAndroidX64C ;;
  esac
  echo "  $dart_sdk_root/out/$output/analyze_snapshot"
done
