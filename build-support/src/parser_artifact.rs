//! Build-time parser artifact identity generation.

use std::{env, fs, path::Path};

/// Emits a content identity covering parser sources, dependency locks, features, and build inputs.
pub fn write_parser_artifact_digest() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let manifest_dir = Path::new(&manifest_dir);
    let mut files = vec![
        manifest_dir.join("Cargo.toml"),
        manifest_dir.join("Cargo.lock"),
    ];
    collect_files(&manifest_dir.join("src"), &mut files);
    files.sort();

    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, b"orgize.parser-artifact.v1");
    for path in files {
        let relative = path.strip_prefix(manifest_dir).expect("workspace file");
        hash_part(&mut hasher, relative.to_string_lossy().as_bytes());
        hash_part(
            &mut hasher,
            &fs::read(&path).expect("read parser artifact input"),
        );
    }
    let mut build_inputs = env::vars()
        .filter(|(key, _)| {
            key.starts_with("CARGO_FEATURE_")
                || matches!(
                    key.as_str(),
                    "TARGET" | "PROFILE" | "OPT_LEVEL" | "DEBUG" | "RUSTC"
                )
        })
        .collect::<Vec<_>>();
    build_inputs.sort();
    for (key, value) in build_inputs {
        hash_part(&mut hasher, key.as_bytes());
        hash_part(&mut hasher, value.as_bytes());
    }

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=src");
    println!(
        "cargo:rustc-env=ORGIZE_PARSER_ARTIFACT_DIGEST={}",
        hasher.finalize().to_hex()
    );
}

fn collect_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .expect("read parser source directory")
        .map(|entry| entry.expect("read parser source entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn hash_part(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}
