# Third-party components in orgizepy wheels

The `MIT` project metadata describes Orgize's own code. A complete `orgizepy`
wheel also contains third-party code. This inventory does not relicense those
components; the accompanying `LICENSES/` files preserve their license texts.
The exact dynamically bundled libraries can vary by build platform and are
visible under `orgizepy/.dylibs/` or the equivalent repaired-wheel directory.

| Component | How it enters the wheel | Upstream terms and source |
| --- | --- | --- |
| Gambit Scheme and Gerbil | Runtime and compiled Scheme modules in `liborgize` | Gerbil on Gambit is offered under LGPL-2.1-or-later and Apache-2.0; see [Gerbil](https://github.com/mighty-gerbils/gerbil) and [Gambit](https://github.com/gambit/gambit). The CI toolchain is pinned in `tools/ci/install-gerbil-release.sh`. |
| gerbil-poo | Compiled Scheme dependency in `liborgize` | Apache-2.0; [source at `e8c57dd44f610c634cf738b130907e0e29fe8ccf`](https://github.com/tao3k/gerbil-poo/tree/e8c57dd44f610c634cf738b130907e0e29fe8ccf). |
| gerbil-parser-rowan | Rust parser dependency in `_orgize` | Apache-2.0 AND LGPL-2.1-or-later; [source at `4b9c1d5f51c5d5c415a53d157301f0609a0b9aa0`](https://github.com/tao3k/gerbil-parser/tree/4b9c1d5f51c5d5c415a53d157301f0609a0b9aa0). |
| gerbil-scheme-rust-ir | Rust parser dependency in `_orgize` | Apache-2.0 OR LGPL-2.1-or-later; [source at `bc8b4bb1209016c7603e1289c9817792fec54887`](https://github.com/tao3k/gerbil-scheme-rust/tree/bc8b4bb1209016c7603e1289c9817792fec54887). |
| OpenSSL 3 | Bundled by wheel repair when linked by the native runtime | Apache-2.0; [source and license](https://github.com/openssl/openssl). |
| zlib | Bundled by wheel repair when linked by the native runtime | Zlib license; [source and license](https://zlib.net/zlib_license.html). |
| SQLite | Bundled by wheel repair when linked by the native runtime | Public domain; [source statement](https://www.sqlite.org/copyright.html). |

The wheel's CycloneDX SBOM enumerates additional Rust crates, their versions,
and their declared license expressions.
Review the exact built wheel and its linked libraries before publication;
including these texts is not, by itself, a completed release-compliance review.
