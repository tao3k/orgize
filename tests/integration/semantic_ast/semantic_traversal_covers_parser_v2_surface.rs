use std::collections::BTreeSet;

use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AstMut, AstRef},
    Org,
};

#[test]
fn semantic_traversal_covers_parser_v2_surface() {
    let mut doc = Org::parse(
        r#"#+TITLE: Traversal
#+ARCHIVE: archive.org::* Archived
#+INCLUDE: "chapter.org" :minlevel 2
#+MACRO: greet Hello $1
#+CAPTION: Link caption
[[https://example.com/image.png][captioned]]

* Heading :tag:
:PROPERTIES:
:CUSTOM_ID: heading-id
:DIR: attachments
:END:
Paragraph with <<target>> <<<radio>>> {{{greet(World)}}} [cite:@doe; see @roe] [[#heading-id][self]] <2026-05-10 Sun>.

*************** TODO Inline :inline:
:PROPERTIES:
:ID: inline
:END:
Body.
*************** END

- term :: item

| A | B |
|---+---|
| 1 | 2 |

#+begin_src rust -n -r -l "ref:%s"
fn main() {} // (ref:main)
#+end_src

[fn:note] Footnote body
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let seen = doc.fold(BTreeSet::new(), |mut seen, node| {
        seen.insert(ast_ref_name(node));
        seen
    });
    assert_traversal_surface(&seen);

    let mut seen_mut = BTreeSet::new();
    doc.visit_mut(|node| {
        seen_mut.insert(ast_mut_name(node));
    });
    assert_traversal_surface(&seen_mut);
}

fn assert_traversal_surface(seen: &BTreeSet<&'static str>) {
    for expected in [
        "Document",
        "IncludeDirective",
        "MacroDefinition",
        "TargetDefinition",
        "FootnoteEntry",
        "ArchiveLocation",
        "AttachmentDirectory",
        "Section",
        "Property",
        "Keyword",
        "Element",
        "Inlinetask",
        "InlinetaskEnd",
        "ListItem",
        "TableRow",
        "TableCell",
        "BlockLine",
        "Object",
    ] {
        assert!(
            seen.contains(expected),
            "expected traversal to visit {expected}, saw {seen:#?}"
        );
    }
}

fn ast_ref_name(node: AstRef<'_, orgize::ast::ParsedAnnotation>) -> &'static str {
    match node {
        AstRef::Document(_) => "Document",
        AstRef::IncludeDirective(_) => "IncludeDirective",
        AstRef::MacroDefinition(_) => "MacroDefinition",
        AstRef::TargetDefinition(_) => "TargetDefinition",
        AstRef::FootnoteEntry(_) => "FootnoteEntry",
        AstRef::ArchiveLocation(_) => "ArchiveLocation",
        AstRef::AttachmentDirectory(_) => "AttachmentDirectory",
        AstRef::Section(_) => "Section",
        AstRef::Property(_) => "Property",
        AstRef::Keyword(_) => "Keyword",
        AstRef::Element(_) => "Element",
        AstRef::Inlinetask(_) => "Inlinetask",
        AstRef::InlinetaskEnd(_) => "InlinetaskEnd",
        AstRef::ListItem(_) => "ListItem",
        AstRef::TableRow(_) => "TableRow",
        AstRef::TableCell(_) => "TableCell",
        AstRef::BlockLine(_) => "BlockLine",
        AstRef::Object(_) => "Object",
    }
}

fn ast_mut_name(node: AstMut<'_, orgize::ast::ParsedAnnotation>) -> &'static str {
    match node {
        AstMut::Document(_) => "Document",
        AstMut::IncludeDirective(_) => "IncludeDirective",
        AstMut::MacroDefinition(_) => "MacroDefinition",
        AstMut::TargetDefinition(_) => "TargetDefinition",
        AstMut::FootnoteEntry(_) => "FootnoteEntry",
        AstMut::ArchiveLocation(_) => "ArchiveLocation",
        AstMut::AttachmentDirectory(_) => "AttachmentDirectory",
        AstMut::Section(_) => "Section",
        AstMut::Property(_) => "Property",
        AstMut::Keyword(_) => "Keyword",
        AstMut::Element(_) => "Element",
        AstMut::Inlinetask(_) => "Inlinetask",
        AstMut::InlinetaskEnd(_) => "InlinetaskEnd",
        AstMut::ListItem(_) => "ListItem",
        AstMut::TableRow(_) => "TableRow",
        AstMut::TableCell(_) => "TableCell",
        AstMut::BlockLine(_) => "BlockLine",
        AstMut::Object(_) => "Object",
    }
}
