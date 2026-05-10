use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AstRef, ElementData, ObjectData},
    Org,
};

#[test]
fn semantic_ast_collects_preprocessing_directives_without_expansion() {
    let doc = Org::parse(
        r#"#+INCLUDE: "./chapter one.org" src org :lines "1-20" :minlevel 2 :only-contents
#+MACRO: issue [[https://tracker.example/$1][$2]]

#+begin_example
#+INCLUDE: ignored.org
#+MACRO: ignored no
#+end_example

{{{issue(42\,A, Fix)}}}
"#,
    )
    .document();

    assert_clean_projection(&doc);

    assert_eq!(doc.includes.len(), 1);
    let include = &doc.includes[0];
    assert_eq!(include.path, "./chapter one.org");
    assert_eq!(include.raw_path, r#""./chapter one.org""#);
    assert_eq!(include.arguments, ["src", "org"]);
    assert_eq!(include.options.len(), 3);
    assert_eq!(include.options[0].key, "lines");
    assert_eq!(include.options[0].value.as_deref(), Some("1-20"));
    assert_eq!(include.options[1].key, "minlevel");
    assert_eq!(include.options[1].value.as_deref(), Some("2"));
    assert_eq!(include.options[2].key, "only-contents");
    assert_eq!(include.options[2].value, None);

    assert_eq!(doc.macro_definitions.len(), 1);
    let definition = &doc.macro_definitions[0];
    assert_eq!(definition.name, "issue");
    assert_eq!(definition.template, "[[https://tracker.example/$1][$2]]");

    let macro_arguments = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => {
                objects.iter().find_map(|object| match &object.data {
                    ObjectData::Macro { name, arguments } if name == "issue" => Some(arguments),
                    _ => None,
                })
            }
            _ => None,
        })
        .expect("macro call remains parsed");
    assert_eq!(macro_arguments, &["42,A".to_string(), "Fix".to_string()]);

    let counts = doc.fold((0, 0), |(includes, definitions), node| match node {
        AstRef::IncludeDirective(_) => (includes + 1, definitions),
        AstRef::MacroDefinition(_) => (includes, definitions + 1),
        _ => (includes, definitions),
    });
    assert_eq!(counts, (1, 1));

    let bare = doc.to_bare();
    assert_eq!(bare.includes[0].ann, ());
    assert_eq!(bare.macro_definitions[0].ann, ());
}

#[test]
fn semantic_ast_diagnoses_invalid_preprocessing_directives() {
    let doc = Org::parse(
        r#"#+INCLUDE:
#+MACRO: 0bad value
"#,
    )
    .document();

    assert!(doc.includes.is_empty());
    assert!(doc.macro_definitions.is_empty());
    assert_eq!(doc.diagnostics.len(), 2);
    assert!(doc
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("INCLUDE keyword")));
    assert!(doc
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("MACRO keyword")));
}
