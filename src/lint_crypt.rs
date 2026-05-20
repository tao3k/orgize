//! Org Crypt lint advice.

use crate::ast::{CryptWarningKind, ParsedAst};

use super::lint_model::{location_for_offsets, LintFinding, LintSeverity};

pub(crate) fn crypt_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    for state in document.crypt_states() {
        if state.body_is_opaque {
            findings.push(LintFinding {
                code: "ORG038",
                severity: LintSeverity::Warning,
                message: format!(
                    "crypt-tagged section `{}` has an opaque body; orgize exposes headline/properties but does not decrypt Org Crypt payloads",
                    state.title
                ),
                location: location_for_offsets(
                    source,
                    state.source.range_start as usize,
                    state.source.range_end as usize,
                ),
            });
        }
        for warning in state.warnings {
            if warning.kind == CryptWarningKind::CryptKeyWithoutCryptTag {
                findings.push(LintFinding {
                    code: "ORG039",
                    severity: LintSeverity::Warning,
                    message: format!(
                        "section `{}` sets CRYPTKEY but has no visible crypt tag",
                        state.title
                    ),
                    location: location_for_offsets(
                        source,
                        state.source.range_start as usize,
                        state.source.range_end as usize,
                    ),
                });
            }
        }
    }
    findings
}
