use std::{fs, path::PathBuf, process::Command};

use orgize::{
    ast::{
        PropertySchemaContract, PropertySchemaField, PropertySchemaRegistry,
        PropertySchemaValueRule,
    },
    lint::{LintOptions, lint_org_with_options},
};

#[test]
fn lint_reports_property_schema_contract_issues() {
    let report = lint_org_with_options(
        r#"* Capture
:PROPERTIES:
:PROPERTY_SCHEMA: file:schemas/capture.schema.json#wendao.capture.v1
:CAPTURE_KIND: surprise
:CAPTURE_SOURCE: conversation
:EXTRA: no
:END:
"#,
        &LintOptions {
            property_schema_registry: PropertySchemaRegistry::new([capture_schema_contract()]),
            ..LintOptions::default()
        },
    );

    let messages = report
        .findings
        .iter()
        .filter(|finding| finding.code == "ORG040")
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        [
            "property schema `wendao.capture.v1` requires `MEMORY_POLICY`",
            "property `CAPTURE_KIND` value `surprise` is not allowed by schema: idea, note",
            "property `EXTRA` is not declared by schema `wendao.capture.v1`",
        ]
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_cli_loads_property_schema_registry_file_with_snapshot() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint");

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&fixture_dir)
        .args([
            "lint",
            "--format",
            "text",
            "--property-schema-registry",
            "property-schema-registry.json",
            "property-schema-registry-capture.org",
        ])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn property_schema_registry_json_contract_has_snapshot() {
    let schema: serde_json::Value = serde_json::from_str(property_schema_json_schema()).unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(property_schema_registry_fixture()).unwrap();

    let registry_properties = json_object_keys(&schema["$defs"]["registry"]["properties"]);
    let contract_properties = json_object_keys(&schema["$defs"]["contract"]["properties"]);
    let field_properties = json_object_keys(&schema["$defs"]["field"]["properties"]);
    let value_rule_options = schema["$defs"]["valueRule"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(value_rule_option_label)
        .collect::<Vec<_>>()
        .join(", ");
    let fixture_contracts = fixture["contracts"].as_array().unwrap();
    let fixture_fields = fixture_contracts[0]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["key"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join(", ");

    insta::assert_snapshot!(format!(
        "schema_id: {}\n\
         title: {}\n\
         registry_properties: {}\n\
         contract_properties: {}\n\
         field_properties: {}\n\
         value_rule_options: {}\n\
         fixture_schema: {}\n\
         fixture_contracts: {}\n\
         fixture_first_contract: {}\n\
         fixture_fields: {}\n",
        schema["$id"].as_str().unwrap(),
        schema["title"].as_str().unwrap(),
        registry_properties.join(", "),
        contract_properties.join(", "),
        field_properties.join(", "),
        value_rule_options,
        fixture["$schema"].as_str().unwrap(),
        fixture_contracts.len(),
        fixture_contracts[0]["id"].as_str().unwrap(),
        fixture_fields
    ));
}

#[test]
fn lint_cli_reports_invalid_property_schema_registry_with_snapshot() {
    let dir = test_dir("lint-invalid-property-schema-registry");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("registry.json"),
        r#"{"contracts":[{"fields":[{"key":"CAPTURE_KIND"}]}]}"#,
    )
    .unwrap();
    fs::write(dir.join("notes.org"), "* Notes\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "lint",
            "--property-schema-registry",
            "registry.json",
            "notes.org",
        ])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

fn capture_schema_contract() -> PropertySchemaContract {
    PropertySchemaContract::new("wendao.capture.v1")
        .alias("file:schemas/capture.schema.json#wendao.capture.v1")
        .allow_unknown_properties(false)
        .field(PropertySchemaField::required(
            "CAPTURE_KIND",
            PropertySchemaValueRule::OneOf(
                ["idea", "note"].into_iter().map(str::to_string).collect(),
            ),
        ))
        .field(PropertySchemaField::required(
            "CAPTURE_SOURCE",
            PropertySchemaValueRule::NonEmpty,
        ))
        .field(PropertySchemaField::required(
            "MEMORY_POLICY",
            PropertySchemaValueRule::OneOf(
                ["none", "candidate", "background", "decision"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            ),
        ))
}

fn property_schema_registry_fixture() -> &'static str {
    include_str!("../fixtures/lint/property-schema-registry.json")
}

fn property_schema_json_schema() -> &'static str {
    include_str!("../../docs/20_parser/20.03_property_schema_registry.schema.json")
}

fn json_object_keys(value: &serde_json::Value) -> Vec<String> {
    let mut keys = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::from)
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

fn value_rule_option_label(value: &serde_json::Value) -> String {
    if let Some(enumerated) = value.get("enum").and_then(serde_json::Value::as_array) {
        return format!(
            "string:{}",
            enumerated
                .iter()
                .map(|value| value.as_str().unwrap())
                .collect::<Vec<_>>()
                .join("|")
        );
    }
    format!(
        "object:{}",
        value["properties"]["kind"]["const"].as_str().unwrap()
    )
}

fn command_snapshot(output: std::process::Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status.code().unwrap_or_default(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    )
}

fn test_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("orgize-cli-tests")
        .join(format!("{name}-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}
