use orgize::{
    export::{from_fn, Container, Event},
    rowan::ast::AstNode,
    Org as Inner,
};
use std::fmt::Write;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Org {
    inner: Inner,
}

#[wasm_bindgen]
impl Org {
    #[wasm_bindgen(constructor)]
    pub fn parse(input: &str) -> Self {
        Org {
            inner: Inner::parse(input),
        }
    }

    pub fn html(&self) -> String {
        self.inner.to_html()
    }

    pub fn org(&self) -> String {
        self.inner.to_org()
    }

    pub fn syntax(&self) -> String {
        format!("{:#?}", self.inner.syntax_document().syntax())
    }

    pub fn update(&mut self, s: &str) {
        self.inner = Inner::parse(s);
    }

    pub fn traverse(&self) -> String {
        let mut result = String::new();
        let mut ident = 0;
        let mut handler = from_fn(|event| {
            let (name, range) = match &event {
                Event::Enter(container) => match container {
                    Container::Document(x) => ("Document", x.text_range()),
                    Container::Section(x) => ("Section", x.text_range()),
                    Container::Paragraph(x) => ("Paragraph", x.text_range()),
                    Container::Headline(x) => ("Headline", x.text_range()),
                    Container::OrgTable(x) => ("OrgTable", x.text_range()),
                    Container::OrgTableRow(x) => ("OrgTableRow", x.text_range()),
                    Container::OrgTableCell(x) => ("OrgTableCell", x.text_range()),
                    Container::TableEl(x) => ("TableEl", x.text_range()),
                    Container::List(x) => ("List", x.text_range()),
                    Container::ListItem(x) => ("ListItem", x.text_range()),
                    Container::Drawer(x) => ("Drawer", x.text_range()),
                    Container::DynBlock(x) => ("DynBlock", x.text_range()),
                    Container::FnDef(x) => ("FnDef", x.text_range()),
                    Container::Comment(x) => ("Comment", x.text_range()),
                    Container::FixedWidth(x) => ("FixedWidth", x.text_range()),
                    Container::SpecialBlock(x) => ("SpecialBlock", x.text_range()),
                    Container::QuoteBlock(x) => ("QuoteBlock", x.text_range()),
                    Container::CenterBlock(x) => ("CenterBlock", x.text_range()),
                    Container::VerseBlock(x) => ("VerseBlock", x.text_range()),
                    Container::CommentBlock(x) => ("CommentBlock", x.text_range()),
                    Container::ExampleBlock(x) => ("ExampleBlock", x.text_range()),
                    Container::ExportBlock(x) => ("ExportBlock", x.text_range()),
                    Container::SourceBlock(x) => ("SourceBlock", x.text_range()),
                    Container::Link(x) => ("Link", x.text_range()),
                    Container::RadioTarget(x) => ("RadioTarget", x.text_range()),
                    Container::FnRef(x) => ("FnRef", x.text_range()),
                    Container::Target(x) => ("Target", x.text_range()),
                    Container::Bold(x) => ("Bold", x.text_range()),
                    Container::Strike(x) => ("Strike", x.text_range()),
                    Container::Italic(x) => ("Italic", x.text_range()),
                    Container::Underline(x) => ("Underline", x.text_range()),
                    Container::Verbatim(x) => ("Verbatim", x.text_range()),
                    Container::Code(x) => ("Code", x.text_range()),
                    Container::Superscript(x) => ("Superscript", x.text_range()),
                    Container::Subscript(x) => ("Subscript", x.text_range()),
                    Container::BabelCall(x) => ("BabelCall", x.text_range()),
                    Container::PropertyDrawer(x) => ("PropertyDrawer", x.text_range()),
                    Container::AffiliatedKeyword(x) => ("AffiliatedKeyword", x.text_range()),
                    Container::Keyword(x) => ("Keyword", x.text_range()),
                    _ => unreachable!(),
                },
                Event::Leave(_) => {
                    ident -= 2;
                    return;
                }
                Event::Text(x) => ("Text", x.text_range()),
                Event::Macros(x) => ("Macros", x.text_range()),
                Event::Cookie(x) => ("Cookie", x.text_range()),
                Event::InlineCall(x) => ("InlineCall", x.text_range()),
                Event::InlineSrc(x) => ("InlineSrc", x.text_range()),
                Event::Clock(x) => ("Clock", x.text_range()),
                Event::LineBreak(x) => ("LineBreak", x.text_range()),
                Event::Snippet(x) => ("Snippet", x.text_range()),
                Event::Rule(x) => ("Rule", x.text_range()),
                Event::Timestamp(x) => ("Timestamp", x.text_range()),
                Event::LatexFragment(x) => ("LatexFragment", x.text_range()),
                Event::LatexEnvironment(x) => ("LatexEnvironment", x.text_range()),
                Event::Entity(x) => ("Entity", x.text_range()),
                _ => unreachable!(),
            };

            let _ = writeln!(
                &mut result,
                "{:ident$}{}@{}..{}",
                "",
                name,
                u32::from(range.start()),
                u32::from(range.end())
            );

            if let Event::Enter(_) = event {
                ident += 2;
            }
        });
        self.inner.traverse(&mut handler);
        result
    }

    #[wasm_bindgen(getter, js_name = "buildTime")]
    pub fn build_time() -> String {
        env!("CARGO_BUILD_TIME").into()
    }

    #[wasm_bindgen(getter, js_name = "gitHash")]
    pub fn git_hash() -> String {
        env!("CARGO_GIT_HASH").into()
    }
}
