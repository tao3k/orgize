//! Semantic AST data model.

use rowan::TextRange;

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
    TableEl { raw: String },
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
    pub formulas: Vec<Keyword<A>>,
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
    Cloze {
        text: Vec<Object<A>>,
        raw_text: String,
        hint: Option<String>,
        id: Option<String>,
        raw: String,
    },
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
    Link(Link<A>),
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
    pub start: Option<TimestampMoment>,
    pub end: Option<TimestampMoment>,
    pub repeater: Option<TimestampRepeater>,
    pub warning: Option<TimestampWarning>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampKind {
    Active,
    Inactive,
    Diary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampMoment {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub day_name: Option<String>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampRepeater {
    pub kind: RepeaterKind,
    pub value: u32,
    pub unit: TimeUnit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampWarning {
    pub kind: WarningKind,
    pub value: u32,
    pub unit: TimeUnit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeaterKind {
    Cumulate,
    CatchUp,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningKind {
    All,
    First,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    Uri { protocol: String, path: String },
    Internal(String),
    Unresolved(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link<A = ()> {
    pub path: String,
    pub target: LinkTarget,
    pub description: Vec<Object<A>>,
    pub raw_description: String,
    pub has_description: bool,
    pub is_image: bool,
    pub caption: Option<Keyword<A>>,
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
