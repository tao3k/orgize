//! Semantic AST data model.

use rowan::TextRange;

use super::attachment_model::{AttachmentDirectory, AttachmentLink, AttachmentState};
use super::block_model::{
    BlockCodeRef, BlockHeaderArg, BlockLine, BlockLineNumbering, BlockSwitches, SemanticFixedWidth,
    joined_block_lines,
};
use super::lifecycle_model::{ArchiveLocation, ArchiveState};
use super::link_model::{
    FileLink, LinkDescriptionState, LinkMediaKind, LinkPath, LinkSearch, LinkTarget,
};
use super::property_model::{OrgDuration, Priority};
use super::timestamp_model::Timestamp;

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
    pub archive_locations: Vec<ArchiveLocation<A>>,
    pub metadata: Vec<Keyword<A>>,
    pub filetags: Vec<String>,
    pub tag_definitions: Vec<TagDefinition>,
    pub export_settings: ExportSettings,
    pub link_abbreviations: Vec<LinkAbbreviation>,
    pub includes: Vec<IncludeDirective<A>>,
    pub macro_definitions: Vec<MacroDefinition<A>>,
    pub targets: Vec<TargetDefinition<A>>,
    pub footnotes: Vec<FootnoteEntry<A>>,
    pub children: Vec<Element<A>>,
    pub sections: Vec<Section<A>>,
    pub diagnostics: Vec<Diagnostic>,
}

/// One tag entry from a document-level `#+TAGS:` vocabulary line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagDefinition {
    pub name: String,
    pub shortcut: Option<String>,
    pub raw: String,
}

/// Document-level export and indexing settings collected from Org keywords.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportSettings {
    pub select_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub headline_levels: Option<usize>,
    pub special_strings: Option<bool>,
    pub expand_entities: Option<bool>,
}

/// `#+LINK:` abbreviation definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkAbbreviation {
    pub name: String,
    pub replacement: String,
    pub raw_value: String,
}

/// Link object with target, description, caption, and image metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link<A = ()> {
    pub path: LinkPath,
    pub target: LinkTarget,
    pub description: Vec<Object<A>>,
    pub default_description: Vec<Object<A>>,
    pub raw_description: String,
    pub description_state: LinkDescriptionState,
    pub media_kind: LinkMediaKind,
    pub caption: Option<Keyword<A>>,
    pub search: Option<LinkSearch>,
    pub attachment: Option<Box<AttachmentLink>>,
    pub file: Option<Box<FileLink>>,
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

    /// Returns the explicit description when present, otherwise target-derived fallback text.
    pub fn description_or_default(&self) -> &[Object<A>] {
        if self.has_description() {
            &self.description
        } else {
            &self.default_description
        }
    }
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
    pub alias: Vec<Object<A>>,
}

/// Source category for a document-local target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    /// Headline title fuzzy target.
    Headline,
    /// `CUSTOM_ID` property target.
    CustomId,
    /// Org-id `ID` property target.
    Id,
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
    pub effective_properties: Vec<Property<A>>,
    pub archive: ArchiveState<A>,
    pub attachment: AttachmentState<A>,
    pub todo: Option<TodoKeyword>,
    pub is_comment: bool,
    pub priority: Priority,
    pub title: Vec<Object<A>>,
    pub raw_title: String,
    pub anchor: Option<String>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub planning: Planning,
    pub children: Vec<Element<A>>,
    pub subsections: Vec<Section<A>>,
}

/// Semantic inlinetask element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inlinetask<A = ()> {
    pub level: usize,
    pub todo: Option<TodoKeyword>,
    pub priority: Priority,
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
    pub duration: Option<OrgDuration>,
}

impl<A> Property<A> {
    /// Returns true when this property carries an Org effort estimate.
    pub fn is_effort(&self) -> bool {
        self.key.eq_ignore_ascii_case("EFFORT")
    }

    /// Returns parsed duration metadata for duration-shaped property values.
    pub fn parsed_duration(&self) -> Option<&OrgDuration> {
        self.duration.as_ref()
    }
}

/// Keyword or affiliated keyword with optional bracket metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyword<A = ()> {
    pub ann: A,
    pub key: String,
    pub optional: Option<String>,
    pub value: String,
    pub parsed: Vec<Object<A>>,
    pub attributes: Vec<KeywordAttribute>,
}

/// Structured `:key value` argument from an `ATTR_*` keyword.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeywordAttribute {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
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
    /// Fixed-width area with line-level metadata.
    FixedWidth(SemanticFixedWidth<A>),
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

/// Block element with normalized kind and block metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block<A = ()> {
    pub kind: BlockKind,
    pub name: Option<String>,
    pub language: Option<String>,
    pub switches: Option<String>,
    pub switch_options: BlockSwitches,
    pub line_numbering: Option<BlockLineNumbering>,
    pub preserve_indentation: bool,
    pub lines: Vec<BlockLine<A>>,
    pub code_refs: Vec<BlockCodeRef>,
    pub parameters: Option<String>,
    pub header_args: Vec<BlockHeaderArg>,
    pub value: String,
    pub children: Vec<Element<A>>,
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

impl<A> Block<A> {
    pub fn normalized_value(&self) -> String {
        joined_block_lines(&self.lines, |line| line.normalized_value.as_str())
    }

    pub fn value_without_code_refs(&self) -> String {
        joined_block_lines(&self.lines, |line| line.value_without_code_ref.as_str())
    }

    pub fn normalized_value_without_code_refs(&self) -> String {
        joined_block_lines(&self.lines, |line| {
            line.normalized_value_without_code_ref.as_str()
        })
    }
}

/// Clock value and optional duration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    pub value: Option<Timestamp>,
    pub duration: Option<String>,
    pub parsed_duration: Option<OrgDuration>,
    pub raw: String,
}

impl Clock {
    /// Returns parsed duration metadata for a clock summary, when present.
    pub fn parsed_duration(&self) -> Option<&OrgDuration> {
        self.parsed_duration.as_ref()
    }
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
    pub parsed_formulas: Vec<TableFormula<A>>,
}

/// Parsed table formula metadata from a `#+TBLFM:` keyword.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFormula<A = ()> {
    pub ann: A,
    pub raw: String,
    pub assignments: Vec<TableFormulaAssignment>,
}

/// One formula assignment inside a `#+TBLFM:` line, split on Org's `::`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFormulaAssignment {
    pub raw: String,
    pub lhs: String,
    pub rhs: String,
    pub flags: Vec<String>,
    pub references: Vec<TableFormulaReference>,
}

/// A table reference token found in a formula expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFormulaReference {
    pub raw: String,
    pub kind: TableFormulaReferenceKind,
}

/// Formula reference category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableFormulaReferenceKind {
    Field,
    Remote,
    Row,
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

/// Footnote definition with label and parsed body elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteDef<A = ()> {
    pub label: String,
    pub children: Vec<Element<A>>,
}

/// Document-level footnote registry entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteEntry<A = ()> {
    pub ann: A,
    pub label: String,
    pub definition: FootnoteDefinition<A>,
}

/// Body storage for standalone and inline footnote definitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FootnoteDefinition<A = ()> {
    Standalone(Vec<Element<A>>),
    Inline(Vec<Object<A>>),
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
        resolved_label: Option<String>,
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
    Link(Box<Link<A>>),
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

/// Options for explicit semantic export projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportProjectionOptions {
    pub prune: bool,
    pub special_strings: bool,
    pub expand_entities: bool,
    pub expand_link_abbreviations: bool,
    pub select_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub headline_level_shift: isize,
}

impl Default for ExportProjectionOptions {
    fn default() -> Self {
        Self {
            prune: false,
            special_strings: false,
            expand_entities: true,
            expand_link_abbreviations: true,
            select_tags: Vec::new(),
            exclude_tags: Vec::new(),
            headline_level_shift: 0,
        }
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
    /// Document-level footnote registry entry.
    FootnoteEntry(&'a FootnoteEntry<A>),
    /// Archive destination collected from `#+ARCHIVE:`.
    ArchiveLocation(&'a ArchiveLocation<A>),
    /// Effective attachment directory visible from a section.
    AttachmentDirectory(&'a AttachmentDirectory<A>),
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
    /// Source/example/fixed-width content line.
    BlockLine(&'a BlockLine<A>),
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
    /// Document-level footnote registry entry.
    FootnoteEntry(&'a mut FootnoteEntry<A>),
    /// Archive destination collected from `#+ARCHIVE:`.
    ArchiveLocation(&'a mut ArchiveLocation<A>),
    /// Effective attachment directory visible from a section.
    AttachmentDirectory(&'a mut AttachmentDirectory<A>),
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
    /// Source/example/fixed-width content line.
    BlockLine(&'a mut BlockLine<A>),
    /// Object node.
    Object(&'a mut Object<A>),
}
