//! Scheme-AOT headline state and source-backed property projection.

macro_rules! check_org_aot_headline_state {
    ($document:expr, $record:expr, $title:expr => $expected:expr) => {{
        assert_eq!($record.kind, "headline");
        assert_eq!($record.field("title"), Some($title));
        assert_eq!($document.headline_todo_type($record.id), $expected);
    }};
}

#[test]
fn scheme_headline_tags_project_without_rust_headline_scanning() {
    let source = "* TODO Plan :agent:plan:\n* Plan :bad::\n* 标题 :中文:\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("Scheme headline tags parse");
    assert_eq!(document.syntax().to_string(), source);
    let headlines = document
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .collect::<Vec<_>>();
    assert_eq!(headlines.len(), 3);
    assert_eq!(headlines[0].field("title"), Some("TODO Plan :agent:plan:"));
    assert_eq!(
        headlines[0].values("tag").collect::<Vec<_>>(),
        ["agent", "plan"]
    );
    assert_eq!(headlines[1].field("title"), Some("Plan :bad::"));
    assert_eq!(headlines[1].values("tag").count(), 0);
    assert_eq!(headlines[2].values("tag").collect::<Vec<_>>(), ["中文"]);
    assert_eq!(
        document.headline_display_title(headlines[0].id).as_deref(),
        Some("Plan")
    );
    assert_eq!(
        document.headline_display_title(headlines[1].id).as_deref(),
        Some("Plan :bad::")
    );
}

#[test]
fn scheme_aot_headline_state_classifies_projected_element_titles() {
    let source = "#+SEQ_TODO: WAIT(w) | DONE(d)\n#+TYP_TODO: HOLD(h) | FINISHED(f)\n* WAIT Parent\n** DONE Child\n* TODO prose\n* HOLD Review\n* FINISHED Shipped\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("headline state reads production Element records");
    let headlines: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .collect();
    assert_eq!(headlines.len(), 5);
    check_org_aot_headline_state!(
        document, headlines[0], "WAIT Parent" => Some("todo")
    );
    check_org_aot_headline_state!(
        document, headlines[1], "DONE Child" => Some("done")
    );
    check_org_aot_headline_state!(document, headlines[2], "TODO prose" => None);
    check_org_aot_headline_state!(
        document, headlines[3], "HOLD Review" => Some("todo")
    );
    check_org_aot_headline_state!(
        document, headlines[4], "FINISHED Shipped" => Some("done")
    );
    assert_eq!(document.headline_todo_type(0), None);
    assert_eq!(
        document.headline_todo_keyword(headlines[0].id),
        Some("WAIT".into())
    );
    assert_eq!(
        document.headline_todo_keyword(headlines[1].id),
        Some("DONE".into())
    );
    assert_eq!(document.headline_todo_keyword(headlines[2].id), None);
    assert_eq!(
        document.headline_todo_keyword(headlines[3].id),
        Some("HOLD".into())
    );
    assert_eq!(
        document.headline_todo_keyword(headlines[4].id),
        Some("FINISHED".into())
    );
    assert_eq!(
        document.headline_content_after_todo(headlines[0].id),
        Some("Parent".into())
    );
    assert_eq!(
        document.headline_content_after_todo(headlines[2].id),
        Some("TODO prose".into())
    );
    assert_eq!(document.headline_content_after_todo(0), None);
}

#[test]
fn headline_state_uses_defaults_only_without_document_directives() {
    let document = orgize::org_aot::parse_org_aot("* TODO Open\n* DONE Closed\n* WAIT Plain\n")
        .expect("default TODO states derive from Org Element graph");
    let headlines: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .collect();
    check_org_aot_headline_state!(document, headlines[0], "TODO Open" => Some("todo"));
    check_org_aot_headline_state!(document, headlines[1], "DONE Closed" => Some("done"));
    check_org_aot_headline_state!(document, headlines[2], "WAIT Plain" => None);
}
