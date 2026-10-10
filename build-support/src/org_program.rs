//! Consume the existing compiler-owned Gerbil program manifest.
use gerbil_scheme_aot_build::{
    NativeHeaderInput, ProgramArchiveContract, ProgramArchiveObservation, ProgramArchiveObserver,
    ProgramArchiveRequest, build_program_archive_with_contract, discover_gambit_gsc_from_env,
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
        eprintln!("orgize: {observation}");
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

pub fn write_org_program() {
    println!("cargo:rerun-if-env-changed=ORGIZE_GERBIL_PROGRAM_MANIFEST");
    println!("cargo:rerun-if-env-changed=GERBIL_GSC");
    println!("cargo:rerun-if-env-changed=GERBIL_HOME");
    println!("cargo:rerun-if-env-changed=GERBIL_GXI");
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=ORGIZE_GERBIL_NATIVE_OUTPUT");
    println!("cargo:rerun-if-env-changed=ORGIZE_FFI_BUNDLE");
    println!("cargo:rerun-if-env-changed=ORGIZE_FFI_EXPORT");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo root"));
    if let Some(bundle) = env::var_os("ORGIZE_FFI_BUNDLE") {
        crate::ffi_bundle::consume(&root, &PathBuf::from(bundle))
            .expect("admit producer-owned Orgize FFI bundle");
        return;
    }
    assert!(
        env::var_os("ORGIZE_FFI_EXPORT").is_some()
            || env::var_os("ORGIZE_GERBIL_PROGRAM_MANIFEST").is_some(),
        "Set ORGIZE_FFI_BUNDLE to a qualified producer artifact; Scheme compilation requires an explicit producer manifest or export request"
    );
    println!(
        "cargo:rerun-if-changed={}",
        root.join("bindings/c/include/orgize.h").display()
    );
    let manifest = env::var_os("ORGIZE_GERBIL_PROGRAM_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/gerbil-parser/program.json"));
    let manifest = manifest.canonicalize().expect(
        "Org::parse requires the canonical Gerbil AOT program: use just scheme-parser-build-isolated then scheme-aligned-program-stage-receipt; no generated-Rust fallback",
    );
    println!("cargo:rerun-if-changed={}", manifest.display());
    let gsc = discover_gambit_gsc_from_env()
        .expect("resolve the Gerbil toolchain's Gambit compiler through the native-build owner");
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
    let include_directory = root.join("bindings/c/include");
    let header_files = [
        include_directory.join("orgize.h"),
        include_directory.join("orgize_runtime.h"),
    ];
    let native_headers = [NativeHeaderInput {
        include_directory: &include_directory,
        header_files: &header_files,
    }];
    let receipt = build_program_archive_with_contract(
        ProgramArchiveRequest {
            manifest: &manifest,
            gsc: &gsc,
            archive_name: "orgize_gerbil_program",
            linker_name: "orgize_gerbil_program",
            out_dir: &native_out,
        },
        ProgramArchiveContract {
            required_modules: &[
                "gerbil-scheme-rust/scheme/runtime",
                "orgize/bindings/c/orgize-parser",
                "orgize/fold/parse-org-compiled-events",
                "orgize/fold/parse-org-compiled-inline-events",
            ],
            forbidden_modules: &[
                "gerbil-parser/src/compiler/event-fold-runtime",
                "orgize/languages/org/modules/org-parser/event-strategy",
                "orgize/languages/org/modules/org-parser/runtime-funs",
            ],
            linker_main_symbol: "gerbil_scheme_rust_program_main",
            additional_objects: &[],
            native_headers: &native_headers,
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
        out.join("org_identity.rs"),
        format!("pub const NATIVE_PARSER_DIGEST: &str = {digest:?};\n"),
    )
    .expect("write native parser identity");
    if let Some(bundle) = env::var_os("ORGIZE_FFI_EXPORT") {
        crate::ffi_bundle::publish(&root, &PathBuf::from(bundle), &digest, &receipt)
            .expect("publish producer-owned Orgize FFI bundle");
    }
    for directive in receipt.cargo_directives {
        println!("{}", directive.line());
    }
}
