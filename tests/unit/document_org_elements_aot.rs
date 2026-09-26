use std::path::Path;

use super::org_elements::index_org;

#[test]
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

#[test]
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
