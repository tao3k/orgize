//! Consume the existing compiler-owned Gerbil program manifest.
use gerbil_scheme_native_build::{
    ProgramArchiveObservation, ProgramArchiveObserver, ProgramArchiveRequest,
    build_program_archive_observed,
};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

struct CargoObserver(Mutex<Sha256>);
impl ProgramArchiveObserver for CargoObserver {
    fn observe(&self, observation: ProgramArchiveObservation<'_>) {
        eprintln!("orgize-native: {observation}");
    }
    fn observe_source_input(&self, source: &Path) {
        println!("cargo:rerun-if-changed={}", source.display());
        // The compiler owner supplies the admitted module sequence. Bind its
        // generated Scheme and declared C header contents, not checkout paths
        // or a deleted Rust IR.
        let bytes = fs::read(source).expect("read native program identity input");
        let mut digest = self.0.lock().expect("native identity lock");
        digest.update(
            u64::try_from(bytes.len())
                .expect("native input length")
                .to_le_bytes(),
        );
        digest.update(bytes);
    }
}

pub fn write_org_native_program() {
    println!("cargo:rerun-if-env-changed=ORGIZE_GERBIL_PROGRAM_MANIFEST");
    println!("cargo:rerun-if-env-changed=GERBIL_GSC");
    println!("cargo:rerun-if-env-changed=ORGIZE_GERBIL_NATIVE_OUTPUT");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo root"));
    println!(
        "cargo:rerun-if-changed={}",
        root.join("bindings/c/include/orgize.h").display()
    );
    let manifest = env::var_os("ORGIZE_GERBIL_PROGRAM_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/gerbil-parser/program.json"));
    let manifest = manifest.canonicalize().expect(
        "Org::parse requires the canonical Gerbil AOT program: prepare it with just scheme-parser-build and scheme-parser-stage; no generated-Rust fallback",
    );
    println!("cargo:rerun-if-changed={}", manifest.display());
    let gsc = PathBuf::from(
        env::var_os("GERBIL_GSC")
            .expect("GERBIL_GSC must name the Gerbil toolchain's Gambit compiler"),
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output"));
    // Optional explicit reuse of the same compiler-owned program across Rust
    // profiles. The existing native owner still validates/fingerprints every
    // input and compiler option; identities stay in this Cargo invocation's OUT_DIR.
    let native_out = env::var_os("ORGIZE_GERBIL_NATIVE_OUTPUT")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| out.clone());
    let observer = CargoObserver(Mutex::new(Sha256::new()));
    observer
        .0
        .lock()
        .expect("native identity lock")
        .update(b"orgize.native-program-inputs.v2\0");
    let receipt = build_program_archive_observed(
        ProgramArchiveRequest {
            manifest: &manifest,
            gsc: &gsc,
            archive_name: "orgize_gerbil_program",
            linker_name: "orgize_gerbil_program",
            out_dir: &native_out,
        },
        &observer,
    )
    .expect("link canonical Org Gerbil program");
    let digest = format!(
        "sha256:{:x}",
        observer
            .0
            .into_inner()
            .expect("native identity lock")
            .finalize()
    );
    fs::write(
        out.join("org_native_identity.rs"),
        format!("pub const NATIVE_PARSER_DIGEST: &str = {digest:?};\n"),
    )
    .expect("write native parser identity");
    for directive in receipt.cargo_directives {
        println!("{}", directive.line());
    }
}
