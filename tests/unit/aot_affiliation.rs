//! AOT graph affiliation comes from the Scheme Org Elements catalog.

use crate::org_aot::parse_org_aot;

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "adjacent_name_keyword_attaches_to_source_block",
        adjacent_name_keyword_attaches_to_source_block,
    ),
    (
        "blank_line_stops_affiliated_keyword_association",
        blank_line_stops_affiliated_keyword_association,
    ),
    (
        "adjacent_affiliated_keywords_attach_as_one_group",
        adjacent_affiliated_keywords_attach_as_one_group,
    ),
];

fn adjacent_name_keyword_attaches_to_source_block() {
    let document = parse_org_aot(
        "* Contract\n#+NAME: task.rule\n#+BEGIN_SRC org-contract\n(assert exists (headline))\n#+END_SRC\n",
    )
    .expect("Scheme AOT parse");
    let keyword = document
        .records()
        .iter()
        .find(|record| record.kind == "keyword" && record.field("key") == Some("NAME"))
        .expect("name keyword");
    let block = document
        .records()
        .iter()
        .find(|record| record.kind == "src-block")
        .expect("source block");
    assert_eq!(document.affiliated_keyword_ids(block.id), &[keyword.id]);
}

fn blank_line_stops_affiliated_keyword_association() {
    let document = parse_org_aot(
        "* Contract\n#+NAME: task.rule\n\n#+BEGIN_SRC org-contract\n(assert exists (headline))\n#+END_SRC\n",
    )
    .expect("Scheme AOT parse");
    let block = document
        .records()
        .iter()
        .find(|record| record.kind == "src-block")
        .expect("source block");
    assert!(document.affiliated_keyword_ids(block.id).is_empty());
}

fn adjacent_affiliated_keywords_attach_as_one_group() {
    let document = parse_org_aot(
        "#+NAME: task.rule\n#+ATTR_HTML: :data-poo-flow task\n#+BEGIN_SRC scheme\n(display 1)\n#+END_SRC\n",
    )
    .expect("Scheme AOT parse");
    let block = document
        .records()
        .iter()
        .find(|record| record.kind == "src-block")
        .expect("source block");
    let keywords = document.affiliated_keyword_ids(block.id);
    assert_eq!(keywords.len(), 2);
    assert_eq!(document.records()[keywords[0]].field("key"), Some("NAME"));
    assert_eq!(
        document.records()[keywords[1]].field("key"),
        Some("ATTR_HTML")
    );
}
