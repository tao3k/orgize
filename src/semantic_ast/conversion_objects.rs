//! Inline object conversion for tokens, markup, timestamps, links, citations, and macros.

use super::attachment_model::attachment_link_from_path;
use super::citation_metadata::{citation_key_range, citation_style};
use super::conversion::Converter;
use super::conversion_util::{strip_pair, strip_wrapping, text_range};
use super::footnote_parts::FootnoteRefParts;
use super::preprocessing::split_macro_args;
use super::settings::{expand_link_abbreviation, file_link, link_search};
use super::targets::{TargetLookup, is_strict_internal_link_path};
use super::timestamp_metadata::{timestamp_moment_range, timestamp_repeater, timestamp_warning};
use super::{
    Citation, CiteReference, DiagnosticKind, Link, LinkDescriptionState, LinkMediaKind, LinkPath,
    LinkTarget, MarkupKind, Object, ObjectData, ParsedAnnotation, Timestamp, TimestampKind,
    UnsupportedSyntaxKind,
};
use crate::{
    syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
    syntax_ast,
};
use rowan::ast::AstNode;
use rowan::{NodeOrToken, TextRange};

impl<'a> Converter<'a> {
    pub(super) fn object(&mut self, element: SyntaxElement) -> Option<Object<ParsedAnnotation>> {
        match element {
            NodeOrToken::Token(token) => self.object_token(token),
            NodeOrToken::Node(node) => self.object_node(node),
        }
    }

    pub(super) fn object_token(&self, token: SyntaxToken) -> Option<Object<ParsedAnnotation>> {
        match token.kind() {
            SyntaxKind::TEXT => Some(Object {
                ann: self.token_ann(&token),
                data: ObjectData::Plain(token.text().to_string()),
            }),
            SyntaxKind::NEW_LINE | SyntaxKind::WHITESPACE | SyntaxKind::BLANK_LINE => {
                Some(Object {
                    ann: self.token_ann(&token),
                    data: ObjectData::Plain(token.text().to_string()),
                })
            }
            _ => None,
        }
    }

    pub(super) fn object_node(&mut self, node: SyntaxNode) -> Option<Object<ParsedAnnotation>> {
        let data = match node.kind() {
            SyntaxKind::AFFILIATED_KEYWORD => return None,
            SyntaxKind::BOLD => self.markup(&node, MarkupKind::Bold),
            SyntaxKind::ITALIC => self.markup(&node, MarkupKind::Italic),
            SyntaxKind::UNDERLINE => self.markup(&node, MarkupKind::Underline),
            SyntaxKind::STRIKE => self.markup(&node, MarkupKind::Strike),
            SyntaxKind::SUPERSCRIPT => self.markup(&node, MarkupKind::Superscript),
            SyntaxKind::SUBSCRIPT => self.markup(&node, MarkupKind::Subscript),
            SyntaxKind::CODE => ObjectData::Code(strip_pair(&node.to_string()).to_string()),
            SyntaxKind::VERBATIM => ObjectData::Verbatim(strip_pair(&node.to_string()).to_string()),
            SyntaxKind::TIMESTAMP_ACTIVE
            | SyntaxKind::TIMESTAMP_INACTIVE
            | SyntaxKind::TIMESTAMP_DIARY => ObjectData::Timestamp(
                self.timestamp_node(&node)
                    .expect("timestamp kind must map to timestamp"),
            ),
            SyntaxKind::ENTITY => ObjectData::Entity(node.to_string()),
            SyntaxKind::LATEX_FRAGMENT => ObjectData::LatexFragment(node.to_string()),
            SyntaxKind::SNIPPET => self.export_snippet(&node),
            SyntaxKind::FN_REF => self.footnote_ref(&node),
            SyntaxKind::CITATION => self.citation(&node),
            #[cfg(feature = "syntax-org-fc")]
            SyntaxKind::CLOZE => self.cloze(&node),
            SyntaxKind::INLINE_CALL => self.inline_call(&node),
            SyntaxKind::INLINE_SRC => self.inline_src(&node),
            SyntaxKind::LINK => self.link(&node),
            SyntaxKind::TARGET => ObjectData::Target(strip_wrapping(&node.to_string(), "<<", ">>")),
            SyntaxKind::RADIO_TARGET => {
                ObjectData::RadioTarget(strip_wrapping(&node.to_string(), "<<<", ">>>"))
            }
            SyntaxKind::MACROS => self.macro_object(&node),
            SyntaxKind::COOKIE => ObjectData::StatisticCookie(node.to_string()),
            SyntaxKind::LINE_BREAK => ObjectData::LineBreak,
            kind => {
                self.diagnostic(
                    node.text_range(),
                    DiagnosticKind::UnsupportedObject,
                    format!("semantic AST has no dedicated object mapping for {kind:?}"),
                );
                ObjectData::Unknown {
                    kind: UnsupportedSyntaxKind::new(format!("{kind:?}")),
                    raw: node.to_string(),
                }
            }
        };

        Some(Object {
            ann: self.node_ann(&node),
            data,
        })
    }

    pub(super) fn markup(
        &mut self,
        node: &SyntaxNode,
        kind: MarkupKind,
    ) -> ObjectData<ParsedAnnotation> {
        ObjectData::Markup {
            kind,
            children: self.objects_from_elements(node.children_with_tokens()),
        }
    }

    pub(super) fn timestamp_node(&self, node: &SyntaxNode) -> Option<Timestamp> {
        let kind = match node.kind() {
            SyntaxKind::TIMESTAMP_ACTIVE => TimestampKind::Active,
            SyntaxKind::TIMESTAMP_INACTIVE => TimestampKind::Inactive,
            SyntaxKind::TIMESTAMP_DIARY => TimestampKind::Diary,
            _ => return None,
        };
        let syntax = syntax_ast::SyntaxTimestamp::cast(node.clone()).expect("timestamp node");
        let is_range = syntax.is_range();
        let (start, end) = timestamp_moment_range(node, is_range);
        Some(Timestamp {
            kind,
            raw: node.to_string(),
            is_range,
            start,
            end,
            repeater: timestamp_repeater(&syntax),
            warning: timestamp_warning(&syntax),
        })
    }

    pub(super) fn export_snippet(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        if let Some(snippet) = syntax_ast::Snippet::cast(node.clone()) {
            ObjectData::ExportSnippet {
                backend: snippet.backend().to_string(),
                value: snippet.value().to_string(),
            }
        } else {
            ObjectData::Unknown {
                kind: UnsupportedSyntaxKind::new("SNIPPET"),
                raw: node.to_string(),
            }
        }
    }

    pub(super) fn footnote_ref(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let parts = node
            .children_with_tokens()
            .take_while(|element| element.kind() != SyntaxKind::R_BRACKET)
            .fold(FootnoteRefParts::default(), FootnoteRefParts::push);

        ObjectData::FootnoteRef {
            label: (!parts.label.is_empty()).then_some(parts.label),
            resolved_label: None,
            definition: self.objects_from_elements(parts.definition),
        }
    }

    pub(super) fn citation(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let raw = node.to_string();
        let Some((head, body)) = raw
            .strip_prefix('[')
            .and_then(|raw| raw.strip_suffix(']'))
            .and_then(|inner| inner.split_once(':'))
        else {
            self.diagnostic(
                node.text_range(),
                DiagnosticKind::Conversion,
                "citation syntax node could not be split into head and body".into(),
            );
            return ObjectData::Unknown {
                kind: UnsupportedSyntaxKind::new("CITATION"),
                raw,
            };
        };

        let (style, variant) = citation_style(head);
        let node_start = usize::from(node.text_range().start());
        let body_start = node_start + 1 + head.len() + 1;
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut references = Vec::new();
        let mut saw_reference = false;
        let mut segment_start = 0;

        for segment in body.split(';') {
            let absolute_start = body_start + segment_start;
            segment_start += segment.len() + 1;

            if let Some((key_start, key_end)) = citation_key_range(segment) {
                saw_reference = true;
                references.push(CiteReference {
                    ann: self.ann(text_range(
                        absolute_start + key_start - 1,
                        absolute_start + key_end,
                    )),
                    id: segment[key_start..key_end].to_string(),
                    prefix: self
                        .objects_from_raw_minimal(&segment[..key_start - 1], absolute_start),
                    suffix: self
                        .objects_from_raw_minimal(&segment[key_end..], absolute_start + key_end),
                });
            } else if saw_reference {
                if segment.contains('@') {
                    self.diagnostic(
                        node.text_range(),
                        DiagnosticKind::Conversion,
                        format!("malformed citation segment `{}`", segment.trim()),
                    );
                }
                suffix.extend(self.objects_from_raw_minimal(segment, absolute_start));
            } else {
                if segment.contains('@') {
                    self.diagnostic(
                        node.text_range(),
                        DiagnosticKind::Conversion,
                        format!("malformed citation segment `{}`", segment.trim()),
                    );
                }
                prefix.extend(self.objects_from_raw_minimal(segment, absolute_start));
            }
        }

        if references.is_empty() {
            self.diagnostic(
                node.text_range(),
                DiagnosticKind::Conversion,
                "citation syntax node did not contain a citation reference".into(),
            );
        }

        ObjectData::Citation(Citation {
            style,
            variant,
            prefix,
            suffix,
            references,
        })
    }

    #[cfg(feature = "syntax-org-fc")]
    pub(super) fn cloze(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let syntax = syntax_ast::Cloze::cast(node.clone()).expect("cloze node");
        let text = syntax.text().collect::<Vec<_>>();
        ObjectData::Cloze {
            text: self.objects_from_elements(text),
            raw_text: syntax.text_raw(),
            hint: syntax.hint().map(|token| token.to_string()),
            id: syntax.id().map(|token| token.to_string()),
            raw: syntax.raw(),
        }
    }

    pub(super) fn inline_call(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let syntax = syntax_ast::InlineCall::cast(node.clone()).expect("inline call node");
        let raw = node.to_string();
        ObjectData::InlineCall {
            name: syntax.call().to_string(),
            arguments: syntax.arguments().to_string(),
            header: syntax.inside_header().map(|token| token.to_string()),
            end_header: syntax.end_header().map(|token| token.to_string()),
            raw,
        }
    }

    pub(super) fn inline_src(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let syntax = syntax_ast::InlineSrc::cast(node.clone()).expect("inline src node");
        let raw = node.to_string();
        ObjectData::InlineSrc {
            language: syntax.language().to_string(),
            parameters: syntax.parameters().map(|token| token.to_string()),
            value: syntax.value().to_string(),
            raw,
        }
    }

    pub(super) fn link(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let syntax = syntax_ast::SyntaxLink::cast(node.clone()).expect("link node");
        let path = syntax.path().to_string();
        let target = self.link_target(&path, node.text_range());
        let search = link_search(&path);
        let attachment = attachment_link_from_path(&path).map(Box::new);
        let file = file_link(&path, search.clone()).map(Box::new);
        let description = syntax.description().collect::<Vec<_>>();
        let caption = syntax
            .caption()
            .map(|caption| self.keyword(&caption.syntax, true));

        ObjectData::Link(Box::new(Link {
            path: LinkPath::new(path),
            target,
            default_description: Vec::new(),
            raw_description: syntax.description_raw(),
            description_state: if syntax.has_description() {
                LinkDescriptionState::Explicit
            } else {
                LinkDescriptionState::None
            },
            media_kind: if syntax.is_image() {
                LinkMediaKind::Image
            } else {
                LinkMediaKind::Normal
            },
            caption,
            search,
            attachment,
            file,
            description: self.objects_from_elements(description),
        }))
    }

    pub(super) fn link_target(&mut self, path: &str, range: TextRange) -> LinkTarget {
        match self.target_index.resolve(path) {
            TargetLookup::Found { key } => {
                return LinkTarget::Internal(key);
            }
            TargetLookup::Ambiguous { key, count } => {
                self.diagnostic(
                    range,
                    DiagnosticKind::Conversion,
                    format!("internal link target `{key}` is ambiguous across {count} definitions"),
                );
                return LinkTarget::Unresolved(path.to_string());
            }
            TargetLookup::Missing { key } if is_strict_internal_link_path(path) => {
                self.diagnostic(
                    range,
                    DiagnosticKind::Conversion,
                    format!("internal link target `{key}` was not found"),
                );
                return LinkTarget::Unresolved(path.to_string());
            }
            TargetLookup::Missing { .. } => {}
        }

        if let Some((protocol, path)) = path.split_once(':') {
            if let Some(expanded) =
                expand_link_abbreviation(protocol, path, &self.link_abbreviations)
            {
                return self.link_target(&expanded, range);
            }
            LinkTarget::Uri {
                protocol: protocol.to_string(),
                path: path.to_string(),
            }
        } else if path.starts_with('#') {
            LinkTarget::Internal(path.to_string())
        } else {
            LinkTarget::Unresolved(path.to_string())
        }
    }

    pub(super) fn macro_object(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let raw = node.to_string();
        let inner = strip_wrapping(&raw, "{{{", "}}}");
        let (name, args) = inner
            .split_once('(')
            .map(|(name, args)| {
                (
                    name,
                    split_macro_args(args.strip_suffix(')').unwrap_or(args)),
                )
            })
            .unwrap_or((inner.as_str(), Vec::new()));
        ObjectData::Macro {
            name: name.to_string(),
            arguments: args,
        }
    }
}
