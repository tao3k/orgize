//! Org syntax AST to document element mapping.

use std::path::Path;

use rowan::ast::AstNode;

use crate::{
    Org, SyntaxNode,
    syntax_ast::{
        ExportBlock, Headline, OrgTable, Paragraph, PropertyDrawer, SourceBlock, SyntaxLink,
        SyntaxList, SyntaxListItem, SyntaxPlanning,
    },
};

use super::{line_index::LineIndex, model::DocumentElement};

pub(super) fn index_org(path: &Path, source: &str) -> Vec<DocumentElement> {
    let org = Org::parse(source);
    let document = org.syntax_document();
    let line_index = LineIndex::new(source);
    let mut facts = Vec::new();

    for node in document.syntax().descendants() {
        collect_org_node(path, source, &line_index, &node, &mut facts);
    }

    facts
}

fn collect_org_node(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    node: &SyntaxNode,
    facts: &mut Vec<DocumentElement>,
) {
    if let Some(headline) = Headline::cast(node.clone()) {
        collect_headline(path, source, line_index, headline, facts);
    } else if let Some(drawer) = PropertyDrawer::cast(node.clone()) {
        collect_property_drawer(path, source, line_index, drawer, facts);
    } else if let Some(planning) = SyntaxPlanning::cast(node.clone()) {
        facts.push(fact(
            "planning",
            "SyntaxPlanning",
            path,
            source,
            line_index,
            planning.syntax(),
            planning_fields(planning.syntax()),
        ));
    } else if let Some(table) = OrgTable::cast(node.clone()) {
        collect_table(path, source, line_index, table, facts);
    } else if let Some(paragraph) = Paragraph::cast(node.clone()) {
        collect_paragraph(path, line_index, paragraph, facts);
    } else if let Some(block) = SourceBlock::cast(node.clone()) {
        collect_source_block(path, source, line_index, block, facts);
    } else if let Some(block) = ExportBlock::cast(node.clone()) {
        collect_export_block(path, source, line_index, block, facts);
    } else if let Some(list) = SyntaxList::cast(node.clone()) {
        collect_list(path, source, line_index, list, facts);
    } else if let Some(item) = SyntaxListItem::cast(node.clone()) {
        collect_list_item(path, source, line_index, item, facts);
    } else if let Some(link) = SyntaxLink::cast(node.clone()) {
        collect_link(path, source, line_index, link, facts);
    }
}

fn collect_headline(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    headline: Headline,
    facts: &mut Vec<DocumentElement>,
) {
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
        line_index,
        headline.syntax(),
        fields.clone(),
    ));
    if headline.todo_keyword().is_some() {
        facts.push(fact(
            "task",
            "Headline",
            path,
            source,
            line_index,
            headline.syntax(),
            fields,
        ));
    }
}

fn collect_property_drawer(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    drawer: PropertyDrawer,
    facts: &mut Vec<DocumentElement>,
) {
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
            line_index,
            drawer.syntax(),
            fields,
        ));
    }
}

fn collect_table(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    table: OrgTable,
    facts: &mut Vec<DocumentElement>,
) {
    let fields = vec![("header".to_string(), table.has_header().to_string())];
    facts.push(fact(
        "table",
        "OrgTable",
        path,
        source,
        line_index,
        table.syntax(),
        fields,
    ));
}

fn collect_paragraph(
    path: &Path,
    line_index: &LineIndex,
    paragraph: Paragraph,
    facts: &mut Vec<DocumentElement>,
) {
    let content = paragraph.raw().trim().to_string();
    facts.push(fact_with_text(
        "paragraph",
        "Paragraph",
        path,
        line_index,
        paragraph.syntax(),
        Vec::new(),
        ElementText {
            text: normalize_inline_text(&content),
            content,
        },
    ));
}

fn collect_source_block(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    block: SourceBlock,
    facts: &mut Vec<DocumentElement>,
) {
    let mut fields = vec![("kind".to_string(), "source".to_string())];
    if let Some(language) = block.language() {
        fields.push(("lang".to_string(), language.to_string()));
    }
    facts.push(fact(
        "block",
        "SourceBlock",
        path,
        source,
        line_index,
        block.syntax(),
        fields,
    ));
}

fn collect_export_block(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    block: ExportBlock,
    facts: &mut Vec<DocumentElement>,
) {
    let mut fields = vec![("kind".to_string(), "export".to_string())];
    if let Some(backend) = block.ty() {
        fields.push(("backend".to_string(), backend.to_string()));
    }
    facts.push(fact(
        "block",
        "ExportBlock",
        path,
        source,
        line_index,
        block.syntax(),
        fields,
    ));
}

fn collect_list(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    list: SyntaxList,
    facts: &mut Vec<DocumentElement>,
) {
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
        line_index,
        list.syntax(),
        fields,
    ));
}

fn collect_list_item(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    item: SyntaxListItem,
    facts: &mut Vec<DocumentElement>,
) {
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
        line_index,
        item.syntax(),
        fields,
    ));
}

fn collect_link(
    path: &Path,
    source: &str,
    line_index: &LineIndex,
    link: SyntaxLink,
    facts: &mut Vec<DocumentElement>,
) {
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
        line_index,
        link.syntax(),
        fields,
    ));
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

fn fact(
    kind: &'static str,
    source_kind: &'static str,
    path: &Path,
    source: &str,
    line_index: &LineIndex,
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
        line_index,
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
    line_index: &LineIndex,
    node: &SyntaxNode,
    fields: Vec<(String, String)>,
    element_text: ElementText,
) -> DocumentElement {
    let range = node.text_range();
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    let line = line_index.line_for(start);
    let end_line = line_index.line_for(end.saturating_sub(1));
    DocumentElement {
        kind,
        source_kind,
        path: path.display().to_string(),
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
