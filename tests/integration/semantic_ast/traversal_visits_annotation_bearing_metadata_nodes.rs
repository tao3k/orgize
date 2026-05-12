use orgize::{
    ast::{AstMut, AstRef, ElementData},
    Org,
};

#[test]
fn traversal_visits_annotation_bearing_metadata_nodes() {
    let mut doc = Org::parse(
        r#"#+TITLE: Demo
#+AUTHOR: Author
* Heading
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
#+ATTR_HTML: :class compact
|   A |   B |
|-----+-----|
|   1 |   2 |

- tag :: item
"#,
    )
    .document();

    assert!(doc.diagnostics.is_empty());

    #[derive(Default)]
    struct Counts {
        keywords: usize,
        properties: usize,
        list_items: usize,
        table_rows: usize,
        table_cells: usize,
    }

    let counts = doc.fold(Counts::default(), |mut counts, node| {
        match node {
            AstRef::Keyword(_) => counts.keywords += 1,
            AstRef::Property(_) => counts.properties += 1,
            AstRef::ListItem(_) => counts.list_items += 1,
            AstRef::TableRow(_) => counts.table_rows += 1,
            AstRef::TableCell(_) => counts.table_cells += 1,
            _ => {}
        }
        counts
    });

    assert_eq!(counts.keywords, 5);
    assert_eq!(counts.properties, 1);
    assert_eq!(counts.list_items, 1);
    assert_eq!(counts.table_rows, 3);
    assert_eq!(counts.table_cells, 4);

    doc.visit_mut(|node| {
        if let AstMut::Keyword(keyword) = node {
            if keyword.key == "TITLE" {
                keyword.value = " Changed".into();
            }
        }
    });
    assert_eq!(
        doc.children
            .iter()
            .find_map(|element| match &element.data {
                ElementData::Keyword(keyword) if keyword.key == "TITLE" => Some(&keyword.value),
                _ => None,
            })
            .map(String::as_str),
        Some(" Changed")
    );
}
