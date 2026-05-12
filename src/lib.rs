#![doc = include_str!("../README.md")]

/// Owned semantic AST projected from the lossless parser tree.
#[path = "semantic_ast/mod.rs"]
pub mod ast;
/// Command-line interface implementation.
#[doc(hidden)]
pub mod cli;
/// Parser configuration.
pub mod config;
mod entities;
/// Event traversal and export helpers built on the lossless syntax tree.
pub mod export;
/// Conservative Org source formatter.
pub mod fmt;
/// Org document linting helpers.
pub mod lint;
mod lint_render;
mod org;
mod replace;
mod syntax;
#[doc(hidden)]
pub mod syntax_ast;
#[path = "ast/mod.rs"]
mod syntax_ast_impl;
#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;
#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config()
            .with_verification_profile_hint(
                rust_lang_project_harness::RustVerificationProfileHint::new(
                    "src/lib.rs",
                    [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
                )
                .without_verification_tasks()
                .with_rationale(
                    "orgize parser-v2 owns public parser and semantic AST APIs; this PR keeps external verification work in the repository cargo gates",
                ),
            )
    }
);

// Re-export of the rowan crate.
pub use rowan;

pub use config::ParseConfig;
pub use org::Org;
pub use rowan::{TextRange, TextSize};
pub use syntax::{
    SyntaxElement, SyntaxElementChildren, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxToken,
};

pub(crate) use syntax::combinator::lossless_parser;
