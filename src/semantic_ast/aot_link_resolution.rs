//! Document-local lookup for Scheme-classified Org link paths.

use std::collections::HashMap;

use crate::org_aot::{
    org_link_file_path, org_link_file_path_kind, org_link_kind, org_link_protocol,
    org_link_protocol_path, org_link_search, org_link_search_kind, org_link_search_value,
    org_link_target_key,
};

use super::settings::expand_link_abbreviation;
use super::{
    AstMut, AstRef, Diagnostic, DiagnosticKind, Document, FileLink, FileLinkPathKind, LinkSearch,
    LinkSearchKind, LinkTarget, ObjectData, ParsedAnnotation, TargetDefinition, TargetKind,
};

pub(super) fn resolve_document_links(document: &mut Document<ParsedAnnotation>) {
    let mut code_refs = Vec::new();
    document.visit(|node| {
        let AstRef::BlockLine(line) = node else {
            return;
        };
        if let Some(reference) = &line.code_ref {
            code_refs.push(TargetDefinition {
                ann: line.ann.clone(),
                kind: TargetKind::CodeRef,
                key: format!("coderef:{}", reference.name),
                value: reference.name.clone(),
                raw: reference.raw.clone(),
                alias: Vec::new(),
            });
        }
    });
    document.targets.extend(code_refs);

    let mut counts = HashMap::<String, usize>::new();
    let mut aliases = HashMap::new();
    for target in &document.targets {
        *counts.entry(target.key.clone()).or_default() += 1;
        if !target.alias.is_empty() {
            aliases.insert(target.key.clone(), target.alias.clone());
        }
    }
    let abbreviations = document.link_abbreviations.clone();
    let mut diagnostics = Vec::new();
    document.visit_mut(|node| {
        let AstMut::Object(object) = node else {
            return;
        };
        let ObjectData::Link(link) = &mut object.data else {
            return;
        };
        let path = link.path().to_string();
        let kind = org_link_kind(&path);
        let key = org_link_target_key(&path);
        let matches = counts.get(key).copied().unwrap_or_default();
        if matches == 1
            && !link.has_description()
            && let Some(alias) = aliases.get(key)
        {
            link.default_description = alias.clone();
        }
        let protocol = org_link_protocol(&path);
        let expanded = (kind == "uri")
            .then(|| {
                expand_link_abbreviation(protocol, org_link_protocol_path(&path), &abbreviations)
            })
            .flatten();
        if protocol == "file" {
            let file_path = org_link_file_path(&path);
            let path_kind = match org_link_file_path_kind(file_path) {
                "empty" => FileLinkPathKind::Empty,
                "absolute" => FileLinkPathKind::Absolute,
                "home-relative" => FileLinkPathKind::HomeRelative,
                "remote" => FileLinkPathKind::Remote,
                _ => FileLinkPathKind::Relative,
            };
            let search = project_link_search(&path);
            link.search = search.clone();
            link.file = Some(Box::new(FileLink {
                protocol: protocol.to_owned(),
                path: file_path.to_owned(),
                path_kind,
                search,
            }));
        } else if kind == "id" {
            link.search = project_link_search(&path);
        }
        link.target = match kind {
            "uri"
                if expanded
                    .as_deref()
                    .is_some_and(|value| org_link_kind(value) == "uri") =>
            {
                let expanded = expanded.as_deref().expect("checked expanded URI");
                LinkTarget::Uri {
                    protocol: org_link_protocol(expanded).to_owned(),
                    path: org_link_protocol_path(expanded).to_owned(),
                }
            }
            "uri" => LinkTarget::Uri {
                protocol: protocol.to_owned(),
                path: org_link_protocol_path(&path).to_owned(),
            },
            "id" if matches == 0 => LinkTarget::Uri {
                protocol: protocol.to_owned(),
                path: org_link_protocol_path(&path).to_owned(),
            },
            "custom-id" => LinkTarget::Internal(key.to_owned()),
            _ if matches == 1 => LinkTarget::Internal(key.to_owned()),
            _ => {
                if matches > 1 {
                    diagnostics.push(Diagnostic {
                        range: object.ann.range,
                        kind: DiagnosticKind::Conversion,
                        message: format!("internal link `{path}` is ambiguous"),
                    });
                } else if matches!(kind, "headline" | "footnote" | "code-ref") {
                    diagnostics.push(Diagnostic {
                        range: object.ann.range,
                        kind: DiagnosticKind::Conversion,
                        message: format!("internal link target `{path}` was not found"),
                    });
                }
                LinkTarget::Unresolved(path)
            }
        };
    });
    document.diagnostics.extend(diagnostics);
}

fn project_link_search(path: &str) -> Option<LinkSearch> {
    let raw = org_link_search(path);
    if raw.is_empty() {
        return None;
    }
    let kind = match org_link_search_kind(raw) {
        "headline" => LinkSearchKind::Headline,
        "custom-id" => LinkSearchKind::CustomId,
        "regexp" => LinkSearchKind::Regexp,
        "line-number" => LinkSearchKind::LineNumber,
        _ => LinkSearchKind::Text,
    };
    Some(LinkSearch {
        raw: raw.to_owned(),
        kind,
        normalized: org_link_search_value(raw).to_lowercase(),
    })
}
