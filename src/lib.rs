#![doc = include_str!("../README.md")]

/// Agent-facing document command API.
pub mod agent;
/// Owned semantic AST projected from the Scheme-AOT Element graph.
#[path = "semantic_ast/mod.rs"]
pub mod ast;
pub mod c_ffi;
/// Command-line interface implementation.
#[doc(hidden)]
pub mod cli;
/// Parser configuration.
pub mod config;
/// Scheme-AOT Org Contract execution over generated Element graphs.
pub mod contract_feature;
/// Document element mapping and parser-owned query API.
pub mod document;
mod entities;
/// Presentation helpers for Scheme-AOT graph exporters.
pub mod export;
/// Conservative Org source formatter.
pub mod fmt;
/// Org document linting helpers.
pub mod lint;
mod org;
mod startup;
pub use startup::initialize_native_runtime;
/// Statically linked Gerbil Org parser and Scheme-declared Element graph.
pub mod org_aot;
pub mod runtime_backend;
pub use runtime_backend::{RuntimeBackend, runtime_backend};
/// Source-bound Org edits validated against the Scheme-AOT Element graph.
pub mod org_aot_edit;
mod org_aot_html;
mod org_aot_latex;
mod org_aot_markdown;
/// Scheme-AOT named Org Element queries over the generated graph.
pub mod org_element_query;
mod runtime;
/// Opt-in diagnostic timings, not a parser or execution-owner selection.
#[doc(hidden)]
pub mod runtime_profile;
#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;

pub use config::ParseConfig;
pub use gerbil_parser_runtime::{SyntaxKind, SyntaxNode, SyntaxToken};
pub use gerbil_parser_runtime::{TextRange, TextSize};
pub use org::Org;

#[cfg(test)]
asp_rust::asp_rust_cargo_test_gate!(
    advice = allow,
    config = {
        let mut config = asp_rust::default_asp_rust_config()
            .with_verification_profile_hint(
                asp_rust::RustVerificationProfileHint::new(
                    "src/lib.rs",
                    [asp_rust::RustOwnerResponsibility::PublicApi],
                )
                .without_verification_tasks()
                .with_rationale(
                    "orgize mounts the ASP Rust policy as a test-only Dev Gate so normal cargo builds and downstream consumers do not compile the policy provider",
                ),
            )
            .with_verification_profile_hint(
                asp_rust::RustVerificationProfileHint::new(
                    "src/lint/rules/file_links.rs",
                    [asp_rust::RustOwnerResponsibility::PureDomainLogic],
                )
                .without_verification_tasks()
                .with_rationale(
                    "orgize file-link lint owns local Org AST and path-token policy, including portable skill-package references; integration tests cover the rule without external verification skills",
                ),
            )
            .with_cargo_test_advice_allow_explanation(
                "scope=orgize cargo-test informational advice; owner=orgize dev gate; finding_category=agent-policy Info findings only; why_safe_now=Warning and Error remain blocking severities with no severity overrides; cleanup_trigger=repair the pre-existing Info backlog under its owning API slices",
            );
        config.ignored_dir_names.insert(".devenv".to_string());
        config.ignored_dir_names.insert(".data".to_string());
        config
    }
);
