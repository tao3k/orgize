//! Scheme-owned Org list metadata projected into the generated Element graph.

use orgize::org_aot::parse_org_aot;

#[test]
fn scheme_aot_list_checkbox_is_a_graph_field_not_paragraph_text() {
    macro_rules! check_checkbox {
        ($source:expr => $expected:expr) => {{
            let document = parse_org_aot($source).expect("Scheme AOT list parser");
            assert_eq!(document.syntax().to_string(), $source);
            let item = document
                .records()
                .iter()
                .find(|record| record.kind == "item")
                .expect("list item projects");
            assert_eq!(item.field("checkbox"), $expected);
        }};
    }
    check_checkbox!("- [X] done\n" => Some("X"));
    check_checkbox!("- [ ] open\n" => Some(" "));
    check_checkbox!("- [-] partial\n" => Some("-"));
    check_checkbox!("- [x] ordinary\n" => None);
}

#[test]
fn scheme_aot_list_counter_and_checkbox_share_one_item() {
    let source = "- [@2] [X] done\n";
    let document = parse_org_aot(source).expect("Scheme AOT list metadata");
    let item = document
        .records()
        .iter()
        .find(|record| record.kind == "item")
        .expect("list item projects");
    assert_eq!(item.field("counter"), Some("2"));
    assert_eq!(item.field("checkbox"), Some("X"));
    assert_eq!(document.syntax().to_string(), source);
}

#[test]
fn scheme_aot_list_counter_rejects_invalid_metadata() {
    macro_rules! check_counter {
        ($source:expr => $expected:expr) => {{
            let document = parse_org_aot($source).expect("Scheme AOT counter parser");
            let item = document
                .records()
                .iter()
                .find(|record| record.kind == "item")
                .expect("list item projects");
            assert_eq!(item.field("counter"), $expected);
            assert_eq!(document.syntax().to_string(), $source);
        }};
    }
    check_counter!("- [@A] item\n" => Some("A"));
    check_counter!("1. [@12] item\n" => Some("12"));
    check_counter!("- [@?] ordinary\n" => None);
    check_counter!("- [@] ordinary\n" => None);
}

#[test]
fn scheme_aot_descriptive_tag_is_not_an_ordered_item_tag() {
    macro_rules! check_tag {
        ($source:expr => $expected:expr) => {{
            let document = parse_org_aot($source).expect("Scheme AOT descriptive item");
            let item = document
                .records()
                .iter()
                .find(|record| record.kind == "item")
                .expect("list item projects");
            assert_eq!(item.field("tag"), $expected);
            assert_eq!(document.syntax().to_string(), $source);
        }};
    }
    check_tag!("- term :: body\n" => Some("term "));
    check_tag!("- [@2] [X] term :: body\n" => Some("term "));
    check_tag!("1. term :: body\n" => None);
    check_tag!("- term: body\n" => None);
}
