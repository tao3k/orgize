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
        "scheme_checkbox_field_drives_document_item_kind",
        scheme_checkbox_field_drives_document_item_kind,
    ),
    (
        "scheme_tag_field_drives_descriptive_list_projection",
        scheme_tag_field_drives_descriptive_list_projection,
    ),
];

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
