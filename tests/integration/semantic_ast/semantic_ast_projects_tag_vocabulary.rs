use crate::semantic_ast::support::assert_clean_projection;
use orgize::Org;
use serde_json::Value;

#[test]
fn semantic_ast_projects_tag_vocabulary_groups_and_exclusive_sets() {
    let doc = Org::parse(
        r#"#+TAGS: { @work(w) @home(h) @tennisclub(t) } laptop(l) pc(p)
#+TAGS: [ GTD : Control Persp ]
#+TAGS: [ Control : Context Task ]
#+TAGS: { Context : @Home @Work @Call }

* TODO Follow up :@Work:Task:
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let work = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "@work")
        .expect("@work tag definition");
    assert_eq!(work.shortcut.as_deref(), Some("w"));
    assert!(!work.is_group);
    assert!(
        work.group
            .as_ref()
            .is_some_and(|group| { group.name.is_none() && group.exclusive })
    );

    let gtd = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "GTD")
        .expect("GTD group tag definition");
    assert!(gtd.is_group);
    assert!(gtd.group.is_none());

    let control = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "Control" && !definition.is_group)
        .expect("Control member tag definition");
    assert!(
        control
            .group
            .as_ref()
            .is_some_and(|group| { group.name.as_deref() == Some("GTD") && !group.exclusive })
    );

    let home = doc
        .tag_definitions
        .iter()
        .find(|definition| definition.name == "@Home")
        .expect("@Home member tag definition");
    assert!(
        home.group
            .as_ref()
            .is_some_and(|group| { group.name.as_deref() == Some("Context") && group.exclusive })
    );

    let payload: Value = serde_json::from_str(&doc.org_elements_json()).expect("Org elements JSON");
    assert_eq!(payload["tagDefinitions"][0]["group"]["exclusive"], true);
    assert_eq!(payload["tagDefinitions"][5]["isGroup"], true);
    assert_eq!(payload["tagDefinitions"][6]["group"]["name"], "GTD");

    insta::assert_debug_snapshot!(
        "semantic_ast__tag_vocabulary_groups",
        doc.to_bare().tag_definitions
    );
}
