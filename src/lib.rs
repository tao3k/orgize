#![doc = include_str!("../README.md")]

/// Agent-facing document command API.
pub mod agent;
/// Owned semantic AST projected from the lossless parser tree.
#[path = "semantic_ast/mod.rs"]
pub mod ast;
/// Command-line interface implementation.
#[doc(hidden)]
pub mod cli;
/// Parser configuration.
pub mod config;
/// Document element mapping and parser-owned query API.
pub mod document;
mod entities;
/// Event traversal and export helpers built on the lossless syntax tree.
pub mod export;
/// Conservative Org source formatter.
pub mod fmt;
/// Org document linting helpers.
pub mod lint;
mod lint_runtime_validation;
mod org;
mod replace;
mod runtime;
mod syntax;
#[doc(hidden)]
pub mod syntax_ast;
#[path = "ast/mod.rs"]
mod syntax_ast_impl;
#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;

// Re-export of the rowan crate.
pub use rowan;

pub use config::ParseConfig;
pub use org::Org;
pub use rowan::{TextRange, TextSize};
pub use syntax::{
    SyntaxElement, SyntaxElementChildren, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxToken,
};

pub(crate) use syntax::combinator::lossless_parser;

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
                    "src/lint_file_links.rs",
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
