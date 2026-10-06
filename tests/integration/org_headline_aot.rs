//! Scheme-AOT headline state and source-backed property projection.

macro_rules! check_org_aot_headline_state {
    ($document:expr, $record:expr, $title:expr => $expected:expr) => {{
        assert_eq!($record.kind, "headline");
        assert_eq!($record.field("title"), Some($title));
        assert_eq!($document.headline_todo_type($record.id), $expected);
    }};
}

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

fn scheme_aot_headline_state_honors_parse_config_until_file_directive_overrides_it() {
    let configured = {
        let config = orgize::ParseConfig {
            todo_keywords: (vec!["WAIT".into()], vec!["FINISHED".into()]),
            ..Default::default()
        };
        orgize::org_aot::parse_org_aot_with_config(
            "* WAIT Open\n* FINISHED Closed\n* TODO Plain\n",
            &config,
        )
        .expect("configured TODO states are Scheme-AOT projected")
    };
    let headlines = configured.headlines().collect::<Vec<_>>();
    assert_eq!(headlines[0].todo_type(), Some("todo"));
    assert_eq!(headlines[0].display_title().as_deref(), Some("Open"));
    assert_eq!(headlines[1].todo_type(), Some("done"));
    assert_eq!(headlines[2].todo_type(), None);
    assert_eq!(headlines[2].display_title().as_deref(), Some("TODO Plain"));

    let config = orgize::ParseConfig {
        todo_keywords: (vec!["WAIT".into()], vec!["FINISHED".into()]),
        ..Default::default()
    };
    let declared = orgize::org_aot::parse_org_aot_with_config(
        "#+SEQ_TODO: HOLD | DONE\n* WAIT Plain\n* HOLD Open\n* DONE Closed\n",
        &config,
    )
    .expect("file-local declarations override configured TODO states");
    let headlines = declared.headlines().collect::<Vec<_>>();
    assert_eq!(headlines[0].todo_type(), None);
    assert_eq!(headlines[1].todo_type(), Some("todo"));
    assert_eq!(headlines[2].todo_type(), Some("done"));
}

fn scheme_headline_properties_are_stable_across_repeated_queries() {
    let source = "#+SEQ_TODO: WAIT | DONE\n* WAIT [#A] Review :work:\n* TODO Plain\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme AOT projects custom headline properties");
    let headlines = document
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .collect::<Vec<_>>();

    for _ in 0..3 {
        assert_eq!(document.headline_todo_type(headlines[0].id), Some("todo"));
        assert_eq!(
            document.headline_todo_keyword(headlines[0].id).as_deref(),
            Some("WAIT")
        );
        assert_eq!(
            document
                .headline_content_after_todo(headlines[0].id)
                .as_deref(),
            Some("[#A] Review :work:")
        );
        assert_eq!(
            document.headline_display_title(headlines[0].id).as_deref(),
            Some("Review")
        );
        assert_eq!(document.headline_todo_keyword(headlines[1].id), None);
        assert_eq!(
            document.headline_display_title(headlines[1].id).as_deref(),
            Some("TODO Plain")
        );
        assert_eq!(
            document.headline_display_title(document.records().len()),
            None
        );
    }
}

fn scheme_priority_cookie_validation_preserves_malformed_headlines() {
    macro_rules! check_display_title {
        ($($source:expr => $expected:expr),+ $(,)?) => {
            $(
                let source = $source;
                let document = orgize::org_aot::parse_org_aot(source)
                    .expect("Scheme priority headline parses");
                assert_eq!(document.syntax().to_string(), source);
                let headline = document.records().iter()
                    .find(|record| record.kind == "headline")
                    .expect("headline Element");
                assert_eq!(document.headline_display_title(headline.id).as_deref(),
                           Some($expected), "source: {source:?}");
            )+
        };
    }
    check_display_title!(
        "* [#A] Work\n" => "Work",
        "* [#64] Work\n" => "Work",
        "* [#65] Work\n" => "[#65] Work",
        "* [#a] Work\n" => "[#a] Work",
        "* [#É] Work\n" => "[#É] Work",
        "* [#AB] Work\n" => "[#AB] Work",
        "* [#A]junk] Work\n" => "[#A]junk] Work",
    );
}

fn typed_aot_headline_view_uses_one_scheme_graph() {
    let source = "#+seq_todo: WAIT(w) | DONE(d)\n\
                  * WAIT Parent :agent:\n\
                  :properties:\n\
                  :session_id: session-a\n\
                  :end:\n\
                  ** DONE [#A] COMMENT Hidden :work:\n\
                  scheduled: <2026-09-27 Sun>\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("Scheme AOT graph");
    let headlines = document.headlines().collect::<Vec<_>>();
    assert_eq!(headlines.len(), 2);
    let parent = headlines[0];
    let child = headlines[1];
    assert_eq!(parent.level(), 1);
    assert_eq!(parent.todo_keyword().as_deref(), Some("WAIT"));
    assert_eq!(parent.todo_type(), Some("todo"));
    assert_eq!(parent.display_title().as_deref(), Some("Parent"));
    assert_eq!(parent.local_tags().collect::<Vec<_>>(), ["agent"]);
    assert_eq!(parent.properties(), [("session_id", "session-a")]);
    assert!(!parent.is_comment());
    assert_eq!(child.level(), 2);
    assert_eq!(
        child.parent().map(|headline| headline.id()),
        Some(parent.id())
    );
    assert_eq!(child.todo_type(), Some("done"));
    assert_eq!(child.display_title().as_deref(), Some("COMMENT Hidden"));
    assert!(child.is_comment());
    assert_eq!(child.effective_tags(), ["agent", "work"]);
    assert_eq!(child.planning(), [("scheduled", "<2026-09-27 Sun>")]);
    assert_eq!(
        document
            .headline(child.id())
            .map(|headline| headline.range()),
        Some(child.range())
    );
    assert!(document.headline(document.records().len()).is_none());
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "org_headline_aot::scheme_headline_tags_project_without_rust_headline_scanning",
        scheme_headline_tags_project_without_rust_headline_scanning,
    ),
    (
        "org_headline_aot::scheme_aot_headline_state_classifies_projected_element_titles",
        scheme_aot_headline_state_classifies_projected_element_titles,
    ),
    (
        "org_headline_aot::headline_state_uses_defaults_only_without_document_directives",
        headline_state_uses_defaults_only_without_document_directives,
    ),
    (
        "org_headline_aot::scheme_aot_headline_state_honors_parse_config_until_file_directive_overrides_it",
        scheme_aot_headline_state_honors_parse_config_until_file_directive_overrides_it,
    ),
    (
        "org_headline_aot::scheme_headline_properties_are_stable_across_repeated_queries",
        scheme_headline_properties_are_stable_across_repeated_queries,
    ),
    (
        "org_headline_aot::scheme_priority_cookie_validation_preserves_malformed_headlines",
        scheme_priority_cookie_validation_preserves_malformed_headlines,
    ),
    (
        "org_headline_aot::typed_aot_headline_view_uses_one_scheme_graph",
        typed_aot_headline_view_uses_one_scheme_graph,
    ),
];
