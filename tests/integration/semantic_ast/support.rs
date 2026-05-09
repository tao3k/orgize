use orgize::ast::{AstRef, ElementData, ObjectData, ParsedAst};

pub(crate) fn assert_clean_projection(doc: &ParsedAst) {
    assert!(
        doc.diagnostics.is_empty(),
        "unexpected diagnostics: {:#?}",
        doc.diagnostics
    );

    let unknowns = doc.fold(Vec::new(), |mut unknowns, node| {
        match node {
            AstRef::Element(element) => {
                if let ElementData::Unknown { kind, .. } = &element.data {
                    unknowns.push(format!("element:{kind}"));
                }
            }
            AstRef::Object(object) => {
                if let ObjectData::Unknown { kind, .. } = &object.data {
                    unknowns.push(format!("object:{kind}"));
                }
            }
            _ => {}
        }
        unknowns
    });

    assert!(
        unknowns.is_empty(),
        "unexpected semantic unknowns: {unknowns:#?}"
    );
}
