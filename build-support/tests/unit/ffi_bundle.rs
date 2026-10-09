use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture() -> (Fixture, PathBuf, Bundle) {
    let root = env::temp_dir().join(format!(
        "orgize-ffi-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let directory = root.join("bundle");
    fs::create_dir_all(&directory).unwrap();
    fs::create_dir_all(root.join("bindings/c/include")).unwrap();
    let mut files = BTreeMap::new();
    for name in [
        "liborgize_gerbil_program.a",
        "libgambit.a",
        "orgize.h",
        "orgize_runtime.h",
    ] {
        fs::write(directory.join(name), name).unwrap();
        files.insert(name.into(), digest(&directory.join(name)).unwrap());
    }
    for name in HEADERS {
        fs::copy(
            directory.join(name),
            root.join("bindings/c/include").join(name),
        )
        .unwrap();
    }
    let bundle = Bundle {
        schema: "orgize.ffi-bundle.v1".into(),
        source_revision: "a".repeat(40),
        target: "aarch64-apple-darwin".into(),
        runtime_revision: RUNTIME_REVISION.into(),
        parser_digest: format!("sha256:{}", "b".repeat(64)),
        files,
        libraries: vec![
            "static=orgize_gerbil_program".into(),
            "static=gambit".into(),
            "m".into(),
        ],
    };
    (Fixture(root), directory, bundle)
}
fn save(directory: &Path, bundle: &Bundle) {
    fs::write(
        directory.join("bundle.json"),
        serde_json::to_vec(bundle).unwrap(),
    )
    .unwrap();
}
#[test]
fn relocation_preserves_admission_without_compiler_inputs() {
    let (root, directory, bundle) = fixture();
    save(&directory, &bundle);
    let moved = root.0.join("relocated");
    fs::rename(directory, &moved).unwrap();
    assert!(validate(&root.0, &moved, &bundle.source_revision, &bundle.target).is_ok());
}
#[test]
fn foreign_identity_and_changed_inputs_are_rejected() {
    let (root, directory, mut bundle) = fixture();
    save(&directory, &bundle);
    assert!(validate(&root.0, &directory, "foreign", &bundle.target).is_err());
    assert!(validate(&root.0, &directory, &bundle.source_revision, "foreign").is_err());
    bundle.runtime_revision = "foreign".into();
    save(&directory, &bundle);
    assert!(validate(&root.0, &directory, &bundle.source_revision, &bundle.target).is_err());
    bundle.runtime_revision = RUNTIME_REVISION.into();
    save(&directory, &bundle);
    fs::write(directory.join("libgambit.a"), "changed").unwrap();
    assert!(validate(&root.0, &directory, &bundle.source_revision, &bundle.target).is_err());
}
#[test]
fn unbound_libraries_headers_and_paths_are_rejected() {
    let (root, directory, mut bundle) = fixture();
    bundle.files.remove("libgambit.a");
    save(&directory, &bundle);
    assert!(validate(&root.0, &directory, &bundle.source_revision, &bundle.target).is_err());
    bundle.files.insert(
        "libgambit.a".into(),
        digest(&directory.join("libgambit.a")).unwrap(),
    );
    bundle.files.insert("../escape".into(), "sha256:bad".into());
    save(&directory, &bundle);
    assert!(validate(&root.0, &directory, &bundle.source_revision, &bundle.target).is_err());
    bundle.files.remove("../escape");
    bundle.files.insert(
        "libgambit.a".into(),
        digest(&directory.join("libgambit.a")).unwrap(),
    );
    save(&directory, &bundle);
    fs::write(root.0.join("bindings/c/include/orgize.h"), "changed").unwrap();
    assert!(validate(&root.0, &directory, &bundle.source_revision, &bundle.target).is_err());
}
