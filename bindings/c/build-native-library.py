#!/usr/bin/env python3
"""Link Orgize's Scheme-owned C ABI into an independently loadable library.

Gambit embedding needs a complete, non-flat link unit plus one explicit
runtime setup object. The Org parser's Rust/Cargo build never invokes this
tool; it is only for consumers that choose the Scheme Contract ABI.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import platform
import re
import shlex
import subprocess
import sys


PROJECT = Path(__file__).resolve().parents[2]


def run(args: list[str], *, capture: bool = False) -> str:
    completed = subprocess.run(
        args, cwd=PROJECT, env=os.environ, check=True, text=True,
        capture_output=capture,
    )
    return completed.stdout if capture else ""


def unique(paths: list[Path]) -> list[Path]:
    return list(dict.fromkeys(path.resolve() for path in paths))


def static_closure() -> tuple[list[tuple[str, Path]], Path]:
    expression = r'''(let* ((ctx (import-module "bindings/c/orgize-native.ss"))
                             (deps (gxc#find-runtime-module-deps ctx)))
                        (for-each
                         (lambda (dep)
                           (displayln (expander-context-id dep) "\t"
                                      (gxc#find-static-module-file dep)))
                         deps)
                        (displayln "ROOT\t" (gxc#find-static-module-file ctx)))'''
    output = run([
        "gxi", "-:max-heap=512M", "-e",
        "(import :gerbil/compiler/driver :gerbil/expander)",
        "-e", expression,
    ], capture=True)
    dependencies: list[tuple[str, Path]] = []
    root: Path | None = None
    for line in output.splitlines():
        module_id, raw_path = line.split("\t", 1)
        if module_id == "ROOT":
            root = Path(raw_path).resolve()
        else:
            dependencies.append((module_id, Path(raw_path).resolve()))
    if root is None:
        raise RuntimeError("Gerbil did not report the Orgize native root")
    return dependencies, root


def generated_define(source: Path, name: str) -> str:
    match = re.search(
        rf"^#define {re.escape(name)} ([A-Za-z0-9_]+)$",
        source.read_text(encoding="utf-8"), re.MULTILINE,
    )
    if match is None:
        raise RuntimeError(f"Gambit {name} is absent from {source}")
    return match.group(1)


def configure_toolchain() -> tuple[str, Path]:
    if sys.platform == "darwin":
        product = platform.mac_ver()[0]
        if not product:
            raise RuntimeError("cannot determine the macOS deployment target")
        os.environ.setdefault("MACOSX_DEPLOYMENT_TARGET", product.split(".", 1)[0] + ".0")
        os.environ.setdefault("CC", "/usr/bin/cc")
        os.environ.pop("SDKROOT", None)
    header_root = PROJECT / "bindings/c"
    previous = os.environ.get("C_INCLUDE_PATH")
    os.environ["C_INCLUDE_PATH"] = os.pathsep.join(
        value for value in (str(header_root), previous) if value
    )
    home = Path(run(["gxi", "-e", "(displayln (gerbil-home))"], capture=True).strip())
    compiler = home / "bin/gsc"
    if not compiler.is_file():
        raise RuntimeError(f"Gerbil release has no Gambit compiler: {compiler}")
    runtime_options = f"~~={home},~~bin={home / 'bin'},~~lib={home / 'lib'}"
    prior = os.environ.get("GAMBOPT", "")
    os.environ["GAMBOPT"] = f"{prior},{runtime_options}" if prior else runtime_options
    return str(compiler), home / "lib"


def compile_missing_objects(gsc: str, sources: list[Path]) -> None:
    for source in sources:
        target = source.with_suffix(".o")
        if target.is_file() and target.stat().st_mtime_ns >= source.stat().st_mtime_ns:
            continue
        c_source = target.with_suffix(".c")
        if not c_source.is_file():
            raise RuntimeError(f"native closure C source is absent: {c_source}")
        run([gsc, "-target", "C", "-obj", "-o", str(target), str(c_source)])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    options = parser.parse_args()
    output = options.output.expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    work_dir = PROJECT / "target/native-contract"
    work_dir.mkdir(parents=True, exist_ok=True)
    gsc, gerbil_lib = configure_toolchain()
    dependencies, root = static_closure()
    libgerbil = unique([
        path for module_id, path in dependencies
        if (module_id.startswith("gerbil/") or module_id.startswith("std/"))
        and not module_id.startswith("gerbil/core")
    ])
    user = unique([
        path for module_id, path in dependencies
        if not module_id.startswith(("gerbil/", "std/"))
        and path.is_file() and path.stat().st_size > 0
    ])
    link_source = work_dir / "orgize-native_.c"
    link_object = work_dir / "orgize-native_.o"
    runtime_object = work_dir / "orgize-runtime.o"
    run([
        gsc, "-target", "C", "-link", "-o", str(link_source),
        *[str(path.with_suffix(".c")) for path in libgerbil],
        *[str(path) for path in user], str(root),
    ])
    run([
        gsc, "-target", "C", "-cc-options", "-D___LIBRARY", "-obj",
        "-o", str(link_object), str(link_source),
    ])
    run([
        gsc, "-target", "C", "-cc-options",
        f"-D___VERSION={generated_define(link_source, '___VERSION')} "
        f"-DORGIZE_LINKER={generated_define(link_source, '___LINKER_ID')}",
        "-obj", "-o", str(runtime_object), "bindings/c/orgize-runtime.c",
    ])
    compile_missing_objects(gsc, [*user, root, *libgerbil])
    module_objects = [path.with_suffix(".o") for path in user]
    gerbil_objects = [path.with_suffix(".o") for path in libgerbil]
    missing = [
        path for path in [*module_objects, root.with_suffix(".o"), *gerbil_objects]
        if not path.is_file()
    ]
    if missing:
        raise RuntimeError(f"native closure object is absent: {missing[0]}")
    flags = list(dict.fromkeys(shlex.split(
        (gerbil_lib / "libgerbil.ldd").read_text().strip().strip("()")
    )))
    if sys.platform == "darwin":
        shared = ["-dynamiclib", "-Wl,-undefined,dynamic_lookup"]
        if platform.machine() == "arm64":
            shared.append("-Wl,-no_compact_unwind")
    else:
        shared = ["-shared"]
    run([
        os.environ.get("CC", "cc"), *shared, "-o", str(output),
        *[str(path) for path in module_objects], str(root.with_suffix(".o")),
        str(link_object), str(runtime_object),
        *[str(path) for path in gerbil_objects], "-L", str(gerbil_lib),
        "-lgambit", *flags,
    ])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
