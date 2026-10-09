//! Tangle an Org-native contract's Scheme body through the generated Org CST.

use std::{collections::HashSet, env, fmt::Write as _, fs, path::Path};

#[path = "support/org_feature_source.rs"]
mod org_feature_source;

use org_feature_source::{feature_block, property, records};

fn tangle(
    source: &str,
    contract_interface: &str,
    elements_interface: &str,
) -> Result<String, String> {
    if contract_interface.is_empty() || elements_interface.is_empty() {
        return Err("Org contract interface modules cannot be empty".into());
    }
    let records = records(source)?;
    let mut output = String::new();
    writeln!(output, ";;; -*- Gerbil -*-").unwrap();
    writeln!(
        output,
        ";;; @generated from the Org Element CST; do not edit."
    )
    .unwrap();
    writeln!(output, "(import (only-in {contract_interface:?}").unwrap();
    writeln!(
        output,
        "                 org-contract-block assert-org-element"
    )
    .unwrap();
    writeln!(output, "                 make-org-contract-definition)").unwrap();
    writeln!(output, "        (only-in {elements_interface:?}").unwrap();
    writeln!(
        output,
        "                 org-elements property property-contains all-of any-of at child-of descendant-of))"
    )
    .unwrap();
    writeln!(output, "(export org-contract-definitions)").unwrap();
    writeln!(output, "(def org-contract-definitions (list").unwrap();
    let mut seen = HashSet::new();
    for section in records.iter().filter(|record| record.kind == "headline") {
        let Some(id) = property(&records, section.id, "CONTRACT_ID")? else {
            continue;
        };
        if id.is_empty() || !seen.insert(id) {
            return Err(format!("empty or duplicate CONTRACT_ID: {id}"));
        }
        let scope = match property(&records, section.id, "CONTRACT_SCOPE")? {
            Some("document") => "document",
            Some("subtree") => "subtree",
            _ => return Err(format!("CONTRACT_SCOPE must be document or subtree: {id}")),
        };
        let body = feature_block(&records, section.id, ":org-contract")?;
        writeln!(
            output,
            "  (make-org-contract-definition {id:?} '{scope} (org-contract-block"
        )
        .unwrap();
        output.push_str(body);
        if !body.ends_with('\n') {
            output.push('\n');
        }
        writeln!(output, "  ))").unwrap();
    }
    if seen.is_empty() {
        return Err("expected at least one CONTRACT_ID section".into());
    }
    writeln!(output, "))").unwrap();
    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // SAFETY: standalone entrypoint, before workers or children and host I/O.
    unsafe { orgize::initialize_native_runtime() }?;
    let args: Vec<_> = env::args_os().collect();
    let (input, output, contract_interface, elements_interface) = match args.as_slice() {
        [_, input, output] => (
            input,
            output,
            "../interface.ss",
            "../../org-elements/interface.ss",
        ),
        [_, input, output, contract, elements] => (
            input,
            output,
            contract
                .to_str()
                .ok_or("contract interface module path must be valid UTF-8")?,
            elements
                .to_str()
                .ok_or("elements interface module path must be valid UTF-8")?,
        ),
        _ => {
            return Err("usage: org_contract_tangle INPUT.org OUTPUT.ss [CONTRACT_INTERFACE ELEMENTS_INTERFACE]".into());
        }
    };
    let source = fs::read_to_string(Path::new(input))?;
    let tangled = tangle(&source, contract_interface, elements_interface)?;
    fs::write(Path::new(output), tangled)?;
    Ok(())
}

#[cfg(test)]
#[path = "support/native_fixture.rs"]
mod native_fixture;

#[cfg(test)]
mod tests {
    use super::tangle;

    const CONTRACT_INTERFACE: &str = "../interface.ss";
    const ELEMENTS_INTERFACE: &str = "../../org-elements/interface.ss";

    fn default_tangle(source: &str) -> Result<String, String> {
        tangle(source, CONTRACT_INTERFACE, ELEMENTS_INTERFACE)
    }

    fn tangling_uses_element_ancestry_and_org_metadata() {
        let source = include_str!("../languages/org/modules/org-contract/contracts.org");
        let tangled = default_tangle(source).unwrap();
        assert!(tangled.contains("make-org-contract-definition \"section.scope.v1\" 'subtree"));
        assert!(
            tangled.contains("make-org-contract-definition \"document.headlines.v1\" 'document")
        );
        assert!(tangled.contains("(org-contract-block\n(assert-org-element"));
        assert!(!tangled.contains("(assert count"));
        assert_eq!(
            tangled,
            include_str!("../languages/org/modules/org-contract/generated/contract-source.ss")
        );
    }

    fn missing_scope_fails_closed() {
        let source = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:END:\n#+begin_src scheme :org-contract\n(list)\n#+end_src\n";
        super::native_fixture::assert_rejected(default_tangle(source));
    }

    fn plain_scheme_babel_block_is_not_a_contract() {
        let source = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme\n(list)\n#+end_src\n";
        super::native_fixture::assert_rejected(default_tangle(source));
    }

    fn elements_query_block_cannot_define_a_contract() {
        let source = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme :org-elements-query\n(org-elements headline)\n#+end_src\n";
        super::native_fixture::assert_rejected(default_tangle(source));
    }

    fn mixed_feature_tags_cannot_define_a_contract() {
        let source = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme :org-contract :org-elements-query\n(list)\n#+end_src\n";
        super::native_fixture::assert_rejected(default_tangle(source));
    }

    fn contract_header_is_case_insensitive_but_language_must_be_scheme() {
        let upper = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:CONTRACT_SCOPE: document\n:END:\n#+BEGIN_SRC SCHEME :ORG-CONTRACT\n(assert-org-element \"x\" error (bindings) (org-elements headline) (expect at-least 1))\n#+END_SRC\n";
        assert!(default_tangle(upper).is_ok());

        let pseudo_language = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:CONTRACT_SCOPE: document\n:END:\n#+BEGIN_SRC org-contract\n(assert-org-element \"x\" error (bindings) (org-elements headline) (expect at-least 1))\n#+END_SRC\n";
        super::native_fixture::assert_rejected(default_tangle(pseudo_language));
    }

    fn nested_block_does_not_satisfy_parent_contract() {
        let source = "* Parent\n:PROPERTIES:\n:CONTRACT_ID: parent\n:CONTRACT_SCOPE: subtree\n:END:\n** Child\n:PROPERTIES:\n:CONTRACT_ID: child\n:CONTRACT_SCOPE: subtree\n:END:\n#+begin_src scheme :org-contract\n(assert-org-element \"a\" error (bindings) (org-elements headline) (expect at-least 1))\n#+end_src\n";
        super::native_fixture::assert_rejected(default_tangle(source));
    }

    fn duplicate_contract_ids_are_rejected() {
        let source = "* One\n:PROPERTIES:\n:CONTRACT_ID: same\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme :org-contract\n(assert-org-element \"a\" error (bindings) (org-elements headline) (expect at-least 1))\n#+end_src\n* Two\n:PROPERTIES:\n:CONTRACT_ID: same\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme :org-contract\n(assert-org-element \"b\" error (bindings) (org-elements headline) (expect at-least 1))\n#+end_src\n";
        super::native_fixture::assert_rejected(default_tangle(source));
    }

    fn consumer_selects_both_owned_interface_imports() {
        let source = include_str!("../languages/org/modules/org-contract/contracts.org");
        let generated = tangle(
            source,
            "/consumer/org-contract/interface.ss",
            "/consumer/org-elements/interface.ss",
        )
        .expect("consumer contracts use upstream interfaces");
        assert!(generated.contains("(only-in \"/consumer/org-contract/interface.ss\""));
        assert!(generated.contains("(only-in \"/consumer/org-elements/interface.ss\""));
        assert_eq!(
            generated.matches("(make-org-contract-definition ").count(),
            5
        );
        super::native_fixture::assert_rejected(tangle(source, "", ELEMENTS_INTERFACE));
        super::native_fixture::assert_rejected(tangle(source, CONTRACT_INTERFACE, ""));
    }

    fn consumer_contract_source_artifact_stays_in_sync() {
        let source = include_str!("../tests/fixtures/org-contract/customer-contracts.org");
        let generated = tangle(
            source,
            "../../../../languages/org/modules/org-contract/interface.ss",
            "../../../../languages/org/modules/org-elements/interface.ss",
        )
        .expect("consumer feature source tangles through the Org Element graph");
        assert_eq!(
            generated,
            include_str!("../tests/fixtures/org-contract/generated/customer-contract-source.ss")
        );
    }
    #[test]
    fn explicit_startup_precedes_parallel_example_cases() {
        super::native_fixture::run(
            "org_contract_tangle",
            10,
            &[
                (
                    "tangling_uses_element_ancestry_and_org_metadata",
                    tangling_uses_element_ancestry_and_org_metadata,
                ),
                ("missing_scope_fails_closed", missing_scope_fails_closed),
                (
                    "plain_scheme_babel_block_is_not_a_contract",
                    plain_scheme_babel_block_is_not_a_contract,
                ),
                (
                    "elements_query_block_cannot_define_a_contract",
                    elements_query_block_cannot_define_a_contract,
                ),
                (
                    "mixed_feature_tags_cannot_define_a_contract",
                    mixed_feature_tags_cannot_define_a_contract,
                ),
                (
                    "contract_header_is_case_insensitive_but_language_must_be_scheme",
                    contract_header_is_case_insensitive_but_language_must_be_scheme,
                ),
                (
                    "nested_block_does_not_satisfy_parent_contract",
                    nested_block_does_not_satisfy_parent_contract,
                ),
                (
                    "duplicate_contract_ids_are_rejected",
                    duplicate_contract_ids_are_rejected,
                ),
                (
                    "consumer_selects_both_owned_interface_imports",
                    consumer_selects_both_owned_interface_imports,
                ),
                (
                    "consumer_contract_source_artifact_stays_in_sync",
                    consumer_contract_source_artifact_stays_in_sync,
                ),
            ],
        );
    }
}
