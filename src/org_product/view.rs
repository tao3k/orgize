//! Read-only views over source offsets and links supplied by the Scheme Fold.

use std::{fmt, sync::Arc};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Index into the published syntax-kind catalog.
pub struct SyntaxKind(pub u16);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// UTF-8 byte offset in the original document.
pub struct TextSize(pub u32);

impl From<u32> for TextSize {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<TextSize> for u32 {
    fn from(value: TextSize) -> Self {
        value.0
    }
}

impl From<TextSize> for usize {
    fn from(value: TextSize) -> Self {
        value.0 as Self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Half-open range in the original document.
pub struct TextRange {
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    pub fn new(start: TextSize, end: TextSize) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    pub fn empty(at: TextSize) -> Self {
        Self::new(at, at)
    }

    pub fn start(self) -> TextSize {
        self.start
    }

    pub fn end(self) -> TextSize {
        self.end
    }

    pub(crate) fn slice(self, source: &str) -> &str {
        &source[usize::from(self.start)..usize::from(self.end)]
    }
}

#[derive(Debug)]
pub(crate) struct SyntaxRow {
    pub kind: SyntaxKind,
    pub token: bool,
    pub range: TextRange,
    pub parent: Option<usize>,
    pub subtree_end: usize,
}

#[derive(Debug)]
struct SourceIndex {
    source: Box<str>,
    rows: Vec<SyntaxRow>,
}

#[derive(Clone, Debug)]
struct Entry {
    index: Arc<SourceIndex>,
    id: usize,
}

impl Entry {
    fn row(&self) -> &SyntaxRow {
        &self.index.rows[self.id]
    }

    fn at(&self, id: usize) -> Self {
        Self {
            index: Arc::clone(&self.index),
            id,
        }
    }

    fn element(&self) -> SyntaxElement {
        if self.row().token {
            SyntaxElement::Token(SyntaxToken(self.clone()))
        } else {
            SyntaxElement::Node(SyntaxNode(self.clone()))
        }
    }
}

#[derive(Clone)]
/// Node in the Scheme-produced navigation index.
pub struct SyntaxNode(Entry);

impl PartialEq for SyntaxNode {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id && Arc::ptr_eq(&self.0.index, &other.0.index)
    }
}

impl Eq for SyntaxNode {}

#[derive(Clone)]
/// Source-backed token in the Scheme-produced navigation index.
pub struct SyntaxToken(Entry);

macro_rules! source_entry {
    ($entry:ident) => {
        impl $entry {
            pub fn kind(&self) -> SyntaxKind {
                self.0.row().kind
            }

            pub fn text_range(&self) -> TextRange {
                self.0.row().range
            }

            pub fn text(&self) -> &str {
                self.text_range().slice(&self.0.index.source)
            }

            pub fn parent(&self) -> Option<SyntaxNode> {
                self.0.row().parent.map(|id| SyntaxNode(self.0.at(id)))
            }
        }

        impl fmt::Display for $entry {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.text())
            }
        }

        impl fmt::Debug for $entry {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($entry))
                    .field("kind", &self.kind())
                    .field("range", &self.text_range())
                    .finish()
            }
        }
    };
}

source_entry!(SyntaxNode);
source_entry!(SyntaxToken);

#[derive(Clone, Debug)]
/// Node or token in producer order.
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

impl SyntaxElement {
    pub fn into_node(self) -> Option<SyntaxNode> {
        match self {
            Self::Node(node) => Some(node),
            Self::Token(_) => None,
        }
    }

    pub fn into_token(self) -> Option<SyntaxToken> {
        match self {
            Self::Token(token) => Some(token),
            Self::Node(_) => None,
        }
    }

    pub fn kind(&self) -> SyntaxKind {
        match self {
            Self::Node(node) => node.kind(),
            Self::Token(token) => token.kind(),
        }
    }

    pub fn text_range(&self) -> TextRange {
        match self {
            Self::Node(node) => node.text_range(),
            Self::Token(token) => token.text_range(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Entry or exit of a navigation row in a preorder traversal.
pub enum WalkEvent<T> {
    Enter(T),
    Leave(T),
}

impl SyntaxNode {
    pub(crate) fn from_rows(source: &str, rows: Vec<SyntaxRow>) -> Self {
        Self(Entry {
            index: Arc::new(SourceIndex {
                source: source.into(),
                rows,
            }),
            id: 0,
        })
    }

    pub fn children_with_tokens(&self) -> impl Iterator<Item = SyntaxElement> + use<> {
        let owner = self.0.clone();
        let mut id = owner.id + 1;
        let end = owner.row().subtree_end;
        std::iter::from_fn(move || {
            if id == end {
                return None;
            }
            let child = owner.at(id);
            id = child.row().subtree_end;
            Some(child.element())
        })
    }

    pub fn children(&self) -> impl Iterator<Item = Self> + use<> {
        self.children_with_tokens()
            .filter_map(SyntaxElement::into_node)
    }

    pub fn first_child(&self) -> Option<Self> {
        self.children().next()
    }

    pub fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement> + use<> {
        let owner = self.0.clone();
        (owner.id..owner.row().subtree_end).map(move |id| owner.at(id).element())
    }

    pub fn descendants(&self) -> impl Iterator<Item = Self> + use<> {
        self.descendants_with_tokens()
            .filter_map(SyntaxElement::into_node)
    }

    pub fn preorder_with_tokens(&self) -> impl Iterator<Item = WalkEvent<SyntaxElement>> + use<> {
        let root = self.0.clone();
        let mut next = Some((root.id, true));
        std::iter::from_fn(move || {
            let (id, entering) = next?;
            let entry = root.at(id);
            let row = entry.row();
            next = if entering {
                if id + 1 < row.subtree_end {
                    Some((id + 1, true))
                } else {
                    Some((id, false))
                }
            } else if id == root.id {
                None
            } else {
                let parent = row.parent.expect("admitted child has a parent");
                if row.subtree_end < root.index.rows[parent].subtree_end {
                    Some((row.subtree_end, true))
                } else {
                    Some((parent, false))
                }
            };
            Some(if entering {
                WalkEvent::Enter(entry.element())
            } else {
                WalkEvent::Leave(entry.element())
            })
        })
    }
}
