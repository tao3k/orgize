//! Org-owned interactive choice projection.

use super::{
    Document, OrgInteractiveCategory, OrgInteractiveChoice, OrgInteractiveChoiceEntry,
    OrgInteractiveParseError, ParsedAnnotation, SourceBlockRecord,
};

impl Document<ParsedAnnotation> {
    /// Projects validated interactive choice windows from Org source blocks.
    ///
    /// The formal surface is `org-contract :type agent-interactive`; consumers
    /// share one parser and one DTO shape instead of recognizing aliases.
    pub fn org_interactive_choices(
        &self,
    ) -> Result<Vec<OrgInteractiveChoice>, OrgInteractiveParseError> {
        self.source_block_records()
            .iter()
            .filter(|record| {
                record.language.as_deref() == Some("org-contract")
                    && record.header_args.iter().any(|arg| {
                        arg.key == "type" && arg.value.as_deref() == Some("agent-interactive")
                    })
            })
            .map(parse_choice)
            .collect()
    }
}

fn parse_choice(
    record: &SourceBlockRecord,
) -> Result<OrgInteractiveChoice, OrgInteractiveParseError> {
    let rows = super::org_native_values::rows("interactive-choice", &[&record.value]);
    let first = rows
        .first()
        .expect("native interactive response must not be empty");
    if first.first().map(String::as_str) == Some("error") {
        assert_eq!(rows.len(), 1, "native interactive error must be exclusive");
        let [_, message] = first.as_slice() else {
            panic!("invalid native interactive error arity");
        };
        return Err(OrgInteractiveParseError::new(message.clone()));
    }
    let [tag, id, method, stage, group, target, create, info] = first.as_slice() else {
        panic!("invalid native interactive choice arity");
    };
    assert_eq!(tag, "choice", "invalid native interactive choice tag");
    let optional = |value: &String| (!value.is_empty()).then(|| value.clone());
    let mut categories = Vec::new();
    let mut entries = Vec::new();
    for row in &rows[1..] {
        match row.first().map(String::as_str) {
            Some("category") => {
                assert!(entries.is_empty(), "native categories must precede entries");
                let [_, key, value, detail] = row.as_slice() else {
                    panic!("invalid native interactive category arity");
                };
                categories.push(OrgInteractiveCategory {
                    key: key.clone(),
                    value: value.clone(),
                    detail: match detail.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => panic!("invalid native interactive detail flag"),
                    },
                });
            }
            Some("entry") => {
                let [_, number, id, contract, full, use_if] = row.as_slice() else {
                    panic!("invalid native interactive entry arity");
                };
                entries.push(OrgInteractiveChoiceEntry {
                    number: number.clone(),
                    id: id.clone(),
                    contract: optional(contract),
                    full: full.clone(),
                    use_if: use_if.clone(),
                });
            }
            _ => panic!("invalid native interactive row tag"),
        }
    }
    assert!(
        !categories.is_empty() && !entries.is_empty(),
        "native interactive choice requires rows"
    );
    Ok(OrgInteractiveChoice {
        source: record.source.clone(),
        id: id.clone(),
        method: method.clone(),
        stage: stage.clone(),
        group: optional(group),
        target: optional(target),
        create: optional(create),
        info: info.clone(),
        categories,
        entries,
    })
}
