use rowan::{GreenNode, GreenToken, Language, NodeOrToken};

use super::{OrgLanguage, SyntaxKind};

pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;

#[inline]
pub(crate) fn token(kind: SyntaxKind, input: &str) -> GreenElement {
    GreenElement::Token(GreenToken::new(OrgLanguage::kind_to_raw(kind), input))
}

#[inline]
pub(crate) fn node<I>(kind: SyntaxKind, children: I) -> GreenElement
where
    I: IntoIterator<Item = GreenElement>,
    I::IntoIter: ExactSizeIterator,
{
    GreenElement::Node(GreenNode::new(OrgLanguage::kind_to_raw(kind), children))
}
