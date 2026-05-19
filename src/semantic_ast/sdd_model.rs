//! Org-native SDD projection model.

use super::{SectionIndexSource, TodoKeyword};

/// One source-grounded SDD node projected from an Org section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SddNodeRecord {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub kind: SddKind,
    pub id: Option<String>,
    pub parent: Option<SddParentRef>,
    pub capability: Option<String>,
    pub viewpoint: Option<String>,
    pub concern: Option<String>,
    pub quality: Option<String>,
    pub rationale: Option<String>,
    pub slug: Option<String>,
    pub status: Option<SddStatusValue>,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
}

impl SddNodeRecord {
    /// Returns true when the node has an explicit machine identifier.
    pub fn has_id(&self) -> bool {
        self.id.as_ref().is_some_and(|id| !id.trim().is_empty())
    }
}

/// SDD node kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SddKind {
    System,
    Capability,
    View,
    Decision,
    Audit,
    Unknown(String),
}

impl SddKind {
    /// Parses an SDD kind property value.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "system" => Self::System,
            "capability" => Self::Capability,
            "view" => Self::View,
            "decision" => Self::Decision,
            "audit" => Self::Audit,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Stable label for compact and DTO consumers.
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => "system",
            Self::Capability => "capability",
            Self::View => "view",
            Self::Decision => "decision",
            Self::Audit => "audit",
            Self::Unknown(value) => value.as_str(),
        }
    }

    /// Returns true when this kind is recognized by the SDD projection.
    pub const fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    /// Returns true when this node kind can omit `SDD_PARENT`.
    pub const fn can_omit_parent(&self) -> bool {
        matches!(self, Self::System | Self::Unknown(_))
    }
}

/// SDD lifecycle/status value projected from `SDD_STATUS`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SddStatusValue {
    Draft,
    Review,
    Accepted,
    Deprecated,
    Superseded,
    Unknown(String),
}

impl SddStatusValue {
    /// Parses an SDD status property value.
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }

        Some(match value.to_ascii_lowercase().as_str() {
            "draft" => Self::Draft,
            "review" => Self::Review,
            "accepted" => Self::Accepted,
            "deprecated" => Self::Deprecated,
            "superseded" => Self::Superseded,
            _ => Self::Unknown(value.to_string()),
        })
    }

    /// Stable label for compact and DTO consumers.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Accepted => "accepted",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

/// Semantic parent edge from one SDD node to another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SddParentRef {
    pub raw: String,
    pub target_id: Option<String>,
    pub label: Option<String>,
}

impl SddParentRef {
    /// Parses an Org `id:` parent reference from a property value.
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }

        if let Some(inner) = raw
            .strip_prefix("[[id:")
            .and_then(|value| value.strip_suffix("]]"))
        {
            if let Some((target, label)) = inner.split_once("][") {
                return Some(Self {
                    raw: raw.to_string(),
                    target_id: non_empty_string(target),
                    label: non_empty_string(label),
                });
            }
            return Some(Self {
                raw: raw.to_string(),
                target_id: non_empty_string(inner),
                label: None,
            });
        }

        if let Some(target) = raw.strip_prefix("id:") {
            return Some(Self {
                raw: raw.to_string(),
                target_id: non_empty_string(target),
                label: None,
            });
        }

        Some(Self {
            raw: raw.to_string(),
            target_id: None,
            label: None,
        })
    }
}

/// Document-local SDD status projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SddStatus {
    pub records: Vec<SddNodeRecord>,
}

impl SddStatus {
    /// Returns true when the projection contains no SDD nodes.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Renders an agent-facing SDD status tree.
    pub fn to_compact_text(&self, path: &str) -> String {
        if self.records.is_empty() {
            return "[ok] orgize sdd status: no SDD nodes\n".to_string();
        }

        let mut output = String::new();
        output.push_str("[SDD] ");
        output.push_str(path);
        output.push('\n');
        output.push_str("architecture nodes: ");
        output.push_str(&self.records.len().to_string());
        output.push('\n');

        for record in &self.records {
            push_record_card(&mut output, path, record);
        }

        output
    }
}

fn push_record_card(output: &mut String, path: &str, record: &SddNodeRecord) {
    output.push_str("- ");
    output.push_str(record.kind.as_str());
    if let Some(status) = &record.status {
        output.push(' ');
        output.push_str(status.as_str());
    }
    output.push_str(": ");
    output.push_str(&record.title);
    output.push('\n');
    output.push_str("  @ ");
    output.push_str(path);
    output.push(':');
    output.push_str(&record.source.start.line.to_string());
    output.push(':');
    output.push_str(&record.source.start.column.to_string());
    output.push('\n');
    if let Some(id) = &record.id {
        output.push_str("  id: ");
        output.push_str(id);
        output.push('\n');
    }
    if let Some(parent) = &record.parent {
        output.push_str("  parent: ");
        if let Some(target_id) = &parent.target_id {
            output.push_str(target_id);
            if let Some(label) = &parent.label {
                output.push_str(" (");
                output.push_str(label);
                output.push(')');
            }
        } else {
            output.push_str(&parent.raw);
        }
        output.push('\n');
    }
    if let Some(capability) = &record.capability {
        output.push_str("  capability: ");
        output.push_str(capability);
        output.push('\n');
    }
    if let Some(viewpoint) = &record.viewpoint {
        output.push_str("  viewpoint: ");
        output.push_str(viewpoint);
        output.push('\n');
    }
    if let Some(concern) = &record.concern {
        output.push_str("  concern: ");
        output.push_str(concern);
        output.push('\n');
    }
    if let Some(quality) = &record.quality {
        output.push_str("  quality: ");
        output.push_str(quality);
        output.push('\n');
    }
    if let Some(rationale) = &record.rationale {
        output.push_str("  rationale: ");
        output.push_str(rationale);
        output.push('\n');
    }
    if let Some(slug) = &record.slug {
        output.push_str("  slug: ");
        output.push_str(slug);
        output.push('\n');
    }
}

fn non_empty_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
