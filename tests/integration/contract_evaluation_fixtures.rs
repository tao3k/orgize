pub(crate) fn reflection_answer_contract_source() -> &'static str {
    r#"* reflection-answers
:PROPERTIES:
:CONTRACT_ID: agent.reflection-answers.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** has-question-table
:PROPERTIES:
:ASSERT_ID: reflection-has-question-table
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table :descendant-of $scope))
#+END_SRC

** has-question-column
:PROPERTIES:
:ASSERT_ID: reflection-has-question-column
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table-cell :descendant-of $scope :text "Question"))
#+END_SRC

** has-value-column
:PROPERTIES:
:ASSERT_ID: reflection-has-value-column
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table-cell :descendant-of $scope :text "Value"))
#+END_SRC

** has-nonempty-answer
:PROPERTIES:
:ASSERT_ID: reflection-has-nonempty-answer
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table-cell :descendant-of $scope :column "Value" :header nil :nonempty t))
#+END_SRC
"#
}

pub(crate) fn reflection_answered_source() -> &'static str {
    r#"* Reflection Questions
:PROPERTIES:
:CONTRACT_ORG: agent.reflection-answers.v1
:END:

| Question | Value |
|----------+-------|
| What should reflection record? | It must answer with a nonempty Value cell. |
"#
}

pub(crate) fn reflection_empty_value_source() -> &'static str {
    r#"* Reflection Questions
:PROPERTIES:
:CONTRACT_ORG: agent.reflection-answers.v1
:END:

| Question | Value |
|----------+-------|
| What should reflection record? | |
"#
}

pub(crate) fn query_expression_contract_source() -> &'static str {
    r#"* query-expression-contract
:PROPERTIES:
:CONTRACT_ID: agent.query-expression.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** evidence-link-from-cell
:PROPERTIES:
:ASSERT_ID: evidence-link-from-cell
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(let ((evidence
       (table-cell :descendant-of $scope :column "Evidence" :header nil :nonempty t)))
  (assert count >= 1
    (link :descendant-of evidence)))
#+END_SRC
"#
}

pub(crate) fn query_expression_target_source() -> &'static str {
    r#"* Evidence Loop
:PROPERTIES:
:CONTRACT_ORG: agent.query-expression.v1
:END:

| Claim | Evidence |
|-------+----------|
| Ready | [[https://example.test][trace]] |
"#
}
