//! Semantic AST data model.

use rowan::TextRange;

/// Parsed semantic document with source annotations on every semantic node.
pub type ParsedAst = Document<ParsedAnnotation>;

/// Semantic document without source annotations, useful for snapshots and equality checks.
pub type BareAst = Document<()>;

/// Source-backed annotation attached to semantic nodes projected from the syntax tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedAnnotation {
    pub range: TextRange,
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub raw: String,
}

/// One-based line and column position in the original Org source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

/// Diagnostic emitted when syntax cannot be projected cleanly into semantic AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: TextRange,
    pub kind: DiagnosticKind,
    pub message: String,
}

/// Category for a semantic projection diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// A syntax element has no semantic element mapping yet.
    UnsupportedElement,
    /// A syntax object has no semantic object mapping yet.
    UnsupportedObject,
    /// A modeled syntax node was present, but its semantic fields could not be derived.
    Conversion,
}

/// Syntax kind name for a node that the semantic projection intentionally keeps unsupported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedSyntaxKind(String);

impl UnsupportedSyntaxKind {
    /// Creates an unsupported syntax kind marker from a parser kind name.
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    /// Returns the parser kind name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the marker and returns the parser kind name.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for UnsupportedSyntaxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for UnsupportedSyntaxKind {
    fn from(kind: String) -> Self {
        Self::new(kind)
    }
}

impl From<&str> for UnsupportedSyntaxKind {
    fn from(kind: &str) -> Self {
        Self::new(kind)
    }
}

/// Root semantic representation of an Org document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document<A = ()> {
    pub ann: A,
    pub properties: Vec<Property<A>>,
    pub includes: Vec<IncludeDirective<A>>,
    pub macro_definitions: Vec<MacroDefinition<A>>,
    pub targets: Vec<TargetDefinition<A>>,
    pub children: Vec<Element<A>>,
    pub sections: Vec<Section<A>>,
    pub diagnostics: Vec<Diagnostic>,
}

/// `#+INCLUDE:` directive collected for explicit preprocessing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeDirective<A = ()> {
    pub ann: A,
    pub path: String,
    pub raw_path: String,
    pub arguments: Vec<String>,
    pub options: Vec<IncludeOption>,
    pub raw_value: String,
}

/// Keyword-style option attached to an `#+INCLUDE:` directive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeOption {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
}

/// `#+MACRO:` definition collected without expanding macro calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroDefinition<A = ()> {
    pub ann: A,
    pub name: String,
    pub template: String,
    pub raw_value: String,
}

/// Opt-in expansion result for one semantic macro call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroExpansion<A = ()> {
    pub ann: A,
    pub name: String,
    pub arguments: Vec<String>,
    pub template: Option<String>,
    pub value: Option<String>,
    pub status: MacroExpansionStatus,
}

/// Expansion status for a semantic macro call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroExpansionStatus {
    /// A matching `#+MACRO:` definition was found.
    Expanded,
    /// The call has no matching definition in this document.
    MissingDefinition,
}

/// Document-local target that can satisfy an internal link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetDefinition<A = ()> {
    pub ann: A,
    pub kind: TargetKind,
    pub key: String,
    pub value: String,
    pub raw: String,
}

/// Source category for a document-local target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    /// Headline title fuzzy target.
    Headline,
    /// `CUSTOM_ID` property target.
    CustomId,
    /// Explicit `<<target>>` object.
    Target,
    /// Explicit `<<<radio target>>>` object.
    RadioTarget,
    /// Footnote definition label.
    FootnoteDefinition,
    /// Source/example code reference.
    CodeRef,
}

/// Semantic section rooted by a headline.
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

/// Semantic inlinetask element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inlinetask<A = ()> {
    pub level: usize,
    pub todo: Option<TodoKeyword>,
    pub priority: Option<String>,
    pub title: Vec<Object<A>>,
    pub raw_title: String,
    pub tags: Vec<String>,
    pub planning: Planning,
    pub properties: Vec<Property<A>>,
    pub children: Vec<Element<A>>,
    pub end: Option<InlinetaskEnd<A>>,
}

/// Closing `END` marker for an inlinetask that contains elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlinetaskEnd<A = ()> {
    pub ann: A,
    pub level: usize,
    pub raw: String,
}

/// TODO keyword plus its configured state class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TodoKeyword {
    pub state: TodoState,
    pub name: String,
}

/// State class for a TODO keyword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodoState {
    /// Keyword belongs to the configured TODO set.
    Todo,
    /// Keyword belongs to the configured DONE set.
    Done,
}

/// Planning timestamps attached to a headline.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Planning {
    pub deadline: Option<Timestamp>,
    pub scheduled: Option<Timestamp>,
    pub closed: Option<Timestamp>,
}

/// Node property from a property drawer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property<A = ()> {
    pub ann: A,
    pub key: String,
    pub value: String,
}

/// Keyword or affiliated keyword with optional bracket metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyword<A = ()> {
    pub ann: A,
    pub key: String,
    pub optional: Option<String>,
    pub value: String,
}

/// Semantic element in a document, section, drawer, list item, or block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element<A = ()> {
    pub ann: A,
    pub affiliated_keywords: Vec<Keyword<A>>,
    pub data: ElementData<A>,
}

/// Element-specific semantic payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementData<A = ()> {
    /// Paragraph objects in source order.
    Paragraph(Vec<Object<A>>),
    /// Standalone `#+KEY: value` keyword.
    Keyword(Keyword<A>),
    /// Inline Babel call represented with keyword-like fields.
    BabelCall(Keyword<A>),
    /// Clock element.
    Clock(Clock),
    /// Named drawer with parsed child elements.
    Drawer(Drawer<A>),
    /// Property drawer projected as owned properties.
    PropertyDrawer(Vec<Property<A>>),
    /// Plain list.
    List(List<A>),
    /// Org table with rows, cells, and formulas.
    Table(Table<A>),
    /// Table.el table kept as raw text for now.
    TableEl { raw: String },
    /// Greater or lesser block.
    Block(Block<A>),
    /// Footnote definition.
    FootnoteDef(FootnoteDef<A>),
    /// Inlinetask.
    Inlinetask(Box<Inlinetask<A>>),
    /// Comment element raw text.
    Comment(String),
    /// Fixed-width area raw text.
    FixedWidth(String),
    /// Horizontal rule.
    Rule,
    /// LaTeX environment raw text.
    LatexEnvironment(String),
    /// Intentionally unsupported syntax element with kind and raw source.
    Unknown {
        kind: UnsupportedSyntaxKind,
        raw: String,
    },
}

/// Clock value and optional duration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    pub value: Option<Timestamp>,
    pub duration: Option<String>,
    pub raw: String,
}

/// Named drawer projected with semantic child elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawer<A = ()> {
    pub name: String,
    pub children: Vec<Element<A>>,
    pub raw: String,
}

/// Plain list with normalized list type and items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List<A = ()> {
    pub list_type: ListType,
    pub items: Vec<ListItem<A>>,
}

/// Normalized Org list category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListType {
    /// Ordered list.
    Ordered,
    /// Unordered list.
    Unordered,
    /// Description list containing tags.
    Descriptive,
}

/// Item inside an Org plain list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem<A = ()> {
    pub ann: A,
    pub bullet: String,
    pub counter: Option<String>,
    pub checkbox: Option<Checkbox>,
    pub tag: Vec<Object<A>>,
    pub children: Vec<Element<A>>,
}

/// Checkbox state for a list item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checkbox {
    /// Checked state `[X]`.
    On,
    /// Unchecked state `[ ]`.
    Off,
    /// Transitional state `[-]`.
    Trans,
}

/// Org table with semantic rows and trailing formula keywords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table<A = ()> {
    pub rows: Vec<TableRow<A>>,
    pub column_alignments: Vec<Option<TableColumnAlignment>>,
    pub formulas: Vec<Keyword<A>>,
}

/// Alignment cookie parsed from an Org table column metadata row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableColumnAlignment {
    /// Left alignment, represented by `<l>`.
    Left,
    /// Center alignment, represented by `<c>`.
    Center,
    /// Right alignment, represented by `<r>`.
    Right,
}

/// Row inside an Org table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow<A = ()> {
    pub ann: A,
    pub is_rule: bool,
    pub cells: Vec<TableCell<A>>,
}

/// Cell inside an Org table row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableCell<A = ()> {
    pub ann: A,
    pub objects: Vec<Object<A>>,
}

/// Block element with normalized kind and block metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block<A = ()> {
    pub kind: BlockKind,
    pub name: Option<String>,
    pub language: Option<String>,
    pub switches: Option<String>,
    pub line_numbering: Option<BlockLineNumbering>,
    pub preserve_indentation: bool,
    pub code_refs: Vec<BlockCodeRef>,
    pub parameters: Option<String>,
    pub header_args: Vec<BlockHeaderArg>,
    pub value: String,
    pub children: Vec<Element<A>>,
}

/// Line-numbering switch metadata for source and example blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLineNumbering {
    pub mode: BlockLineNumberMode,
    pub start: Option<usize>,
}

/// Org source/example block line-numbering mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockLineNumberMode {
    /// Start a fresh numbered listing with `-n`.
    New,
    /// Continue from the previous numbered listing with `+n`.
    Continued,
}

/// Code reference cookie found inside a source or example block line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockCodeRef {
    /// One-based line number inside the block value.
    pub line: usize,
    /// Reference name extracted from the active label format.
    pub name: String,
    /// Raw reference cookie as it appears in the block line.
    pub raw: String,
}

/// Header argument parsed from a source block parameter string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockHeaderArg {
    /// Header argument key without the leading colon.
    pub key: String,
    /// Header argument value, if present, preserving inner spacing.
    pub value: Option<String>,
    /// Raw header argument fragment as it appears in the begin line.
    pub raw: String,
}

/// Semantic category for an Org block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// Source block.
    Source,
    /// Example block.
    Example,
    /// Export block.
    Export,
    /// Quote block.
    Quote,
    /// Verse block.
    Verse,
    /// Center block.
    Center,
    /// Comment block.
    Comment,
    /// Dynamic block.
    Dynamic,
    /// Named special block.
    Special(String),
}

/// Footnote definition with label and parsed body elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteDef<A = ()> {
    pub label: String,
    pub children: Vec<Element<A>>,
}

/// Semantic object inside paragraph-like content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object<A = ()> {
    pub ann: A,
    pub data: ObjectData<A>,
}

/// Object-specific semantic payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectData<A = ()> {
    /// Plain text segment.
    Plain(String),
    /// Explicit line break object.
    LineBreak,
    /// Markup object with parsed child objects.
    Markup {
        kind: MarkupKind,
        children: Vec<Object<A>>,
    },
    /// Inline code contents without delimiters.
    Code(String),
    /// Verbatim contents without delimiters.
    Verbatim(String),
    /// Timestamp object.
    Timestamp(Timestamp),
    /// Entity raw text.
    Entity(String),
    /// LaTeX fragment raw text.
    LatexFragment(String),
    /// Export snippet with backend and value.
    ExportSnippet { backend: String, value: String },
    /// Footnote reference, optionally with inline definition objects.
    FootnoteRef {
        label: Option<String>,
        definition: Vec<Object<A>>,
    },
    /// Citation object with parsed references and affixes.
    Citation(Citation<A>),
    /// Org-fc cloze object.
    Cloze {
        text: Vec<Object<A>>,
        raw_text: String,
        hint: Option<String>,
        id: Option<String>,
        raw: String,
    },
    /// Inline Babel call.
    InlineCall {
        name: String,
        arguments: String,
        header: Option<String>,
        end_header: Option<String>,
        raw: String,
    },
    /// Inline source block.
    InlineSrc {
        language: String,
        parameters: Option<String>,
        value: String,
        raw: String,
    },
    /// Link object with parsed target and description metadata.
    Link(Link<A>),
    /// Target object without angle delimiters.
    Target(String),
    /// Radio target object without angle delimiters.
    RadioTarget(String),
    /// Macro call with name and parsed arguments.
    Macro {
        name: String,
        arguments: Vec<String>,
    },
    /// Statistics cookie raw text.
    StatisticCookie(String),
    /// Intentionally unsupported syntax object with kind and raw source.
    Unknown {
        kind: UnsupportedSyntaxKind,
        raw: String,
    },
}

/// Semantic markup kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupKind {
    /// Bold markup.
    Bold,
    /// Italic markup.
    Italic,
    /// Underline markup.
    Underline,
    /// Strike-through markup.
    Strike,
    /// Superscript markup.
    Superscript,
    /// Subscript markup.
    Subscript,
}

/// Timestamp metadata projected from Org timestamp syntax.
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

/// Org timestamp delimiter category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampKind {
    /// Active timestamp, for example `<2026-05-01 Fri>`.
    Active,
    /// Inactive timestamp, for example `[2026-05-01 Fri]`.
    Inactive,
    /// Diary sexp timestamp, for example `<%%(diary-date 5 1)>`.
    Diary,
}

/// Parsed date and optional time within a timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampMoment {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub day_name: Option<String>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
}

/// Repeater cookie attached to a timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampRepeater {
    pub kind: RepeaterKind,
    pub value: u32,
    pub unit: TimeUnit,
}

/// Warning delay cookie attached to a timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampWarning {
    pub kind: WarningKind,
    pub value: u32,
    pub unit: TimeUnit,
}

/// Org timestamp repeater mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeaterKind {
    /// Cumulate repeater, written with `++`.
    Cumulate,
    /// Catch-up repeater, written with `+`.
    CatchUp,
    /// Restart repeater, written with `.+`.
    Restart,
}

/// Org timestamp warning delay mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningKind {
    /// Warn for all matching occurrences.
    All,
    /// Warn only for the first occurrence.
    First,
}

/// Unit used by timestamp repeater and warning cookies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    /// Hour unit.
    Hour,
    /// Day unit.
    Day,
    /// Week unit.
    Week,
    /// Month unit.
    Month,
    /// Year unit.
    Year,
}

/// Normalized link target classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// URI-like link target split into protocol and path.
    Uri { protocol: String, path: String },
    /// Internal target such as `#custom-id`.
    Internal(String),
    /// Link target without a dedicated semantic classifier yet.
    Unresolved(String),
}

/// Original link path text as it appears in the link target position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkPath(String);

impl LinkPath {
    /// Creates a link path from parser-owned text.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the path and returns its owned text.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for LinkPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for LinkPath {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<&str> for LinkPath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl AsRef<str> for LinkPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Whether a link had an explicit Org description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkDescriptionState {
    /// Link has no explicit description.
    None,
    /// Link was written with a description part.
    Explicit,
}

impl LinkDescriptionState {
    /// Returns true when the source link had an explicit description.
    pub const fn has_description(self) -> bool {
        matches!(self, Self::Explicit)
    }
}

/// Media classification for link exporter behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkMediaKind {
    /// Normal link.
    Normal,
    /// Image link.
    Image,
}

impl LinkMediaKind {
    /// Returns true when the link should be treated as an image.
    pub const fn is_image(self) -> bool {
        matches!(self, Self::Image)
    }
}

/// Link object with target, description, caption, and image metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link<A = ()> {
    pub path: LinkPath,
    pub target: LinkTarget,
    pub description: Vec<Object<A>>,
    pub raw_description: String,
    pub description_state: LinkDescriptionState,
    pub media_kind: LinkMediaKind,
    pub caption: Option<Keyword<A>>,
}

impl<A> Link<A> {
    /// Returns the original link path text.
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    /// Returns true when the source link had an explicit description.
    pub fn has_description(&self) -> bool {
        self.description_state.has_description()
    }

    /// Returns true when the link should be treated as an image.
    pub fn is_image(&self) -> bool {
        self.media_kind.is_image()
    }
}

/// Citation object with global affixes and per-reference metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation<A = ()> {
    pub style: String,
    pub variant: String,
    pub prefix: Vec<Object<A>>,
    pub suffix: Vec<Object<A>>,
    pub references: Vec<CiteReference<A>>,
}

/// Single citation reference inside a citation object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CiteReference<A = ()> {
    pub id: String,
    pub prefix: Vec<Object<A>>,
    pub suffix: Vec<Object<A>>,
}

/// Borrowed node exposed to semantic traversal callbacks.
pub enum AstRef<'a, A> {
    /// Document node.
    Document(&'a Document<A>),
    /// Include preprocessing directive.
    IncludeDirective(&'a IncludeDirective<A>),
    /// Macro definition preprocessing directive.
    MacroDefinition(&'a MacroDefinition<A>),
    /// Document-local internal link target.
    TargetDefinition(&'a TargetDefinition<A>),
    /// Section node.
    Section(&'a Section<A>),
    /// Property node.
    Property(&'a Property<A>),
    /// Keyword node.
    Keyword(&'a Keyword<A>),
    /// Element node.
    Element(&'a Element<A>),
    /// Inlinetask node.
    Inlinetask(&'a Inlinetask<A>),
    /// Inlinetask closing marker.
    InlinetaskEnd(&'a InlinetaskEnd<A>),
    /// List item node.
    ListItem(&'a ListItem<A>),
    /// Table row node.
    TableRow(&'a TableRow<A>),
    /// Table cell node.
    TableCell(&'a TableCell<A>),
    /// Object node.
    Object(&'a Object<A>),
}

/// Mutable node exposed to semantic traversal callbacks.
pub enum AstMut<'a, A> {
    /// Document node.
    Document(&'a mut Document<A>),
    /// Include preprocessing directive.
    IncludeDirective(&'a mut IncludeDirective<A>),
    /// Macro definition preprocessing directive.
    MacroDefinition(&'a mut MacroDefinition<A>),
    /// Document-local internal link target.
    TargetDefinition(&'a mut TargetDefinition<A>),
    /// Section node.
    Section(&'a mut Section<A>),
    /// Property node.
    Property(&'a mut Property<A>),
    /// Keyword node.
    Keyword(&'a mut Keyword<A>),
    /// Element node.
    Element(&'a mut Element<A>),
    /// Inlinetask node.
    Inlinetask(&'a mut Inlinetask<A>),
    /// Inlinetask closing marker.
    InlinetaskEnd(&'a mut InlinetaskEnd<A>),
    /// List item node.
    ListItem(&'a mut ListItem<A>),
    /// Table row node.
    TableRow(&'a mut TableRow<A>),
    /// Table cell node.
    TableCell(&'a mut TableCell<A>),
    /// Object node.
    Object(&'a mut Object<A>),
}
