//! Non-executing link protocol registry over semantic Org links.

use super::settings::{expand_link_abbreviation, link_abbreviation};
use super::{
    AstRef, Document, Keyword, LinkProtocolKind, LinkProtocolRecord, LinkProtocolSource,
    LinkSearch, ObjectData, OrgProtocolCall, OrgProtocolKind, OrgProtocolParameter,
    ParsedAnnotation,
};

impl Document<ParsedAnnotation> {
    /// Projects link protocols, `#+LINK` abbreviations, and org-protocol calls.
    ///
    /// This registry is intentionally inert: it records parser-visible link
    /// intent for lint, search, frontend, and agent consumers without opening
    /// files, dispatching custom handlers, or executing `shell:`/`elisp:`.
    pub fn link_protocol_records(&self) -> Vec<LinkProtocolRecord> {
        let mut records = link_abbreviation_records(&self.metadata);
        self.visit(|node| {
            let AstRef::Object(object) = node else {
                return;
            };
            let ObjectData::Link(link) = &object.data else {
                return;
            };
            if let Some(record) = link_record(
                &object.ann,
                link.path(),
                link.search.clone(),
                &self.link_abbreviations,
            ) {
                records.push(record);
            }
        });
        records.sort_by_key(|record| record.ann.range.start());
        records
    }
}

fn link_abbreviation_records(keywords: &[Keyword<ParsedAnnotation>]) -> Vec<LinkProtocolRecord> {
    keywords
        .iter()
        .filter(|keyword| keyword.key.eq_ignore_ascii_case("LINK"))
        .filter_map(|keyword| {
            let abbreviation = link_abbreviation(keyword)?;
            let org_protocol = org_protocol_for_target(&abbreviation.replacement);
            Some(LinkProtocolRecord {
                ann: keyword.ann.clone(),
                source: LinkProtocolSource::AbbreviationDefinition,
                protocol: abbreviation.name,
                kind: LinkProtocolKind::Abbreviation,
                raw: keyword.value.clone(),
                target: abbreviation.replacement.clone(),
                search: None,
                replacement: Some(abbreviation.replacement),
                org_protocol,
            })
        })
        .collect()
}

fn link_record(
    ann: &ParsedAnnotation,
    raw: &str,
    search: Option<LinkSearch>,
    abbreviations: &[super::LinkAbbreviation],
) -> Option<LinkProtocolRecord> {
    let (protocol, target) = raw.split_once(':')?;
    let protocol = protocol.to_ascii_lowercase();
    let replacement = abbreviations
        .iter()
        .find(|abbreviation| abbreviation.name.eq_ignore_ascii_case(&protocol))
        .map(|abbreviation| abbreviation.replacement.clone());
    let expanded = expand_link_abbreviation(&protocol, target, abbreviations);
    let kind = link_protocol_kind(&protocol, replacement.is_some());
    let org_protocol = if protocol == "org-protocol" {
        org_protocol_call(target)
    } else {
        expanded.as_deref().and_then(org_protocol_for_target)
    };
    Some(LinkProtocolRecord {
        ann: ann.clone(),
        source: LinkProtocolSource::Link,
        protocol: protocol.clone(),
        kind,
        raw: raw.to_string(),
        target: target.to_string(),
        search,
        replacement,
        org_protocol,
    })
}

fn link_protocol_kind(protocol: &str, is_abbreviation: bool) -> LinkProtocolKind {
    if is_abbreviation {
        return LinkProtocolKind::Abbreviation;
    }
    match protocol {
        "file" | "file+sys" | "file+emacs" | "file+shell" => LinkProtocolKind::File,
        "attachment" => LinkProtocolKind::Attachment,
        "id" => LinkProtocolKind::InternalId,
        "coderef" => LinkProtocolKind::CodeReference,
        "http" | "https" | "ftp" | "doi" | "irc" => LinkProtocolKind::Web,
        "mailto" | "news" | "gnus" | "rmail" | "mhe" | "bbdb" => LinkProtocolKind::Message,
        "docview" | "help" | "info" | "shortdoc" => LinkProtocolKind::Documentation,
        "shell" | "elisp" => LinkProtocolKind::Executable,
        "org-protocol" => LinkProtocolKind::OrgProtocol,
        _ => LinkProtocolKind::Custom,
    }
}

fn org_protocol_call(target: &str) -> Option<OrgProtocolCall> {
    let body = target.trim_start_matches('/');
    let subprotocol_end = body
        .find(['?', '/', ':'])
        .unwrap_or_else(|| body.trim_end_matches('/').len());
    let subprotocol = body[..subprotocol_end].trim();
    if subprotocol.is_empty() {
        return None;
    }
    Some(OrgProtocolCall {
        subprotocol: subprotocol.to_string(),
        kind: org_protocol_kind(subprotocol),
        parameters: org_protocol_parameters(body),
    })
}

fn org_protocol_for_target(raw: &str) -> Option<OrgProtocolCall> {
    let (protocol, target) = raw.trim().split_once(':')?;
    protocol
        .eq_ignore_ascii_case("org-protocol")
        .then(|| org_protocol_call(target))
        .flatten()
}

fn org_protocol_kind(subprotocol: &str) -> OrgProtocolKind {
    match subprotocol {
        "store-link" => OrgProtocolKind::StoreLink,
        "capture" => OrgProtocolKind::Capture,
        "open-source" => OrgProtocolKind::OpenSource,
        _ => OrgProtocolKind::Custom,
    }
}

fn org_protocol_parameters(body: &str) -> Vec<OrgProtocolParameter> {
    let Some((_, query)) = body.split_once('?') else {
        return Vec::new();
    };
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (key, value) = part
                .split_once('=')
                .map(|(key, value)| (key, Some(value)))
                .unwrap_or((part, None));
            OrgProtocolParameter {
                key: percent_decode(key),
                value: value.map(percent_decode),
                raw: part.to_string(),
            }
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let Some(byte) = hex_byte(bytes[index + 1], bytes[index + 2]) {
                    decoded.push(byte);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    match String::from_utf8(decoded) {
        Ok(value) => value,
        Err(error) => String::from_utf8_lossy(&error.into_bytes()).into_owned(),
    }
}

fn hex_byte(high: u8, low: u8) -> Option<u8> {
    Some(hex_value(high)? * 16 + hex_value(low)?)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
