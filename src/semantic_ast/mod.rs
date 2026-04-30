//! Owned semantic AST for Org documents.
//!
//! The parser still builds the lossless rowan syntax tree. This module is the
//! semantic, org-element-like layer projected from that syntax tree.

use rowan::{NodeOrToken, TextRange, TextSize};

use crate::{
    syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
    syntax_ast,
};
use rowan::ast::AstNode;

pub type ParsedAst = Document<ParsedAnnotation>;
pub type BareAst = Document<()>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedAnnotation {
    pub range: TextRange,
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub raw: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: TextRange,
    pub kind: DiagnosticKind,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    UnsupportedElement,
    UnsupportedObject,
    Conversion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document<A = ()> {
    pub ann: A,
    pub properties: Vec<Property<A>>,
    pub children: Vec<Element<A>>,
    pub sections: Vec<Section<A>>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section<A = ()> {
    pub ann: A,
    pub level: usize,
    pub properties: Vec<Property<A>>,
    pub todo: Option<TodoKeyword>,
    pub is_comment: bool,
    pub priority: Option<String>,
    pub title: Vec<Object<A>>,
    pub raw_title: String,
    pub anchor: Option<String>,
    pub tags: Vec<String>,
    pub planning: Planning,
    pub children: Vec<Element<A>>,
    pub subsections: Vec<Section<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TodoKeyword {
    pub state: TodoState,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodoState {
    Todo,
    Done,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Planning {
    pub deadline: Option<Timestamp>,
    pub scheduled: Option<Timestamp>,
    pub closed: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property<A = ()> {
    pub ann: A,
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyword<A = ()> {
    pub ann: A,
    pub key: String,
    pub optional: Option<String>,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element<A = ()> {
    pub ann: A,
    pub affiliated_keywords: Vec<Keyword<A>>,
    pub data: ElementData<A>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementData<A = ()> {
    Paragraph(Vec<Object<A>>),
    Keyword(Keyword<A>),
    BabelCall(Keyword<A>),
    Clock(Clock),
    Drawer(Drawer<A>),
    PropertyDrawer(Vec<Property<A>>),
    List(List<A>),
    Table(Table<A>),
    Block(Block<A>),
    FootnoteDef(FootnoteDef<A>),
    Comment(String),
    FixedWidth(String),
    Rule,
    LatexEnvironment(String),
    Unknown { kind: String, raw: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    pub value: Option<Timestamp>,
    pub duration: Option<String>,
    pub raw: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawer<A = ()> {
    pub name: String,
    pub children: Vec<Element<A>>,
    pub raw: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List<A = ()> {
    pub list_type: ListType,
    pub items: Vec<ListItem<A>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListType {
    Ordered,
    Unordered,
    Descriptive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem<A = ()> {
    pub ann: A,
    pub bullet: String,
    pub counter: Option<String>,
    pub checkbox: Option<Checkbox>,
    pub tag: Vec<Object<A>>,
    pub children: Vec<Element<A>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checkbox {
    On,
    Off,
    Trans,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table<A = ()> {
    pub rows: Vec<TableRow<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow<A = ()> {
    pub ann: A,
    pub is_rule: bool,
    pub cells: Vec<TableCell<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableCell<A = ()> {
    pub ann: A,
    pub objects: Vec<Object<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block<A = ()> {
    pub kind: BlockKind,
    pub name: Option<String>,
    pub language: Option<String>,
    pub switches: Option<String>,
    pub parameters: Option<String>,
    pub value: String,
    pub children: Vec<Element<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Source,
    Example,
    Export,
    Quote,
    Verse,
    Center,
    Comment,
    Dynamic,
    Special(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteDef<A = ()> {
    pub label: String,
    pub children: Vec<Element<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object<A = ()> {
    pub ann: A,
    pub data: ObjectData<A>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectData<A = ()> {
    Plain(String),
    LineBreak,
    Markup {
        kind: MarkupKind,
        children: Vec<Object<A>>,
    },
    Code(String),
    Verbatim(String),
    Timestamp(Timestamp),
    Entity(String),
    LatexFragment(String),
    ExportSnippet {
        backend: String,
        value: String,
    },
    FootnoteRef {
        label: Option<String>,
        definition: Vec<Object<A>>,
    },
    Citation(Citation<A>),
    InlineCall {
        name: String,
        arguments: String,
        header: Option<String>,
        end_header: Option<String>,
        raw: String,
    },
    InlineSrc {
        language: String,
        parameters: Option<String>,
        value: String,
        raw: String,
    },
    Link {
        target: LinkTarget,
        description: Vec<Object<A>>,
        raw_description: String,
    },
    Target(String),
    RadioTarget(String),
    Macro {
        name: String,
        arguments: Vec<String>,
    },
    StatisticCookie(String),
    Unknown {
        kind: String,
        raw: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupKind {
    Bold,
    Italic,
    Underline,
    Strike,
    Superscript,
    Subscript,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub kind: TimestampKind,
    pub raw: String,
    pub is_range: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampKind {
    Active,
    Inactive,
    Diary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    Uri { protocol: String, path: String },
    Internal(String),
    Unresolved(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation<A = ()> {
    pub style: String,
    pub variant: String,
    pub prefix: Vec<Object<A>>,
    pub suffix: Vec<Object<A>>,
    pub references: Vec<CiteReference<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CiteReference<A = ()> {
    pub id: String,
    pub prefix: Vec<Object<A>>,
    pub suffix: Vec<Object<A>>,
}

pub enum AstRef<'a, A> {
    Document(&'a Document<A>),
    Section(&'a Section<A>),
    Property(&'a Property<A>),
    Keyword(&'a Keyword<A>),
    Element(&'a Element<A>),
    ListItem(&'a ListItem<A>),
    TableRow(&'a TableRow<A>),
    TableCell(&'a TableCell<A>),
    Object(&'a Object<A>),
}

pub enum AstMut<'a, A> {
    Document(&'a mut Document<A>),
    Section(&'a mut Section<A>),
    Property(&'a mut Property<A>),
    Keyword(&'a mut Keyword<A>),
    Element(&'a mut Element<A>),
    ListItem(&'a mut ListItem<A>),
    TableRow(&'a mut TableRow<A>),
    TableCell(&'a mut TableCell<A>),
    Object(&'a mut Object<A>),
}

impl ParsedAst {
    pub fn from_syntax_tree(root: &SyntaxNode, source: &str) -> Self {
        Converter::new(source).document(root)
    }
}

impl<A> Document<A> {
    pub fn to_bare(&self) -> BareAst {
        self.map_ann(|_| ())
    }

    pub fn map_ann<B, F>(&self, mut f: F) -> Document<B>
    where
        F: FnMut(&A) -> B,
    {
        self.map_ann_with(&mut f)
    }

    pub fn try_map_ann<B, E, F>(&self, mut f: F) -> Result<Document<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        self.try_map_ann_with(&mut f)
    }

    pub fn visit<F>(&self, mut f: F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        self.visit_with(&mut f);
    }

    pub fn visit_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        self.visit_mut_with(&mut f);
    }

    pub fn fold<T, F>(&self, init: T, mut f: F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        self.fold_with(init, &mut f)
    }

    fn map_ann_with<B, F>(&self, f: &mut F) -> Document<B>
    where
        F: FnMut(&A) -> B,
    {
        Document {
            ann: f(&self.ann),
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            sections: self.sections.iter().map(|x| x.map_ann_with(f)).collect(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Document<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Document {
            ann: f(&self.ann)?,
            properties: self
                .properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            sections: self
                .sections
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            diagnostics: self.diagnostics.clone(),
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Document(self));
        for property in &self.properties {
            property.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
        for section in &self.sections {
            section.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Document(self));
        for property in &mut self.properties {
            property.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
        for section in &mut self.sections {
            section.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Document(self));
        for property in &self.properties {
            acc = property.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        for section in &self.sections {
            acc = section.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Section<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Section<B>
    where
        F: FnMut(&A) -> B,
    {
        Section {
            ann: f(&self.ann),
            level: self.level,
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            todo: self.todo.clone(),
            is_comment: self.is_comment,
            priority: self.priority.clone(),
            title: self.title.iter().map(|x| x.map_ann_with(f)).collect(),
            raw_title: self.raw_title.clone(),
            anchor: self.anchor.clone(),
            tags: self.tags.clone(),
            planning: self.planning.clone(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            subsections: self.subsections.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Section<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Section {
            ann: f(&self.ann)?,
            level: self.level,
            properties: self
                .properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            todo: self.todo.clone(),
            is_comment: self.is_comment,
            priority: self.priority.clone(),
            title: self
                .title
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            raw_title: self.raw_title.clone(),
            anchor: self.anchor.clone(),
            tags: self.tags.clone(),
            planning: self.planning.clone(),
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            subsections: self
                .subsections
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Section(self));
        for property in &self.properties {
            property.visit_with(f);
        }
        for object in &self.title {
            object.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
        for section in &self.subsections {
            section.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Section(self));
        for property in &mut self.properties {
            property.visit_mut_with(f);
        }
        for object in &mut self.title {
            object.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
        for section in &mut self.subsections {
            section.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Section(self));
        for property in &self.properties {
            acc = property.fold_with(acc, f);
        }
        for object in &self.title {
            acc = object.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        for section in &self.subsections {
            acc = section.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Property<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Property<B>
    where
        F: FnMut(&A) -> B,
    {
        Property {
            ann: f(&self.ann),
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Property<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Property {
            ann: f(&self.ann)?,
            key: self.key.clone(),
            value: self.value.clone(),
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Property(self));
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Property(self));
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::Property(self))
    }
}

impl<A> Keyword<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Keyword<B>
    where
        F: FnMut(&A) -> B,
    {
        Keyword {
            ann: f(&self.ann),
            key: self.key.clone(),
            optional: self.optional.clone(),
            value: self.value.clone(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Keyword<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Keyword {
            ann: f(&self.ann)?,
            key: self.key.clone(),
            optional: self.optional.clone(),
            value: self.value.clone(),
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Keyword(self));
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Keyword(self));
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::Keyword(self))
    }
}

impl<A> Element<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Element<B>
    where
        F: FnMut(&A) -> B,
    {
        Element {
            ann: f(&self.ann),
            affiliated_keywords: self
                .affiliated_keywords
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            data: self.data.map_ann_with(f),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Element<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Element {
            ann: f(&self.ann)?,
            affiliated_keywords: self
                .affiliated_keywords
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            data: self.data.try_map_ann_with(f)?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Element(self));
        for keyword in &self.affiliated_keywords {
            keyword.visit_with(f);
        }
        self.data.visit_with(f);
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Element(self));
        for keyword in &mut self.affiliated_keywords {
            keyword.visit_mut_with(f);
        }
        self.data.visit_mut_with(f);
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Element(self));
        for keyword in &self.affiliated_keywords {
            acc = keyword.fold_with(acc, f);
        }
        self.data.fold_with(acc, f)
    }
}

impl<A> ElementData<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> ElementData<B>
    where
        F: FnMut(&A) -> B,
    {
        match self {
            ElementData::Paragraph(objects) => {
                ElementData::Paragraph(objects.iter().map(|x| x.map_ann_with(f)).collect())
            }
            ElementData::Keyword(keyword) => ElementData::Keyword(keyword.map_ann_with(f)),
            ElementData::BabelCall(keyword) => ElementData::BabelCall(keyword.map_ann_with(f)),
            ElementData::Clock(clock) => ElementData::Clock(clock.clone()),
            ElementData::Drawer(drawer) => ElementData::Drawer(Drawer {
                name: drawer.name.clone(),
                children: drawer.children.iter().map(|x| x.map_ann_with(f)).collect(),
                raw: drawer.raw.clone(),
            }),
            ElementData::PropertyDrawer(properties) => {
                ElementData::PropertyDrawer(properties.iter().map(|x| x.map_ann_with(f)).collect())
            }
            ElementData::List(list) => ElementData::List(list.map_ann_with(f)),
            ElementData::Table(table) => ElementData::Table(table.map_ann_with(f)),
            ElementData::Block(block) => ElementData::Block(block.map_ann_with(f)),
            ElementData::FootnoteDef(def) => ElementData::FootnoteDef(FootnoteDef {
                label: def.label.clone(),
                children: def.children.iter().map(|x| x.map_ann_with(f)).collect(),
            }),
            ElementData::Comment(value) => ElementData::Comment(value.clone()),
            ElementData::FixedWidth(value) => ElementData::FixedWidth(value.clone()),
            ElementData::Rule => ElementData::Rule,
            ElementData::LatexEnvironment(value) => ElementData::LatexEnvironment(value.clone()),
            ElementData::Unknown { kind, raw } => ElementData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ElementData<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(match self {
            ElementData::Paragraph(objects) => ElementData::Paragraph(
                objects
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            ),
            ElementData::Keyword(keyword) => ElementData::Keyword(keyword.try_map_ann_with(f)?),
            ElementData::BabelCall(keyword) => ElementData::BabelCall(keyword.try_map_ann_with(f)?),
            ElementData::Clock(clock) => ElementData::Clock(clock.clone()),
            ElementData::Drawer(drawer) => ElementData::Drawer(Drawer {
                name: drawer.name.clone(),
                children: drawer
                    .children
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
                raw: drawer.raw.clone(),
            }),
            ElementData::PropertyDrawer(properties) => ElementData::PropertyDrawer(
                properties
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            ),
            ElementData::List(list) => ElementData::List(list.try_map_ann_with(f)?),
            ElementData::Table(table) => ElementData::Table(table.try_map_ann_with(f)?),
            ElementData::Block(block) => ElementData::Block(block.try_map_ann_with(f)?),
            ElementData::FootnoteDef(def) => ElementData::FootnoteDef(FootnoteDef {
                label: def.label.clone(),
                children: def
                    .children
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            }),
            ElementData::Comment(value) => ElementData::Comment(value.clone()),
            ElementData::FixedWidth(value) => ElementData::FixedWidth(value.clone()),
            ElementData::Rule => ElementData::Rule,
            ElementData::LatexEnvironment(value) => ElementData::LatexEnvironment(value.clone()),
            ElementData::Unknown { kind, raw } => ElementData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        match self {
            ElementData::Paragraph(objects) => {
                for object in objects {
                    object.visit_with(f);
                }
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                keyword.visit_with(f);
            }
            ElementData::Drawer(drawer) => {
                for child in &drawer.children {
                    child.visit_with(f);
                }
            }
            ElementData::PropertyDrawer(properties) => {
                for property in properties {
                    property.visit_with(f);
                }
            }
            ElementData::List(list) => {
                list.visit_with(f);
            }
            ElementData::Table(table) => {
                table.visit_with(f);
            }
            ElementData::Block(block) => {
                for child in &block.children {
                    child.visit_with(f);
                }
            }
            ElementData::FootnoteDef(def) => {
                for child in &def.children {
                    child.visit_with(f);
                }
            }
            _ => {}
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        match self {
            ElementData::Paragraph(objects) => {
                for object in objects {
                    object.visit_mut_with(f);
                }
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                keyword.visit_mut_with(f);
            }
            ElementData::Drawer(drawer) => {
                for child in &mut drawer.children {
                    child.visit_mut_with(f);
                }
            }
            ElementData::PropertyDrawer(properties) => {
                for property in properties {
                    property.visit_mut_with(f);
                }
            }
            ElementData::List(list) => {
                list.visit_mut_with(f);
            }
            ElementData::Table(table) => {
                table.visit_mut_with(f);
            }
            ElementData::Block(block) => {
                for child in &mut block.children {
                    child.visit_mut_with(f);
                }
            }
            ElementData::FootnoteDef(def) => {
                for child in &mut def.children {
                    child.visit_mut_with(f);
                }
            }
            _ => {}
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        match self {
            ElementData::Paragraph(objects) => {
                for object in objects {
                    acc = object.fold_with(acc, f);
                }
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                acc = keyword.fold_with(acc, f);
            }
            ElementData::Drawer(drawer) => {
                for child in &drawer.children {
                    acc = child.fold_with(acc, f);
                }
            }
            ElementData::PropertyDrawer(properties) => {
                for property in properties {
                    acc = property.fold_with(acc, f);
                }
            }
            ElementData::List(list) => {
                acc = list.fold_with(acc, f);
            }
            ElementData::Table(table) => {
                acc = table.fold_with(acc, f);
            }
            ElementData::Block(block) => {
                for child in &block.children {
                    acc = child.fold_with(acc, f);
                }
            }
            ElementData::FootnoteDef(def) => {
                for child in &def.children {
                    acc = child.fold_with(acc, f);
                }
            }
            _ => {}
        }
        acc
    }
}

impl<A> List<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> List<B>
    where
        F: FnMut(&A) -> B,
    {
        List {
            list_type: self.list_type,
            items: self
                .items
                .iter()
                .map(|item| ListItem {
                    ann: f(&item.ann),
                    bullet: item.bullet.clone(),
                    counter: item.counter.clone(),
                    checkbox: item.checkbox,
                    tag: item.tag.iter().map(|x| x.map_ann_with(f)).collect(),
                    children: item.children.iter().map(|x| x.map_ann_with(f)).collect(),
                })
                .collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<List<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(List {
            list_type: self.list_type,
            items: self
                .items
                .iter()
                .map(|item| {
                    Ok(ListItem {
                        ann: f(&item.ann)?,
                        bullet: item.bullet.clone(),
                        counter: item.counter.clone(),
                        checkbox: item.checkbox,
                        tag: item
                            .tag
                            .iter()
                            .map(|x| x.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                        children: item
                            .children
                            .iter()
                            .map(|x| x.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for item in &self.items {
            item.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for item in &mut self.items {
            item.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for item in &self.items {
            acc = item.fold_with(acc, f);
        }
        acc
    }
}

impl<A> ListItem<A> {
    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::ListItem(self));
        for object in &self.tag {
            object.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::ListItem(self));
        for object in &mut self.tag {
            object.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::ListItem(self));
        for object in &self.tag {
            acc = object.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Table<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Table<B>
    where
        F: FnMut(&A) -> B,
    {
        Table {
            rows: self
                .rows
                .iter()
                .map(|row| TableRow {
                    ann: f(&row.ann),
                    is_rule: row.is_rule,
                    cells: row
                        .cells
                        .iter()
                        .map(|cell| TableCell {
                            ann: f(&cell.ann),
                            objects: cell.objects.iter().map(|x| x.map_ann_with(f)).collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Table<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Table {
            rows: self
                .rows
                .iter()
                .map(|row| {
                    Ok(TableRow {
                        ann: f(&row.ann)?,
                        is_rule: row.is_rule,
                        cells: row
                            .cells
                            .iter()
                            .map(|cell| {
                                Ok(TableCell {
                                    ann: f(&cell.ann)?,
                                    objects: cell
                                        .objects
                                        .iter()
                                        .map(|x| x.try_map_ann_with(f))
                                        .collect::<Result<_, _>>()?,
                                })
                            })
                            .collect::<Result<_, E>>()?,
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for row in &self.rows {
            row.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for row in &mut self.rows {
            row.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for row in &self.rows {
            acc = row.fold_with(acc, f);
        }
        acc
    }
}

impl<A> TableRow<A> {
    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TableRow(self));
        for cell in &self.cells {
            cell.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TableRow(self));
        for cell in &mut self.cells {
            cell.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::TableRow(self));
        for cell in &self.cells {
            acc = cell.fold_with(acc, f);
        }
        acc
    }
}

impl<A> TableCell<A> {
    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TableCell(self));
        for object in &self.objects {
            object.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TableCell(self));
        for object in &mut self.objects {
            object.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::TableCell(self));
        for object in &self.objects {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Block<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Block<B>
    where
        F: FnMut(&A) -> B,
    {
        Block {
            kind: self.kind.clone(),
            name: self.name.clone(),
            language: self.language.clone(),
            switches: self.switches.clone(),
            parameters: self.parameters.clone(),
            value: self.value.clone(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Block<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Block {
            kind: self.kind.clone(),
            name: self.name.clone(),
            language: self.language.clone(),
            switches: self.switches.clone(),
            parameters: self.parameters.clone(),
            value: self.value.clone(),
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<A> Object<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Object<B>
    where
        F: FnMut(&A) -> B,
    {
        Object {
            ann: f(&self.ann),
            data: self.data.map_ann_with(f),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Object<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Object {
            ann: f(&self.ann)?,
            data: self.data.try_map_ann_with(f)?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Object(self));
        self.data.visit_with(f);
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Object(self));
        self.data.visit_mut_with(f);
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let acc = f(init, AstRef::Object(self));
        self.data.fold_with(acc, f)
    }
}

impl<A> ObjectData<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> ObjectData<B>
    where
        F: FnMut(&A) -> B,
    {
        match self {
            ObjectData::Plain(value) => ObjectData::Plain(value.clone()),
            ObjectData::LineBreak => ObjectData::LineBreak,
            ObjectData::Markup { kind, children } => ObjectData::Markup {
                kind: *kind,
                children: children.iter().map(|x| x.map_ann_with(f)).collect(),
            },
            ObjectData::Code(value) => ObjectData::Code(value.clone()),
            ObjectData::Verbatim(value) => ObjectData::Verbatim(value.clone()),
            ObjectData::Timestamp(value) => ObjectData::Timestamp(value.clone()),
            ObjectData::Entity(value) => ObjectData::Entity(value.clone()),
            ObjectData::LatexFragment(value) => ObjectData::LatexFragment(value.clone()),
            ObjectData::ExportSnippet { backend, value } => ObjectData::ExportSnippet {
                backend: backend.clone(),
                value: value.clone(),
            },
            ObjectData::FootnoteRef { label, definition } => ObjectData::FootnoteRef {
                label: label.clone(),
                definition: definition.iter().map(|x| x.map_ann_with(f)).collect(),
            },
            ObjectData::Citation(citation) => ObjectData::Citation(citation.map_ann_with(f)),
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                raw,
            } => ObjectData::InlineCall {
                name: name.clone(),
                arguments: arguments.clone(),
                header: header.clone(),
                end_header: end_header.clone(),
                raw: raw.clone(),
            },
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                raw,
            } => ObjectData::InlineSrc {
                language: language.clone(),
                parameters: parameters.clone(),
                value: value.clone(),
                raw: raw.clone(),
            },
            ObjectData::Link {
                target,
                description,
                raw_description,
            } => ObjectData::Link {
                target: target.clone(),
                description: description.iter().map(|x| x.map_ann_with(f)).collect(),
                raw_description: raw_description.clone(),
            },
            ObjectData::Target(value) => ObjectData::Target(value.clone()),
            ObjectData::RadioTarget(value) => ObjectData::RadioTarget(value.clone()),
            ObjectData::Macro { name, arguments } => ObjectData::Macro {
                name: name.clone(),
                arguments: arguments.clone(),
            },
            ObjectData::StatisticCookie(value) => ObjectData::StatisticCookie(value.clone()),
            ObjectData::Unknown { kind, raw } => ObjectData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ObjectData<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(match self {
            ObjectData::Plain(value) => ObjectData::Plain(value.clone()),
            ObjectData::LineBreak => ObjectData::LineBreak,
            ObjectData::Markup { kind, children } => ObjectData::Markup {
                kind: *kind,
                children: children
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            },
            ObjectData::Code(value) => ObjectData::Code(value.clone()),
            ObjectData::Verbatim(value) => ObjectData::Verbatim(value.clone()),
            ObjectData::Timestamp(value) => ObjectData::Timestamp(value.clone()),
            ObjectData::Entity(value) => ObjectData::Entity(value.clone()),
            ObjectData::LatexFragment(value) => ObjectData::LatexFragment(value.clone()),
            ObjectData::ExportSnippet { backend, value } => ObjectData::ExportSnippet {
                backend: backend.clone(),
                value: value.clone(),
            },
            ObjectData::FootnoteRef { label, definition } => ObjectData::FootnoteRef {
                label: label.clone(),
                definition: definition
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            },
            ObjectData::Citation(citation) => ObjectData::Citation(citation.try_map_ann_with(f)?),
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                raw,
            } => ObjectData::InlineCall {
                name: name.clone(),
                arguments: arguments.clone(),
                header: header.clone(),
                end_header: end_header.clone(),
                raw: raw.clone(),
            },
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                raw,
            } => ObjectData::InlineSrc {
                language: language.clone(),
                parameters: parameters.clone(),
                value: value.clone(),
                raw: raw.clone(),
            },
            ObjectData::Link {
                target,
                description,
                raw_description,
            } => ObjectData::Link {
                target: target.clone(),
                description: description
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
                raw_description: raw_description.clone(),
            },
            ObjectData::Target(value) => ObjectData::Target(value.clone()),
            ObjectData::RadioTarget(value) => ObjectData::RadioTarget(value.clone()),
            ObjectData::Macro { name, arguments } => ObjectData::Macro {
                name: name.clone(),
                arguments: arguments.clone(),
            },
            ObjectData::StatisticCookie(value) => ObjectData::StatisticCookie(value.clone()),
            ObjectData::Unknown { kind, raw } => ObjectData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        match self {
            ObjectData::Markup { children, .. }
            | ObjectData::FootnoteRef {
                definition: children,
                ..
            }
            | ObjectData::Link {
                description: children,
                ..
            } => {
                for child in children {
                    child.visit_with(f);
                }
            }
            ObjectData::Citation(citation) => citation.visit_with(f),
            _ => {}
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        match self {
            ObjectData::Markup { children, .. }
            | ObjectData::FootnoteRef {
                definition: children,
                ..
            }
            | ObjectData::Link {
                description: children,
                ..
            } => {
                for child in children {
                    child.visit_mut_with(f);
                }
            }
            ObjectData::Citation(citation) => citation.visit_mut_with(f),
            _ => {}
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        match self {
            ObjectData::Markup { children, .. }
            | ObjectData::FootnoteRef {
                definition: children,
                ..
            }
            | ObjectData::Link {
                description: children,
                ..
            } => {
                for child in children {
                    acc = child.fold_with(acc, f);
                }
            }
            ObjectData::Citation(citation) => {
                acc = citation.fold_with(acc, f);
            }
            _ => {}
        }
        acc
    }
}

impl<A> Citation<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Citation<B>
    where
        F: FnMut(&A) -> B,
    {
        Citation {
            style: self.style.clone(),
            variant: self.variant.clone(),
            prefix: self.prefix.iter().map(|x| x.map_ann_with(f)).collect(),
            suffix: self.suffix.iter().map(|x| x.map_ann_with(f)).collect(),
            references: self
                .references
                .iter()
                .map(|x| CiteReference {
                    id: x.id.clone(),
                    prefix: x.prefix.iter().map(|o| o.map_ann_with(f)).collect(),
                    suffix: x.suffix.iter().map(|o| o.map_ann_with(f)).collect(),
                })
                .collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Citation<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Citation {
            style: self.style.clone(),
            variant: self.variant.clone(),
            prefix: self
                .prefix
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            suffix: self
                .suffix
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            references: self
                .references
                .iter()
                .map(|x| {
                    Ok(CiteReference {
                        id: x.id.clone(),
                        prefix: x
                            .prefix
                            .iter()
                            .map(|o| o.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                        suffix: x
                            .suffix
                            .iter()
                            .map(|o| o.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for object in &self.prefix {
            object.visit_with(f);
        }
        for reference in &self.references {
            for object in &reference.prefix {
                object.visit_with(f);
            }
            for object in &reference.suffix {
                object.visit_with(f);
            }
        }
        for object in &self.suffix {
            object.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for object in &mut self.prefix {
            object.visit_mut_with(f);
        }
        for reference in &mut self.references {
            for object in &mut reference.prefix {
                object.visit_mut_with(f);
            }
            for object in &mut reference.suffix {
                object.visit_mut_with(f);
            }
        }
        for object in &mut self.suffix {
            object.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for object in &self.prefix {
            acc = object.fold_with(acc, f);
        }
        for reference in &self.references {
            for object in &reference.prefix {
                acc = object.fold_with(acc, f);
            }
            for object in &reference.suffix {
                acc = object.fold_with(acc, f);
            }
        }
        for object in &self.suffix {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}

struct Converter<'a> {
    source: &'a str,
    lines: LineIndex,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Converter<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            lines: LineIndex::new(source),
            diagnostics: Vec::new(),
        }
    }

    fn document(mut self, root: &SyntaxNode) -> ParsedAst {
        let ann = self.node_ann(root);
        let mut children = Vec::new();
        let mut sections = Vec::new();
        let mut properties = Vec::new();

        for node in root.children() {
            match node.kind() {
                SyntaxKind::SECTION => {
                    let section_children = self.elements_from_container(&node);
                    for child in section_children {
                        if let ElementData::PropertyDrawer(props) = &child.data {
                            properties.extend(props.clone());
                        }
                        children.push(child);
                    }
                }
                SyntaxKind::HEADLINE => sections.push(self.section(&node)),
                _ => {}
            }
        }

        Document {
            ann,
            properties,
            children,
            sections,
            diagnostics: self.diagnostics,
        }
    }

    fn section(&mut self, node: &SyntaxNode) -> Section<ParsedAnnotation> {
        let legacy = syntax_ast::Headline::cast(node.clone()).expect("headline node");
        let properties = legacy
            .properties()
            .map(|drawer| self.properties(&drawer.syntax))
            .unwrap_or_default();
        let anchor = properties
            .iter()
            .find(|property| property.key.eq_ignore_ascii_case("CUSTOM_ID"))
            .map(|property| property.value.clone());
        let planning = legacy
            .planning()
            .map(|planning| self.planning(&planning.syntax))
            .unwrap_or_default();
        let children = legacy
            .section()
            .map(|section| self.elements_from_container(&section.syntax))
            .unwrap_or_default();
        let subsections = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::HEADLINE)
            .map(|child| self.section(&child))
            .collect();
        let todo = legacy.todo_keyword().map(|name| TodoKeyword {
            state: match legacy.todo_type() {
                Some(syntax_ast::TodoType::Done) => TodoState::Done,
                _ => TodoState::Todo,
            },
            name: name.to_string(),
        });
        let title = legacy.title().collect::<Vec<_>>();

        Section {
            ann: self.node_ann(node),
            level: legacy.level(),
            properties,
            todo,
            is_comment: legacy.is_commented(),
            priority: legacy.priority().map(|x| x.to_string()),
            title: self.objects_from_elements(title),
            raw_title: legacy.title_raw(),
            anchor,
            tags: legacy.tags().map(|x| x.to_string()).collect(),
            planning,
            children,
            subsections,
        }
    }

    fn elements_from_container(&mut self, node: &SyntaxNode) -> Vec<Element<ParsedAnnotation>> {
        node.children()
            .filter_map(|child| self.element(&child))
            .collect()
    }

    fn element(&mut self, node: &SyntaxNode) -> Option<Element<ParsedAnnotation>> {
        let affiliated_keywords = self.affiliated_keywords(node);
        let data = match node.kind() {
            SyntaxKind::AFFILIATED_KEYWORD => return None,
            SyntaxKind::PARAGRAPH => {
                ElementData::Paragraph(self.objects_from_elements(node.children_with_tokens()))
            }
            SyntaxKind::KEYWORD => ElementData::Keyword(self.keyword(node, false)),
            SyntaxKind::BABEL_CALL => ElementData::BabelCall(self.keyword(node, false)),
            SyntaxKind::CLOCK => ElementData::Clock(self.clock(node)),
            SyntaxKind::DRAWER => ElementData::Drawer(self.drawer(node)),
            SyntaxKind::PROPERTY_DRAWER => ElementData::PropertyDrawer(self.properties(node)),
            SyntaxKind::LIST => ElementData::List(self.list(node)),
            SyntaxKind::ORG_TABLE => ElementData::Table(self.table(node)),
            SyntaxKind::SOURCE_BLOCK
            | SyntaxKind::EXAMPLE_BLOCK
            | SyntaxKind::EXPORT_BLOCK
            | SyntaxKind::QUOTE_BLOCK
            | SyntaxKind::VERSE_BLOCK
            | SyntaxKind::CENTER_BLOCK
            | SyntaxKind::COMMENT_BLOCK
            | SyntaxKind::SPECIAL_BLOCK
            | SyntaxKind::DYN_BLOCK => ElementData::Block(self.block(node)),
            SyntaxKind::FN_DEF => ElementData::FootnoteDef(self.footnote_def(node)),
            SyntaxKind::COMMENT => ElementData::Comment(node.to_string()),
            SyntaxKind::FIXED_WIDTH => ElementData::FixedWidth(node.to_string()),
            SyntaxKind::RULE => ElementData::Rule,
            SyntaxKind::LATEX_ENVIRONMENT => ElementData::LatexEnvironment(node.to_string()),
            kind => {
                self.diagnostic(
                    node.text_range(),
                    DiagnosticKind::UnsupportedElement,
                    format!("semantic AST has no dedicated element mapping for {kind:?}"),
                );
                ElementData::Unknown {
                    kind: format!("{kind:?}"),
                    raw: node.to_string(),
                }
            }
        };

        Some(Element {
            ann: self.node_ann(node),
            affiliated_keywords,
            data,
        })
    }

    fn affiliated_keywords(&mut self, node: &SyntaxNode) -> Vec<Keyword<ParsedAnnotation>> {
        node.children()
            .take_while(|child| child.kind() == SyntaxKind::AFFILIATED_KEYWORD)
            .map(|child| self.keyword(&child, true))
            .collect()
    }

    fn keyword(&self, node: &SyntaxNode, affiliated: bool) -> Keyword<ParsedAnnotation> {
        if affiliated {
            let legacy = syntax_ast::AffiliatedKeyword::cast(node.clone()).expect("keyword node");
            Keyword {
                ann: self.node_ann(node),
                key: legacy.key().to_string(),
                optional: legacy.optional().map(|x| x.to_string()),
                value: legacy.value().map(|x| x.to_string()).unwrap_or_default(),
            }
        } else {
            let legacy = syntax_ast::Keyword::cast(node.clone());
            if let Some(legacy) = legacy {
                Keyword {
                    ann: self.node_ann(node),
                    key: legacy.key().to_string(),
                    optional: None,
                    value: legacy.value().to_string(),
                }
            } else {
                Keyword {
                    ann: self.node_ann(node),
                    key: format!("{:?}", node.kind()),
                    optional: None,
                    value: node.to_string(),
                }
            }
        }
    }

    fn properties(&self, node: &SyntaxNode) -> Vec<Property<ParsedAnnotation>> {
        syntax_ast::PropertyDrawer::cast(node.clone())
            .map(|drawer| {
                drawer
                    .iter()
                    .map(|(key, value)| Property {
                        ann: self.token_ann(value.syntax()),
                        key: key.to_string(),
                        value: value.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn planning(&self, node: &SyntaxNode) -> Planning {
        let mut planning = Planning::default();
        for child in node.children() {
            let timestamp = child.children().find_map(|n| self.timestamp_node(&n));
            match child.kind() {
                SyntaxKind::PLANNING_DEADLINE => planning.deadline = timestamp,
                SyntaxKind::PLANNING_SCHEDULED => planning.scheduled = timestamp,
                SyntaxKind::PLANNING_CLOSED => planning.closed = timestamp,
                _ => {}
            }
        }
        planning
    }

    fn clock(&self, node: &SyntaxNode) -> Clock {
        let legacy = syntax_ast::Clock::cast(node.clone()).expect("clock node");
        let value = node
            .children()
            .find_map(|child| self.timestamp_node(&child));

        Clock {
            value,
            duration: legacy.duration().map(|token| token.to_string()),
            raw: node.to_string(),
        }
    }

    fn drawer(&mut self, node: &SyntaxNode) -> Drawer<ParsedAnnotation> {
        let name = syntax_ast::Drawer::cast(node.clone())
            .map(|drawer| drawer.name().to_string())
            .unwrap_or_default();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::DRAWER_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();

        Drawer {
            name,
            children,
            raw: node.to_string(),
        }
    }

    fn list(&mut self, node: &SyntaxNode) -> List<ParsedAnnotation> {
        let legacy = syntax_ast::List::cast(node.clone()).expect("list node");
        let has_descriptive_item = node.children().any(|item| {
            item.kind() == SyntaxKind::LIST_ITEM
                && item
                    .children()
                    .any(|child| child.kind() == SyntaxKind::LIST_ITEM_TAG)
        });
        let list_type = if has_descriptive_item || legacy.is_descriptive() {
            ListType::Descriptive
        } else if legacy.is_ordered() {
            ListType::Ordered
        } else {
            ListType::Unordered
        };
        let items = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::LIST_ITEM)
            .map(|child| self.list_item(&child))
            .collect();

        List { list_type, items }
    }

    fn list_item(&mut self, node: &SyntaxNode) -> ListItem<ParsedAnnotation> {
        let legacy = syntax_ast::ListItem::cast(node.clone()).expect("list item node");
        let tag = legacy.tag().collect::<Vec<_>>();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::LIST_ITEM_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();
        let checkbox = legacy.checkbox().and_then(|token| match token.as_ref() {
            "X" => Some(Checkbox::On),
            " " => Some(Checkbox::Off),
            "-" => Some(Checkbox::Trans),
            _ => None,
        });

        ListItem {
            ann: self.node_ann(node),
            bullet: legacy.bullet().to_string(),
            counter: legacy.counter().map(|x| x.to_string()),
            checkbox,
            tag: self.objects_from_elements(tag),
            children,
        }
    }

    fn table(&mut self, node: &SyntaxNode) -> Table<ParsedAnnotation> {
        let rows = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::ORG_TABLE_RULE_ROW | SyntaxKind::ORG_TABLE_STANDARD_ROW
                )
            })
            .map(|child| TableRow {
                ann: self.node_ann(&child),
                is_rule: child.kind() == SyntaxKind::ORG_TABLE_RULE_ROW,
                cells: child
                    .children()
                    .filter(|cell| cell.kind() == SyntaxKind::ORG_TABLE_CELL)
                    .map(|cell| TableCell {
                        ann: self.node_ann(&cell),
                        objects: self.objects_from_elements(cell.children_with_tokens()),
                    })
                    .collect(),
            })
            .collect();

        Table { rows }
    }

    fn block(&mut self, node: &SyntaxNode) -> Block<ParsedAnnotation> {
        let kind = match node.kind() {
            SyntaxKind::SOURCE_BLOCK => BlockKind::Source,
            SyntaxKind::EXAMPLE_BLOCK => BlockKind::Example,
            SyntaxKind::EXPORT_BLOCK => BlockKind::Export,
            SyntaxKind::QUOTE_BLOCK => BlockKind::Quote,
            SyntaxKind::VERSE_BLOCK => BlockKind::Verse,
            SyntaxKind::CENTER_BLOCK => BlockKind::Center,
            SyntaxKind::COMMENT_BLOCK => BlockKind::Comment,
            SyntaxKind::DYN_BLOCK => BlockKind::Dynamic,
            SyntaxKind::SPECIAL_BLOCK => {
                BlockKind::Special(block_name(node).unwrap_or_else(|| "special".into()))
            }
            _ => BlockKind::Special(format!("{:?}", node.kind())),
        };

        let source = syntax_ast::SourceBlock::cast(node.clone());
        let export = syntax_ast::ExportBlock::cast(node.clone());
        let value = node
            .children()
            .find(|child| child.kind() == SyntaxKind::BLOCK_CONTENT)
            .map(|child| child.to_string())
            .unwrap_or_default();
        let children = node
            .children()
            .find(|child| child.kind() == SyntaxKind::BLOCK_CONTENT)
            .map(|child| self.elements_from_container(&child))
            .unwrap_or_default();

        Block {
            kind,
            name: semantic_block_name(node),
            language: source
                .as_ref()
                .and_then(|block| block.language().map(|x| x.to_string())),
            switches: source
                .as_ref()
                .and_then(|block| block.switches().map(|x| x.to_string())),
            parameters: source
                .as_ref()
                .and_then(|block| block.parameters().map(|x| x.to_string())),
            value: source
                .as_ref()
                .map(|block| block.value())
                .or_else(|| export.as_ref().map(|block| block.value()))
                .unwrap_or(value),
            children,
        }
    }

    fn footnote_def(&mut self, node: &SyntaxNode) -> FootnoteDef<ParsedAnnotation> {
        let mut saw_fn_prefix = false;
        let mut saw_label_colon = false;
        let mut after_marker = false;
        let mut label = String::new();
        let mut content = Vec::new();

        for element in node.children_with_tokens() {
            match element.kind() {
                SyntaxKind::AFFILIATED_KEYWORD | SyntaxKind::L_BRACKET => {}
                SyntaxKind::TEXT if !saw_fn_prefix => {
                    saw_fn_prefix = true;
                }
                SyntaxKind::COLON if saw_fn_prefix && !saw_label_colon => {
                    saw_label_colon = true;
                }
                SyntaxKind::R_BRACKET if saw_label_colon => {
                    after_marker = true;
                }
                _ if after_marker => content.push(element),
                SyntaxKind::TEXT if saw_label_colon => {
                    label.push_str(
                        element
                            .as_token()
                            .map(|token| token.text())
                            .unwrap_or_default(),
                    );
                }
                _ => {}
            }
        }
        let children = self
            .paragraph_from_elements(content)
            .into_iter()
            .collect::<Vec<_>>();

        FootnoteDef { label, children }
    }

    fn paragraph_from_elements(
        &mut self,
        elements: Vec<SyntaxElement>,
    ) -> Option<Element<ParsedAnnotation>> {
        let range = range_from_elements(&elements)?;
        let objects = self.objects_from_elements(elements);

        Some(Element {
            ann: self.ann(range),
            affiliated_keywords: Vec::new(),
            data: ElementData::Paragraph(objects),
        })
    }

    fn objects_from_elements(
        &mut self,
        elements: impl IntoIterator<Item = SyntaxElement>,
    ) -> Vec<Object<ParsedAnnotation>> {
        elements
            .into_iter()
            .filter_map(|element| self.object(element))
            .collect()
    }

    fn object(&mut self, element: SyntaxElement) -> Option<Object<ParsedAnnotation>> {
        match element {
            NodeOrToken::Token(token) => self.object_token(token),
            NodeOrToken::Node(node) => self.object_node(node),
        }
    }

    fn object_token(&self, token: SyntaxToken) -> Option<Object<ParsedAnnotation>> {
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

    fn object_node(&mut self, node: SyntaxNode) -> Option<Object<ParsedAnnotation>> {
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
                    kind: format!("{kind:?}"),
                    raw: node.to_string(),
                }
            }
        };

        Some(Object {
            ann: self.node_ann(&node),
            data,
        })
    }

    fn markup(&mut self, node: &SyntaxNode, kind: MarkupKind) -> ObjectData<ParsedAnnotation> {
        ObjectData::Markup {
            kind,
            children: self.objects_from_elements(node.children_with_tokens()),
        }
    }

    fn timestamp_node(&self, node: &SyntaxNode) -> Option<Timestamp> {
        let kind = match node.kind() {
            SyntaxKind::TIMESTAMP_ACTIVE => TimestampKind::Active,
            SyntaxKind::TIMESTAMP_INACTIVE => TimestampKind::Inactive,
            SyntaxKind::TIMESTAMP_DIARY => TimestampKind::Diary,
            _ => return None,
        };
        let legacy = syntax_ast::Timestamp::cast(node.clone()).expect("timestamp node");
        Some(Timestamp {
            kind,
            raw: node.to_string(),
            is_range: legacy.is_range(),
        })
    }

    fn export_snippet(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        if let Some(snippet) = syntax_ast::Snippet::cast(node.clone()) {
            ObjectData::ExportSnippet {
                backend: snippet.backend().to_string(),
                value: snippet.value().to_string(),
            }
        } else {
            ObjectData::Unknown {
                kind: "SNIPPET".into(),
                raw: node.to_string(),
            }
        }
    }

    fn footnote_ref(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let mut saw_fn_prefix = false;
        let mut saw_label_colon = false;
        let mut in_definition = false;
        let mut label = String::new();
        let mut definition = Vec::new();

        for element in node.children_with_tokens() {
            match element.kind() {
                SyntaxKind::L_BRACKET => {}
                SyntaxKind::TEXT if !saw_fn_prefix => {
                    saw_fn_prefix = true;
                }
                SyntaxKind::COLON if saw_fn_prefix && !saw_label_colon => {
                    saw_label_colon = true;
                }
                SyntaxKind::COLON if saw_label_colon && !in_definition => {
                    in_definition = true;
                }
                SyntaxKind::R_BRACKET => break,
                _ if in_definition => definition.push(element),
                SyntaxKind::TEXT if saw_label_colon => {
                    label.push_str(
                        element
                            .as_token()
                            .map(|token| token.text())
                            .unwrap_or_default(),
                    );
                }
                _ => {}
            }
        }

        ObjectData::FootnoteRef {
            label: (!label.is_empty()).then_some(label),
            definition: self.objects_from_elements(definition),
        }
    }

    fn inline_call(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::InlineCall::cast(node.clone()).expect("inline call node");
        let raw = node.to_string();
        ObjectData::InlineCall {
            name: legacy.call().to_string(),
            arguments: legacy.arguments().to_string(),
            header: legacy.inside_header().map(|token| token.to_string()),
            end_header: legacy.end_header().map(|token| token.to_string()),
            raw,
        }
    }

    fn inline_src(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::InlineSrc::cast(node.clone()).expect("inline src node");
        let raw = node.to_string();
        ObjectData::InlineSrc {
            language: legacy.language().to_string(),
            parameters: legacy.parameters().map(|token| token.to_string()),
            value: legacy.value().to_string(),
            raw,
        }
    }

    fn link(&mut self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let legacy = syntax_ast::Link::cast(node.clone()).expect("link node");
        let path = legacy.path().to_string();
        let target = if let Some((protocol, path)) = path.split_once(':') {
            LinkTarget::Uri {
                protocol: protocol.to_string(),
                path: path.to_string(),
            }
        } else if path.starts_with('#') {
            LinkTarget::Internal(path)
        } else {
            LinkTarget::Unresolved(path)
        };
        let description = legacy.description().collect::<Vec<_>>();
        ObjectData::Link {
            target,
            raw_description: legacy.description_raw(),
            description: self.objects_from_elements(description),
        }
    }

    fn macro_object(&self, node: &SyntaxNode) -> ObjectData<ParsedAnnotation> {
        let raw = node.to_string();
        let inner = strip_wrapping(&raw, "{{{", "}}}");
        let (name, args) = inner
            .split_once('(')
            .map(|(name, args)| {
                (
                    name,
                    args.strip_suffix(')')
                        .unwrap_or(args)
                        .split(',')
                        .map(|arg| arg.trim().to_string())
                        .filter(|arg| !arg.is_empty())
                        .collect(),
                )
            })
            .unwrap_or((inner.as_str(), Vec::new()));
        ObjectData::Macro {
            name: name.to_string(),
            arguments: args,
        }
    }

    fn node_ann(&self, node: &SyntaxNode) -> ParsedAnnotation {
        self.ann(node.text_range())
    }

    fn token_ann(&self, token: &SyntaxToken) -> ParsedAnnotation {
        self.ann(token.text_range())
    }

    fn ann(&self, range: TextRange) -> ParsedAnnotation {
        let start = self.lines.position(range.start());
        let end = self.lines.position(range.end());
        let raw = self.raw(range).to_string();
        ParsedAnnotation {
            range,
            start,
            end,
            raw,
        }
    }

    fn raw(&self, range: TextRange) -> &str {
        let start: usize = range.start().into();
        let end: usize = range.end().into();
        self.source.get(start..end).unwrap_or_default()
    }

    fn diagnostic(&mut self, range: TextRange, kind: DiagnosticKind, message: String) {
        self.diagnostics.push(Diagnostic {
            range,
            kind,
            message,
        });
    }
}

struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(idx + 1);
            }
        }
        Self { starts }
    }

    fn position(&self, offset: TextSize) -> SourcePosition {
        let offset = usize::from(offset);
        let line = match self.starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        SourcePosition {
            line: line + 1,
            column: offset.saturating_sub(self.starts[line]) + 1,
        }
    }
}

fn block_name(node: &SyntaxNode) -> Option<String> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::BLOCK_BEGIN)
        .and_then(|begin| {
            begin
                .children_with_tokens()
                .filter_map(|child| child.into_token())
                .find(|token| token.kind() == SyntaxKind::TEXT)
                .map(|token| token.text().to_string())
        })
}

fn semantic_block_name(node: &SyntaxNode) -> Option<String> {
    match node.kind() {
        SyntaxKind::SOURCE_BLOCK => Some("src".into()),
        SyntaxKind::EXAMPLE_BLOCK => Some("example".into()),
        SyntaxKind::EXPORT_BLOCK => Some("export".into()),
        SyntaxKind::QUOTE_BLOCK => Some("quote".into()),
        SyntaxKind::VERSE_BLOCK => Some("verse".into()),
        SyntaxKind::CENTER_BLOCK => Some("center".into()),
        SyntaxKind::COMMENT_BLOCK => Some("comment".into()),
        SyntaxKind::DYN_BLOCK => Some("dynamic".into()),
        SyntaxKind::SPECIAL_BLOCK => block_name(node),
        _ => None,
    }
}

fn range_from_elements(elements: &[SyntaxElement]) -> Option<TextRange> {
    let start = elements.first()?.text_range().start();
    let end = elements.last()?.text_range().end();
    Some(TextRange::new(start, end))
}

fn strip_pair(value: &str) -> &str {
    value
        .char_indices()
        .nth(1)
        .and_then(|(start, _)| {
            value
                .char_indices()
                .last()
                .map(|(end, _)| &value[start..end])
        })
        .unwrap_or_default()
}

fn strip_wrapping(value: &str, prefix: &str, suffix: &str) -> String {
    value
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .unwrap_or(value)
        .to_string()
}
