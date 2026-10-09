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
