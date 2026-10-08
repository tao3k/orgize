# Third-party components in orgizepy wheels

The `MIT` project metadata describes Orgize's own code. A complete `orgizepy`
wheel also contains third-party code. This inventory does not relicense those
components; the accompanying `LICENSES/` files preserve their license texts.
The exact dynamically bundled libraries can vary by build platform and are
visible under `orgizepy/.dylibs/` or the equivalent repaired-wheel directory.

| Component | How it enters the wheel | Upstream terms and source |
| --- | --- | --- |
| Gambit Scheme and Gerbil | Runtime and compiled Scheme modules in `liborgize` | Gerbil on Gambit is offered under LGPL-2.1-or-later and Apache-2.0; see [Gerbil](https://github.com/mighty-gerbils/gerbil) and [Gambit](https://github.com/gambit/gambit). The CI toolchain is pinned in `tools/ci/install-gerbil-release.sh`. |
| gerbil-poo | Compiled Scheme dependency in `liborgize` | Apache-2.0; [source at `099b381588360a8a49fd772f666a1a00351366f5`](https://github.com/tao3k/gerbil-poo/tree/099b381588360a8a49fd772f666a1a00351366f5). |
| gerbil-parser runtime and artifact | Rust admission and navigation projections in `_orgize`, with Scheme parser modules in the native program | Apache-2.0 AND LGPL-2.1-or-later; [source at `d066e1a4`](https://github.com/tao3k/gerbil-parser/tree/d066e1a472359b9e3fcced67326b15bb5cf0cd4f). |
| gerbil-scheme and gerbil-scheme-sys | Rust/C native runtime lifecycle and FFI in `_orgize` | Apache-2.0 OR LGPL-2.1-or-later; [source at `1e4f1f65`](https://github.com/tao3k/gerbil-scheme-rust/tree/1e4f1f65c7ff8e95fe764f2e8549fa9d92275db4). |
| OpenSSL 3 | Bundled by wheel repair when linked by the native runtime | Apache-2.0; [source and license](https://github.com/openssl/openssl). |
| zlib | Bundled by wheel repair when linked by the native runtime | Zlib license; [source and license](https://zlib.net/zlib_license.html). |
| SQLite | Bundled by wheel repair when linked by the native runtime | Public domain; [source statement](https://www.sqlite.org/copyright.html). |

The wheel's CycloneDX SBOM enumerates additional Rust crates, their versions,
and their declared license expressions.
Review the exact built wheel and its linked libraries before publication;
including these texts is not, by itself, a completed release-compliance review.
