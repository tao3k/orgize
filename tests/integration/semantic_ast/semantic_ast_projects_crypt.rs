use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{CryptWarningKind, ParsedAst},
};

const SOURCE: &str = r#"* Secret note :crypt:
:PROPERTIES:
:CRYPTKEY: 0x0123456789012345678901234567890123456789
:END:
-----BEGIN PGP MESSAGE-----
opaque
-----END PGP MESSAGE-----
** Inherited child
Plain child should still be treated as sensitive when crypt is inherited.
* Plaintext secret :crypt:
This body has not been encrypted yet.
* Key only
:PROPERTIES:
:CRYPTKEY: 0xfeed
:END:
"#;

#[test]
fn semantic_ast_projects_org_crypt_states() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let states = doc.crypt_states();
    assert_eq!(states.len(), 4);
    assert!(states[0].has_direct_tag);
    assert!(states[0].body_is_opaque);
    assert!(states[0].encrypted_payload);
    assert_eq!(
        states[0].crypt_key.as_ref().map(|key| key.inherited),
        Some(false)
    );
    assert!(states[1].has_inherited_tag);
    assert!(
        states[1]
            .warnings
            .iter()
            .any(|warning| warning.kind == CryptWarningKind::InheritedCryptTag)
    );
    assert!(
        states[2]
            .warnings
            .iter()
            .any(|warning| warning.kind == CryptWarningKind::PlaintextCryptBody)
    );
    assert!(!states[3].body_is_opaque);
    assert!(
        states[3]
            .warnings
            .iter()
            .any(|warning| warning.kind == CryptWarningKind::CryptKeyWithoutCryptTag)
    );

    insta::assert_snapshot!(
        "semantic_ast__semantic_crypt_states",
        render_crypt_states(&doc)
    );
}

fn render_crypt_states(doc: &ParsedAst) -> String {
    let mut output = String::new();
    for state in doc.crypt_states() {
        output.push_str(&format!(
            "{} level={} direct={} inherited={} opaque={} encrypted={} key={} path={}\n",
            state.title,
            state.level,
            state.has_direct_tag,
            state.has_inherited_tag,
            state.body_is_opaque,
            state.encrypted_payload,
            state
                .crypt_key
                .as_ref()
                .map(|key| if key.inherited { "inherited" } else { "local" })
                .unwrap_or("none"),
            state.outline_path.join(" > ")
        ));
        for warning in state.warnings {
            output.push_str(&format!("  warning {}\n", warning.kind.as_str()));
        }
    }
    output
}
