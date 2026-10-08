use std::path::Path;

use super::org_elements::index_org;

pub(crate) fn batch_index_matches_individual_projection() {
    use super::elements::{DocumentSource, index_sources};
    use super::model::DocumentLanguage;
    let sources = [
        "* α\n:PROPERTIES:\n:ID: one\n:END:\n",
        "- [X] checked\n",
        "* duplicate\n",
        "* duplicate\n",
    ]
    .iter()
    .enumerate()
    .map(|(index, source)| DocumentSource {
        path: std::path::PathBuf::from(format!("note-{index}.org")),
        source: source.to_string(),
    })
    .collect::<Vec<_>>();
    let expected = sources
        .iter()
        .flat_map(|source| index_org(&source.path, &source.source).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        format!(
            "{:?}",
            index_sources(DocumentLanguage::Org, &sources).unwrap()
        ),
        format!("{expected:?}")
    );
    assert!(
        index_sources(DocumentLanguage::Org, &[])
            .unwrap()
            .is_empty()
    );
}

pub(crate) fn batch_index_spans_bounded_packets_and_oversized_sources() {
    use super::elements::{DocumentSource, bounded_org_batch_len, index_sources};
    use super::model::DocumentLanguage;
    let mut sources = (0..130)
        .map(|index| DocumentSource {
            path: std::path::PathBuf::from(format!("note-{index}.org")),
            source: format!("* Head {index}\n:PROPERTIES:\n:ID: id-{index}\n:END:\n"),
        })
        .collect::<Vec<_>>();
    assert_eq!(bounded_org_batch_len(&sources), 64);
    sources[64].source = "x".repeat(65537);
    assert_eq!(bounded_org_batch_len(&sources[64..]), 0);
    let expected = sources
        .iter()
        .flat_map(|source| index_org(&source.path, &source.source).unwrap())
        .collect::<Vec<_>>();
    let actual = index_sources(DocumentLanguage::Org, &sources).unwrap();
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

pub(crate) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "query_pushdown_preserves_complete_native_projection",
        query_pushdown_preserves_complete_native_projection,
    ),
    (
        "scheme_checkbox_field_drives_document_item_kind",
        scheme_checkbox_field_drives_document_item_kind,
    ),
    (
        "scheme_tag_field_drives_descriptive_list_projection",
        scheme_tag_field_drives_descriptive_list_projection,
    ),
];

fn query_pushdown_preserves_complete_native_projection() {
    let blocks = crate::org_aot::parse_org_aot(
        "* Same\n#+begin_src text\n  λ (ref:a)\n#+end_src\n** Same\n#+begin_example\n#+end_example\n* Same\n:PROPERTIES:\n:CUSTOM_ID: fixed\n:EFFORT: 1:30\n:TOTAL: 1e308\n:BAD: λ\n:END:\n#+begin_quote\n  β\n#+end_quote\n* Same\n",
    ).unwrap();
    for record in blocks.records() {
        let _ = blocks.affiliated_keyword_ids(record.id);
    }
    let (projected, stages) = crate::runtime_profile::measure(|| blocks.document());
    assert_eq!(
        stages["native.requests_completed"], 3,
        "blocks, anchors and property durations each have one document owner: {stages:?}"
    );
    assert_eq!(projected.sections[0].anchor.as_deref(), Some("same"));
    assert_eq!(
        projected.sections[0].subsections[0].anchor.as_deref(),
        Some("same-1")
    );
    assert_eq!(projected.sections[1].anchor.as_deref(), Some("fixed"));
    assert_eq!(
        projected.sections[1].properties[1]
            .duration
            .as_ref()
            .unwrap()
            .total_seconds,
        5400
    );
    assert_eq!(
        projected.sections[1].properties[2]
            .duration
            .as_ref()
            .unwrap()
            .total_seconds,
        u64::MAX
    );
    assert!(projected.sections[1].properties[3].duration.is_none());
    assert_eq!(projected.sections[2].anchor.as_deref(), Some("same-2"));
    let blocks =
        crate::org_aot::parse_org_aot(&"#+begin_src text\nλ\n#+end_src\n".repeat(33)).unwrap();
    for record in blocks.records() {
        let _ = blocks.affiliated_keyword_ids(record.id);
    }
    let (projected, stages) = crate::runtime_profile::measure(|| blocks.document());
    assert_eq!(projected.children.len(), 33);
    assert_eq!(
        stages["native.requests_completed"], 2,
        "block plan yields at the 32-record boundary"
    );
    let keywords = crate::org_aot::parse_org_aot(
        "#+options: H:2 H:3 -:nil e:YES\n#+FILETAGS: :α:β:α:\n#+lınk: ignored\n",
    )
    .unwrap();
    // Affiliation is an independent, lazy Scheme policy. Admit that policy
    // before measuring keyword reuse; whole-catalog performance stays cold.
    for record in keywords.records() {
        let _ = keywords.affiliated_keyword_ids(record.id);
    }
    let (projected, stages) = crate::runtime_profile::measure(|| keywords.document());
    assert_eq!(
        stages
            .get("native.requests_completed")
            .copied()
            .unwrap_or(0),
        0,
        "keyword projection must reuse the admitted Scheme document plan"
    );
    assert_eq!(projected.filetags, ["α", "β"]);
    assert_eq!(projected.export_settings.headline_levels, Some(3));
    assert_eq!(projected.export_settings.special_strings, Some(false));
    assert_eq!(projected.export_settings.expand_entities, Some(true));
    let properties = crate::Org::parse(
        "* Parent\n:PROPERTIES:\n:Color_ALL: 'two words' blue\n:END:\n** Child\n:PROPERTIES:\n:Color: blue\n:END:\n",
    ).document();
    let ((profile, plan), stages) = crate::runtime_profile::measure(|| {
        properties.property_profile_with_native_plan(&crate::ast::PropertySchemaRegistry::default())
    });
    assert_eq!(
        stages["native.requests_completed"], 1,
        "property descriptors and tokens must share one native request"
    );
    let child = &properties.sections[0].subsections[0];
    let (_, stages) = crate::runtime_profile::measure(|| {
        assert_eq!(
            crate::ast::property_allowed_values(
                &child.effective_properties,
                &profile,
                &child.properties[0],
                &plan,
            ),
            Some(vec!["two words".into(), "blue".into()])
        );
    });
    assert!(
        !stages.contains_key("native.requests_completed"),
        "allowed-value lint must reuse the admitted native property plan"
    );
    use super::{
        elements::filter_elements_by_query,
        org_elements::{index_org_document, index_org_document_with_query},
        query_match::PreparedQuery,
    };
    let source = "#+TODO: WAIT | FINISHED\n* FINISHED α :tag:\n:PROPERTIES:\n:ID: ID-Ä\n:END:\nBody *strong* α.\n- [X] checked\n- term :: body\n[[file:photo.png]]\n#+begin_src Rust\nlet VALUE = 1;\n#+end_src\n| A | B |\n|---+---|\n| 1 | 2 |\n";
    let path = Path::new("Mixed/α file.org");
    let document = crate::org_aot::parse_org_aot(source).unwrap();
    let full = index_org_document(path, source, &document).unwrap();
    for terms in [
        vec![],
        vec!["absent".into()],
        vec!["α".into()],
        vec!["DONE".into()],
        vec!["file".into()],
        vec!["rust VALUE".into()],
    ] {
        for fields in [
            vec![],
            vec!["todoType=Done".into()],
            vec!["title".into()],
            vec!["text=Body".into()],
            vec!["ID=ID-Ä".into()],
        ] {
            let expected = filter_elements_by_query(full.clone(), &terms, &[], &fields);
            let query = PreparedQuery::new(&terms, &[], &fields);
            let actual =
                index_org_document_with_query(path, source, &document, Some(&query)).unwrap();
            assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
        }
    }
    assert!(
        !filter_elements_by_query(full, &["DONE".into()], &[], &[]).is_empty(),
        "match Scheme-projected lifecycle value absent from raw source"
    );
}

fn scheme_checkbox_field_drives_document_item_kind() {
    macro_rules! check_item {
        ($source:expr => $kind:expr, $counter:expr, $checkbox:expr, $checked:expr) => {{
            let facts = index_org(Path::new("list.org"), $source).expect("AOT list index");
            let item = facts
                .iter()
                .find(|fact| matches!(fact.kind, "checklistItem" | "listItem"))
                .expect("projected list item");
            let field = |name| {
                item.fields
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| value.as_str())
            };
            assert_eq!(item.kind, $kind);
            assert_eq!(field("counter"), $counter);
            assert_eq!(field("checkbox"), $checkbox);
            assert_eq!(field("checked"), $checked);
        }};
    }
    check_item!("- [X] done\n" => "checklistItem", None, Some("X"), Some("true"));
    check_item!("- [@2] [X] done\n" => "checklistItem", Some("2"), Some("X"), Some("true"));
    check_item!("- [@A] item\n" => "listItem", Some("A"), None, None);
    check_item!("- [ ] open\n" => "checklistItem", None, Some(" "), Some("false"));
    check_item!("- [-] partial\n" => "checklistItem", None, Some("-"), Some("false"));
    check_item!("- [x] ordinary\n" => "listItem", None, None, None);
}

fn scheme_tag_field_drives_descriptive_list_projection() {
    let facts =
        index_org(Path::new("list.org"), "- term :: body\n").expect("AOT descriptive list index");
    let list = facts
        .iter()
        .find(|fact| fact.kind == "list")
        .expect("plain list fact");
    assert!(
        list.fields
            .iter()
            .any(|(name, value)| name == "descriptive" && value == "true")
    );
    let item = facts
        .iter()
        .find(|fact| fact.kind == "listItem")
        .expect("descriptive item fact");
    assert!(
        item.fields
            .iter()
            .any(|(name, value)| name == "tag" && value == "term")
    );
}
