use orgize::ast::{
    ElementData, OrgSourceBlock, OrgSourceBlockDocument, OrgSourceBlockHeader,
    OrgSourceBlockHeaderValue, OrgSourceBlockKeyword,
};
use orgize::{Org, syntax_ast::SourceBlock};

#[test]
fn gql_remains_a_generic_keyword_before_a_source_block() {
    let org = Org::parse(
        "#+gql: Registry::refresh --calls--> Registry::publish\n#+begin_src rust\nfn refresh() {}\n#+end_src\n",
    );
    let document = org.document();
    assert_eq!(document.children.len(), 2);
    assert!(matches!(
        &document.children[0].data,
        ElementData::Keyword(keyword) if keyword.key == "gql"
    ));
    assert_eq!(
        org.first_node::<SourceBlock>().unwrap().language().unwrap(),
        "rust"
    );
}

#[test]
fn source_block_languages_do_not_require_a_global_registration() {
    let org = Org::parse("#+begin_src any-language\nbody\n#+end_src\n");
    let document = org.document();
    assert_eq!(document.children.len(), 1);
    assert_eq!(
        org.first_node::<SourceBlock>().unwrap().language().unwrap(),
        "any-language"
    );
}

#[test]
fn typed_source_block_document_round_trips_through_orgize() {
    let block = OrgSourceBlock::new(
        "rust",
        vec![
            OrgSourceBlockHeader::new(
                "query",
                OrgSourceBlockHeaderValue::text(
                    "rust://src/registry.rs#item/method/refresh/type/Registry",
                )
                .unwrap(),
            )
            .unwrap(),
            OrgSourceBlockHeader::new(
                "filename",
                OrgSourceBlockHeaderValue::text("src/registry.rs").unwrap(),
            )
            .unwrap(),
        ],
        vec![
            OrgSourceBlockKeyword::new("gql", "Registry::refresh --calls--> Registry::publish")
                .unwrap(),
        ],
        "fn refresh() {}",
    )
    .unwrap();
    let rendered = OrgSourceBlockDocument::new(vec![block])
        .unwrap()
        .render()
        .unwrap();

    assert_eq!(
        rendered,
        "#+GQL: Registry::refresh --calls--> Registry::publish\n#+begin_src rust :query \"rust://src/registry.rs#item/method/refresh/type/Registry\" :filename \"src/registry.rs\"\nfn refresh() {}\n#+end_src\n"
    );
    assert_eq!(
        Org::parse(rendered).document().source_block_records().len(),
        1
    );
}

#[test]
fn typed_source_block_document_rejects_org_end_marker_in_source() {
    let error = OrgSourceBlock::new("rust", vec![], vec![], "#+end_src\nnot source").unwrap_err();
    assert_eq!(error.reason_kind(), "org-source-block-input-invalid");
}

#[test]
fn typed_source_block_document_rejects_case_variant_header_duplicates() {
    let headers = vec![
        OrgSourceBlockHeader::new("runtime", OrgSourceBlockHeaderValue::text("bash").unwrap())
            .unwrap(),
        OrgSourceBlockHeader::new("RUNTIME", OrgSourceBlockHeaderValue::text("sh").unwrap())
            .unwrap(),
    ];
    let error = OrgSourceBlock::new("sh", headers, vec![], "true").unwrap_err();
    assert_eq!(error.reason_kind(), "org-source-block-input-invalid");
}
