//! Behavioral admission cases for the public Scheme AOT Org parser.

use orgize::Org;

macro_rules! assert_org_records {
    ($source:expr, $($kind:literal => $count:expr),+ $(,)?) => {{
        let source = $source;
        let parsed = Org::parse(source);
        assert_eq!(parsed.to_org(), source);
        assert_eq!(parsed.syntax().to_string(), source);
        $(
            assert_eq!(
                parsed.records().iter().filter(|record| record.kind == $kind).count(),
                $count,
                "{} count in {source:?}",
                $kind,
            );
        )+
        parsed
    }};
}

#[test]
fn nested_headlines_and_section_boundaries() {
    let parsed = assert_org_records!(
        "preamble\n* Parent\nbody\n** Child\nmore\n* Sibling\n",
        "headline" => 3,
        "paragraph" => 3,
    );
    let headlines: Vec<_> = parsed
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .collect();
    assert_eq!(headlines[0].field("title"), Some("Parent"));
    assert_eq!(headlines[1].field("title"), Some("Child"));
    assert_eq!(headlines[2].field("title"), Some("Sibling"));
    assert_eq!(headlines[1].parent_id, Some(headlines[0].id));
    assert_ne!(headlines[2].parent_id, Some(headlines[0].id));
}

#[test]
fn block_contents_do_not_start_headlines() {
    assert_org_records!(
        "#+begin_src text\n* not a headline\n#+end_src\n* Real\n",
        "src-block" => 1,
        "headline" => 1,
    );
}

#[test]
fn headline_title_reuses_scheme_inline_objects_without_losing_title_fields() {
    let parsed = assert_org_records!(
        "* TODO A *bold* [[id:target]] :work:\n",
        "headline" => 1,
        "bold" => 1,
        "link" => 1,
    );
    let headline = parsed.headlines().next().expect("headline view");
    assert_eq!(
        headline.display_title().as_deref(),
        Some("A *bold* [[id:target]]")
    );
    assert_eq!(headline.todo_keyword().as_deref(), Some("TODO"));
    assert_eq!(headline.local_tags().collect::<Vec<_>>(), ["work"]);
}

#[test]
fn affiliated_keyword_and_nested_objects() {
    let parsed = assert_org_records!(
        "#+NAME: example\n#+CAPTION: A *bold* caption\n| a | b |\n\n* Link [[https://example.org][*bold*]]\n",
        "keyword" => 2,
        "table" => 1,
        "headline" => 1,
        "link" => 1,
        "bold" => 2,
    );
    let table = parsed
        .records()
        .iter()
        .find(|record| record.kind == "table")
        .expect("table record");
    assert_eq!(parsed.affiliated_keyword_ids(table.id).len(), 2);
}

#[test]
fn malformed_constructs_remain_lossless() {
    assert_org_records!(
        "* Open\r\n#+begin_src rust\r\nlet x = 1;\r\n",
        "headline" => 1,
    );
}
