//! Revision-bound, relocatable Orgize program archives. Admission never compiles Scheme.
use gerbil_scheme_aot_build::{CargoDirectiveKind, NativeArchiveLinkReceipt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const HEADERS: &[&str] = &["orgize.h", "orgize_runtime.h"];
static NEXT_COPY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    schema: String,
    source_revision: String,
    target: String,
    runtime_revision: String,
    parser_digest: String,
    files: BTreeMap<String, String>,
    libraries: Vec<String>,
}

#[derive(Deserialize)]
struct WorkspaceManifest {
    workspace: Workspace,
}
#[derive(Deserialize)]
struct Workspace {
    dependencies: BridgeDependencies,
}
#[derive(Deserialize)]
struct BridgeDependencies {
    #[serde(rename = "gerbil-scheme")]
    runtime: GitSource,
    #[serde(rename = "gerbil-scheme-sys")]
    system: GitSource,
    #[serde(rename = "gerbil-scheme-aot-build")]
    builder: GitSource,
}
#[derive(Deserialize)]
struct GitSource {
    git: String,
    rev: String,
}

fn runtime_revision(root: &Path) -> Result<String, String> {
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).map_err(|error| error.to_string())?;
    let manifest: WorkspaceManifest =
        toml::from_str(&manifest).map_err(|error| error.to_string())?;
    let bindings = manifest.workspace.dependencies;
    // Bind admission to the manifest that compiled this Cargo adapter. A later
    // caller-side edit cannot relabel an already compiled bridge or bundle.
    let compiled: WorkspaceManifest =
        toml::from_str(include_str!("../../Cargo.toml")).map_err(|error| error.to_string())?;
    let runtime = &compiled.workspace.dependencies.runtime;
    let valid = [&bindings.runtime, &bindings.system, &bindings.builder]
        .into_iter()
        .all(|source| {
            source.git == "https://github.com/tao3k/gerbil-scheme-rust"
                && source.rev == runtime.rev
                && source.rev.len() == 40
                && source.rev.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    if !valid {
        return Err("FFI runtime requires one immutable Cargo workspace bridge cohort".into());
    }
    Ok(bindings.runtime.rev)
}

fn digest(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| format!("sha256:{:x}", Sha256::digest(bytes)))
        .map_err(|error| format!("read {}: {error}", path.display()))
}

fn revision(root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("Orgize checkout revision unavailable".into());
    }
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| error.to_string())
}

fn library_file(name: &str) -> Result<String, String> {
    let name = name.strip_prefix("static=").unwrap_or(name);
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
    {
        return Err("invalid FFI link library".into());
    }
    Ok(format!("lib{name}.a"))
}

fn bundle_library(library: &str) -> String {
    // These are Gambit dependencies, not platform-provided C libraries.
    // Carry them with the producer artifact instead of retaining host paths.
    match library {
        "crypto" | "ssl" | "z" | "sqlite3" | "zstd" | "jitterentropy" => {
            format!("static={library}")
        }
        _ => library.to_owned(),
    }
}

fn validate_link_owners(target_os: &str, libraries: &[String]) -> Result<(), String> {
    // The pinned bridge scopes Gambit's signal handlers on Linux and emits
    // the resulting archive as the runtime owner in its Cargo receipt.
    let gambit = if target_os == "linux" {
        "static=gerbil_scheme_rust_gambit"
    } else {
        "static=gambit"
    };
    if libraries.first().map(String::as_str) != Some("static=orgize_gerbil_program")
        || !libraries.iter().any(|library| library == gambit)
    {
        return Err(format!(
            "missing program or target Gambit link owner {gambit}"
        ));
    }
    Ok(())
}

fn copy_file(
    source: &Path,
    bundle: &Path,
    name: &str,
    files: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    // Gerbil installation archives may be read-only. Do not propagate their permissions into
    // the export, or overwrite a previous export in place. Publish each file
    // by rename so repeated Cargo profiles can replace it without chmod.
    let bytes = fs::read(source).map_err(|error| format!("read {}: {error}", source.display()))?;
    let temporary = bundle.join(format!(
        ".{name}.{}-{}.tmp",
        std::process::id(),
        NEXT_COPY.fetch_add(1, Ordering::Relaxed)
    ));
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("create {name}: {error}"))?;
    let result = (|| {
        output.write_all(&bytes)?;
        drop(output);
        fs::rename(&temporary, bundle.join(name))
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(format!("publish {name}: {error}"));
    }
    files.insert(
        name.to_owned(),
        format!("sha256:{:x}", Sha256::digest(bytes)),
    );
    Ok(())
}

fn receipt_dependency(name: &str, search: &[PathBuf]) -> Result<PathBuf, String> {
    search
        .iter()
        .map(|path| path.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| format!("incomplete native-build receipt: missing static dependency {name}"))
}

pub(crate) fn publish(
    root: &Path,
    directory: &Path,
    parser_digest: &str,
    receipt: &NativeArchiveLinkReceipt,
) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let search: Vec<PathBuf> = receipt
        .cargo_directives
        .iter()
        .filter(|directive| directive.kind == CargoDirectiveKind::RustcLinkSearch)
        .map(|directive| {
            PathBuf::from(
                directive
                    .value
                    .strip_prefix("native=")
                    .unwrap_or(&directive.value),
            )
        })
        .collect();
    let libraries: Vec<String> = receipt
        .cargo_directives
        .iter()
        .filter(|directive| directive.kind == CargoDirectiveKind::RustcLinkLib)
        .map(|directive| bundle_library(&directive.value))
        .collect();
    let target = env::var("TARGET").map_err(|error| error.to_string())?;
    let target_os = env::var("CARGO_CFG_TARGET_OS").map_err(|error| error.to_string())?;
    validate_link_owners(&target_os, &libraries)?;
    let mut files = BTreeMap::new();
    for library in &libraries {
        let name = library_file(library)?;
        if library.starts_with("static=") {
            let source = receipt_dependency(&name, &search)?;
            copy_file(&source, &directory, &name, &mut files)?;
        }
    }
    for header in HEADERS {
        copy_file(
            &root.join("bindings/c/include").join(header),
            &directory,
            header,
            &mut files,
        )?;
    }
    let bundle = Bundle {
        schema: "orgize.ffi-bundle.v1".into(),
        source_revision: revision(root)?,
        target,
        runtime_revision: runtime_revision(root)?,
        parser_digest: parser_digest.into(),
        files,
        libraries,
    };
    fs::write(
        directory.join("bundle.json"),
        serde_json::to_vec_pretty(&bundle).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn validate(
    root: &Path,
    directory: &Path,
    expected_revision: &str,
    target: &str,
    target_os: &str,
) -> Result<Bundle, String> {
    let bundle: Bundle = serde_json::from_slice(
        &fs::read(directory.join("bundle.json")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if bundle.schema != "orgize.ffi-bundle.v1"
        || bundle.source_revision != expected_revision
        || bundle.target != target
        || bundle.runtime_revision != runtime_revision(root)?
    {
        return Err("foreign FFI schema, source revision, target or runtime".into());
    }
    let hash = bundle
        .parser_digest
        .strip_prefix("sha256:")
        .ok_or("invalid parser digest")?;
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("invalid parser digest".into());
    }
    validate_link_owners(target_os, &bundle.libraries)?;
    for (name, expected) in &bundle.files {
        if Path::new(name).file_name().and_then(|name| name.to_str()) != Some(name.as_str())
            || name.contains(['/', '\\'])
            || directory.join(name).is_symlink()
        {
            return Err("FFI input must be a regular local file".into());
        }
        if digest(&directory.join(name))? != *expected {
            return Err(format!("changed FFI input {name}"));
        }
    }
    for library in &bundle.libraries {
        let name = library_file(library)?;
        if library.starts_with("static=") && !bundle.files.contains_key(&name) {
            return Err(format!("unbound FFI library {library}"));
        }
    }
    for header in HEADERS {
        let expected = bundle.files.get(*header).ok_or("missing FFI header")?;
        if digest(&root.join("bindings/c/include").join(header))? != *expected {
            return Err(format!("FFI header mismatch {header}"));
        }
    }
    Ok(bundle)
}

pub(crate) fn consume(root: &Path, directory: &Path) -> Result<(), String> {
    let directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let bundle = validate(
        root,
        &directory,
        &revision(root)?,
        &env::var("TARGET").map_err(|error| error.to_string())?,
        &env::var("CARGO_CFG_TARGET_OS").map_err(|error| error.to_string())?,
    )?;
    println!(
        "cargo:rerun-if-changed={}",
        directory.join("bundle.json").display()
    );
    for name in bundle.files.keys() {
        println!("cargo:rerun-if-changed={}", directory.join(name).display());
    }
    for header in HEADERS {
        println!(
            "cargo:rerun-if-changed={}",
            root.join("bindings/c/include").join(header).display()
        );
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    fs::write(
        out.join("org_identity.rs"),
        format!(
            "pub const NATIVE_PARSER_DIGEST: &str = {:?};\n",
            bundle.parser_digest
        ),
    )
    .map_err(|error| error.to_string())?;
    println!("cargo:rustc-link-search=native={}", directory.display());
    for library in bundle.libraries {
        println!("cargo:rustc-link-lib={library}");
    }
    eprintln!("orgize-ffi: admitted producer archive; no Scheme compilation");
    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/ffi_bundle.rs"]
mod tests;
