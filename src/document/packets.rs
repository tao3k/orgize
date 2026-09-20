use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{
    elements::display_path,
    model::{DocumentElement, DocumentLanguage},
    source_selection::{SourceSelector, structural_selector_fragment},
};

const DOCUMENT_QUERY_PACKET_SCHEMA_ID: &str =
    "agent.semantic-protocols.semantic-document-query-packet";
const DOCUMENT_QUERY_PACKET_SCHEMA_VERSION: &str = "1";
const DOCUMENT_QUERY_PACKET_SCHEMA_AUTHORITY: &str =
    "https://tao3k.github.io/agent-semantic-protocols/schemas/";

pub(super) struct DocumentQueryEvidence {
    pub(super) source_snapshot: Value,
    resolution_evidence: Value,
    snapshot_root: String,
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
    language: DocumentLanguage,
    paths: impl IntoIterator<Item = PathBuf>,
    owner_path: Option<&Path>,
    project_root: &Path,
    logical_args: &[String],
) -> Result<DocumentQueryEvidence, String> {
    let execution_command_digest = provider_execution_command_digest(language, logical_args)?;
    document_query_evidence_with_digest(
        language,
        paths,
        owner_path,
        project_root,
        execution_command_digest,
    )
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
    let selected_parent = selection.path.parent().unwrap_or_else(|| Path::new("."));
    let current_directory = std::env::current_dir()
        .ok()
        .and_then(|path| fs::canonicalize(path).ok());
    let root = if fs::canonicalize(selected_parent).ok() == current_directory {
        Path::new(".")
    } else {
        selected_parent
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
    language: DocumentLanguage,
    paths: impl IntoIterator<Item = PathBuf>,
    owner_path: Option<&Path>,
    project_root: &Path,
    execution_command_digest: String,
) -> Result<DocumentQueryEvidence, String> {
    let snapshot_base = if project_root.is_file() {
        project_root
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    } else {
        project_root
    };
    let snapshot_base = fs::canonicalize(snapshot_base).map_err(|error| {
        format!(
            "could not resolve document query snapshot root {}: {error}",
            snapshot_base.display()
        )
    })?;
    let mut leaves = BTreeMap::new();
    for path in paths {
        let absolute_path = fs::canonicalize(&path).map_err(|error| {
            format!(
                "could not resolve document query source {}: {error}",
                path.display()
            )
        })?;
        let snapshot_path = relative_snapshot_path(&snapshot_base, &absolute_path)?;
        let bytes = fs::read(&absolute_path).map_err(|error| {
            format!(
                "could not read document query source {}: {error}",
                absolute_path.display()
            )
        })?;
        let source_digest = blake3::hash(&bytes).to_hex().to_string();
        if leaves
            .insert(snapshot_path.clone(), source_digest)
            .is_some()
        {
            return Err(format!(
                "document query snapshot contains duplicate source path `{snapshot_path}`"
            ));
        }
    }
    let snapshot_root = workspace_merkle_root(&leaves);
    let provider_digest = document_provider_digest(language)?;
    let owner_evidence = owner_path
        .map(|path| {
            let absolute_path = fs::canonicalize(path).map_err(|error| {
                format!(
                    "could not resolve document query owner {}: {error}",
                    path.display()
                )
            })?;
            let snapshot_path = relative_snapshot_path(&snapshot_base, &absolute_path)?;
            let source_digest = leaves.get(&snapshot_path).ok_or_else(|| {
                format!("document query owner `{snapshot_path}` is absent from the source snapshot")
            })?;
            Ok::<_, String>((snapshot_path, format!("blake3:{source_digest}")))
        })
        .transpose()?;
    let mut resolution_evidence = json!({
        "schemaId": "asp.source-resolution.v1",
        "snapshotRoot": snapshot_root,
        "authority": "live-parser",
        "state": "live-hit",
        "parserArtifactDigest": provider_digest
    });
    if let Some((owner_path, owner_blob_digest)) = owner_evidence {
        resolution_evidence["ownerPath"] = json!(owner_path);
        resolution_evidence["ownerBlobDigest"] = json!(owner_blob_digest);
    }
    let source_snapshot = json!({
        "schemaId": "asp.source-snapshot.v1",
        "algorithm": "blake3-merkle-v1",
        "rootDigest": snapshot_root,
        "sourceKind": "filesystem",
        "leafCount": leaves.len(),
        "providerDigest": provider_digest
    });
    Ok(DocumentQueryEvidence {
        source_snapshot,
        resolution_evidence,
        snapshot_root,
        execution_command_digest,
    })
}

fn build_document_query_packet(input: DocumentQueryPacketInput<'_>) -> Result<Value, String> {
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
    let item_digest = query_projection_digest(
        input.language,
        &input.evidence.snapshot_root,
        input.query_kind,
        &input.query,
        &document_facts,
        &content_blocks,
    )?;
    Ok(json!({
        "schemaId": DOCUMENT_QUERY_PACKET_SCHEMA_ID,
        "schemaVersion": DOCUMENT_QUERY_PACKET_SCHEMA_VERSION,
        "schemaAuthority": DOCUMENT_QUERY_PACKET_SCHEMA_AUTHORITY,
        "protocolId": "agent.semantic-protocols.semantic-language",
        "protocolVersion": "1",
        "languageId": input.language.id(),
        "providerId": input.language.provider_id(),
        "binary": env!("CARGO_PKG_NAME"),
        "namespace": input.language.provider_namespace(),
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
        "sourceSnapshot": input.evidence.source_snapshot,
        "resolutionEvidence": input.evidence.resolution_evidence,
        "itemDigest": item_digest,
        "executionCommandDigest": input.evidence.execution_command_digest,
        "truncated": input.facts.len() > 80
    }))
}

fn print_json(packet: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(packet)
        .map_err(|error| format!("failed to render JSON packet: {error}"))?;
    println!("{text}");
    Ok(())
}

fn packet_project_root(root: &Path) -> String {
    let candidate = display_path(root).replace('\\', "/");
    if candidate.is_empty() {
        ".".to_string()
    } else {
        candidate
    }
}

fn document_fact_json(language: DocumentLanguage, root: &Path, fact: &DocumentElement) -> Value {
    let path = packet_path(root, &fact.path);
    let structural_selector = packet_structural_selector(language, &fact.path, fact);
    json!({
        "id": structural_selector.as_str(),
        "kind": fact.kind,
        "sourceKind": fact.source_kind,
        "name": fact_name(fact),
        "documentPath": path,
        "structuralSelector": structural_selector.as_str(),
        "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
        "parserAuthority": language.parser_authority(),
        "queryKeys": query_keys(fact),
        "attributes": fact_fields_json(fact),
        "textSnippet": fact.text
    })
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
                let structural_selector = packet_structural_selector(language, &fact.path, fact);
                let item_digest = content_block_digest(
                    language,
                    structural_selector.as_str(),
                    content.as_bytes(),
                );
                let block = json!({
                    "kind": "element",
                    "documentPath": path,
                    "structuralSelector": structural_selector.as_str(),
                    "location": location_json(&packet_path(root, &fact.path), fact.line, fact.end_line),
                    "parserAuthority": language.parser_authority(),
                    "content": content,
                    "itemDigest": item_digest
                });
                Ok(block)
            })
        })
        .collect()
}

fn relative_snapshot_path(snapshot_base: &Path, absolute_path: &Path) -> Result<String, String> {
    let relative = absolute_path.strip_prefix(snapshot_base).map_err(|_| {
        format!(
            "document query source {} is outside snapshot root {}",
            absolute_path.display(),
            snapshot_base.display()
        )
    })?;
    let mut components = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "document query snapshot path is not normalized: {}",
                    relative.display()
                ));
            }
        }
    }
    if components.is_empty() {
        return Err("document query snapshot path must name a source file".to_string());
    }
    Ok(components.join("/"))
}

fn workspace_merkle_root(leaves: &BTreeMap<String, String>) -> String {
    if leaves.is_empty() {
        return canonical_blake3_digest(b"asp.workspace-merkle-empty.v1", &[]);
    }
    let mut level = leaves
        .iter()
        .map(|(path, source_digest)| {
            canonical_blake3_digest(
                b"asp.workspace-file-leaf.v1",
                &[path.as_bytes(), source_digest.as_bytes()],
            )
        })
        .collect::<Vec<_>>();
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| {
                let left = &pair[0];
                let right = pair.get(1).unwrap_or(left);
                canonical_blake3_digest(
                    b"asp.workspace-merkle-node.v1",
                    &[left.as_bytes(), right.as_bytes()],
                )
            })
            .collect();
    }
    level.pop().expect("non-empty Merkle level")
}

fn document_provider_digest(language: DocumentLanguage) -> Result<String, String> {
    Ok(format!(
        "blake3:{}",
        canonical_blake3_digest(
            b"asp.semantic-document-parser-artifact.v1",
            &[
                language.provider_id().as_bytes(),
                env!("ORGIZE_PARSER_ARTIFACT_DIGEST").as_bytes(),
            ],
        )
    ))
}

fn content_block_digest(
    language: DocumentLanguage,
    structural_selector: &str,
    content: &[u8],
) -> String {
    format!(
        "blake3:{}",
        canonical_blake3_digest(
            b"asp.semantic-document-content-block.v1",
            &[
                language.provider_id().as_bytes(),
                structural_selector.as_bytes(),
                content,
            ],
        )
    )
}

fn query_projection_digest(
    language: DocumentLanguage,
    snapshot_root: &str,
    query_kind: &str,
    query: &str,
    document_facts: &[Value],
    content_blocks: &[Value],
) -> Result<String, String> {
    let facts = serde_json::to_vec(document_facts)
        .map_err(|error| format!("could not encode document facts for identity: {error}"))?;
    let blocks = serde_json::to_vec(content_blocks)
        .map_err(|error| format!("could not encode document content for identity: {error}"))?;
    Ok(format!(
        "blake3:{}",
        canonical_blake3_digest(
            b"asp.semantic-document-query-projection.v1",
            &[
                language.provider_id().as_bytes(),
                snapshot_root.as_bytes(),
                query_kind.as_bytes(),
                query.as_bytes(),
                &facts,
                &blocks,
            ],
        )
    ))
}

fn canonical_blake3_digest(domain: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(domain.len() as u64).to_be_bytes());
    hasher.update(domain);
    hasher.update(&(parts.len() as u64).to_be_bytes());
    for part in parts {
        hasher.update(&(part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher.finalize().to_hex().to_string()
}

fn provider_execution_command_digest(
    language: DocumentLanguage,
    logical_args: &[String],
) -> Result<String, String> {
    let executable = std::env::current_exe().map_err(|error| {
        format!("could not resolve current executable for JSON query evidence: {error}")
    })?;
    let mut hasher = Sha256::new();
    hash_command_component(&mut hasher, executable.as_os_str().as_encoded_bytes());
    hash_command_component(&mut hasher, language.id().as_bytes());
    hash_command_component(&mut hasher, b"query");
    for argument in logical_args {
        hash_command_component(&mut hasher, argument.as_bytes());
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn hash_command_component(hasher: &mut Sha256, component: &[u8]) {
    hasher.update((component.len() as u64).to_be_bytes());
    hasher.update(component);
}

fn location_json(path: &str, line: usize, end_line: usize) -> Value {
    json!({
        "path": path,
        "lineRange": format!("{}:{}", line.max(1), end_line.max(line).max(1))
    })
}

fn packet_structural_selector(
    language: DocumentLanguage,
    source_path: &str,
    fact: &DocumentElement,
) -> String {
    let source_path = Path::new(source_path);
    let source_path = if source_path.is_absolute() {
        source_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(source_path))
            .unwrap_or_else(|_| source_path.to_path_buf())
    };
    format!(
        "{}://{}#{}",
        language.id(),
        source_path.display(),
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
