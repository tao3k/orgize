//! Document-local lookup for Scheme-classified Org link paths.

use std::collections::HashMap;

use crate::org_aot::{org_link_kind, org_link_protocol, org_link_protocol_path};

use super::settings::expand_link_abbreviation;
use super::{
    AstMut, AstRef, AttachmentLink, AttachmentLinkSearch, AttachmentLinkSearchKind, Diagnostic,
    DiagnosticKind, Document, FileLink, FileLinkPathKind, LinkSearch, LinkSearchKind, LinkTarget,
    ObjectData, ParsedAnnotation, TargetDefinition, TargetKind,
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
    let mut paths = Vec::new();
    document.visit(|node| {
        if let AstRef::Object(object) = node
            && let ObjectData::Link(link) = &object.data
            && matches!(link.target, LinkTarget::Unresolved(_))
        {
            paths.push(link.path().to_string());
        }
    });
    let plans = if paths.is_empty() {
        HashMap::new()
    } else {
        let refs = paths.iter().map(String::as_str).collect::<Vec<_>>();
        let rows =
            crate::org_aot::native_semantic_rows(18, &refs).expect("initialized native link batch");
        assert_eq!(rows.len(), paths.len(), "native link count");
        paths
            .into_iter()
            .zip(rows)
            .map(|(path, row)| {
                assert_eq!(row.len(), 10, "native link row arity");
                (path, row)
            })
            .collect::<HashMap<_, _>>()
    };
    let mut diagnostics = Vec::new();
    document.visit_mut(|node| {
        let AstMut::Object(object) = node else {
            return;
        };
        let ObjectData::Link(link) = &mut object.data else {
            return;
        };
        // Parser-owned explicit links enter as Unresolved.  Scheme-AOT radio
        // matches already carry their document-local target decision; running
        // them through ordinary path lookup would invent ambiguity diagnostics.
        if !matches!(link.target, LinkTarget::Unresolved(_)) {
            return;
        }
        let path = link.path().to_string();
        let row = plans.get(&path).expect("admitted native link path");
        let kind = row[0].as_str();
        let key = row[1].as_str();
        let matches = counts.get(key).copied().unwrap_or_default();
        if matches == 1
            && !link.has_description()
            && let Some(alias) = aliases.get(key)
        {
            link.default_description = alias.clone();
        }
        let protocol = row[2].as_str();
        if protocol.eq_ignore_ascii_case("attachment") {
            let projected_search = project_link_search(row);
            let search = projected_search
                .as_ref()
                .map(|search| AttachmentLinkSearch {
                    raw: search.raw.clone(),
                    kind: match search.kind {
                        LinkSearchKind::Headline => AttachmentLinkSearchKind::Headline,
                        LinkSearchKind::CustomId => AttachmentLinkSearchKind::CustomId,
                        LinkSearchKind::Regexp => AttachmentLinkSearchKind::Regexp,
                        LinkSearchKind::LineNumber => AttachmentLinkSearchKind::LineNumber,
                        LinkSearchKind::Text => AttachmentLinkSearchKind::Text,
                    },
                });
            link.search = projected_search;
            link.attachment = Some(Box::new(AttachmentLink {
                path: row[5].clone(),
                search,
            }));
        }
        let expanded = (kind == "uri")
            .then(|| expand_link_abbreviation(protocol, &row[3], &abbreviations))
            .flatten();
        if protocol == "file" {
            let file_path = row[4].as_str();
            let path_kind = match row[6].as_str() {
                "empty" => FileLinkPathKind::Empty,
                "absolute" => FileLinkPathKind::Absolute,
                "home-relative" => FileLinkPathKind::HomeRelative,
                "remote" => FileLinkPathKind::Remote,
                _ => FileLinkPathKind::Relative,
            };
            let search = project_link_search(row);
            link.search = search.clone();
            link.file = Some(Box::new(FileLink {
                protocol: protocol.to_owned(),
                path: file_path.to_owned(),
                path_kind,
                search,
            }));
        } else if kind == "id" {
            link.search = project_link_search(row);
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
                path: row[3].clone(),
            },
            "id" if matches == 0 => LinkTarget::Uri {
                protocol: protocol.to_owned(),
                path: row[3].clone(),
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

fn project_link_search(row: &[String]) -> Option<LinkSearch> {
    let raw = row[7].as_str();
    if raw.is_empty() {
        return None;
    }
    let kind = match row[8].as_str() {
        "headline" => LinkSearchKind::Headline,
        "custom-id" => LinkSearchKind::CustomId,
        "regexp" => LinkSearchKind::Regexp,
        "line-number" => LinkSearchKind::LineNumber,
        _ => LinkSearchKind::Text,
    };
    Some(LinkSearch {
        raw: raw.to_owned(),
        kind,
        normalized: super::org_native_values::scalar(
            "link-search-normalize",
            &[&row[8], raw, &row[9]],
        ),
    })
}
