//! Task-blocker lint advice for ordered TODO siblings.

use crate::ast::{ParsedAst, TaskBlockerRecord};

use super::lint_model::{LintFinding, LintSeverity, location_for_offsets};

pub(crate) fn task_blocker_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    document
        .task_blocker_records()
        .iter()
        .map(|record| task_blocker_finding(record, source))
        .collect()
}

fn task_blocker_finding(record: &TaskBlockerRecord, source: &str) -> LintFinding {
    LintFinding {
        code: "ORG029",
        severity: LintSeverity::Warning,
        message: format!(
            "task `{}` is blocked by previous open sibling `{}` because parent `{}` has local ORDERED property",
            record.blocked.title, record.blocker.title, record.parent.title
        ),
        location: location_for_offsets(
            source,
            record.blocked.source.range_start as usize,
            record.blocked.source.range_end as usize,
        ),
    }
}
