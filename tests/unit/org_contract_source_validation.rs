use std::path::Path;

use crate::{Org, ast::validate_contract_source};

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "rejects_an_org_file_without_contract_definitions",
        rejects_an_org_file_without_contract_definitions,
    ),
    (
        "rejects_a_contract_without_valid_assertions",
        rejects_a_contract_without_valid_assertions,
    ),
    (
        "accepts_a_contract_with_a_valid_selector_assertion",
        accepts_a_contract_with_a_valid_selector_assertion,
    ),
    (
        "rejects_an_unsupported_contract_kind",
        rejects_an_unsupported_contract_kind,
    ),
    (
        "rejects_an_assertion_without_an_id_even_when_another_assertion_is_valid",
        rejects_an_assertion_without_an_id_even_when_another_assertion_is_valid,
    ),
    (
        "rejects_an_assertion_without_a_query",
        rejects_an_assertion_without_a_query,
    ),
];

fn rejects_an_org_file_without_contract_definitions() {
    let document = Org::parse("#+TITLE: Documentation policy\n* Rules\nPlain prose\n").document();
    let result = validate_contract_source(
        &document,
        Some(Path::new("docs/contracts/document-policy.org")),
    );

    assert!(!result.is_valid());
    assert_eq!(result.diagnostics[0].code, "CONTRACT-E001");
    assert_eq!(
        result.diagnostics[0].path.as_deref(),
        Some("docs/contracts/document-policy.org")
    );
}

fn rejects_a_contract_without_valid_assertions() {
    let document = Org::parse(
        "* Empty contract\n:PROPERTIES:\n:CONTRACT_ID: empty.contract\n:CONTRACT_SCOPE: document\n:END:\n",
    )
    .document();
    let result = validate_contract_source(&document, Some(Path::new("contracts/empty.org")));

    assert_eq!(result.registry.contracts.len(), 1);
    assert_eq!(result.diagnostics[0].code, "CONTRACT-E003");
    assert_eq!(
        result.diagnostics[0].contract_id.as_deref(),
        Some("empty.contract")
    );
}

fn accepts_a_contract_with_a_valid_selector_assertion() {
    let document = Org::parse(
        "* Document contract\n:PROPERTIES:\n:CONTRACT_ID: document.contract\n:CONTRACT_SCOPE: document\n:END:\n** Has text\n:PROPERTIES:\n:ASSERT_ID: document.has-text\n:END:\n#+begin_src org-elements-selector\n(:org-element (:type paragraph))\n#+end_src\n",
    )
    .document();
    let run = || validate_contract_source(&document, Some(Path::new("contracts/document.org")));
    #[cfg(feature = "runtime-profile")]
    let (result, stages) = crate::runtime_profile::measure(run);
    #[cfg(not(feature = "runtime-profile"))]
    let result = run();
    #[cfg(feature = "runtime-profile")]
    assert_eq!(
        stages.get("native.requests_completed"),
        Some(&1),
        "Contract registry and diagnostics must share one Scheme plan: {stages:?}"
    );

    assert!(result.is_valid(), "{:?}", result.diagnostics);
    assert_eq!(result.registry.contracts.len(), 1);
    assert_eq!(result.registry.contracts[0].assertions.len(), 1);
}

fn rejects_an_unsupported_contract_kind() {
    let document = Org::parse(
        "* Contract\n:PROPERTIES:\n:CONTRACT_ID: invalid.kind\n:CONTRACT_KIND: imaginary\n:END:\n",
    )
    .document();
    let result = validate_contract_source(&document, Some(Path::new("contracts/kind.org")));

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "CONTRACT-E005")
    );
}

fn rejects_an_assertion_without_an_id_even_when_another_assertion_is_valid() {
    let document = Org::parse(
        "* Contract\n:PROPERTIES:\n:CONTRACT_ID: mixed.assertions\n:CONTRACT_SCOPE: document\n:END:\n** Anonymous assertion\n#+begin_src org-elements-selector\n(:org-element (:type paragraph))\n#+end_src\n** Valid assertion\n:PROPERTIES:\n:ASSERT_ID: mixed.has-text\n:END:\n#+begin_src org-elements-selector\n(:org-element (:type paragraph))\n#+end_src\n",
    )
    .document();
    let result = validate_contract_source(&document, Some(Path::new("contracts/mixed.org")));

    assert_eq!(result.registry.contracts[0].assertions.len(), 1);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "CONTRACT-E007")
    );
}

fn rejects_an_assertion_without_a_query() {
    let document = Org::parse(
        "* Contract\n:PROPERTIES:\n:CONTRACT_ID: missing.query\n:CONTRACT_SCOPE: document\n:END:\n** Broken assertion\n:PROPERTIES:\n:ASSERT_ID: missing.query.assertion\n:END:\n",
    )
    .document();
    let result = validate_contract_source(&document, Some(Path::new("contracts/query.org")));

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "CONTRACT-E009")
    );
}
