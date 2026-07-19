{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  wasmPackRelease =
    {
      aarch64-darwin = {
        target = "aarch64-apple-darwin";
        hash = "sha256-Cr/0oD1nC2wA6jHQ4WCKckB+NV89N2Xpww60XNW34xg=";
      };
      x86_64-darwin = {
        target = "x86_64-apple-darwin";
        hash = "sha256-0/Gkoz6V+PDXgBsCTghiTEeZmayWqhUJCLI5QBXNA2M=";
      };
      aarch64-linux = {
        target = "aarch64-unknown-linux-musl";
        hash = "sha256-4X7wgGOBw6CsucndrWQ6SfrKpaLs9lekIdTY8zV6JLc=";
      };
      x86_64-linux = {
        target = "x86_64-unknown-linux-musl";
        hash = "sha256-wJ+XHsrtmi78gP3Op6AO9rU8f63IxX0fYbU6aqZrZoo=";
      };
    }
    .${pkgs.stdenv.hostPlatform.system};
  wasmBindgenRelease =
    {
      aarch64-darwin = {
        target = "aarch64-apple-darwin";
        hash = "sha256-SzaoqSg4CK9tI6DS1UmimeQARauLInnBOc3/Yxm28MM=";
      };
      x86_64-darwin = {
        target = "x86_64-apple-darwin";
        hash = "sha256-GZykGE3DW1dDLoIMuThbW+ZbKKFS9Wn3Eeboo+VjI8g=";
      };
      aarch64-linux = {
        target = "aarch64-unknown-linux-musl";
        hash = "sha256-clOzqm4ZmAsV6leHbg7TqZluGSqLqfxOKHcxjV8IMOE=";
      };
      x86_64-linux = {
        target = "x86_64-unknown-linux-musl";
        hash = "sha256-MDnzj2X+I3tkDPBqFAyRnKjXF+xQEhRtFF0/J7tNayg=";
      };
    }
    .${pkgs.stdenv.hostPlatform.system};
  wasm-pack = pkgs.stdenvNoCC.mkDerivation {
    pname = "wasm-pack";
    version = "0.15.0";
    src = pkgs.fetchurl {
      url = "https://github.com/rustwasm/wasm-pack/releases/download/v0.15.0/wasm-pack-v0.15.0-${wasmPackRelease.target}.tar.gz";
      inherit (wasmPackRelease) hash;
    };
    sourceRoot = "wasm-pack-v0.15.0-${wasmPackRelease.target}";
    dontBuild = true;
    installPhase = ''
      runHook preInstall
      install -Dm755 wasm-pack $out/bin/wasm-pack
      runHook postInstall
    '';
    meta = {
      description = "Utility that builds Rust-generated WebAssembly packages";
      homepage = "https://github.com/rustwasm/wasm-pack";
      license = with lib.licenses; [
        asl20
        mit
      ];
      mainProgram = "wasm-pack";
    };
  };
  wasm-bindgen-cli = pkgs.stdenvNoCC.mkDerivation {
    pname = "wasm-bindgen-cli";
    version = "0.2.121";
    src = pkgs.fetchurl {
      url = "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.121/wasm-bindgen-0.2.121-${wasmBindgenRelease.target}.tar.gz";
      inherit (wasmBindgenRelease) hash;
    };
    sourceRoot = "wasm-bindgen-0.2.121-${wasmBindgenRelease.target}";
    dontBuild = true;
    installPhase = ''
      runHook preInstall
      install -Dm755 wasm-bindgen $out/bin/wasm-bindgen
      install -Dm755 wasm-bindgen-test-runner $out/bin/wasm-bindgen-test-runner
      install -Dm755 wasm2es6js $out/bin/wasm2es6js
      runHook postInstall
    '';
    meta = {
      description = "WebAssembly bindings generator for Rust";
      homepage = "https://github.com/wasm-bindgen/wasm-bindgen";
      license = with lib.licenses; [
        asl20
        mit
      ];
      mainProgram = "wasm-bindgen";
    };
  };
in
{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [
    pkgs.binaryen
    pkgs.just
    pkgs.pkg-config
    pkgs.protobuf
    wasm-bindgen-cli
    wasm-pack
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [ "wasm32-unknown-unknown" ];
    # Ensure rust can link python library
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  # https://devenv.sh/languages/
  # languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # https://devenv.sh/basics/
  enterShell = "";

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = "";

  # https://devenv.sh/git-hooks/
  git-hooks.hooks = {
    shellcheck.enable = true;
    nixfmt.enable = true;
    clippy.enable = true;
    clippy.packageOverrides.cargo = config.languages.rust.toolchain.cargo;
    clippy.packageOverrides.clippy = config.languages.rust.toolchainPackage;
    clippy.settings.allFeatures = true;
  };
  # See full reference at https://devenv.sh/reference/options/
}
