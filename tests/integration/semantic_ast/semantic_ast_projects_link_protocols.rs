use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{LinkProtocolKind, LinkProtocolSource, OrgProtocolKind},
    Org,
};

const SOURCE: &str = r#"#+LINK: gh https://github.com/%s
#+LINK: clip org-protocol://capture?template=c&url=%h
See [[gh:tao3k/orgize][repo]] and [[clip:https://example.com/a path][clip]].
Open [[org-protocol://open-source?url=https%3A%2F%2Fexample.com%2Fpost][source]].
Risk [[shell:echo hi][shell]] and [[elisp:org-agenda][agenda]].
Refs [[file:notes.org::*Target]] [[attachment:diagram.png]] [[id:abc::*Heading]].
Custom [[man:printf][manual]] and mail [[mailto:dev@example.com]].
"#;

#[test]
fn semantic_ast_projects_link_protocol_registry() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.link_protocol_records();
    assert_eq!(records.len(), 12);
    assert!(records.iter().any(|record| {
        record.source == LinkProtocolSource::AbbreviationDefinition
            && record.protocol == "gh"
            && record.kind == LinkProtocolKind::Abbreviation
            && record.replacement.as_deref() == Some("https://github.com/%s")
    }));
    assert!(records.iter().any(|record| {
        record.source == LinkProtocolSource::Link
            && record.protocol == "shell"
            && record.kind == LinkProtocolKind::Executable
    }));
    assert!(records.iter().any(|record| {
        record.source == LinkProtocolSource::Link
            && record.protocol == "man"
            && record.kind == LinkProtocolKind::Custom
    }));
    assert!(records.iter().any(|record| {
        record.source == LinkProtocolSource::Link
            && record.protocol == "clip"
            && record.kind == LinkProtocolKind::Abbreviation
            && record.org_protocol.as_ref().is_some_and(|call| {
                call.subprotocol == "capture" && call.kind == OrgProtocolKind::Capture
            })
    }));
    assert!(records.iter().any(|record| {
        record.source == LinkProtocolSource::Link
            && record.protocol == "org-protocol"
            && record.org_protocol.as_ref().is_some_and(|call| {
                call.subprotocol == "open-source"
                    && call.kind == OrgProtocolKind::OpenSource
                    && call.parameters.iter().any(|parameter| {
                        parameter.key == "url"
                            && parameter.value.as_deref() == Some("https://example.com/post")
                    })
            })
    }));

    insta::assert_debug_snapshot!("semantic_ast__semantic_link_protocol_records", records);
}
