use orgize::{
    Org,
    ast::{ElementData, MarkupKind, ObjectData},
};

#[test]
fn semantic_ast_projects_inline_babel_and_footnote_details() {
    let source = r#"call_square[:results output](4)[:results html] and src_rust[:exports code]{let x = 1;} and [fn:note:See *bold* text]."#;
    let doc = Org::parse(source).document();

    assert!(doc.diagnostics.is_empty());
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };

    let inline_call = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                ..
            } => Some((name, arguments, header, end_header)),
            _ => None,
        })
        .expect("inline call object");
    assert_eq!(inline_call.0, "square");
    assert_eq!(inline_call.1, "4");
    assert_eq!(inline_call.2.as_deref(), Some(":results output"));
    assert_eq!(inline_call.3.as_deref(), Some(":results html"));

    let inline_src = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                ..
            } => Some((language, parameters, value)),
            _ => None,
        })
        .expect("inline src object");
    assert_eq!(inline_src.0, "rust");
    assert_eq!(inline_src.1.as_deref(), Some(":exports code"));
    assert_eq!(inline_src.2, "let x = 1;");

    let footnote = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::FootnoteRef {
                label, definition, ..
            } => Some((label, definition)),
            _ => None,
        })
        .expect("footnote ref object");
    assert_eq!(footnote.0.as_deref(), Some("note"));
    assert!(footnote.1.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    let bold = footnote
        .1
        .iter()
        .find(|object| {
            matches!(
                object.data,
                ObjectData::Markup {
                    kind: MarkupKind::Bold,
                    ..
                }
            )
        })
        .expect("bold footnote object");
    assert_eq!(bold.ann.raw, "*bold*");
    assert_eq!(
        usize::from(bold.ann.range.start()),
        source.find("*bold*").expect("source markup")
    );
}

#[test]
fn inline_footnote_fragments_use_graph_children_and_preserve_complex_fallback() {
    let source = "[fn:n:See *大胆* text] [fn:m:See [brackets] *bold*]\n";
    let graph = orgize::org_aot::parse_org_aot(source).expect("Scheme inline graph");
    assert_eq!(graph.syntax().to_string(), source);
    let records = graph.records();
    let notes = records
        .iter()
        .filter(|record| record.kind == "footnote-reference")
        .collect::<Vec<_>>();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].field("definition"), Some("See *大胆* text"));
    assert!(notes[0].field("fragment-fallback").is_none());
    assert!(
        notes[0]
            .child_ids
            .iter()
            .any(|&id| records[id].kind == "bold")
    );
    assert_eq!(
        notes[1].field("fragment-fallback"),
        Some("See [brackets] *bold*")
    );

    let doc = Org::parse(source).document();
    assert!(doc.diagnostics.is_empty());
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };
    let definitions = paragraph
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::FootnoteRef { definition, .. } => Some(definition),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 2);
    for expected in ["*大胆*", "*bold*"] {
        let bold = definitions
            .iter()
            .flat_map(|definition| definition.iter())
            .find(|object| object.ann.raw == expected)
            .expect("bold markup in inline definition");
        assert!(matches!(
            bold.data,
            ObjectData::Markup {
                kind: MarkupKind::Bold,
                ..
            }
        ));
        assert_eq!(
            usize::from(bold.ann.range.start()),
            source.find(expected).unwrap()
        );
    }
}

#[cfg(feature = "syntax-org-fc")]
#[test]
fn nested_cloze_in_footnote_uses_explicit_complex_fragment_fallback() {
    let source = "[fn:n:See {{*term*}} now]\n";
    let graph = orgize::org_aot::parse_org_aot(source).expect("Scheme inline graph");
    let note = graph
        .records()
        .iter()
        .find(|record| record.kind == "footnote-reference")
        .expect("footnote graph record");
    assert_eq!(note.field("fragment-fallback"), Some("See {{*term*}} now"));

    let doc = Org::parse(source).document();
    assert!(doc.diagnostics.is_empty());
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };
    let definition = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::FootnoteRef { definition, .. } => Some(definition),
            _ => None,
        })
        .expect("footnote definition");
    let cloze = definition
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Cloze { text, .. } => Some(text),
            _ => None,
        })
        .expect("nested cloze");
    let bold = cloze
        .iter()
        .find(|object| object.ann.raw == "*term*")
        .expect("nested bold");
    assert!(matches!(
        bold.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    ));
    assert_eq!(
        usize::from(bold.ann.range.start()),
        source.find("*term*").unwrap()
    );
}
