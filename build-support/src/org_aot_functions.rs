//! Compile Scheme-authored typed function IR during a Cargo build.

use std::{env, fs, path::PathBuf};

/// Generate Rust functions from the checked-in Scheme IR, without Gerbil at build time.
pub fn write_org_aot_functions() {
    let source_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest dir"))
        .join("languages/org/v1/modules/org-elements/generated");
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output dir"));
    for name in [
        "todo_directive_p",
        "todo_state_from_directives",
        "todo_keyword_matches_p",
        "todo_keyword_from_directives",
        "headline_content_after_todo",
    ] {
        let source = source_dir.join(format!("{name}.ir.json"));
        println!("cargo:rerun-if-changed={}", source.display());
        let ir = fs::read_to_string(&source).expect("read Scheme-authored headline IR");
        let generated = gerbil_scheme_rust_ir::compile_function_json(&ir)
            .expect("Scheme-authored headline IR must compile to Rust");
        fs::write(output_dir.join(format!("{name}.rs")), generated)
            .expect("write generated headline function");
    }
}

/// Compile the Org-owned contextual event algorithm for Cargo-only consumers.
pub fn write_org_aot_events() {
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest dir"))
        .join("languages/org/v1/generated/rowan-events.ir.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let ir = fs::read_to_string(source).expect("read Scheme-authored Org event IR");
    let generated = gerbil_scheme_rust_ir::compile_event_function_json(&ir)
        .expect("Scheme-authored Org events must compile to Rust");
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output dir"));
    fs::write(output_dir.join("org_rowan_events.rs"), generated)
        .expect("write generated Org event function");
}
