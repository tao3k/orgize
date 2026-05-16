use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{BlockKind, ElementData},
    Org,
};

#[test]
fn semantic_ast_projects_source_and_example_block_code_refs() {
    let doc = Org::parse(
        r#"#+begin_src rust -l "// ref:%s" -r
let value = 1; // ref:init
println!("{value}"); // ref:print_value
#+end_src

#+begin_example
example line (ref:sample)
#+end_example
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let blocks = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Block(block) => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 2);

    assert_eq!(blocks[0].kind, BlockKind::Source);
    assert_eq!(blocks[0].switches.as_deref(), Some(r#"-l "// ref:%s" -r"#));
    assert_eq!(blocks[0].code_refs.len(), 2);
    assert_eq!(blocks[0].code_refs[0].line, 1);
    assert_eq!(blocks[0].code_refs[0].column, 16);
    assert_eq!(blocks[0].code_refs[0].name, "init");
    assert_eq!(blocks[0].code_refs[0].raw, "// ref:init");
    assert_eq!(blocks[0].code_refs[1].line, 2);
    assert_eq!(blocks[0].code_refs[1].column, 22);
    assert_eq!(blocks[0].code_refs[1].name, "print_value");
    assert_eq!(blocks[0].code_refs[1].raw, "// ref:print_value");
    assert!(blocks[0].value.contains("// ref:init"));
    assert_eq!(blocks[0].lines.len(), 2);
    assert_eq!(
        blocks[0].lines[0].code_ref,
        Some(blocks[0].code_refs[0].clone())
    );

    assert_eq!(blocks[1].kind, BlockKind::Example);
    assert_eq!(blocks[1].switches, None);
    assert_eq!(blocks[1].code_refs.len(), 1);
    assert_eq!(blocks[1].code_refs[0].line, 1);
    assert_eq!(blocks[1].code_refs[0].column, 14);
    assert_eq!(blocks[1].code_refs[0].name, "sample");
    assert_eq!(blocks[1].code_refs[0].raw, "(ref:sample)");
    assert_eq!(blocks[1].lines.len(), 1);
    assert_eq!(
        blocks[1].lines[0].code_ref,
        Some(blocks[1].code_refs[0].clone())
    );
    assert!(blocks[1].value.contains("(ref:sample)"));
}
