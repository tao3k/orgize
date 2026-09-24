#!/usr/bin/env bash
set -euo pipefail

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    tag=gerbil-v0.19-2591dcd9b7c6d2c4e9dd8611a17c5b1a5d82bbdb-linux-x86_64-portable-full-single-host-unlimited-patchf5cedd8168cb
    sha256=9844ba362fdf6f1c5e8a4411e462a466d52dee0d541f8e84f61091cec0e5e740 ;;
  Darwin-arm64)
    tag=gerbil-v0.19-2591dcd9b7c6d2c4e9dd8611a17c5b1a5d82bbdb-darwin-aarch64-gcc16-arm64-aot-tools-single-host-unlimited-patchf5cedd8168cb
    sha256=8d9c88434aed6301eaebb719bf52472f05c2368eafc630e8a9f1d67cc9895a21 ;;
  *) echo "unsupported Gerbil release host" >&2; exit 64 ;;
esac

archive="${RUNNER_TEMP:?}/${tag}.tar.gz"
root="${RUNNER_TEMP}/orgize-gerbil-release"
curl --fail --location --retry 3 --output "$archive" \
  "https://github.com/tao3k/gerbil-bazel/releases/download/${tag}/${tag}.tar.gz"
echo "$sha256  $archive" | shasum -a 256 --check
mkdir -p "$root"
tar -xzf "$archive" -C "$root" --strip-components=1
source "$root/activate"
echo "$GERBIL_PREFIX/bin" >> "${GITHUB_PATH:?}"
{
  echo "GERBIL_PREFIX=$GERBIL_PREFIX"
  echo "GERBIL_HOME=$GERBIL_HOME"
  echo "GAMBOPT=$GAMBOPT"
  echo "GERBIL_PATH=${GITHUB_WORKSPACE:?}/.gerbil-native"
} >> "${GITHUB_ENV:?}"
