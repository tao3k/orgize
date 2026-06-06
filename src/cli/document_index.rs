use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Org, SyntaxNode,
    syntax_ast::{ExportBlock, Headline, OrgTable, PropertyDrawer, SourceBlock},
};
use rowan::ast::AstNode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentLanguage {
    Org,
    Markdown,
}

#[derive(Clone, Debug)]
pub(super) struct DocumentFact {
    pub(super) kind: &'static str,
    pub(super) path: String,
    pub(super) line: usize,
    pub(super) end_line: usize,
    pub(super) fields: Vec<(String, String)>,
    pub(super) text: String,
}

pub(super) struct SourceSelector {
    pub(super) path: PathBuf,
    pub(super) range: Option<(usize, usize)>,
}

pub(super) fn index_project(
    language: DocumentLanguage,
    root: &Path,
) -> Result<Vec<DocumentFact>, String> {
    let mut files = Vec::new();
    collect_document_paths(language, root, &mut files)?;
    files.sort();
    files.dedup();

    let mut facts = Vec::new();
    for path in files {
        facts.extend(index_path(language, &path)?);
    }
    Ok(facts)
}

pub(super) fn index_path(
    language: DocumentLanguage,
    path: &Path,
) -> Result<Vec<DocumentFact>, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    match language {
        DocumentLanguage::Org => Ok(index_org(path, &source)),
        DocumentLanguage::Markdown => index_markdown(path, &source),
    }
}

pub(super) fn filter_facts(facts: &[DocumentFact], query: &str) -> Vec<DocumentFact> {
    facts
        .iter()
        .filter(|fact| fact.matches(query))
        .cloned()
        .collect()
}

pub(super) fn count_kind(facts: &[DocumentFact], kind: &str) -> usize {
    facts.iter().filter(|fact| fact.kind == kind).count()
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

pub(super) fn select_source(source: &str, range: Option<(usize, usize)>) -> String {
    let Some((start, end)) = range else {
        return source.to_string();
    };
    let mut output = String::new();
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let line_no = index + 1;
        if line_no >= start && line_no <= end {
            output.push_str(line);
        }
    }
    output
}

pub(super) fn display_path(path: &Path) -> String {
    path.display().to_string()
}

pub(super) fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', " ")
}

fn index_org(path: &Path, source: &str) -> Vec<DocumentFact> {
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
            for tag in headline.tags() {
                fields.push(("tag".to_string(), tag.0.text().to_string()));
            }
            facts.push(fact("heading", path, source, headline.syntax(), fields));
        } else if let Some(drawer) = PropertyDrawer::cast(node.clone()) {
            for (key, value) in drawer.iter() {
                let fields = vec![
                    ("key".to_string(), key.0.text().to_string()),
                    ("value".to_string(), value.0.text().to_string()),
                ];
                facts.push(fact("property", path, source, drawer.syntax(), fields));
            }
        } else if let Some(table) = OrgTable::cast(node.clone()) {
            let fields = vec![("header".to_string(), table.has_header().to_string())];
            facts.push(fact("table", path, source, table.syntax(), fields));
        } else if let Some(block) = SourceBlock::cast(node.clone()) {
            let mut fields = vec![("kind".to_string(), "source".to_string())];
            if let Some(language) = block.language() {
                fields.push(("lang".to_string(), language.to_string()));
            }
            facts.push(fact("block", path, source, block.syntax(), fields));
        } else if let Some(block) = ExportBlock::cast(node.clone()) {
            let mut fields = vec![("kind".to_string(), "export".to_string())];
            if let Some(backend) = block.ty() {
                fields.push(("backend".to_string(), backend.to_string()));
            }
            facts.push(fact("block", path, source, block.syntax(), fields));
        }
    }

    facts
}

#[cfg(feature = "md")]
fn index_markdown(path: &Path, source: &str) -> Result<Vec<DocumentFact>, String> {
    use comrak::{Arena, Options, nodes::NodeValue};

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
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
                    path,
                    data.sourcepos.start.line,
                    data.sourcepos.end.line,
                    vec![
                        ("level".to_string(), heading.level.to_string()),
                        ("title".to_string(), title),
                    ],
                ));
            }
            NodeValue::Link(link) => facts.push(markdown_fact(
                "link",
                path,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                vec![("target".to_string(), link.url.clone())],
            )),
            NodeValue::CodeBlock(block) => facts.push(markdown_fact(
                "block",
                path,
                data.sourcepos.start.line,
                data.sourcepos.end.line,
                vec![
                    ("kind".to_string(), "code".to_string()),
                    ("lang".to_string(), block.info.clone()),
                ],
            )),
            NodeValue::Table(_) => facts.push(markdown_fact(
                "table",
                path,
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
fn index_markdown(_path: &Path, _source: &str) -> Result<Vec<DocumentFact>, String> {
    Err("orgize md requires the `md` feature".to_string())
}

fn fact(
    kind: &'static str,
    path: &Path,
    source: &str,
    node: &SyntaxNode,
    fields: Vec<(String, String)>,
) -> DocumentFact {
    let range = node.text_range();
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    let line = offset_to_line(source, start);
    let end_line = offset_to_line(source, end.saturating_sub(1));
    DocumentFact {
        kind,
        path: display_path(path),
        line,
        end_line,
        text: node
            .to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string(),
        fields,
    }
}

#[cfg(feature = "md")]
fn markdown_fact(
    kind: &'static str,
    path: &Path,
    line: usize,
    end_line: usize,
    fields: Vec<(String, String)>,
) -> DocumentFact {
    DocumentFact {
        kind,
        path: display_path(path),
        line: line.max(1),
        end_line: end_line.max(line).max(1),
        text: String::new(),
        fields,
    }
}

fn collect_document_paths(
    language: DocumentLanguage,
    path: &Path,
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
        if matches!(name, ".git" | "target" | "node_modules" | ".venv") {
            continue;
        }
        let entry_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry_path.display()))?;
        if entry_type.is_dir() {
            collect_document_paths(language, &entry_path, files)?;
        } else if entry_type.is_file() && language.matches_path(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn offset_to_line(source: &str, offset: usize) -> usize {
    source
        .as_bytes()
        .iter()
        .take(offset.min(source.len()))
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

impl DocumentFact {
    pub(super) fn render(&self) -> String {
        let mut output = format!(
            "|{} {}:{}-{}",
            self.kind, self.path, self.line, self.end_line
        );
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
            "{} {} {:?} {}",
            self.kind, self.path, self.fields, self.text
        )
        .to_ascii_lowercase();
        query.split_whitespace().all(|term| haystack.contains(term))
    }
}

impl DocumentLanguage {
    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::Markdown => "md",
        }
    }

    pub(super) fn command_prefix(self) -> &'static str {
        match self {
            Self::Org => "orgize",
            Self::Markdown => "orgize md",
        }
    }

    pub(super) fn parser_authority(self) -> &'static str {
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

impl SourceSelector {
    pub(super) fn parse(selector: &str) -> Result<Self, String> {
        let Some((path, range)) = selector.rsplit_once(':') else {
            return Ok(Self {
                path: PathBuf::from(selector),
                range: None,
            });
        };
        if path.is_empty() {
            return Err(format!("invalid selector `{selector}`"));
        }
        let range = parse_line_range(range)?;
        Ok(Self {
            path: PathBuf::from(path),
            range: Some(range),
        })
    }
}

fn parse_line_range(value: &str) -> Result<(usize, usize), String> {
    let (start, end) = value.split_once('-').unwrap_or((value, value));
    let start = start
        .parse::<usize>()
        .map_err(|_| format!("invalid selector line `{value}`"))?;
    let end = end
        .parse::<usize>()
        .map_err(|_| format!("invalid selector line `{value}`"))?;
    Ok((start, end.max(start)))
}
