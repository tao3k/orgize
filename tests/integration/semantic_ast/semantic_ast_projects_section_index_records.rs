use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{LifecycleRecordKind, LinkSearchKind, TargetKind},
    Org,
};

const SOURCE: &str = r#"#+CATEGORY: research
* TODO Parent :agent:
:PROPERTIES:
:ID: parent-id
:CUSTOM_ID: parent-custom
:PREF: parser projection
:END:
Body links to [[id:child-id::*Child][child heading]] and <<local-target>>.
:LOGBOOK:
- Refiled on [2026-05-14 Thu 09:30] from [[file:old.org::*Old]]
:END:
#+begin_src rust -r
let answer = 42; // (ref:answer)
#+end_src
** DONE Child :ARCHIVE:
CLOSED: [2026-05-14 Thu]
:PROPERTIES:
:ID: child-id
:CATEGORY: archive-cat
:END:
Archived child body.
"#;

#[test]
fn semantic_ast_projects_source_grounded_section_index_records() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.section_index_records();
    assert_eq!(records.len(), 2);

    let parent = records
        .iter()
        .find(|record| record.title == "Parent")
        .expect("parent record");
    assert_eq!(parent.outline_path, ["Parent"]);
    assert_eq!(parent.level, 1);
    assert_eq!(
        parent.category.as_ref().map(|category| category.as_str()),
        Some("research")
    );
    assert_eq!(
        parent.todo.as_ref().map(|todo| todo.name.as_str()),
        Some("TODO")
    );
    assert_eq!(parent.tags, ["agent"]);
    assert_eq!(parent.effective_tags, ["agent"]);
    assert!(parent
        .properties
        .iter()
        .any(|property| property.key == "PREF" && property.value == "parser projection"));
    assert!(parent
        .effective_properties
        .iter()
        .any(|property| property.key == "PREF" && property.value == "parser projection"));
    assert!(parent
        .body
        .iter()
        .any(|slice| slice.text.contains("Body links to")));
    assert!(parent
        .links
        .iter()
        .any(|link| link.path == "id:child-id::*Child"
            && link.search.as_ref().is_some_and(|search| {
                search.raw == "*Child" && search.kind == LinkSearchKind::Headline
            })));
    assert!(parent.targets.iter().any(|target| {
        target.kind == TargetKind::CustomId
            && target.key == "#parent-custom"
            && target.value == "parent-custom"
    }));
    assert!(parent
        .targets
        .iter()
        .any(|target| target.kind == TargetKind::Id && target.key == "id:parent-id"));
    assert!(parent
        .targets
        .iter()
        .any(|target| target.kind == TargetKind::Target && target.value == "local-target"));
    assert!(parent
        .targets
        .iter()
        .any(|target| target.kind == TargetKind::CodeRef && target.key == "coderef:answer"));
    assert!(parent
        .lifecycle
        .iter()
        .any(|record| matches!(record.kind, LifecycleRecordKind::Refile { .. })));

    let child = records
        .iter()
        .find(|record| record.title == "Child")
        .expect("child record");
    assert_eq!(child.outline_path, ["Parent", "Child"]);
    assert_eq!(
        child.category.as_ref().map(|category| category.as_str()),
        Some("archive-cat")
    );
    assert!(child.archive.archived);
    assert!(child.archive.has_archive_tag);
    assert_eq!(
        child.todo.as_ref().map(|todo| todo.name.as_str()),
        Some("DONE")
    );
    assert!(child.planning.closed.is_some());
    assert!(child
        .targets
        .iter()
        .any(|target| target.kind == TargetKind::Id && target.key == "id:child-id"));
    assert!(child.source.start.line < child.source.end.line);
}
