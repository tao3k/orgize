use orgize::Org;

const INTERACTIVE: &str = r#"
#+BEGIN_SRC org-contract :type agent-interactive
id: principle-pressure
method: choice
stage: presentation
group: PRINCIPLE
create: deferred
target: tao3k.principles
info: Choose the pressure that should be evaluated.
categories: 1=EVIDENCE,2=AUTHORITY,?=detail
details:
|n|id|contract|full|use-if|
|1|EVIDENCE|tao3k.evidence|Evidence before confidence|provenance is missing|
|2|AUTHORITY|tao3k.human-agency|Human agency|consequential authority is missing|
#+END_SRC
"#;

#[test]
fn projects_canonical_org_interactive_choice() {
    let document = Org::parse(INTERACTIVE).document();
    let choices = document.org_interactive_choices().unwrap();
    assert_eq!(choices.len(), 1);
    let choice = &choices[0];
    assert_eq!(choice.id, "principle-pressure");
    assert_eq!(choice.method, "choice");
    assert_eq!(choice.stage, "presentation");
    assert_eq!(choice.group.as_deref(), Some("PRINCIPLE"));
    assert_eq!(choice.target.as_deref(), Some("tao3k.principles"));
    assert_eq!(choice.entries.len(), 2);
    assert_eq!(choice.categories.len(), 3);
    assert!(choice.categories[2].detail);
}

#[test]
fn rejects_categories_that_do_not_resolve_to_details() {
    let source = INTERACTIVE.replace("1=EVIDENCE", "1=UNKNOWN");
    let document = Org::parse(&source).document();
    let error = document.org_interactive_choices().unwrap_err();
    assert!(error.to_string().contains("must match a detail row"));
}
