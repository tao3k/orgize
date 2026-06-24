use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AgendaDate, AgendaQuery, PriorityProfile, PriorityRangeStatus, PriorityValue,
        PropertyAllowedValueScope, PropertyInheritancePolicy, PropertySchemaApplication,
        PropertySchemaContract, PropertySchemaField, PropertySchemaFindingKind,
        PropertySchemaReferenceKind, PropertySchemaRegistry, PropertySchemaScope,
        PropertySchemaValueRule,
    },
};

#[test]
fn semantic_ast_projects_priority_effort_and_effective_properties() {
    let doc = Org::parse(
        r#"#+PROPERTY: Effort 2h
* TODO [#A] Parent
:PROPERTIES:
:Owner: Sarah
:END:
** TODO Child
SCHEDULED: <2026-05-15 Fri>
* TODO [#2] Numeric
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.properties[0].key, "Effort");
    assert_eq!(
        doc.properties[0].duration.as_ref().unwrap().total_seconds,
        7_200
    );

    let parent = &doc.sections[0];
    assert_eq!(parent.priority.raw_cookie(), Some("A"));
    assert_eq!(parent.priority.effective_text(), "A");
    assert_eq!(parent.priority.org_priority_score(), Some(2_000));
    assert_eq!(parent.priority.range_status(), PriorityRangeStatus::InRange);
    assert!(
        parent
            .effective_properties
            .iter()
            .any(|property| property.key == "Effort" && property.value == "2h")
    );

    let child = &parent.subsections[0];
    assert!(child.priority.is_default());
    assert_eq!(child.priority.effective_text(), "B");
    assert_eq!(child.priority.org_priority_score(), Some(1_000));
    assert_eq!(child.priority.range_status(), PriorityRangeStatus::InRange);
    assert!(
        child
            .effective_properties
            .iter()
            .any(|property| property.key == "Owner" && property.value == "Sarah")
    );

    let numeric = &doc.sections[1];
    assert_eq!(numeric.priority.raw_cookie(), Some("2"));
    assert_eq!(numeric.priority.effective_text(), "2");
    assert_eq!(numeric.priority.org_priority_score(), Some(65_000));
    assert_eq!(
        numeric.priority.range_status(),
        PriorityRangeStatus::OutOfRange
    );

    let numeric_profile = PriorityProfile::new(
        PriorityValue::Numeric(0),
        PriorityValue::Numeric(10),
        PriorityValue::Numeric(5),
    )
    .expect("valid numeric priority profile");
    assert_eq!(
        numeric.priority.score_with_profile(&numeric_profile),
        Some(8_000)
    );
    assert_eq!(
        numeric.priority.range_status_with_profile(&numeric_profile),
        PriorityRangeStatus::InRange
    );

    let query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 15))
        .match_expression(r#"Owner="Sarah"+Effort>=120+PRIORITY="B""#)
        .expect("valid inherited property match");
    let entries = doc.to_bare().agenda_entries(&query);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].raw_title, "Child");

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_priority_properties", doc.to_bare());
    });
}

#[test]
fn semantic_ast_projects_clock_duration_metadata() {
    let doc = Org::parse("* Work\nCLOCK: [2003-09-16 Tue 09:39] =>  1:02\n").document();
    assert_clean_projection(&doc);

    let clock = doc.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            orgize::ast::ElementData::Clock(clock) => Some(clock),
            _ => None,
        })
        .expect("clock element");

    assert_eq!(clock.duration.as_deref(), Some("1:02"));
    assert_eq!(clock.parsed_duration.as_ref().unwrap().total_seconds, 3_720);
}

#[test]
fn semantic_ast_projects_two_digit_numeric_priority() {
    let doc = Org::parse("* TODO [#64] Numeric high\n").document();
    assert_clean_projection(&doc);

    let section = &doc.sections[0];
    assert_eq!(section.priority.raw_cookie(), Some("64"));
    assert_eq!(section.priority.effective_text(), "64");
    assert_eq!(section.priority.org_priority_score(), Some(3_000));
    assert_eq!(
        section.priority.range_status(),
        PriorityRangeStatus::OutOfRange
    );
}

#[test]
fn semantic_ast_projects_letter_priority_outside_default_profile() {
    let doc = Org::parse("* TODO [#D] Outside default profile\n").document();
    assert_clean_projection(&doc);

    let section = &doc.sections[0];
    assert_eq!(section.priority.raw_cookie(), Some("D"));
    assert_eq!(section.priority.effective_text(), "D");
    assert_eq!(section.priority.org_priority_score(), Some(-1_000));
    assert_eq!(
        section.priority.range_status(),
        PriorityRangeStatus::OutOfRange
    );
}

#[test]
fn semantic_ast_ignores_invalid_priority_cookie_shapes() {
    for invalid_cookie in ["[#a]", "[#xx]", "[#65]"] {
        let input = format!("* TODO {invalid_cookie} Not a priority\n");
        let doc = Org::parse(input.as_str()).document();
        assert_clean_projection(&doc);

        let section = &doc.sections[0];
        assert!(section.priority.is_default());
        assert_eq!(section.priority.raw_cookie(), None);
        assert_eq!(section.priority.effective_text(), "B");
        assert_eq!(section.priority.org_priority_score(), Some(1_000));
        assert_eq!(
            section.priority.range_status(),
            PriorityRangeStatus::InRange
        );
    }
}

#[test]
fn semantic_ast_projects_property_profile_allowed_values() {
    let doc = Org::parse(
        r#"#+PROPERTY: Effort_ALL 0 0:30 "1 hour"
#+PROPERTY: Status_all todo done
* Project
:PROPERTIES:
:Owner_ALL: "Sarah Connor" Jim ""
:Owner: Sarah Connor
:END:
** Child
:PROPERTIES:
:Owner: Jim
:END:
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let profile = doc.property_profile();
    assert_eq!(profile.inheritance, PropertyInheritancePolicy::All);
    assert!(
        profile
            .inherited_keys
            .iter()
            .any(|key| key.eq_ignore_ascii_case("Owner"))
    );

    let fixed = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "VISIBILITY_ALL")
        .expect("fixed Org visibility values");
    assert_eq!(fixed.scope, PropertyAllowedValueScope::FixedGlobal);
    assert_eq!(fixed.values, ["folded", "children", "content", "all"]);

    let effort = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "Effort_ALL")
        .expect("document Effort_ALL values");
    assert_eq!(effort.scope, PropertyAllowedValueScope::Document);
    assert_eq!(effort.property, "Effort");
    assert_eq!(effort.values, ["0", "0:30", "1 hour"]);

    let status = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "Status_all")
        .expect("document Status_all values");
    assert_eq!(status.scope, PropertyAllowedValueScope::Document);
    assert_eq!(status.property, "Status");
    assert_eq!(status.values, ["todo", "done"]);

    let owner = profile
        .allowed_values
        .iter()
        .find(|record| record.descriptor_key == "Owner_ALL")
        .expect("section Owner_ALL values");
    assert!(matches!(
        &owner.scope,
        PropertyAllowedValueScope::Section { title, .. } if title == "Project"
    ));
    assert_eq!(owner.values, ["Sarah Connor", "Jim", ""]);
}

#[test]
fn semantic_ast_projects_property_schema_registry_validates_loaded_contracts() {
    let doc = Org::parse(
        r#"* File-backed capture
:PROPERTIES:
:PROPERTY_SCHEMA: [[file:schemas/capture.schema.json#wendao.capture.v1][Capture schema]]
:CAPTURE_KIND: surprise
:CAPTURE_SOURCE: conversation
:EXTRA: no
:END:
* Macro-backed capture
:PROPERTIES:
:PROPERTY_SCHEMA: {{{property_schema(wendao.capture.v1)}}}
:CAPTURE_KIND: idea
:CAPTURE_SOURCE: article
:MEMORY_POLICY: candidate
:END:
* Direct file capture
:PROPERTIES:
:PROPERTY_SCHEMA: capture.schema.json#wendao.capture.v1
:CAPTURE_KIND: note
:CAPTURE_SOURCE: article
:MEMORY_POLICY: candidate
:END:
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let registry = PropertySchemaRegistry::new([capture_schema_contract()]);
    let profile = doc.property_profile_with_schema_registry(&registry);
    assert_eq!(profile.schema_applications.len(), 3);

    let file_application = &profile.schema_applications[0];
    assert_eq!(
        file_application.reference.kind,
        PropertySchemaReferenceKind::OrgFileLink
    );
    assert_eq!(
        file_application.reference.normalized,
        "file:schemas/capture.schema.json#wendao.capture.v1"
    );
    assert_eq!(
        file_application.contract_id.as_deref(),
        Some("wendao.capture.v1")
    );
    assert!(matches!(
        &file_application.scope,
        PropertySchemaScope::Section { title, .. } if title == "File-backed capture"
    ));
    assert!(file_application.findings.iter().any(|finding| {
        finding.kind == PropertySchemaFindingKind::MissingRequiredProperty
            && finding.property.as_deref() == Some("MEMORY_POLICY")
    }));
    assert!(file_application.findings.iter().any(|finding| {
        finding.kind == PropertySchemaFindingKind::DisallowedValue
            && finding.property.as_deref() == Some("CAPTURE_KIND")
    }));
    assert!(file_application.findings.iter().any(|finding| {
        finding.kind == PropertySchemaFindingKind::UnknownProperty
            && finding.property.as_deref() == Some("EXTRA")
    }));

    let macro_application = &profile.schema_applications[1];
    assert_eq!(
        macro_application.reference.kind,
        PropertySchemaReferenceKind::Macro
    );
    assert_eq!(macro_application.reference.normalized, "wendao.capture.v1");
    assert_eq!(
        macro_application.contract_id.as_deref(),
        Some("wendao.capture.v1")
    );
    assert!(macro_application.findings.is_empty());

    let direct_file_application = &profile.schema_applications[2];
    assert_eq!(
        direct_file_application.reference.kind,
        PropertySchemaReferenceKind::File
    );
    assert_eq!(
        direct_file_application.reference.normalized,
        "capture.schema.json#wendao.capture.v1"
    );
    assert_eq!(
        direct_file_application.contract_id.as_deref(),
        Some("wendao.capture.v1")
    );
    assert!(direct_file_application.findings.is_empty());

    insta::assert_snapshot!(
        "semantic_ast__property_schema_applications",
        render_property_schema_applications(&profile.schema_applications)
    );
}

fn render_property_schema_applications(applications: &[PropertySchemaApplication]) -> String {
    let mut out = String::new();
    out.push_str(&format!("schema_applications={}\n", applications.len()));
    for application in applications {
        out.push_str(&format!(
            "application scope={} path={} level={} title={} reference={} raw={} normalized={} contract={}\n",
            application.scope.as_str(),
            property_schema_scope_path(&application.scope),
            property_schema_scope_level(&application.scope),
            property_schema_scope_title(&application.scope),
            application.reference.kind.as_str(),
            application.reference.raw,
            application.reference.normalized,
            application.contract_id.as_deref().unwrap_or("none")
        ));
        for finding in &application.findings {
            out.push_str(&format!(
                "  finding {} property={} actual={} expected={} source={}:{}-{}:{} message={}\n",
                finding.kind.as_str(),
                finding.property.as_deref().unwrap_or("none"),
                finding.actual.as_deref().unwrap_or("none"),
                if finding.expected.is_empty() {
                    "none".to_string()
                } else {
                    finding.expected.join("|")
                },
                finding.source.start.line,
                finding.source.start.column,
                finding.source.end.line,
                finding.source.end.column,
                finding.message
            ));
        }
    }
    out
}

fn property_schema_scope_path(scope: &PropertySchemaScope) -> String {
    match scope {
        PropertySchemaScope::Document => "document".to_string(),
        PropertySchemaScope::Section { outline_path, .. } => outline_path.join(" > "),
    }
}

fn property_schema_scope_level(scope: &PropertySchemaScope) -> String {
    match scope {
        PropertySchemaScope::Document => "none".to_string(),
        PropertySchemaScope::Section { level, .. } => level.to_string(),
    }
}

fn property_schema_scope_title(scope: &PropertySchemaScope) -> &str {
    match scope {
        PropertySchemaScope::Document => "document",
        PropertySchemaScope::Section { title, .. } => title,
    }
}

fn capture_schema_contract() -> PropertySchemaContract {
    PropertySchemaContract::new("wendao.capture.v1")
        .alias("file:schemas/capture.schema.json#wendao.capture.v1")
        .alias("capture.schema.json#wendao.capture.v1")
        .allow_unknown_properties(false)
        .field(PropertySchemaField::required(
            "CAPTURE_KIND",
            PropertySchemaValueRule::OneOf(
                [
                    "idea",
                    "articleNote",
                    "task",
                    "decision",
                    "preference",
                    "correction",
                    "memoryCandidate",
                    "evidence",
                    "note",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            ),
        ))
        .field(PropertySchemaField::required(
            "CAPTURE_SOURCE",
            PropertySchemaValueRule::OneOf(
                [
                    "conversation",
                    "url",
                    "file",
                    "selection",
                    "article",
                    "code",
                    "other",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            ),
        ))
        .field(PropertySchemaField::required(
            "MEMORY_POLICY",
            PropertySchemaValueRule::OneOf(
                [
                    "none",
                    "candidate",
                    "background",
                    "decision",
                    "transient",
                    "supersedes",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            ),
        ))
}
