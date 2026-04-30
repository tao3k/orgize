use orgize::{
    ast::{
        AstMut, AstRef, ElementData, LinkTarget, MarkupKind, ObjectData, ParsedAst, RepeaterKind,
        SourcePosition, TimeUnit, TodoState, WarningKind,
    },
    config::UseSubSuperscript,
    Org, ParseConfig,
};

fn assert_clean_projection(doc: &ParsedAst) {
    assert!(
        doc.diagnostics.is_empty(),
        "unexpected diagnostics: {:#?}",
        doc.diagnostics
    );

    let unknowns = doc.fold(Vec::new(), |mut unknowns, node| {
        match node {
            AstRef::Element(element) => {
                if let ElementData::Unknown { kind, .. } = &element.data {
                    unknowns.push(format!("element:{kind}"));
                }
            }
            AstRef::Object(object) => {
                if let ObjectData::Unknown { kind, .. } = &object.data {
                    unknowns.push(format!("object:{kind}"));
                }
            }
            _ => {}
        }
        unknowns
    });

    assert!(
        unknowns.is_empty(),
        "unexpected semantic unknowns: {unknowns:#?}"
    );
}

#[test]
fn semantic_ast_projection_and_bare_snapshot() {
    let doc = Org::parse(
        r#"#+TITLE: Demo
* TODO Heading :work:
SCHEDULED: <2026-04-30 Thu>
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
Paragraph with *bold*, [[https://example.com][a link]], and <2026-04-30 Thu>.

- [X] item one
- tag :: item two

#+begin_src rust
fn main() {}
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.sections.len(), 1);

    let section = &doc.sections[0];
    assert_eq!(section.level, 1);
    assert_eq!(section.todo.as_ref().unwrap().state, TodoState::Todo);
    assert_eq!(section.raw_title, "Heading ");
    assert_eq!(section.tags, ["work"]);
    assert_eq!(section.anchor.as_deref(), Some("heading-id"));
    let scheduled = section.planning.scheduled.as_ref().unwrap();
    assert_eq!(scheduled.start.as_ref().unwrap().year, 2026);
    assert_eq!(scheduled.start.as_ref().unwrap().month, 4);
    assert_eq!(scheduled.start.as_ref().unwrap().day, 30);
    assert_eq!(
        scheduled.start.as_ref().unwrap().day_name.as_deref(),
        Some("Thu")
    );

    let paragraph = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .expect("paragraph element");
    assert!(paragraph.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert!(paragraph
        .iter()
        .any(|object| matches!(object.data, ObjectData::Link(_))));

    insta::assert_debug_snapshot!("semantic_bare_ast", doc.to_bare());
}

#[test]
fn semantic_ast_covers_current_lossless_projection_surface() {
    let fixtures = [
        "#+TITLE: Demo\n",
        r#"* TODO Heading :tag:
DEADLINE: <2026-05-01 Fri> SCHEDULED: <2026-04-30 Thu> CLOSED: [2026-04-29 Wed]
:PROPERTIES:
:CUSTOM_ID: id
:END:
Body.
"#,
        r#"Paragraph with *bold* /italic/ _underline_ +strike+ ~code~ =verbatim= H_2 x^2 <2026-04-30 Thu> [2026-04-30 Thu] <%%(diary-date 4 30)> @@html:<span>@@ \alpha $x$ <<target>> <<<radio>>> {{{macro(1\,a, two)}}} [fn:note:See /inner/] [cite:@doe2020] src_rust[:exports code]{let x = 1;} call_square(4) [50%]\\
Next.
"#,
        r#"#+ATTR_HTML: :class compact
| A | B |
|---+---|
| 1 | 2 |
#+TBLFM: $1=$2
"#,
        r#"#+begin_quote
quoted
#+end_quote

#+begin_src rust
fn main() {}
#+end_src

#+begin_export html
<b>x</b>
#+end_export
"#,
        r#":DRAWER:
inside
:END:

[fn:note] Footnote body

# comment
: fixed
-----
\begin{equation}
x
\end{equation}
"#,
        "- [X] item\n- term :: description\n",
        "  +---+\n  | a |\n  +---+\n",
    ];

    for fixture in fixtures {
        let doc = Org::parse(fixture).document();
        assert_clean_projection(&doc);
    }
}

#[test]
fn semantic_ast_projects_object_gap_repairs() {
    let doc = Org::parse(
        r#"[[https://example.com][*bold* description]] [2003-09-16 Tue 09:39]--[2003-09-16 Tue 10:39] {{{macro(1\,a, two)}}}"#,
    )
    .document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };

    let link_description = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Link(link) => Some(&link.description),
            _ => None,
        })
        .expect("link object");
    assert!(link_description.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));

    let timestamp = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Timestamp(timestamp) => Some(timestamp),
            _ => None,
        })
        .expect("timestamp object");
    assert!(timestamp.is_range);
    assert_eq!(timestamp.start.as_ref().unwrap().hour, Some(9));
    assert_eq!(timestamp.start.as_ref().unwrap().minute, Some(39));
    assert_eq!(timestamp.end.as_ref().unwrap().hour, Some(10));
    assert_eq!(timestamp.end.as_ref().unwrap().minute, Some(39));

    let macro_arguments = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Macro { name, arguments } if name == "macro" => Some(arguments),
            _ => None,
        })
        .expect("macro object");
    assert_eq!(macro_arguments, &["1,a".to_string(), "two".to_string()]);
}

#[test]
fn semantic_ast_projects_link_metadata() {
    let image_doc = Org::parse("#+CAPTION: Logo\n[[file:/tmp/logo.svg]]").document();

    assert_clean_projection(&image_doc);
    let image_link = match &image_doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("image link"),
        other => panic!("expected paragraph, got {other:#?}"),
    };
    assert_eq!(image_link.path, "file:/tmp/logo.svg");
    assert!(matches!(
        &image_link.target,
        LinkTarget::Uri { protocol, path }
            if protocol == "file" && path == "/tmp/logo.svg"
    ));
    assert!(!image_link.has_description);
    assert!(image_link.is_image);
    assert_eq!(image_link.caption.as_ref().unwrap().key, "CAPTION");
    assert_eq!(image_link.caption.as_ref().unwrap().value, " Logo");

    let doc =
        Org::parse("Links [[#heading][*Jump*]] and [[https://example.com][Site]].").document();

    assert_clean_projection(&doc);
    let links = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .filter_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .collect::<Vec<_>>(),
        other => panic!("expected paragraph, got {other:#?}"),
    };

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].path, "#heading");
    assert!(matches!(
        &links[0].target,
        LinkTarget::Internal(target) if target == "#heading"
    ));
    assert!(links[0].has_description);
    assert_eq!(links[0].raw_description, "*Jump*");
    assert!(links[0].description.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert!(!links[0].is_image);

    assert_eq!(links[1].path, "https://example.com");
    assert!(matches!(
        &links[1].target,
        LinkTarget::Uri { protocol, path }
            if protocol == "https" && path == "//example.com"
    ));
    assert!(links[1].has_description);
    assert_eq!(links[1].raw_description, "Site");
    assert!(!links[1].is_image);
}

#[test]
fn semantic_ast_projects_timestamp_metadata() {
    let doc = Org::parse("SCHEDULED: <2003-09-16 Tue 09:39-10:39 +1w --2d>\n").document();

    assert_clean_projection(&doc);
    let timestamp = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Timestamp(timestamp) => Some(timestamp),
                _ => None,
            })
            .expect("timestamp object"),
        other => panic!("expected paragraph, got {other:#?}"),
    };

    let start = timestamp.start.as_ref().expect("timestamp start");
    assert_eq!(start.year, 2003);
    assert_eq!(start.month, 9);
    assert_eq!(start.day, 16);
    assert_eq!(start.day_name.as_deref(), Some("Tue"));
    assert_eq!(start.hour, Some(9));
    assert_eq!(start.minute, Some(39));

    let end = timestamp.end.as_ref().expect("timestamp range end");
    assert_eq!(end.year, 2003);
    assert_eq!(end.month, 9);
    assert_eq!(end.day, 16);
    assert_eq!(end.day_name.as_deref(), Some("Tue"));
    assert_eq!(end.hour, Some(10));
    assert_eq!(end.minute, Some(39));

    let repeater = timestamp.repeater.as_ref().expect("timestamp repeater");
    assert_eq!(repeater.kind, RepeaterKind::Cumulate);
    assert_eq!(repeater.value, 1);
    assert_eq!(repeater.unit, TimeUnit::Week);

    let warning = timestamp.warning.as_ref().expect("timestamp warning");
    assert_eq!(warning.kind, WarningKind::First);
    assert_eq!(warning.value, 2);
    assert_eq!(warning.unit, TimeUnit::Day);
}

#[test]
fn semantic_ast_projects_citations() {
    let doc = Org::parse(
        "See [cite/text:global *prefix* ; see /also/ @doe2020 p. *42*; cf. @roe2021; global suffix] and [cite/noauthor/bare:@smith].",
    )
    .document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let citations = paragraph
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Citation(citation) => Some(citation),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(citations.len(), 2);
    assert_eq!(citations[0].style, "text");
    assert_eq!(citations[0].variant, "");
    assert!(matches!(
        &citations[0].prefix[0].data,
        ObjectData::Plain(value) if value == "global "
    ));
    assert!(citations[0].prefix.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert_eq!(citations[0].references[0].id, "doe2020");
    assert!(matches!(
        &citations[0].references[0].prefix[0].data,
        ObjectData::Plain(value) if value == "see "
    ));
    assert!(citations[0].references[0]
        .prefix
        .iter()
        .any(|object| matches!(
            object.data,
            ObjectData::Markup {
                kind: MarkupKind::Italic,
                ..
            }
        )));
    assert!(matches!(
        &citations[0].references[0].suffix[0].data,
        ObjectData::Plain(value) if value == "p. "
    ));
    assert!(citations[0].references[0]
        .suffix
        .iter()
        .any(|object| matches!(
            object.data,
            ObjectData::Markup {
                kind: MarkupKind::Bold,
                ..
            }
        )));
    assert_eq!(citations[0].references[1].id, "roe2021");
    assert!(matches!(
        &citations[0].suffix[0].data,
        ObjectData::Plain(value) if value == "global suffix"
    ));

    assert_eq!(citations[1].style, "noauthor");
    assert_eq!(citations[1].variant, "bare");
    assert_eq!(citations[1].references[0].id, "smith");
}

#[test]
fn semantic_citation_affixes_respect_parse_config() {
    let config = ParseConfig {
        use_sub_superscript: UseSubSuperscript::Nil,
        ..Default::default()
    };
    let doc = config.parse("See [cite:@doe x_1].").document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let citation = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Citation(citation) => Some(citation),
            _ => None,
        })
        .expect("citation object");

    assert_eq!(citation.references[0].id, "doe");
    assert_eq!(citation.references[0].suffix.len(), 1);
    assert!(matches!(
        &citation.references[0].suffix[0].data,
        ObjectData::Plain(value) if value == "x_1"
    ));
}

#[test]
fn html_export_preserves_citation_raw_text() {
    let html = Org::parse("See [cite:@doe2020].").to_html();

    assert_eq!(
        html,
        "<main><section><p>See [cite:@doe2020].</p></section></main>"
    );
}

#[test]
fn semantic_ast_projects_table_el() {
    let doc = Org::parse("  +---+\n  | a |\n  +---+\n").document();

    assert_clean_projection(&doc);
    match &doc.children[0].data {
        ElementData::TableEl { raw } => {
            assert!(raw.contains("| a |"));
            assert!(raw.starts_with("  +---+"));
        }
        other => panic!("expected table.el element, got {other:#?}"),
    }
}

#[test]
fn semantic_ast_projects_org_table_formulas() {
    let doc = Org::parse("| a |\n#+TBLFM: $1=$2\n#+tblfm: @2=$3\n").document();

    assert_clean_projection(&doc);
    let table = match &doc.children[0].data {
        ElementData::Table(table) => table,
        other => panic!("expected org table, got {other:#?}"),
    };

    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.formulas.len(), 2);
    assert_eq!(table.formulas[0].key, "TBLFM");
    assert_eq!(table.formulas[0].value, " $1=$2");
    assert_eq!(table.formulas[1].key, "tblfm");
    assert_eq!(table.formulas[1].value, " @2=$3");

    let formula_keyword_count = doc.fold(0usize, |count, node| match node {
        AstRef::Keyword(keyword) if keyword.key.eq_ignore_ascii_case("TBLFM") => count + 1,
        _ => count,
    });
    assert_eq!(formula_keyword_count, 2);
}

#[cfg(feature = "syntax-org-fc")]
#[test]
fn semantic_ast_projects_cloze_objects_with_metadata() {
    let doc = Org::parse("{{*text*}{hint}@card-id}").document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let cloze = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Cloze {
                text,
                raw_text,
                hint,
                id,
                raw,
            } => Some((text, raw_text, hint, id, raw)),
            _ => None,
        })
        .expect("cloze object");

    assert_eq!(cloze.1, "*text*");
    assert_eq!(cloze.2.as_deref(), Some("hint"));
    assert_eq!(cloze.3.as_deref(), Some("card-id"));
    assert_eq!(cloze.4, "{{*text*}{hint}@card-id}");
    assert!(cloze.0.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}

#[test]
fn annotations_map_and_fold_work_across_the_tree() {
    let doc = Org::parse("* DONE A\nBody with /italic/ text.").document();

    assert_eq!(doc.ann.start.line, 1);
    assert_eq!(doc.ann.start.column, 1);
    assert_eq!(doc.sections[0].ann.start.line, 1);
    assert_eq!(doc.sections[0].children[0].ann.start.line, 2);

    let object_count = doc.fold(0usize, |count, node| match node {
        AstRef::Object(_) => count + 1,
        _ => count,
    });
    assert!(object_count >= 3);

    let ranges = doc.map_ann(|ann| ann.range);
    assert_eq!(ranges.ann, doc.ann.range);

    let bare = doc
        .try_map_ann(|_| Ok::<_, std::convert::Infallible>(()))
        .unwrap();
    assert_eq!(bare, doc.to_bare());
}

#[test]
fn semantic_annotations_handle_parser_line_endings_and_utf8_columns() {
    let doc = Org::parse("* A\réé *bold*").document();

    assert_clean_projection(&doc);
    let paragraph = &doc.sections[0].children[0];
    assert_eq!(paragraph.ann.start, SourcePosition { line: 2, column: 1 });

    let objects = match &paragraph.data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let bold = objects
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
        .expect("bold object");

    assert_eq!(bold.ann.start, SourcePosition { line: 2, column: 4 });
    assert_eq!(
        bold.ann.end,
        SourcePosition {
            line: 2,
            column: 10
        }
    );
}

#[test]
fn semantic_traversal_supports_exporter_and_indexer_shapes() {
    let doc = Org::parse(
        r#"* Export Me
Paragraph with [[https://example.com][link]] and <2026-04-30 Thu>.

#+begin_quote
quoted
#+end_quote

- one
- two

| A | B |
|---+---|
| 1 | 2 |

[fn:note] Footnote body
"#,
    )
    .document();

    assert_clean_projection(&doc);

    #[derive(Default)]
    struct TraversalShape {
        headlines: Vec<String>,
        paragraphs: usize,
        blocks: usize,
        links: usize,
        timestamps: usize,
        list_items: usize,
        table_rows: usize,
        table_cells: usize,
        footnotes: Vec<String>,
    }

    let shape = doc.fold(TraversalShape::default(), |mut shape, node| {
        match node {
            AstRef::Section(section) => shape.headlines.push(section.raw_title.clone()),
            AstRef::Element(element) => match &element.data {
                ElementData::Paragraph(_) => shape.paragraphs += 1,
                ElementData::Block(_) => shape.blocks += 1,
                ElementData::FootnoteDef(definition) => {
                    shape.footnotes.push(definition.label.clone());
                }
                _ => {}
            },
            AstRef::Object(object) => match &object.data {
                ObjectData::Link(_) => shape.links += 1,
                ObjectData::Timestamp(_) => shape.timestamps += 1,
                _ => {}
            },
            AstRef::ListItem(_) => shape.list_items += 1,
            AstRef::TableRow(_) => shape.table_rows += 1,
            AstRef::TableCell(_) => shape.table_cells += 1,
            _ => {}
        }
        shape
    });

    assert_eq!(shape.headlines, ["Export Me".to_string()]);
    assert_eq!(shape.paragraphs, 5);
    assert_eq!(shape.blocks, 1);
    assert_eq!(shape.links, 1);
    assert_eq!(shape.timestamps, 1);
    assert_eq!(shape.list_items, 2);
    assert_eq!(shape.table_rows, 3);
    assert_eq!(shape.table_cells, 4);
    assert_eq!(shape.footnotes, ["note".to_string()]);
}

#[test]
fn traversal_visits_annotation_bearing_metadata_nodes() {
    let mut doc = Org::parse(
        r#"#+TITLE: Demo
#+AUTHOR: Author
* Heading
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
#+ATTR_HTML: :class compact
|   A |   B |
|-----+-----|
|   1 |   2 |

- tag :: item
"#,
    )
    .document();

    assert!(doc.diagnostics.is_empty());

    #[derive(Default)]
    struct Counts {
        keywords: usize,
        properties: usize,
        list_items: usize,
        table_rows: usize,
        table_cells: usize,
    }

    let counts = doc.fold(Counts::default(), |mut counts, node| {
        match node {
            AstRef::Keyword(_) => counts.keywords += 1,
            AstRef::Property(_) => counts.properties += 1,
            AstRef::ListItem(_) => counts.list_items += 1,
            AstRef::TableRow(_) => counts.table_rows += 1,
            AstRef::TableCell(_) => counts.table_cells += 1,
            _ => {}
        }
        counts
    });

    assert_eq!(counts.keywords, 3);
    assert_eq!(counts.properties, 1);
    assert_eq!(counts.list_items, 1);
    assert_eq!(counts.table_rows, 3);
    assert_eq!(counts.table_cells, 4);

    doc.visit_mut(|node| {
        if let AstMut::Keyword(keyword) = node {
            if keyword.key == "TITLE" {
                keyword.value = " Changed".into();
            }
        }
    });
    assert_eq!(
        doc.children
            .iter()
            .find_map(|element| match &element.data {
                ElementData::Keyword(keyword) if keyword.key == "TITLE" => Some(&keyword.value),
                _ => None,
            })
            .map(String::as_str),
        Some(" Changed")
    );
}

#[test]
fn semantic_ast_projects_inline_babel_and_footnote_details() {
    let doc = Org::parse(
        r#"call_square[:results output](4)[:results html] and src_rust[:exports code]{let x = 1;} and [fn:note:See *bold* text]."#,
    )
    .document();

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
            ObjectData::FootnoteRef { label, definition } => Some((label, definition)),
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
}

#[test]
fn semantic_ast_projects_footnote_definition_label_and_body() {
    let doc = Org::parse("#+CAPTION: A note\n[fn:WORD-1] See *bold* text\n").document();

    assert!(doc.diagnostics.is_empty());
    let element = &doc.children[0];
    assert_eq!(element.affiliated_keywords[0].key, "CAPTION");
    assert_eq!(element.affiliated_keywords[0].value, " A note");

    let definition = match &element.data {
        ElementData::FootnoteDef(definition) => definition,
        _ => panic!("expected footnote definition"),
    };
    assert_eq!(definition.label, "WORD-1");

    let body = match &definition.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected footnote body paragraph"),
    };
    assert!(body.iter().any(|object| matches!(
        &object.data,
        ObjectData::Plain(value) if value.contains("*bold*")
    )));
}

#[test]
fn semantic_ast_keeps_affiliated_keywords_out_of_paragraph_objects() {
    let doc = Org::parse("#+ATTR_HTML: :width 300px\n[[./img/a.jpg]]").document();

    assert!(doc.diagnostics.is_empty());
    let paragraph = &doc.children[0];
    assert_eq!(paragraph.affiliated_keywords.len(), 1);
    assert_eq!(paragraph.affiliated_keywords[0].key, "ATTR_HTML");
    assert_eq!(paragraph.affiliated_keywords[0].value, " :width 300px");

    let objects = match &paragraph.data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };
    assert_eq!(objects.len(), 1);
    assert!(matches!(objects[0].data, ObjectData::Link(_)));
}

#[test]
fn semantic_ast_projects_clean_clock_duration() {
    let doc = Org::parse("* Work\nCLOCK: [2003-09-16 Tue 09:39] =>  1:00\n").document();

    assert!(doc.diagnostics.is_empty());
    let clock = doc.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Clock(clock) => Some(clock),
            _ => None,
        })
        .expect("clock element");

    assert!(clock.value.is_some());
    let timestamp = clock.value.as_ref().unwrap();
    assert_eq!(timestamp.start.as_ref().unwrap().hour, Some(9));
    assert_eq!(timestamp.start.as_ref().unwrap().minute, Some(39));
    assert_eq!(clock.duration.as_deref(), Some("1:00"));
    assert!(clock.raw.contains("=>  1:00"));
}

#[test]
fn existing_html_traversal_still_uses_the_lossless_substrate() {
    let html = Org::parse(
        r#"* title
paragraph with [[https://example.com][link]]

- one
- two

#+begin_quote
quoted
#+end_quote
"#,
    )
    .to_html();

    insta::assert_snapshot!("semantic_ast_html_compatibility", html);
}
