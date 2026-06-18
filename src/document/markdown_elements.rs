use std::path::Path;

use super::model::DocumentElement;

#[cfg(feature = "md")]
pub(super) fn index_markdown(path: &Path, source: &str) -> Result<Vec<DocumentElement>, String> {
    use comrak::{Arena, Options, nodes::NodeValue};

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    let root = comrak::parse_document(&arena, source, &options);
    let mut facts = Vec::new();

    for node in root.descendants() {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Heading(heading) => {
                let title = node
                    .children()
                    .filter_map(|child| match &child.data.borrow().value {
                        NodeValue::Text(text) => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                facts.push(markdown_fact(
                    "heading",
                    "NodeValue::Heading",
                    path,
                    source,
                    data.sourcepos.start.line,
                    data.sourcepos.end.line,
                    vec![
                        ("level".to_string(), heading.level.to_string()),
                        ("title".to_string(), title),
                    ],
                ));
            }
            NodeValue::Paragraph => {
                let text = markdown_inline_text(node);
                facts.push(markdown_fact_with_text(
                    "paragraph",
                    "NodeValue::Paragraph",
                    path,
                    source,
                    MarkdownFactPayload::new(
                        data.sourcepos.start.line,
                        data.sourcepos.end.line,
                        Vec::new(),
                        text,
                    ),
                ));
            }
            NodeValue::Link(link) => facts.push(markdown_fact(
                "link",
                "NodeValue::Link",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                vec![("target".to_string(), link.url.clone())],
            )),
            NodeValue::Image(link) => facts.push(markdown_fact(
                "image",
                "NodeValue::Image",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                vec![("target".to_string(), link.url.clone())],
            )),
            NodeValue::CodeBlock(block) => facts.push(markdown_fact(
                "block",
                "NodeValue::CodeBlock",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                vec![
                    ("kind".to_string(), "code".to_string()),
                    ("lang".to_string(), block.info.clone()),
                ],
            )),
            NodeValue::Table(_) => facts.push(markdown_fact(
                "table",
                "NodeValue::Table",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                Vec::new(),
            )),
            NodeValue::List(list) => facts.push(markdown_fact(
                "list",
                "NodeValue::List",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                vec![
                    ("listKind".to_string(), format!("{:?}", list.list_type)),
                    ("start".to_string(), list.start.to_string()),
                ],
            )),
            NodeValue::Item(_) => facts.push(markdown_fact(
                "listItem",
                "NodeValue::Item",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                Vec::new(),
            )),
            NodeValue::TaskItem(task) => {
                let mut fields = vec![("checked".to_string(), task.symbol.is_some().to_string())];
                if let Some(symbol) = task.symbol {
                    fields.push(("checkbox".to_string(), symbol.to_string()));
                }
                facts.push(markdown_fact(
                    "checklistItem",
                    "NodeValue::TaskItem",
                    path,
                    source,
                    data.sourcepos.start.line,
                    data.sourcepos.end.line,
                    fields,
                ));
            }
            NodeValue::FrontMatter(_) => facts.push(markdown_fact(
                "frontMatter",
                "NodeValue::FrontMatter",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                Vec::new(),
            )),
            NodeValue::ThematicBreak => facts.push(markdown_fact(
                "thematicBreak",
                "NodeValue::ThematicBreak",
                path,
                source,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                Vec::new(),
            )),
            _ => {}
        }
    }

    Ok(facts)
}

#[cfg(not(feature = "md"))]
pub(super) fn index_markdown(_path: &Path, _source: &str) -> Result<Vec<DocumentElement>, String> {
    Err("orgize md requires the `md` feature".to_string())
}

#[cfg(feature = "md")]
fn markdown_fact(
    kind: &'static str,
    source_kind: &'static str,
    path: &Path,
    source: &str,
    line: usize,
    end_line: usize,
    fields: Vec<(String, String)>,
) -> DocumentElement {
    markdown_fact_with_text(
        kind,
        source_kind,
        path,
        source,
        MarkdownFactPayload::new(line, end_line, fields, String::new()),
    )
}

#[cfg(feature = "md")]
struct MarkdownFactPayload {
    line: usize,
    end_line: usize,
    fields: Vec<(String, String)>,
    text: String,
}

#[cfg(feature = "md")]
impl MarkdownFactPayload {
    fn new(line: usize, end_line: usize, fields: Vec<(String, String)>, text: String) -> Self {
        Self {
            line,
            end_line,
            fields,
            text,
        }
    }
}

#[cfg(feature = "md")]
fn markdown_fact_with_text(
    kind: &'static str,
    source_kind: &'static str,
    path: &Path,
    source: &str,
    payload: MarkdownFactPayload,
) -> DocumentElement {
    DocumentElement {
        kind,
        source_kind,
        path: path.display().to_string(),
        line: payload.line.max(1),
        end_line: payload.end_line.max(payload.line).max(1),
        content: markdown_source_content(source, payload.line, payload.end_line)
            .unwrap_or_else(|| payload.text.clone()),
        text: payload.text,
        fields: payload.fields,
    }
}

#[cfg(feature = "md")]
fn markdown_source_content(source: &str, line: usize, end_line: usize) -> Option<String> {
    let start_line = line.max(1);
    let end_line = end_line.max(start_line);
    let mut output = String::new();
    for (index, source_line) in source.split_inclusive('\n').enumerate() {
        let line_no = index + 1;
        if line_no >= start_line && line_no <= end_line {
            output.push_str(source_line);
        }
    }
    (!output.is_empty()).then_some(output)
}

#[cfg(feature = "md")]
fn markdown_inline_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    use comrak::nodes::NodeValue;

    fn collect<'a>(node: &'a comrak::nodes::AstNode<'a>, output: &mut Vec<String>) {
        match &node.data.borrow().value {
            NodeValue::Text(text) => output.push(text.to_string()),
            NodeValue::Code(code) => output.push(code.literal.clone()),
            _ => {
                for child in node.children() {
                    collect(child, output);
                }
            }
        }
    }

    let mut parts = Vec::new();
    for child in node.children() {
        collect(child, &mut parts);
    }
    parts.join(" ").trim().to_string()
}
