#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 4 ]]; then
  echo "usage: $0 DART_SDK_SOURCE ANDROID_NDK_ROOT WORK_ROOT DART_COMMIT [arm|arm64c|x64c ...]" >&2
  exit 2
fi

dart_sdk_source=$(realpath "$1")
android_ndk_root=$(realpath "$2")
work_root_input=$3
dart_commit=$4
shift 4

mkdir -p "$work_root_input"
work_root=$(realpath "$work_root_input")
script_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
sdk_root="$work_root/sdk"
depot_tools_root="$work_root/depot_tools"
patch_path="$script_root/dart-sdk-3.11.4.patch"

if [[ ! -d "$dart_sdk_source/.git" ]]; then
  echo "not a Dart SDK git checkout: $dart_sdk_source" >&2
  exit 2
fi
if [[ ! -x "$android_ndk_root/toolchains/llvm/prebuilt/linux-x86_64/bin/clang++" ]]; then
  echo "Android NDK toolchain not found below: $android_ndk_root" >&2
  exit 2
fi
if ! git -C "$dart_sdk_source" cat-file -e "$dart_commit^{commit}" 2>/dev/null; then
  echo "Dart commit is not available in the source checkout: $dart_commit" >&2
  exit 2
fi
if [[ "$work_root" == "$dart_sdk_source" || "$work_root" == "$dart_sdk_source"/* ]]; then
  echo "work root must be outside the source checkout" >&2
  exit 2
fi

source_head_before=$(git -C "$dart_sdk_source" rev-parse HEAD)
source_status_before=$(git -C "$dart_sdk_source" status --porcelain=v1)

if [[ ! -d "$depot_tools_root/.git" ]]; then
  if [[ -e "$depot_tools_root" ]]; then
    echo "existing depot_tools path is not a git checkout: $depot_tools_root" >&2
    exit 3
  fi
  git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git \
    "$depot_tools_root"
fi

if [[ ! -d "$sdk_root/.git" ]]; then
  if [[ -e "$sdk_root" ]]; then
    echo "existing SDK work path is not a git checkout: $sdk_root" >&2
    exit 3
  fi
  git clone --shared --no-checkout "$dart_sdk_source" "$sdk_root"
  git -C "$sdk_root" checkout --detach "$dart_commit"
else
  current_commit=$(git -C "$sdk_root" rev-parse HEAD)
  if [[ "$current_commit" != "$dart_commit" ]]; then
    if [[ -n $(git -C "$sdk_root" status --porcelain=v1 --untracked-files=no) ]]; then
      echo "SDK work checkout has tracked changes at a different commit: $sdk_root" >&2
      exit 3
    fi
    git -C "$sdk_root" checkout --detach "$dart_commit"
  fi
fi

tools_ready=true
for tool in \
  "$sdk_root/buildtools/gn" \
  "$sdk_root/buildtools/ninja/ninja" \
  "$sdk_root/tools/sdks/dart-sdk/bin/dart"; do
  if [[ ! -x "$tool" ]]; then
    tools_ready=false
  fi
done

if [[ "$tools_ready" != true ]]; then
  if git -C "$sdk_root" apply --check --reverse "$patch_path" 2>/dev/null; then
    git -C "$sdk_root" apply --reverse "$patch_path"
  fi
  if [[ -n $(git -C "$sdk_root" status --porcelain=v1 --untracked-files=no) ]]; then
    echo "SDK work checkout has changes unrelated to the Clutter analyzer patch" >&2
    exit 3
  fi

  (
    cd "$work_root"
    DEPOT_TOOLS_UPDATE=0 "$depot_tools_root/gclient" config \
      --name sdk "file://$dart_sdk_source"
    DEPOT_TOOLS_UPDATE=0 "$depot_tools_root/gclient" sync \
      --revision "sdk@$dart_commit" \
      --no-history
  )
fi

architectures=("$@")
if [[ ${#architectures[@]} -eq 0 ]]; then
  architectures=(arm arm64c x64c)
fi

"$script_root/build-analyzers.sh" \
  "$sdk_root" \
  "$android_ndk_root" \
  "${architectures[@]}"

source_head_after=$(git -C "$dart_sdk_source" rev-parse HEAD)
source_status_after=$(git -C "$dart_sdk_source" status --porcelain=v1)
if [[ "$source_head_before" != "$source_head_after" || "$source_status_before" != "$source_status_after" ]]; then
  echo "source Dart SDK checkout changed while building analyzers" >&2
  exit 4
fi

echo
echo "Analyzer identities:"
for architecture in "${architectures[@]}"; do
  case "$architecture" in
    arm) output=ProductAndroidARM ;;
    arm64c) output=ProductAndroidARM64C ;;
    x64c) output=ProductAndroidX64C ;;
    *)
      echo "unsupported analyzer architecture: $architecture" >&2
      exit 2
      ;;
  esac
  analyzer="$sdk_root/out/$output/analyze_snapshot"
  printf '  %s  %s\n' "$(sha256sum "$analyzer" | cut -d' ' -f1)" "$analyzer"
done
printf 'Dart commit: %s\n' "$(git -C "$sdk_root" rev-parse HEAD)"
printf 'Disposable SDK: %s\n' "$sdk_root"
