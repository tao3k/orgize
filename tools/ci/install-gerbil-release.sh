#!/usr/bin/env bash
set -euo pipefail

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    tag=gerbil-v0.19-2591dcd9b7c6d2c4e9dd8611a17c5b1a5d82bbdb-linux-x86_64-portable-full-single-host-unlimited-patcha379bb244bfa
    sha256=3b345b1af68dfbbc0e73bae64941e4c8910d0ca192d81432ce699c5af385b565 ;;
  Darwin-arm64)
    tag=gerbil-v0.19-2591dcd9b7c6d2c4e9dd8611a17c5b1a5d82bbdb-darwin-aarch64-gcc16-arm64-single-host-unlimited-patche35312b4a951
    sha256=c53d62f26ff382f8956576963edc3eb9193098dc48de78d44b8a3df63636bd18 ;;
  *) echo "unsupported Gerbil release host" >&2; exit 64 ;;
esac

archive="${RUNNER_TEMP:?}/${tag}.tar.gz"
root="${RUNNER_TEMP}/orgize-gerbil-release"
curl --fail --location --retry 3 --output "$archive" \
  "https://github.com/tao3k/gerbil-bazel/releases/download/${tag}/${tag}.tar.gz"
echo "$sha256  $archive" | shasum -a 256 --check
mkdir -p "$root"
tar -xzf "$archive" -C "$root" --strip-components=1
# Downloaded and checksum-verified above; absent from the source checkout.
# shellcheck source=/dev/null
source "$root/activate"
# The activated distribution owns its compiler and platform link inputs.
eval "$("$GERBIL_HOME/bin/gambuild-C")"
command -v "$C_COMPILER" >/dev/null
echo "$GERBIL_PREFIX/bin" >> "${GITHUB_PATH:?}"
{
  echo "GERBIL_PREFIX=$GERBIL_PREFIX"
  echo "GERBIL_HOME=$GERBIL_HOME"
  echo "GERBIL_GSC=$GERBIL_HOME/bin/gsc"
  echo "GERBIL_GCC=$C_COMPILER"
  echo "GAMBOPT=$GAMBOPT"
  echo "GERBIL_PATH=${GITHUB_WORKSPACE:?}/.gerbil-native"
  if [[ "$(uname -s)" == Darwin ]]; then
    # The packaged Gerbil was linked against OpenSSL 3. Select the matching
    # package metadata even when Homebrew exposes OpenSSL 4 by default.
    openssl_pc="$(brew --prefix openssl@3)/lib/pkgconfig"
    echo "PKG_CONFIG_PATH=$openssl_pc${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  fi
} >> "${GITHUB_ENV:?}"
