use std::path::Path;

use serde_json::{Value, json};

use super::{
    elements::display_path,
    model::{DocumentElement, DocumentLanguage},
    source_selection::{SourceSelector, structural_selector_fragment},
};

pub(super) fn print_search_json(
    language: DocumentLanguage,
    view: &str,
    root: &Path,
    facts: &[DocumentElement],
    query: Option<&str>,
) -> Result<(), String> {
    let packet = json!({
        "schemaId": "agent.semantic-protocols.semantic-document-search-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": language.id(),
        "providerId": "orgize",
        "binary": "asp",
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", language.id()),
        "method": format!("search/{view}"),
        "projectRoot": packet_project_root(root),
        "view": view,
        "documentMode": "metadata",
        "query": query.unwrap_or_default(),
        "documentCount": document_count(facts),
        "factCount": facts.len(),
        "owners": owners_json(language, root, facts),
        "documentFacts": facts.iter().map(|fact| document_fact_json(language, root, fact)).collect::<Vec<_>>(),
        "nextActions": [
            {
                "kind": "query",
                "target": "term",
                "command": format!("{} query --term <term> --view metadata", language.command_prefix())
            },
            {
                "kind": "query",
                "target": "selector",
                "command": format!("{} query --selector <structural-selector> --view metadata", language.command_prefix())
            },
            {
                "kind": "query",
                "target": "kind",
                "command": format!("{} query --kind <element-kind> --view metadata", language.command_prefix())
            },
            {
                "kind": "query",
                "target": "field",
                "command": format!("{} query --field <key=value> --view metadata", language.command_prefix())
            },
            {
                "kind": "query",
                "target": "content",
                "command": format!("{} query --term <term> --content", language.command_prefix())
            }
        ],
        "notes": [{
            "kind": "search-document",
            "message": "Document facts are parser-owned element metadata. Use emitted structuralSelector values for query --selector and --content."
        }]
    });
    print_json(&packet)
}

const DOCUMENT_QUERY_PACKET_SCHEMA_ID: &str =
    "agent.semantic-protocols.semantic-document-query-packet";
const DOCUMENT_QUERY_PACKET_SCHEMA_VERSION: &str = "1";
const DOCUMENT_QUERY_PACKET_SCHEMA_AUTHORITY: &str =
    "https://tao3k.github.io/agent-semantic-protocols/schemas/";

pub(super) struct DocumentQueryEvidence {
    execution_command_digest: String,
}

struct DocumentQueryPacketInput<'a> {
    language: DocumentLanguage,
    query: String,
    query_terms: Vec<String>,
    query_kind: &'static str,
    root: &'a Path,
    facts: &'a [DocumentElement],
    content_output: bool,
    evidence: DocumentQueryEvidence,
}

pub(super) fn document_query_evidence(
    paths: impl IntoIterator<Item = std::path::PathBuf>,
    owner_path: Option<&Path>,
) -> Result<DocumentQueryEvidence, String> {
    let execution_command_digest = provider_execution_command_digest()?;
    document_query_evidence_with_digest(paths, owner_path, execution_command_digest)
}

pub(super) fn print_query_json(
    language: DocumentLanguage,
    terms: &[String],
    root: &Path,
    facts: &[DocumentElement],
    content_output: bool,
    evidence: DocumentQueryEvidence,
) -> Result<(), String> {
    let query_terms = if terms.is_empty() {
        vec!["*".to_string()]
    } else {
        terms.to_vec()
    };
    let packet = build_document_query_packet(DocumentQueryPacketInput {
        language,
        query: query_terms.join(" "),
        query_terms,
        query_kind: "term",
        root,
        facts,
        content_output,
        evidence,
    })?;
    print_json(&packet)
}

pub(super) fn print_selector_query_json(
    language: DocumentLanguage,
    selector: &str,
    selection: &SourceSelector,
    facts: &[DocumentElement],
    content_output: bool,
    evidence: DocumentQueryEvidence,
) -> Result<(), String> {
    let root = if selection.structural_selector.is_some() {
        Path::new(".")
    } else {
        selection.path.parent().unwrap_or_else(|| Path::new("."))
    };
    let packet = build_document_query_packet(DocumentQueryPacketInput {
        language,
        query: selector.to_string(),
        query_terms: vec![selector.to_string()],
        query_kind: "selector",
        root,
        facts,
        content_output,
        evidence,
    })?;
    print_json(&packet)
}

fn document_query_evidence_with_digest(
    paths: impl IntoIterator<Item = std::path::PathBuf>,
    owner_path: Option<&Path>,
    execution_command_digest: String,
) -> Result<DocumentQueryEvidence, String> {
    let _ = (paths.into_iter(), owner_path);
    Ok(DocumentQueryEvidence {
        execution_command_digest,
    })
}

fn build_document_query_packet(input: DocumentQueryPacketInput<'_>) -> Result<Value, String> {
    let execution_command_digest = input.evidence.execution_command_digest.clone();
    let query_surface = if input.content_output {
        "content"
    } else {
        "metadata"
    };
    let document_facts = input
        .facts
        .iter()
        .take(80)
        .map(|fact| document_fact_json(input.language, input.root, fact))
        .collect::<Vec<_>>();
    let content_blocks = if input.content_output {
        content_blocks_json(input.language, input.root, input.facts)?
    } else {
        Vec::new()
    };
    Ok(json!({
        "schemaId": DOCUMENT_QUERY_PACKET_SCHEMA_ID,
        "schemaVersion": DOCUMENT_QUERY_PACKET_SCHEMA_VERSION,
        "schemaAuthority": DOCUMENT_QUERY_PACKET_SCHEMA_AUTHORITY,
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": input.language.id(),
        "providerId": "orgize",
        "binary": env!("CARGO_PKG_NAME"),
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", input.language.id()),
        "method": "query/document",
        "projectRoot": packet_project_root(input.root),
        "query": input.query,
        "queryTerms": input.query_terms,
        "queryKind": input.query_kind,
        "querySurface": query_surface,
        "documentMode": query_surface,
        "matchCount": input.facts.len(),
        "matchLimit": 80,
        "matchesTruncated": input.facts.len() > 80,
        "documentFacts": document_facts,
        "contentBlocks": content_blocks,
        "executionCommandDigest": execution_command_digest,
        "truncated": input.facts.len() > 80
    }))
}

fn print_json(packet: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(packet)
        .map_err(|error| format!("failed to render JSON packet: {error}"))?;
    println!("{text}");
    Ok(())
}

fn document_count(facts: &[DocumentElement]) -> usize {
    facts
        .iter()
        .map(|fact| fact.path.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn packet_project_root(root: &Path) -> String {
    let candidate = display_path(root).replace('\\', "/");
    if candidate.is_empty() {
        ".".to_string()
    } else {
        candidate
    }
}

fn owners_json(language: DocumentLanguage, root: &Path, facts: &[DocumentElement]) -> Vec<Value> {
    facts
        .iter()
        .map(|fact| packet_path(root, &fact.path))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|path| {
            json!({
                "path": path,
                "role": "document",
                "parserAuthority": language.parser_authority()
            })
        })
        .collect()
}

fn document_fact_json(language: DocumentLanguage, root: &Path, fact: &DocumentElement) -> Value {
    let path = packet_path(root, &fact.path);
    let structural_selector = packet_structural_selector(language, &path, fact);
    let mut value = json!({
        "id": structural_selector.as_str(),
        "kind": fact.kind,
        "sourceKind": fact.source_kind,
        "name": fact_name(fact),
        "documentPath": path,
        "structuralSelector": structural_selector.as_str(),
        "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
        "parserAuthority": language.parser_authority(),
        "queryKeys": query_keys(fact),
        "attributes": fact_fields_json(fact)
    });
    if !fact.text.is_empty() {
        value["textSnippet"] = json!(fact.text);
    }
    value
}

fn content_blocks_json(
    language: DocumentLanguage,
    root: &Path,
    facts: &[DocumentElement],
) -> Result<Vec<Value>, String> {
    facts
        .iter()
        .take(80)
        .filter_map(|fact| {
            let content = fact.content_text();
            (!content.trim().is_empty()).then(|| {
                let path = packet_path(root, &fact.path);
                let structural_selector = packet_structural_selector(language, &path, fact);
                let block = json!({
                    "kind": "element",
                    "documentPath": path,
                    "structuralSelector": structural_selector.as_str(),
                    "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
                    "parserAuthority": language.parser_authority(),
                    "content": content
                });
                Ok(block)
            })
        })
        .collect()
}

fn provider_execution_command_digest() -> Result<String, String> {
    parse_provider_execution_command_digest(
        std::env::var("ASP_PROVIDER_EXECUTION_COMMAND_DIGEST").ok(),
    )
}

fn parse_provider_execution_command_digest(digest: Option<String>) -> Result<String, String> {
    let digest = digest.ok_or_else(|| {
        "ASP_PROVIDER_EXECUTION_COMMAND_DIGEST is required for JSON document queries".to_string()
    })?;
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err("ASP_PROVIDER_EXECUTION_COMMAND_DIGEST must use sha256:<64hex>".to_string());
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err("ASP_PROVIDER_EXECUTION_COMMAND_DIGEST must use sha256:<64hex>".to_string());
    }
    Ok(digest)
}

fn location_json(path: &str, line: usize, end_line: usize) -> Value {
    json!({
        "path": path,
        "lineRange": format!("{}:{}", line.max(1), end_line.max(line).max(1))
    })
}

fn packet_structural_selector(
    language: DocumentLanguage,
    packet_path: &str,
    fact: &DocumentElement,
) -> String {
    format!(
        "{}://{}#{}",
        language.id(),
        packet_path,
        structural_selector_fragment(&fact.structural_selector)
    )
}

fn packet_path(root: &Path, path: &str) -> String {
    let path = Path::new(path);
    let relative = if path.is_absolute() {
        path.strip_prefix(root).ok()
    } else {
        Some(path)
    };
    let mut candidate = relative
        .map(display_path)
        .or_else(|| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| ".".to_string())
        .replace('\\', "/");
    while let Some(stripped) = candidate.strip_prefix("./") {
        if stripped.is_empty() {
            break;
        }
        candidate = stripped.to_string();
    }
    if candidate.is_empty() {
        ".".to_string()
    } else {
        candidate
    }
}

fn fact_fields_json(fact: &DocumentElement) -> Value {
    let mut fields = serde_json::Map::new();
    for (key, value) in &fact.fields {
        fields.insert(key.clone(), json!(value));
    }
    if !fact.text.is_empty() {
        fields.insert("text".to_string(), json!(fact.text));
    }
    Value::Object(fields)
}

fn fact_name(fact: &DocumentElement) -> String {
    fact.fields
        .iter()
        .find(|(key, _)| matches!(key.as_str(), "title" | "key" | "target" | "lang"))
        .map(|(_, value)| value.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fact.kind.to_string())
}

fn query_keys(fact: &DocumentElement) -> Vec<String> {
    let mut keys = std::collections::BTreeSet::new();
    keys.insert(fact.kind.to_string());
    keys.insert(fact_name(fact));
    for (_, value) in &fact.fields {
        if !value.is_empty() {
            keys.insert(value.clone());
        }
    }
    if !fact.text.is_empty() {
        keys.insert(fact.text.clone());
    }
    keys.into_iter().collect()
}
