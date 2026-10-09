# Producer-owned FFI bundles

Orgize owns compilation and qualification of its Scheme parser. Downstream Rust
applications can consume its static program archive with `ORGIZE_FFI_BUNDLE`
pointing to an extracted producer bundle. This branch validates the bundle and
emits linker directives without reading a Scheme program manifest or invoking
its compiler. A rejected bundle fails the build; it never falls back to rebuilding.

The `orgize.ffi-bundle.v1` manifest binds the exact Orgize Git revision, Rust target
triple, pinned Gerbil FFI runtime revision, parser identity, C headers, archive
contents and ordered libraries. Static Gambit and compiler runtime dependencies
are included. System libraries remain host linker requirements. Only consume
artifacts from a trusted successful producer run for the pinned source revision;
file hashes provide integrity, not publisher authentication.

Plain downstream Cargo builds require `ORGIZE_FFI_BUNDLE`. Scheme compilation
is enabled only by an explicit producer manifest or export request. The Orgize
producer recipes supply that manifest; an absent consumer bundle never triggers
a parser rebuild.

Producer builds set `ORGIZE_FFI_EXPORT` to package the compiler-owned link receipt.
CI runs the existing Scheme and Rust qualification, relocates the bundle, reruns
Rust tests with a deliberately absent program manifest, and uploads a separate
artifact for each qualified platform. Artifact names contain the exact source
revision, OS and architecture. CI checks out the PR head rather than a merge SHA.

This removes downstream **Orgize Scheme compilation**. The separately pinned
`gerbil-scheme-sys` external-program dependency still builds its small C lifecycle
bridge with the installed Gambit toolchain. It does not rebuild the Orgize parser.
A completely compiler-free FFI runtime distribution requires producer support in
that dependency as well; this bundle does not claim that capability.

The private corpus handoff batches both syntax events and document metadata.
Metadata packets admit at most 64 documents and 64 KiB of encoded fields;
each document delegates to the existing Scheme document semantic function.
The Rust adapter preserves source order and per-document failures, checks
response framing, and builds the same source-backed syntax/Element projections. Oversized
single-document configuration fields retain the existing individual call.
This batching does not change public contract versions or performance budgets.

POO strategy declarations stay in the build lane. The engine lowers both
document and secondary inline strategies into bounded native Scheme units.
The runtime imports these generated products, not the generic EventFold
interpreter or the declaration factory. Parameter overrides remain call-scoped.
The producer archive contract requires both products and rejects an interpreted
Fold or Org declaration module in the executable closure.

The bundle carries static OpenSSL, zlib, and SQLite dependencies declared by Gambit. Platform C libraries remain platform links. Every carried archive is bound by its digest; consumer builds do not inherit producer library search paths.

Library discovery belongs to the shared native-build owner. Orgize packages
only archives located by its link receipt; missing static archives fail export,
without downstream package probing or environment-path discovery.
Repeated exports replace read-only SDK
inputs by publishing new files, without changing permissions on the SDK.
