#![doc = include_str!("../README.md")]

#[path = "semantic_ast/mod.rs"]
pub mod ast;
pub mod config;
mod entities;
pub mod export;
mod org;
mod replace;
mod syntax;
#[path = "ast/mod.rs"]
mod syntax_ast_impl;
#[doc(hidden)]
pub mod syntax_ast {
    pub use crate::syntax_ast_impl::*;
}
#[cfg(test)]
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
