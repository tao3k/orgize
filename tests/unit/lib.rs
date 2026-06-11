//! Test helpers mounted into the crate during `cargo test --lib`.

use nom::IResult;
use rowan::{SyntaxNode, ast::AstNode};

use crate::{
    ParseConfig,
    syntax::{combinator::GreenElement, input::Input},
};

#[path = "document_source_selection.rs"]
mod document_source_selection;
#[path = "elements_bridge_query_json.rs"]
mod elements_bridge_query_json;

pub fn to_ast<N: AstNode>(
    parser: impl Fn(Input) -> IResult<Input, GreenElement, ()>,
) -> impl Fn(&str) -> N {
    move |s: &str| {
        let input = Input {
            s,
            c: &ParseConfig::default(),
        };
        let element = parser(input).unwrap().1;
        let node = element.into_node().unwrap();
        let node = SyntaxNode::<N::Language>::new_root(node);
        AstNode::cast(node).unwrap()
    }
}
