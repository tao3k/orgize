//! Document element indexing, filtering, and source selection helpers.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Org, SyntaxNode,
    syntax_ast::{
        ExportBlock, Headline, OrgTable, Paragraph, PropertyDrawer, SourceBlock, SyntaxLink,
        SyntaxList, SyntaxListItem, SyntaxPlanning,
    },
};
use rowan::ast::AstNode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentLanguage {
    Org,
    Markdown,
}

#[derive(Clone, Debug)]
pub struct DocumentElement {
    pub kind: &'static str,
    pub source_kind: &'static str,
    pub path: String,
    pub line: usize,
    pub end_line: usize,
    pub fields: Vec<(String, String)>,
    pub text: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentWalkConfig {
    pub ignore_dirs: Vec<String>,
    pub include_hidden_dirs: Vec<String>,
}

impl Default for DocumentWalkConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: default_ignore_dirs()
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            include_hidden_dirs: Vec::new(),
        }
    }
}

impl DocumentWalkConfig {
    pub fn new(ignore_dirs: Vec<String>, include_hidden_dirs: Vec<String>) -> Self {
        Self {
            ignore_dirs,
            include_hidden_dirs,
        }
    }
}

pub fn index_project(
    language: DocumentLanguage,
    root: &Path,
) -> Result<Vec<DocumentElement>, String> {
    index_project_with_config(language, root, &DocumentWalkConfig::default())
}

pub fn index_project_with_config(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
) -> Result<Vec<DocumentElement>, String> {
    let mut files = Vec::new();
    collect_document_paths(language, root, walk_config, &mut files)?;
    files.sort();
    files.dedup();

    let mut facts = Vec::new();
    for path in files {
        if !path.exists() {
            continue;
        }
        facts.extend(index_path(language, &path)?);
    }
    Ok(facts)
}

pub fn index_path(language: DocumentLanguage, path: &Path) -> Result<Vec<DocumentElement>, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    match language {
        DocumentLanguage::Org => Ok(index_org(path, &source)),
        DocumentLanguage::Markdown => index_markdown(path, &source),
    }
}

pub fn filter_elements(elements: &[DocumentElement], query: &str) -> Vec<DocumentElement> {
    elements
        .iter()
        .filter(|element| element.matches(query))
        .cloned()
        .collect()
}

pub fn filter_elements_by_query(
    elements: Vec<DocumentElement>,
    terms: &[String],
    kinds: &[String],
    fields: &[String],
) -> Vec<DocumentElement> {
    elements
        .into_iter()
        .filter(|element| {
            terms.iter().all(|term| element.matches(term))
                && kinds.iter().all(|kind| element.kind_matches(kind))
                && fields.iter().all(|field| element.field_matches(field))
        })
        .collect()
}

pub(super) fn count_kind(elements: &[DocumentElement], kind: &str) -> usize {
    elements
        .iter()
        .filter(|element| element.kind == kind)
        .count()
}

pub(super) fn last_existing_path(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .rev()
        .filter(|arg| !arg.starts_with('-'))
        .map(PathBuf::from)
        .find(|path| path.exists())
}

pub(super) fn option_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then_some(window[1].as_str()))
}

pub(super) fn option_values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter_map(|window| (window[0] == name).then_some(window[1].clone()))
        .collect()
}

pub(super) fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

pub(super) fn display_path(path: &Path) -> String {
    path.display().to_string()
}

pub(super) fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

fn index_org(path: &Path, source: &str) -> Vec<DocumentElement> {
    let org = Org::parse(source);
    let document = org.syntax_document();
    let mut facts = Vec::new();

    for node in document.syntax().descendants() {
        if let Some(headline) = Headline::cast(node.clone()) {
            let mut fields = vec![
                ("level".to_string(), headline.level().to_string()),
                ("title".to_string(), headline.title_raw().trim().to_string()),
            ];
            if let Some(todo) = headline.todo_keyword() {
                fields.push(("todo".to_string(), todo.0.text().to_string()));
            }
            if let Some(todo_type) = headline.todo_type() {
                fields.push(("todoType".to_string(), format!("{todo_type:?}")));
            }
            if let Some(priority) = headline.priority() {
                fields.push(("priority".to_string(), priority.0.text().to_string()));
            }
            for tag in headline.tags() {
                fields.push(("tag".to_string(), tag.0.text().to_string()));
            }
            facts.push(fact(
                "heading",
                "Headline",
                path,
                source,
                headline.syntax(),
                fields.clone(),
            ));
            if headline.todo_keyword().is_some() {
                facts.push(fact(
                    "task",
                    "Headline",
                    path,
                    source,
                    headline.syntax(),
                    fields,
                ));
            }
        } else if let Some(drawer) = PropertyDrawer::cast(node.clone()) {
            for (key, value) in drawer.iter() {
                let fields = vec![
                    ("key".to_string(), key.0.text().to_string()),
                    ("value".to_string(), value.0.text().to_string()),
                ];
                facts.push(fact(
                    "property",
                    "PropertyDrawer",
                    path,
                    source,
                    drawer.syntax(),
                    fields,
                ));
            }
        } else if let Some(planning) = SyntaxPlanning::cast(node.clone()) {
            facts.push(fact(
                "planning",
                "SyntaxPlanning",
                path,
                source,
                planning.syntax(),
                planning_fields(planning.syntax()),
            ));
        } else if let Some(table) = OrgTable::cast(node.clone()) {
            let fields = vec![("header".to_string(), table.has_header().to_string())];
            facts.push(fact(
                "table",
                "OrgTable",
                path,
                source,
                table.syntax(),
                fields,
            ));
        } else if let Some(paragraph) = Paragraph::cast(node.clone()) {
            let content = paragraph.raw().trim().to_string();
            facts.push(fact_with_text(
                "paragraph",
                "Paragraph",
                path,
                source,
                paragraph.syntax(),
                Vec::new(),
                ElementText {
                    text: normalize_inline_text(&content),
                    content,
                },
            ));
        } else if let Some(block) = SourceBlock::cast(node.clone()) {
            let mut fields = vec![("kind".to_string(), "source".to_string())];
            if let Some(language) = block.language() {
                fields.push(("lang".to_string(), language.to_string()));
            }
            facts.push(fact(
                "block",
                "SourceBlock",
                path,
                source,
                block.syntax(),
                fields,
            ));
        } else if let Some(block) = ExportBlock::cast(node.clone()) {
            let mut fields = vec![("kind".to_string(), "export".to_string())];
            if let Some(backend) = block.ty() {
                fields.push(("backend".to_string(), backend.to_string()));
            }
            facts.push(fact(
                "block",
                "ExportBlock",
                path,
                source,
                block.syntax(),
                fields,
            ));
        } else if let Some(list) = SyntaxList::cast(node.clone()) {
            let fields = vec![
                (
                    "listKind".to_string(),
                    if list.is_ordered() {
                        "ordered"
                    } else {
                        "unordered"
                    }
                    .to_string(),
                ),
                ("descriptive".to_string(), list.is_descriptive().to_string()),
            ];
            facts.push(fact(
                "list",
                "SyntaxList",
                path,
                source,
                list.syntax(),
                fields,
            ));
        } else if let Some(item) = SyntaxListItem::cast(node.clone()) {
            let checkbox = item
                .checkbox()
                .map(|checkbox| checkbox.0.text().to_string());
            let mut fields = vec![
                ("bullet".to_string(), item.bullet().0.text().to_string()),
                ("indent".to_string(), item.indent().to_string()),
            ];
            if let Some(counter) = item.counter() {
                fields.push(("counter".to_string(), counter.0.text().to_string()));
            }
            if let Some(checkbox) = checkbox.as_deref() {
                fields.push(("checkbox".to_string(), checkbox.to_string()));
                fields.push(("checked".to_string(), (checkbox == "X").to_string()));
            }
            let tag = item
                .tag()
                .map(|element| element.to_string())
                .collect::<String>();
            if !tag.trim().is_empty() {
                fields.push(("tag".to_string(), tag.trim().to_string()));
            }
            facts.push(fact(
                if checkbox.is_some() {
                    "checklistItem"
                } else {
                    "listItem"
                },
                "SyntaxListItem",
                path,
                source,
                item.syntax(),
                fields,
            ));
        } else if let Some(link) = SyntaxLink::cast(node.clone()) {
            let mut fields = vec![("target".to_string(), link.path().0.text().to_string())];
            let description = link.description_raw();
            if !description.is_empty() {
                fields.push(("description".to_string(), description));
            }
            facts.push(fact(
                if link.is_image() { "image" } else { "link" },
                "SyntaxLink",
                path,
                source,
                link.syntax(),
                fields,
            ));
        }
    }

    facts
}

fn planning_fields(node: &SyntaxNode) -> Vec<(String, String)> {
    let raw = node.to_string();
    let mut fields = Vec::new();
    for marker in ["SCHEDULED", "DEADLINE", "CLOSED"] {
        if raw.contains(marker) {
            fields.push((marker.to_ascii_lowercase(), "true".to_string()));
        }
    }
    fields
}

#[cfg(feature = "md")]
fn index_markdown(path: &Path, source: &str) -> Result<Vec<DocumentElement>, String> {
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
fn index_markdown(_path: &Path, _source: &str) -> Result<Vec<DocumentElement>, String> {
    Err("orgize md requires the `md` feature".to_string())
}

fn fact(
    kind: &'static str,
    source_kind: &'static str,
    path: &Path,
    source: &str,
    node: &SyntaxNode,
    fields: Vec<(String, String)>,
) -> DocumentElement {
    let node_text = node.to_string();
    let text = node_text
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    let content = if uses_source_backed_content(kind, source_kind) {
        source_node_content(source, node).unwrap_or_else(|| text.clone())
    } else {
        text.clone()
    };
    fact_with_text(
        kind,
        source_kind,
        path,
        source,
        node,
        fields,
        ElementText { text, content },
    )
}

fn uses_source_backed_content(kind: &str, source_kind: &str) -> bool {
    matches!(
        (kind, source_kind),
        ("block", "SourceBlock")
            | ("block", "ExportBlock")
            | ("list", "SyntaxList")
            | ("listItem", "SyntaxListItem")
            | ("checklistItem", "SyntaxListItem")
    )
}

fn source_node_content(source: &str, node: &SyntaxNode) -> Option<String> {
    let range = node.text_range();
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    source
        .get(start..end)
        .map(str::trim)
        .filter(|content| !content.is_empty())
        .map(str::to_string)
}

struct ElementText {
    text: String,
    content: String,
}

fn fact_with_text(
    kind: &'static str,
    source_kind: &'static str,
    path: &Path,
    source: &str,
    node: &SyntaxNode,
    fields: Vec<(String, String)>,
    element_text: ElementText,
) -> DocumentElement {
    let range = node.text_range();
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    let line = position_to_line(source, start);
    let end_line = position_to_line(source, end.saturating_sub(1));
    DocumentElement {
        kind,
        source_kind,
        path: display_path(path),
        line,
        end_line,
        text: element_text.text,
        content: element_text.content,
        fields,
    }
}

fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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
        path: display_path(path),
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

fn collect_document_paths(
    language: DocumentLanguage,
    path: &Path,
    walk_config: &DocumentWalkConfig,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.is_file() {
        if language.matches_path(path) {
            files.push(path.to_path_buf());
            return Ok(());
        }
        return Err(format!(
            "{}: expected {} file",
            path.display(),
            language.id()
        ));
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", path.display()));
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        let Some(name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let entry_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry_path.display()))?;
        if entry_type.is_dir() {
            if should_skip_project_directory(name, walk_config) {
                continue;
            }
            collect_document_paths(language, &entry_path, walk_config, files)?;
        } else if entry_type.is_file() && language.matches_path(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn should_skip_project_directory(name: &str, walk_config: &DocumentWalkConfig) -> bool {
    if walk_config
        .include_hidden_dirs
        .iter()
        .any(|included| included == name)
    {
        return false;
    }
    if walk_config
        .ignore_dirs
        .iter()
        .any(|ignored| ignored == name)
    {
        return true;
    }
    name.starts_with('.')
}

fn default_ignore_dirs() -> &'static [&'static str] {
    &[
        "target",
        "node_modules",
        "dist",
        "build",
        "__pycache__",
        "venv",
        "vendor",
    ]
}

fn position_to_line(source: &str, byte_index: usize) -> usize {
    source
        .as_bytes()
        .iter()
        .take(byte_index.min(source.len()))
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

impl DocumentElement {
    pub(super) fn render(&self) -> String {
        let mut output = format!(
            "|{} {}:{}-{}",
            self.kind, self.path, self.line, self.end_line
        );
        output.push_str(" sourceKind=\"");
        output.push_str(self.source_kind);
        output.push('"');
        for (key, value) in &self.fields {
            output.push(' ');
            output.push_str(key);
            output.push_str("=\"");
            output.push_str(&escape_field(value));
            output.push('"');
        }
        if !self.text.is_empty() {
            output.push_str(" text=\"");
            output.push_str(&escape_field(&self.text));
            output.push('"');
        }
        output
    }

    pub(super) fn matches(&self, query: &str) -> bool {
        let query = query.to_ascii_lowercase();
        if query.is_empty() {
            return true;
        }
        let haystack = format!(
            "{} {} {} {:?} {} {}",
            self.kind, self.source_kind, self.path, self.fields, self.text, self.content
        )
        .to_ascii_lowercase();
        query.split_whitespace().all(|term| haystack.contains(term))
    }

    fn kind_matches(&self, kind: &str) -> bool {
        self.kind.eq_ignore_ascii_case(kind.trim())
    }

    fn field_matches(&self, field: &str) -> bool {
        let field = field.trim();
        if field.is_empty() {
            return true;
        }
        let Some((key, value)) = field.split_once('=') else {
            return self
                .fields
                .iter()
                .any(|(existing_key, _)| existing_key.eq_ignore_ascii_case(field));
        };
        let key = key.trim();
        let value = value.trim();
        if key.eq_ignore_ascii_case("text") {
            return self.text.contains(value);
        }
        self.fields.iter().any(|(existing_key, existing_value)| {
            existing_key.eq_ignore_ascii_case(key) && existing_value.contains(value)
        })
    }

    pub(super) fn content_text(&self) -> String {
        if !self.content.trim().is_empty() {
            return self.content.clone();
        }
        if !self.text.trim().is_empty() {
            return self.text.clone();
        }
        self.fields
            .iter()
            .find(|(key, value)| {
                matches!(
                    key.as_str(),
                    "title" | "value" | "description" | "target" | "lang"
                ) && !value.trim().is_empty()
            })
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    }
}

impl DocumentLanguage {
    pub fn id(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::Markdown => "md",
        }
    }

    pub fn command_prefix(self) -> &'static str {
        match self {
            Self::Org => "asp org",
            Self::Markdown => "asp md",
        }
    }

    pub fn parser_authority(self) -> &'static str {
        match self {
            Self::Org => "orgize",
            Self::Markdown => "comrak",
        }
    }

    fn matches_path(self, path: &Path) -> bool {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return false;
        };
        match self {
            Self::Org => matches!(extension, "org" | "org_archive"),
            Self::Markdown => matches!(extension, "md" | "markdown"),
        }
    }
}
