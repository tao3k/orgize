use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AstRef, ElementData, ObjectData},
    Org,
};

#[test]
fn semantic_traversal_supports_exporter_and_indexer_shapes() {
    let doc = Org::parse(
        r#"* Export Me
Paragraph with [[https://example.com][link]] and <2026-04-30 Thu>.

#+begin_quote
quoted
#+end_quote

- one
- two

| A | B |
|---+---|
| 1 | 2 |

[fn:note] Footnote body
"#,
    )
    .document();

    assert_clean_projection(&doc);

    #[derive(Default)]
    struct TraversalShape {
        headlines: Vec<String>,
        paragraphs: usize,
        blocks: usize,
        links: usize,
        timestamps: usize,
        list_items: usize,
        table_rows: usize,
        table_cells: usize,
        footnotes: Vec<String>,
    }

    let shape = doc.fold(TraversalShape::default(), |mut shape, node| {
        match node {
            AstRef::Section(section) => shape.headlines.push(section.raw_title.clone()),
            AstRef::Element(element) => match &element.data {
                ElementData::Paragraph(_) => shape.paragraphs += 1,
                ElementData::Block(_) => shape.blocks += 1,
                ElementData::FootnoteDef(definition) => {
                    shape.footnotes.push(definition.label.clone());
                }
                _ => {}
            },
            AstRef::Object(object) => match &object.data {
                ObjectData::Link(_) => shape.links += 1,
                ObjectData::Timestamp(_) => shape.timestamps += 1,
                _ => {}
            },
            AstRef::ListItem(_) => shape.list_items += 1,
            AstRef::TableRow(_) => shape.table_rows += 1,
            AstRef::TableCell(_) => shape.table_cells += 1,
            _ => {}
        }
        shape
    });

    assert_eq!(shape.headlines, ["Export Me".to_string()]);
    assert_eq!(shape.paragraphs, 6);
    assert_eq!(shape.blocks, 1);
    assert_eq!(shape.links, 1);
    assert_eq!(shape.timestamps, 1);
    assert_eq!(shape.list_items, 2);
    assert_eq!(shape.table_rows, 3);
    assert_eq!(shape.table_cells, 4);
    assert_eq!(shape.footnotes, ["note".to_string()]);
}
