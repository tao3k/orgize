use std::path::Path;

use serde_json::{Value, json};

use super::document_index::{
    DocumentFact, DocumentLanguage, SourceSelector, display_path, select_source,
};

pub(super) fn print_search_json(
    language: DocumentLanguage,
    view: &str,
    root: &Path,
    facts: &[DocumentFact],
    query: Option<&str>,
) -> Result<(), String> {
    let packet = json!({
        "schemaId": "agent.semantic-protocols.semantic-search-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": language.id(),
        "providerId": "orgize",
        "binary": "orgize",
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", language.id()),
        "method": format!("search/{view}"),
        "projectRoot": display_path(root),
        "view": view,
        "renderMode": "facts",
        "query": query.unwrap_or_default(),
        "header": {
            "kind": format!("search-{view}"),
            "fields": {
                "language": language.id(),
                "provider": "orgize",
                "documents": document_count(facts),
                "facts": facts.len()
            }
        },
        "nodes": [],
        "edges": [],
        "owners": owners_json(root, facts),
        "hits": facts.iter().map(|fact| search_hit_json(root, fact)).collect::<Vec<_>>(),
        "findings": [],
        "nextActions": [{
            "kind": "query",
            "target": "selector",
            "fields": { "command": format!("{} query --selector <path:start-end>", language.command_prefix()) }
        }],
        "notes": [{
            "kind": "search-document",
            "message": "Document facts are parser-owned and can be expanded with query --selector."
        }],
        "nativeSyntaxFacts": facts.iter().map(|fact| native_fact_json(root, fact)).collect::<Vec<_>>()
    });
    print_json(&packet)
}

pub(super) fn print_query_json(
    language: DocumentLanguage,
    terms: &[String],
    root: &Path,
    facts: &[DocumentFact],
) -> Result<(), String> {
    let query_terms = if terms.is_empty() {
        vec!["*".to_string()]
    } else {
        terms.to_vec()
    };
    let packet = json!({
        "schemaId": "agent.semantic-protocols.semantic-query-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": language.id(),
        "providerId": "orgize",
        "binary": "orgize",
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", language.id()),
        "method": "query/document",
        "projectRoot": display_path(root),
        "query": query_terms.join(" "),
        "queryTerms": query_terms,
        "outputMode": "outline",
        "matchCount": facts.len(),
        "matchLimit": 80,
        "matchesTruncated": facts.len() > 80,
        "matches": facts.iter().take(80).map(|fact| query_match_json(root, fact)).collect::<Vec<_>>(),
        "nativeSyntaxFacts": facts.iter().take(80).map(|fact| native_fact_json(root, fact)).collect::<Vec<_>>(),
        "truncated": facts.len() > 80
    });
    print_json(&packet)
}

pub(super) fn print_selector_query_json(
    language: DocumentLanguage,
    selector: &str,
    selection: &SourceSelector,
    source: &str,
) -> Result<(), String> {
    let root = selection.path.parent().unwrap_or_else(|| Path::new("."));
    let (line, end_line) = selection
        .range
        .unwrap_or_else(|| (1, source.lines().count().max(1)));
    let path = packet_path(root, &display_path(&selection.path));
    let selected = select_source(source, selection.range);
    let packet = json!({
        "schemaId": "agent.semantic-protocols.semantic-query-packet",
        "schemaVersion": "1",
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": language.id(),
        "providerId": "orgize",
        "binary": "orgize",
        "namespace": format!("agent.semantic-protocols.languages.{}.orgize", language.id()),
        "method": "query/document",
        "projectRoot": display_path(root),
        "query": selector,
        "queryTerms": [selector],
        "outputMode": "outline",
        "matchCount": 1,
        "matchLimit": 1,
        "matchesTruncated": false,
        "matches": [{
            "name": "selector",
            "kind": "selector",
            "location": location_json(&path, line, end_line),
            "read": format!("{path}:{line}:{end_line}"),
            "truncated": false,
            "fields": { "bytes": selected.len(), "command": format!("{} query --selector {selector} --code", language.command_prefix()) }
        }],
        "nativeSyntaxFacts": [],
        "truncated": false
    });
    print_json(&packet)
}

fn print_json(packet: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(packet)
        .map_err(|error| format!("failed to render JSON packet: {error}"))?;
    println!("{text}");
    Ok(())
}

fn document_count(facts: &[DocumentFact]) -> usize {
    facts
        .iter()
        .map(|fact| fact.path.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn owners_json(root: &Path, facts: &[DocumentFact]) -> Vec<Value> {
    facts
        .iter()
        .map(|fact| packet_path(root, &fact.path))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|path| {
            json!({
                "path": path,
                "role": "document",
                "public": false,
                "fields": {}
            })
        })
        .collect()
}

fn search_hit_json(root: &Path, fact: &DocumentFact) -> Value {
    let path = packet_path(root, &fact.path);
    json!({
        "kind": fact.kind,
        "ownerPath": path,
        "symbol": fact_name(fact),
        "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
        "score": 1.0,
        "reason": "document-fact",
        "fields": fact_fields_json(fact)
    })
}

fn query_match_json(root: &Path, fact: &DocumentFact) -> Value {
    let path = packet_path(root, &fact.path);
    json!({
        "name": fact_name(fact),
        "kind": fact.kind,
        "location": location_json(&path, fact.line, fact.end_line),
        "read": format!("{path}:{}:{}", fact.line, fact.end_line),
        "truncated": false,
        "fields": fact_fields_json(fact)
    })
}

fn native_fact_json(root: &Path, fact: &DocumentFact) -> Value {
    let path = packet_path(root, &fact.path);
    json!({
        "id": format!("{}:{}:{}:{}", fact.kind, path, fact.line, fact.end_line),
        "kind": fact.kind,
        "source": "native-parser",
        "languageKind": fact.kind,
        "name": fact_name(fact),
        "ownerPath": path,
        "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
        "queryKeys": query_keys(fact),
        "fields": fact_fields_json(fact)
    })
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
    let candidate = relative
        .map(display_path)
        .or_else(|| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| ".".to_string())
        .replace('\\', "/");
    if candidate.is_empty() {
        ".".to_string()
    } else {
        candidate
    }
}

fn fact_fields_json(fact: &DocumentFact) -> Value {
    let mut fields = serde_json::Map::new();
    for (key, value) in &fact.fields {
        fields.insert(key.clone(), json!(value));
    }
    if !fact.text.is_empty() {
        fields.insert("text".to_string(), json!(fact.text));
    }
    Value::Object(fields)
}

fn fact_name(fact: &DocumentFact) -> String {
    fact.fields
        .iter()
        .find(|(key, _)| matches!(key.as_str(), "title" | "key" | "target" | "lang"))
        .map(|(_, value)| value.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fact.kind.to_string())
}

fn query_keys(fact: &DocumentFact) -> Vec<String> {
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
