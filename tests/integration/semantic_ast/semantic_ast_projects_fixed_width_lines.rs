use crate::semantic_ast::support::assert_clean_projection;
use orgize::{Org, ast::ElementData};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/fixed-width-lines.org");

#[test]
fn semantic_ast_projects_fixed_width_lines() {
    let doc = Org::parse(SOURCE).document();

    assert_clean_projection(&doc);

    let fixed_width = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::FixedWidth(fixed_width) => Some(fixed_width),
            _ => None,
        })
        .expect("fixed-width element");

    assert_eq!(fixed_width.value, "  one\n\t\ttwo\n");
    assert_eq!(fixed_width.lines.len(), 2);
    assert_eq!(fixed_width.lines[0].source, ":   one");
    assert_eq!(fixed_width.lines[0].value, "  one");
    assert_eq!(fixed_width.lines[0].normalized_value, "one");
    assert_eq!(fixed_width.lines[0].removed_indent, 2);
    assert_eq!(fixed_width.lines[1].source, ": \t\ttwo");
    assert_eq!(fixed_width.lines[1].value, "\t\ttwo");
    assert_eq!(fixed_width.lines[1].normalized_value, "      two");
    assert_eq!(fixed_width.normalized_value(), "one\n      two\n");

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_fixed_width_lines", fixed_width);
    });
}
