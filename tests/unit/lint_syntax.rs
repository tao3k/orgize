use crate::{
    Org,
    ast::parse_contracts_from_document,
    lint::{LintSeverity, lint_document, lint_org},
};

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "syntax_checks_unregistered_blocks",
        syntax_checks_unregistered_blocks,
    ),
    (
        "syntax_rejects_malformed_and_unsupported_forms",
        syntax_rejects_malformed_and_unsupported_forms,
    ),
    (
        "syntax_accepts_native_query_contract_and_selector",
        syntax_accepts_native_query_contract_and_selector,
    ),
    (
        "syntax_rejects_trailing_contract_forms",
        syntax_rejects_trailing_contract_forms,
    ),
    (
        "syntax_validates_projected_document_with_unicode_locations",
        syntax_validates_projected_document_with_unicode_locations,
    ),
    (
        "syntax_invalid_expectation_never_defaults_to_exists",
        syntax_invalid_expectation_never_defaults_to_exists,
    ),
];

fn block(language: &str, body: &str) -> String {
    format!("#+begin_src {language}\n{body}\n#+end_src\n")
}

fn syntax_checks_unregistered_blocks() {
    syntax_projection_preserves_source_fields_without_native_requests();
    #[cfg(feature = "runtime-profile")]
    {
        let source = block("org-elements-query", "(")
            + &block("org-elements-selector", "(:org-element (:type paragraph))")
            + &block("org-elements-expect", "count >= 1");
        let document = Org::parse(&source).document();
        let (errors, stages) = crate::runtime_profile::measure(|| {
            crate::ast::contract_source_blocks(&document)
                .iter()
                .filter(|block| crate::ast::contract_block_syntax_error(block).is_some())
                .count()
        });
        assert_eq!(errors, 1, "malformed forms must not poison adjacent blocks");
        assert_eq!(
            stages.get("native.requests_completed"),
            Some(&1),
            "syntax admission must share one native plan: {stages:?}"
        );
    }
    let source = block("org-contract", "(assert exists (paragraph)");
    let document = Org::parse(&source).document();
    assert!(
        parse_contracts_from_document(&document, None)
            .contracts
            .is_empty()
    );
    let report = lint_org(&source);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "ORG046")
            .count(),
        1
    );
}

fn syntax_projection_preserves_source_fields_without_native_requests() {
    let source = r#"#+PROPERTY: header-args :results output :tangle no
#+NAME: 中文块
#+begin_src org-contract :name 测试 :severity error
(assert exists (paragraph))
#+end_src

中文 src_rust[:results raw]{1 + 1} and *src_scheme{(+ 1 2)}*.

* Scope
:PROPERTIES:
:header-args: :exports both
:END:
:DRAWER:
#+begin_src rust
42
#+end_src
:END:

- item
  #+begin_src scheme
  (+ 2 3)
  #+end_src

| src_rust{7} |

[fn:note] src_scheme{8}
"#;
    let document = Org::parse(source).document();
    #[cfg(feature = "runtime-profile")]
    let lean = {
        let (records, observations) =
            crate::runtime_profile::measure(|| document.source_block_syntax_records());
        assert!(
            observations.is_empty(),
            "syntax projection must not call native policies"
        );
        records
    };
    #[cfg(not(feature = "runtime-profile"))]
    let lean = document.source_block_syntax_records();
    let full = document.source_block_records();
    assert!(lean.len() >= 6, "exercise nested and inline source syntax");
    assert_eq!(lean.len(), full.len());
    for (syntax, babel) in lean.iter().zip(&full) {
        assert_eq!(syntax.source, babel.source);
        assert_eq!(syntax.kind, babel.kind);
        assert_eq!(syntax.name, babel.name);
        assert_eq!(syntax.language, babel.language);
        assert_eq!(syntax.parameters, babel.parameters);
        assert_eq!(syntax.value, babel.value);
    }
}

fn syntax_rejects_malformed_and_unsupported_forms() {
    for (language, body) in [
        ("org-elements-query", ")"),
        ("org-elements-query", "(kind paragraph ignored)"),
        ("org-elements-query", "(category imaginary)"),
        ("org-elements-query", "(= (summary text) \"x\" ignored)"),
        (
            "org-elements-selector",
            "(:org-element (:type paragraph)) ignored",
        ),
        ("org-elements-query-expr", "(imaginary paragraph)"),
        ("org-elements-expr", "\"unclosed"),
        ("org-elements-query", "category = \"section\""),
        (
            "org-elements-selector",
            "(:org-element (:imaginary paragraph))",
        ),
        ("org-elements-expect", "count >= nope"),
        ("org-elements-expect", "exists\nnot exists"),
        ("org-elements-expect", "count >= -1"),
        ("org-elements-expect", "notexists"),
        ("org-elements-expect", "count >= 1x"),
        ("org-elements-expect", "count >= 18446744073709551616"),
        ("org-elements-expect", "count >= 1 ignored"),
        ("org-contract", ""),
    ] {
        assert!(
            lint_org(&block(language, body))
                .findings
                .iter()
                .any(|f| f.code == "ORG046" && f.severity == LintSeverity::Error),
            "{language}: {body}"
        );
    }
}

fn syntax_accepts_native_query_contract_and_selector() {
    for body in [
        "exists",
        "not exists",
        "count <= 0",
        "count < 1",
        "count >= 1",
        "count > 0",
        "count == 1",
        "count != 2",
        "# note\r\ncount\t>= 1 # note",
    ] {
        assert!(
            !lint_org(&block("org-elements-expect", body))
                .findings
                .iter()
                .any(|f| f.code == "ORG046"),
            "{body}"
        );
    }
    for (language, body) in [
        ("ORG-ELEMENTS-QUERY", "(paragraph)"),
        (
            "org-elements-query",
            "(org-elements-query (kind paragraph) (limit 3))",
        ),
        ("org-contract", "(assert exists (paragraph))"),
        (
            "org-contract",
            "(assert pair-document-properties-equal (properties ID))",
        ),
        (
            "org-contract",
            "(assert pair-node-properties-equal (identity ID) (properties TITLE))",
        ),
        ("org-elements-selector", "(:org-element (:type paragraph))"),
        ("org-elements-expect", "# note\ncount >= 1 # note"),
        (
            "org-contract",
            "(let (($p (paragraph))) (assert count >= 1 (paragraph)))",
        ),
    ] {
        assert!(
            !lint_org(&block(language, body))
                .findings
                .iter()
                .any(|f| f.code == "ORG046"),
            "{language}: {body}"
        );
    }
    assert!(
        !lint_org(&block("text", "(unclosed"))
            .findings
            .iter()
            .any(|f| f.code == "ORG046")
    );
}

fn syntax_rejects_trailing_contract_forms() {
    for body in [
        "(assert exists (paragraph) ignored)",
        "(assert exists (paragraph)) (imaginary)",
        "(imaginary) (assert exists (paragraph))",
        "(let (($p (paragraph)) ($p (headline))) (assert exists (paragraph)))",
        "(let (($p (paragraph))) (assert exists (paragraph)) (imaginary))",
        "(let (($p (paragraph))) (let ((p (headline))) (assert exists (paragraph))))",
    ] {
        assert!(
            lint_org(&block("org-contract", body))
                .findings
                .iter()
                .any(|f| f.code == "ORG046"),
            "{body}"
        );
    }
}

fn syntax_validates_projected_document_with_unicode_locations() {
    let source = format!("π\r\n{}", block("org-elements-query", "(query"));
    let document = Org::parse(&source).document();
    let direct = lint_org(&source);
    let projected = lint_document(&document, &source);
    let direct = direct.findings.iter().find(|f| f.code == "ORG046").unwrap();
    let projected = projected
        .findings
        .iter()
        .find(|f| f.code == "ORG046")
        .unwrap();
    assert_eq!(direct, projected);
    assert_eq!(direct.location.range_start, 4);
    assert_eq!(direct.location.start.line, 2);
}

fn syntax_invalid_expectation_never_defaults_to_exists() {
    let source = format!(
        "* Policy\n:PROPERTIES:\n:CONTRACT_ID: policy\n:END:\n** Assertion\n:PROPERTIES:\n:ASSERT_ID: p\n:END:\n{}{}",
        block("org-elements-query", "(paragraph)"),
        block("org-elements-expect", "exists\nignored")
    );
    let document = Org::parse(&source).document();
    let registry = parse_contracts_from_document(&document, None);
    assert_eq!(registry.contracts.len(), 1);
    assert!(registry.contracts[0].assertions.is_empty());
    assert!(
        lint_org(&source)
            .findings
            .iter()
            .any(|f| f.code == "ORG046")
    );
}
