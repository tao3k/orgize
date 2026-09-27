//! AOT graph affiliation comes from the Scheme Org Elements catalog.

use crate::org_aot::parse_org_aot;

#[test]
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

#[test]
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
