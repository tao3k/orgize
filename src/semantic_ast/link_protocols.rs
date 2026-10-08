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
    let row = super::org_native_values::optional("uri-split", raw)?;
    let [protocol, target]: [String; 2] = row.try_into().expect("native URI arity");
    let protocol = super::org_native_values::scalar("ascii-lower", &[&protocol]);
    let replacement = super::settings::abbreviation_index(&protocol, abbreviations)
        .map(|index| abbreviations[index].replacement.clone());
    let expanded = expand_link_abbreviation(&protocol, &target, abbreviations);
    let kind = link_protocol_kind(&protocol, replacement.is_some());
    let org_protocol = if protocol == "org-protocol" {
        org_protocol_call(&target)
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
    let kind = super::org_native_values::scalar(
        "link-protocol-kind",
        &[protocol, if is_abbreviation { "true" } else { "false" }],
    );
    match kind.as_str() {
        "abbreviation" => LinkProtocolKind::Abbreviation,
        "file" => LinkProtocolKind::File,
        "attachment" => LinkProtocolKind::Attachment,
        "internal-id" => LinkProtocolKind::InternalId,
        "code-reference" => LinkProtocolKind::CodeReference,
        "web" => LinkProtocolKind::Web,
        "message" => LinkProtocolKind::Message,
        "documentation" => LinkProtocolKind::Documentation,
        "executable" => LinkProtocolKind::Executable,
        "org-protocol" => LinkProtocolKind::OrgProtocol,
        "custom" => LinkProtocolKind::Custom,
        _ => panic!("native protocol kind"),
    }
}
fn org_protocol_call(target: &str) -> Option<OrgProtocolCall> {
    native_org_protocol(target, "target")
}
fn org_protocol_for_target(raw: &str) -> Option<OrgProtocolCall> {
    native_org_protocol(raw, "raw")
}
fn native_org_protocol(raw: &str, mode: &str) -> Option<OrgProtocolCall> {
    let rows = super::org_native_values::rows("org-protocol", &[raw, mode]);
    let mut rows = rows.into_iter();
    let [subprotocol, kind]: [String; 2] =
        rows.next()?.try_into().expect("native org-protocol header");
    let kind = match kind.as_str() {
        "store-link" => OrgProtocolKind::StoreLink,
        "capture" => OrgProtocolKind::Capture,
        "open-source" => OrgProtocolKind::OpenSource,
        "custom" => OrgProtocolKind::Custom,
        _ => panic!("native org-protocol kind"),
    };
    let parameters = rows
        .map(|row| {
            let [raw, key, value, present]: [String; 4] =
                row.try_into().expect("native protocol parameter");
            OrgProtocolParameter {
                raw,
                key: percent_decode(&key),
                value: match present.as_str() {
                    "true" => Some(percent_decode(&value)),
                    "false" => None,
                    _ => panic!("native parameter presence"),
                },
            }
        })
        .collect();
    Some(OrgProtocolCall {
        subprotocol,
        kind,
        parameters,
    })
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
