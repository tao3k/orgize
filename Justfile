set dotenv-load := false
set shell := ["bash", "-euo", "pipefail", "-c"]
host_os := os()
scheme_home := env_var_or_default("GERBIL_HOME", "")
# Keep the fixed heap envelope while honoring an explicitly selected SDK.
scheme_gambopt := "max-heap=1G,debug=q" + if scheme_home == "" { "" } else { ",~~bin=" + scheme_home + "/bin,~~lib=" + scheme_home + "/lib,~~include=" + scheme_home + "/include" }
lib_ext := if host_os == "macos" { "dylib" } else { "so" }
scheme_env := if host_os == "macos" { "env -u SDKROOT CC=/usr/bin/cc GERBIL_GCC=/usr/bin/cc" } else { "env" }
native_env := scheme_env + (if host_os == "macos" { " CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/cc" } else { "" })
native_link_args := if host_os == "macos" { "-C linker=/usr/bin/cc -C link-arg=-Wl,-ld_classic" } else { "" }
native_rust_flags := if host_os == "macos" { "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS=\"" + native_link_args + "\"" } else { "" }
rust_linker := if host_os == "macos" { "-C linker=/usr/bin/cc" } else { "" }
# Gerbil's loadable module resolves OS functions in its existing host process.
clock_linker := if host_os == "macos" { "-Wl,-undefined,dynamic_lookup" } else { "" }

repair_command := if host_os == "macos" { "delocate-wheel -w" } else { "auditwheel repair --wheel-dir" }

# Native architecture task owners; imported recipes share these platform settings.
import 'just/scheme.just'
import 'just/runtime.just'
import 'just/benchmark.just'
import 'just/python.just'
import 'just/checks.just'
import 'just/development.just'
import 'just/ci.just'

# Daily entry points; inspect private owner recipes with just --dump.
default:
    @just --list

format:
    cargo fmt --all
