use std::path::Path;

use serde_json::{Value, json};

use super::{
    elements::display_path,
    model::{DocumentElement, DocumentLanguage},
    source_selection::SourceSelector,
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
                "command": format!("{} query --selector <path:start-end> --view metadata", language.command_prefix())
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
            },
            {
                "kind": "query",
                "target": "direct-read",
                "command": format!("{} query --from-hook direct-source-read --selector <path:start-end> .", language.command_prefix())
            }
        ],
        "notes": [{
            "kind": "search-document",
            "message": "Document facts are parser-owned element metadata. Use direct-source-read only after an exact selector is selected."
        }]
    });
    print_json(&packet)
}

pub(super) fn print_query_json(
    language: DocumentLanguage,
    terms: &[String],
    root: &Path,
    facts: &[DocumentElement],
    content_output: bool,
) -> Result<(), String> {
    let query_terms = if terms.is_empty() {
        vec!["*".to_string()]
    } else {
        terms.to_vec()
    };
    let query_surface = if content_output {
        "content"
    } else {
        "metadata"
    };
    let packet = json!({
        "schemaId": "agent.semantic-protocols.semantic-document-query-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": language.id(),
        "providerId": "orgize",
        "binary": "asp",
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", language.id()),
        "method": "query/document",
        "projectRoot": packet_project_root(root),
        "query": query_terms.join(" "),
        "queryTerms": query_terms,
        "queryKind": "term",
        "querySurface": query_surface,
        "documentMode": query_surface,
        "matchCount": facts.len(),
        "matchLimit": 80,
        "matchesTruncated": facts.len() > 80,
        "documentFacts": facts.iter().take(80).map(|fact| document_fact_json(language, root, fact)).collect::<Vec<_>>(),
        "contentBlocks": if content_output { content_blocks_json(language, root, facts) } else { Vec::new() },
        "truncated": facts.len() > 80
    });
    print_json(&packet)
}

pub(super) fn print_selector_query_json(
    language: DocumentLanguage,
    selector: &str,
    selection: &SourceSelector,
    facts: &[DocumentElement],
    content_output: bool,
) -> Result<(), String> {
    let root = selection.path.parent().unwrap_or_else(|| Path::new("."));
    let query_surface = if content_output {
        "content"
    } else {
        "metadata"
    };
    let packet = json!({
        "schemaId": "agent.semantic-protocols.semantic-document-query-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": language.id(),
        "providerId": "orgize",
        "binary": "asp",
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", language.id()),
        "method": "query/document",
        "projectRoot": packet_project_root(root),
        "query": selector,
        "queryTerms": [selector],
        "queryKind": "selector",
        "querySurface": query_surface,
        "documentMode": query_surface,
        "matchCount": facts.len(),
        "matchLimit": 80,
        "matchesTruncated": facts.len() > 80,
        "documentFacts": facts.iter().take(80).map(|fact| document_fact_json(language, root, fact)).collect::<Vec<_>>(),
        "contentBlocks": if content_output { content_blocks_json(language, root, facts) } else { Vec::new() },
        "truncated": facts.len() > 80
    });
    print_json(&packet)
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
    let mut value = json!({
        "id": format!("{}:{}:{}:{}", fact.kind, path, fact.line, fact.end_line),
        "kind": fact.kind,
        "sourceKind": fact.source_kind,
        "name": fact_name(fact),
        "documentPath": path,
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
) -> Vec<Value> {
    facts
        .iter()
        .take(80)
        .filter_map(|fact| {
            let content = fact.content_text();
            (!content.trim().is_empty()).then(|| {
                let path = packet_path(root, &fact.path);
                json!({
                    "kind": "element",
                    "documentPath": path,
                    "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
                    "parserAuthority": language.parser_authority(),
                    "content": content
                })
            })
        })
        .collect()
}

fn location_json(path: &str, line: usize, end_line: usize) -> Value {
    json!({
        "path": path,
        "lineRange": format!("{}:{}", line.max(1), end_line.max(line).max(1))
    })
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
