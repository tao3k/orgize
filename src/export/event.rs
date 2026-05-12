//! Event model emitted by lossless syntax tree traversal.

use crate::syntax_ast::{
    AffiliatedKeyword, BabelCall, Bold, CenterBlock, Code, Comment, CommentBlock, Cookie, DynBlock,
    Entity, ExampleBlock, ExportBlock, FixedWidth, FnDef, FnRef, Headline, InlineCall, InlineSrc,
    Italic, LatexEnvironment, LatexFragment, LineBreak, Macros, OrgTable, OrgTableCell,
    OrgTableRow, Paragraph, PropertyDrawer, QuoteBlock, RadioTarget, Rule, Snippet, SourceBlock,
    SpecialBlock, Strike, Subscript, Superscript, SyntaxCitation as Citation, SyntaxClock as Clock,
    SyntaxDocument as Document, SyntaxDrawer as Drawer, SyntaxKeyword as Keyword,
    SyntaxLink as Link, SyntaxList as List, SyntaxListItem as ListItem, SyntaxSection as Section,
    SyntaxTimestamp as Timestamp, TableEl, Target, Token, Underline, Verbatim, VerseBlock,
};

#[cfg(feature = "syntax-org-fc")]
use crate::syntax_ast::Cloze;

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
/// Container event payload for syntax tree enter and leave events.
pub enum Container {
    Document(Document),
    Section(Section),
    Paragraph(Paragraph),
    Headline(Headline),

    OrgTable(OrgTable),
    OrgTableRow(OrgTableRow),
    OrgTableCell(OrgTableCell),
    TableEl(TableEl),

    List(List),
    ListItem(ListItem),
    Drawer(Drawer),
    DynBlock(DynBlock),

    FnDef(FnDef),
    Comment(Comment),
    FixedWidth(FixedWidth),
    SpecialBlock(SpecialBlock),
    QuoteBlock(QuoteBlock),
    CenterBlock(CenterBlock),
    VerseBlock(VerseBlock),
    CommentBlock(CommentBlock),
    ExampleBlock(ExampleBlock),
    ExportBlock(ExportBlock),
    SourceBlock(SourceBlock),

    Link(Link),
    RadioTarget(RadioTarget),
    FnRef(FnRef),
    Target(Target),
    Bold(Bold),
    Strike(Strike),
    Italic(Italic),
    Underline(Underline),
    Verbatim(Verbatim),
    Code(Code),
    Superscript(Superscript),
    Subscript(Subscript),
    BabelCall(BabelCall),
    PropertyDrawer(PropertyDrawer),
    AffiliatedKeyword(AffiliatedKeyword),
    Keyword(Keyword),
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
/// Traversal event emitted by [`Traverser`](crate::export::Traverser).
pub enum Event {
    Enter(Container),
    Leave(Container),

    Text(Token),
    Macros(Macros),
    Cookie(Cookie),
    Citation(Citation),
    InlineCall(InlineCall),
    InlineSrc(InlineSrc),
    Clock(Clock),
    LineBreak(LineBreak),
    Snippet(Snippet),
    Rule(Rule),
    Timestamp(Timestamp),
    LatexFragment(LatexFragment),
    LatexEnvironment(LatexEnvironment),
    Entity(Entity),

    #[cfg(feature = "syntax-org-fc")]
    Cloze(Cloze),
}
