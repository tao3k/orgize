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
/// Document element mapping and search/query API.
pub mod document;
mod entities;
/// Event traversal and export helpers built on the lossless syntax tree.
pub mod export;
/// Conservative Org source formatter.
pub mod fmt;
/// Org document linting helpers.
pub mod lint;
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

// Re-export of the rowan crate.
pub use rowan;

pub use config::ParseConfig;
pub use org::Org;
pub use rowan::{TextRange, TextSize};
pub use syntax::{
    SyntaxElement, SyntaxElementChildren, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxToken,
};

pub(crate) use syntax::combinator::lossless_parser;
