use std::env;
use std::path::PathBuf;

fn main() {
    // Maturin sets this only when producing a Python extension, not for the
    // repository's ordinary Cargo test/clippy jobs.
    if env::var_os("PYO3_BUILD_EXTENSION_MODULE").is_none() {
        return;
    }

    let extension = match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => "dylib",
        Ok("linux") => "so",
        _ => panic!("the complete orgizepy SDK is currently supported on macOS and Linux"),
    };
    let library = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/orgizepy/lib")
        .join(format!("liborgize.{extension}"));
    println!("cargo:rerun-if-changed={}", library.display());
    assert!(
        library.is_file(),
        "the complete orgizepy SDK requires {}; build the Scheme Contract library before building a wheel (just python-contract-library)",
        library.display()
    );
}
