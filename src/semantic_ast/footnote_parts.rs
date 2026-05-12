//! Small state machines for semantic footnote projection.

use crate::syntax::{SyntaxElement, SyntaxKind};

#[derive(Default)]
pub(super) struct FootnoteDefParts {
    pub(super) label: String,
    pub(super) content: Vec<SyntaxElement>,
    saw_fn_prefix: bool,
    saw_label_colon: bool,
    after_marker: bool,
}

impl FootnoteDefParts {
    pub(super) fn push(mut self, element: SyntaxElement) -> Self {
        match element.kind() {
            SyntaxKind::AFFILIATED_KEYWORD | SyntaxKind::L_BRACKET => {}
            SyntaxKind::TEXT if !self.saw_fn_prefix => {
                self.saw_fn_prefix = true;
            }
            SyntaxKind::COLON if self.saw_fn_prefix && !self.saw_label_colon => {
                self.saw_label_colon = true;
            }
            SyntaxKind::R_BRACKET if self.saw_label_colon => {
                self.after_marker = true;
            }
            _ if self.after_marker => self.content.push(element),
            SyntaxKind::TEXT if self.saw_label_colon => {
                self.label.push_str(
                    element
                        .as_token()
                        .map(|token| token.text())
                        .unwrap_or_default(),
                );
            }
            _ => {}
        }
        self
    }
}

#[derive(Default)]
pub(super) struct FootnoteRefParts {
    pub(super) label: String,
    pub(super) definition: Vec<SyntaxElement>,
    saw_fn_prefix: bool,
    saw_label_colon: bool,
    in_definition: bool,
}

impl FootnoteRefParts {
    pub(super) fn push(mut self, element: SyntaxElement) -> Self {
        match element.kind() {
            SyntaxKind::L_BRACKET => {}
            SyntaxKind::TEXT if !self.saw_fn_prefix => {
                self.saw_fn_prefix = true;
            }
            SyntaxKind::COLON if self.saw_fn_prefix && !self.saw_label_colon => {
                self.saw_label_colon = true;
            }
            SyntaxKind::COLON if self.saw_label_colon && !self.in_definition => {
                self.in_definition = true;
            }
            _ if self.in_definition => self.definition.push(element),
            SyntaxKind::TEXT if self.saw_label_colon => {
                self.label.push_str(
                    element
                        .as_token()
                        .map(|token| token.text())
                        .unwrap_or_default(),
                );
            }
            _ => {}
        }
        self
    }
}
