//! Lossless syntax tree traversal traits and helpers.

use crate::syntax::{SyntaxElement, SyntaxKind};
use crate::syntax_ast::{
    BabelCall, Bold, CenterBlock, Code, Comment, CommentBlock, Cookie, DynBlock, Entity,
    ExampleBlock, ExportBlock, FixedWidth, FnDef, FnRef, Headline, InlineCall, InlineSrc, Italic,
    LatexEnvironment, LatexFragment, LineBreak, Macros, OrgTable, OrgTableCell, OrgTableRow,
    Paragraph, PropertyDrawer, QuoteBlock, RadioTarget, Rule, Snippet, SourceBlock, SpecialBlock,
    Strike, Subscript, Superscript, SyntaxCitation as Citation, SyntaxClock as Clock,
    SyntaxDocument as Document, SyntaxDrawer as Drawer, SyntaxKeyword as Keyword,
    SyntaxLink as Link, SyntaxList as List, SyntaxListItem as ListItem, SyntaxSection as Section,
    SyntaxTimestamp as Timestamp, Target, Token, Underline, Verbatim, VerseBlock,
};

#[cfg(feature = "syntax-org-fc")]
use crate::syntax_ast::Cloze;
use rowan::ast::AstNode;

use super::event::{Container, Event};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
enum TraversalControl {
    Up,
    Stop,
    Skip,
    #[default]
    Continue,
}

#[derive(Default)]
/// Mutable control state passed through syntax traversal callbacks.
pub struct TraversalContext {
    control: TraversalControl,
}

impl TraversalContext {
    /// Stops traversal completely
    pub fn stop(&mut self) {
        self.control = TraversalControl::Stop;
    }
    /// Skips traversal of the current node's siblings
    pub fn up(&mut self) {
        self.control = TraversalControl::Up;
    }
    /// Skips traversal of the current node's descendants
    pub fn skip(&mut self) {
        self.control = TraversalControl::Skip;
    }
    /// Continues traversal
    pub fn r#continue(&mut self) {
        self.control = TraversalControl::Continue;
    }
}

/// A trait for enumerating org syntax tree
///
/// ### `TraversalContext`
///
/// `TraversalContext` can be used to control the traversal.
///
/// For example, `ctx.skip()` will skips the traversal for current
/// element and its descendants and improve the traversal performance.
///
/// ```rust
/// use orgize::{
///     export::{Container, Event, HtmlExport, TraversalContext, Traverser},
///     Org,
/// };
/// use slugify::slugify;
///
/// #[derive(Default)]
/// struct Toc(HtmlExport);
///
/// impl Traverser for Toc {
///     fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
///         match event {
///             Event::Enter(Container::Headline(headline)) => {
///                 let title = headline.title().map(|e| e.to_string()).collect::<String>();
///                 self.0.push_str(&format!("<a href='#{}'>", slugify!(&title)));
///                 for elem in headline.title() {
///                     self.element(elem, ctx);
///                 }
///                 self.0.push_str("</a>");
///                 if headline.headlines().count() > 0 {
///                     self.0.push_str("<ul>");
///                 }
///             }
///             Event::Leave(Container::Headline(headline)) => {
///                 if headline.headlines().count() > 0 {
///                     self.0.push_str("</ul>");
///                 }
///             }
///             Event::Enter(Container::Section(_)) | Event::Leave(Container::Section(_)) => ctx.skip(),
///             Event::Enter(Container::Document(_)) | Event::Leave(Container::Document(_)) => {}
///             _ => self.0.event(event, ctx),
///         }
///     }
/// }
///
/// let org = Org::parse(r#"
/// * heading 1
/// section 1
/// ** heading 1.1
/// ** heading 1.2
/// * heading 2
/// section 2
/// * heading 3
/// **** heading 3.1"#);
/// let mut toc = Toc::default();
/// org.traverse(&mut toc);
/// assert_eq!(toc.0.finish(), "\
/// <a href='#heading-1'>heading 1</a>\
/// <ul><a href='#heading-1-1'>heading 1.1</a><a href='#heading-1-2'>heading 1.2</a></ul>\
/// <a href='#heading-2'>heading 2</a>\
/// <a href='#heading-3'>heading 3</a>\
/// <ul><a href='#heading-3-1'>heading 3.1</a></ul>");
/// ```
pub trait Traverser {
    /// Handles traversal event
    fn event(&mut self, event: Event, ctx: &mut TraversalContext);

    fn element(&mut self, element: SyntaxElement, ctx: &mut TraversalContext) {
        macro_rules! take_control {
            () => {
                match ctx.control {
                    TraversalControl::Stop => {
                        ctx.control = TraversalControl::Stop;
                        return;
                    }
                    TraversalControl::Up => {
                        ctx.control = TraversalControl::Skip;
                        return;
                    }
                    TraversalControl::Skip => {
                        ctx.control = TraversalControl::Continue;
                        return;
                    }
                    TraversalControl::Continue => {}
                }
            };
        }

        match element {
            SyntaxElement::Node(node) => {
                macro_rules! walk {
                    ($ast:ident) => {{
                        debug_assert!($ast::can_cast(node.kind()));
                        let node = $ast { syntax: node };
                        self.event(Event::Enter(Container::$ast(node.clone())), ctx);
                        take_control!();
                        for child in node.syntax.children_with_tokens() {
                            self.element(child, ctx);
                            take_control!();
                        }
                        self.event(Event::Leave(Container::$ast(node.clone())), ctx);
                        take_control!();
                    }};
                    (@$ast:ident) => {{
                        debug_assert!($ast::can_cast(node.kind()));
                        let node = $ast { syntax: node };
                        self.event(Event::$ast(node), ctx);
                        take_control!();
                    }};
                }

                match node.kind() {
                    SyntaxKind::DOCUMENT => walk!(Document),
                    SyntaxKind::HEADLINE => walk!(Headline),
                    SyntaxKind::SECTION => walk!(Section),
                    SyntaxKind::PARAGRAPH => walk!(Paragraph),
                    SyntaxKind::BOLD => walk!(Bold),
                    SyntaxKind::ITALIC => walk!(Italic),
                    SyntaxKind::STRIKE => walk!(Strike),
                    SyntaxKind::UNDERLINE => walk!(Underline),
                    SyntaxKind::LIST => walk!(List),
                    SyntaxKind::LIST_ITEM => walk!(ListItem),
                    SyntaxKind::CODE => walk!(Code),
                    SyntaxKind::INLINE_CALL => walk!(@InlineCall),
                    SyntaxKind::INLINE_SRC => walk!(@InlineSrc),
                    SyntaxKind::RULE => walk!(@Rule),
                    SyntaxKind::VERBATIM => walk!(Verbatim),
                    SyntaxKind::SPECIAL_BLOCK => walk!(SpecialBlock),
                    SyntaxKind::QUOTE_BLOCK => walk!(QuoteBlock),
                    SyntaxKind::CENTER_BLOCK => walk!(CenterBlock),
                    SyntaxKind::VERSE_BLOCK => walk!(VerseBlock),
                    SyntaxKind::COMMENT_BLOCK => walk!(CommentBlock),
                    SyntaxKind::EXAMPLE_BLOCK => walk!(ExampleBlock),
                    SyntaxKind::EXPORT_BLOCK => walk!(ExportBlock),
                    SyntaxKind::SOURCE_BLOCK => walk!(SourceBlock),
                    SyntaxKind::BABEL_CALL => walk!(BabelCall),
                    SyntaxKind::CLOCK => walk!(@Clock),
                    SyntaxKind::COOKIE => walk!(@Cookie),
                    SyntaxKind::CITATION => walk!(@Citation),
                    SyntaxKind::RADIO_TARGET => walk!(RadioTarget),
                    SyntaxKind::DRAWER => walk!(Drawer),
                    SyntaxKind::DYN_BLOCK => walk!(DynBlock),
                    SyntaxKind::FN_DEF => walk!(FnDef),
                    SyntaxKind::FN_REF => walk!(FnRef),
                    SyntaxKind::MACROS => walk!(@Macros),
                    SyntaxKind::SNIPPET => walk!(@Snippet),
                    SyntaxKind::TIMESTAMP_ACTIVE
                    | SyntaxKind::TIMESTAMP_INACTIVE
                    | SyntaxKind::TIMESTAMP_DIARY => walk!(@Timestamp),
                    SyntaxKind::TARGET => walk!(Target),
                    SyntaxKind::COMMENT => walk!(Comment),
                    SyntaxKind::FIXED_WIDTH => walk!(FixedWidth),
                    SyntaxKind::ORG_TABLE => walk!(OrgTable),
                    SyntaxKind::ORG_TABLE_RULE_ROW | SyntaxKind::ORG_TABLE_STANDARD_ROW => {
                        walk!(OrgTableRow)
                    }
                    SyntaxKind::ORG_TABLE_CELL => walk!(OrgTableCell),
                    SyntaxKind::LINK => walk!(Link),
                    SyntaxKind::LATEX_FRAGMENT => walk!(@LatexFragment),
                    SyntaxKind::LATEX_ENVIRONMENT => walk!(@LatexEnvironment),
                    SyntaxKind::ENTITY => walk!(@Entity),
                    SyntaxKind::LINE_BREAK => walk!(@LineBreak),
                    SyntaxKind::SUPERSCRIPT => walk!(Superscript),
                    SyntaxKind::SUBSCRIPT => walk!(Subscript),
                    SyntaxKind::KEYWORD => walk!(Keyword),
                    SyntaxKind::PROPERTY_DRAWER => walk!(PropertyDrawer),
                    #[cfg(feature = "syntax-org-fc")]
                    SyntaxKind::CLOZE => walk!(@Cloze),
                    SyntaxKind::BLOCK_CONTENT | SyntaxKind::LIST_ITEM_CONTENT => {
                        for child in node.children_with_tokens() {
                            self.element(child, ctx);
                            take_control!();
                        }
                    }
                    _ => {}
                }
            }
            SyntaxElement::Token(token) => {
                if token.kind() == SyntaxKind::TEXT {
                    self.event(Event::Text(Token(token)), ctx);
                    take_control!();
                }
            }
        };
    }
}

/// Traverser adapter backed by a closure that receives events.
pub struct FromFn<F: FnMut(Event)>(F);

impl<F: FnMut(Event)> Traverser for FromFn<F> {
    fn event(&mut self, event: Event, _: &mut TraversalContext) {
        (self.0)(event)
    }
}

/// Traverser adapter backed by a closure that can also control traversal.
pub struct FromFnWithCtx<F: FnMut(Event, &mut TraversalContext)>(F);

impl<F: FnMut(Event, &mut TraversalContext)> Traverser for FromFnWithCtx<F> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        (self.0)(event, ctx)
    }
}

/// A helper for creating traverser
///
/// ```rust
/// use orgize::{
///     export::{from_fn, Container, Event, Traverser},
///     Org,
/// };
///
/// let mut count = 0;
/// let mut handler = from_fn(|event| {
///     if matches!(event, Event::Enter(Container::Headline(_))) {
///         count += 1;
///     }
/// });
/// Org::parse("* 1\n** 2\n*** 3\n****4").traverse(&mut handler);
/// assert_eq!(count, 3);
/// ```
pub fn from_fn<F: FnMut(Event)>(f: F) -> FromFn<F> {
    FromFn(f)
}

/// A helper for creating traverser
///
/// ```rust
/// use orgize::{
///     export::{from_fn_with_ctx, Container, Event, Traverser},
///     Org,
/// };
///
/// let mut count = 0;
/// let mut handler = from_fn_with_ctx(|event, ctx| {
///     if let Event::Enter(Container::Headline(hdl)) = event {
///         count += 1;
///         if &hdl.title_raw() == "cow" {
///             ctx.stop();
///         }
///     }
/// });
/// Org::parse("* 1\n* cow\n* 3").traverse(&mut handler);
/// assert_eq!(count, 2);
/// ```
pub fn from_fn_with_ctx<F: FnMut(Event, &mut TraversalContext)>(f: F) -> FromFnWithCtx<F> {
    FromFnWithCtx(f)
}
