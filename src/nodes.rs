use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceInfo {
    line: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    start: SourceInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListEnumType {
    Unordered,
    Arabic,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "PathBuf", into = "PathBuf")]
pub struct FileId {
    pub path: PathBuf,
}

impl From<FileId> for PathBuf {
    fn from(val: FileId) -> Self {
        val.path
    }
}

impl From<PathBuf> for FileId {
    fn from(value: PathBuf) -> Self {
        FileId { path: value }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Node {
    Code(Code),
    Comment(Comment),
    Label(Label),
    Section(Section),
    Paragraph(Paragraph),
    Footnote(Footnote),
    FootnoteReference(FootnoteReference),
    SubstitutionDefinition(SubstitutionDefinition),
    SubstitutionReference(SubstitutionReference),
    Root(Root),
    Heading(Heading),

    #[serde(rename = "definitionListItem")]
    DefinitionListItem(DefinitionListItem),

    #[serde(rename = "definitionList")]
    DefinitionList(DefinitionList),

    #[serde(rename = "listItem")]
    ListItem(ListItem),

    List(List),

    Line(Line),
    LineBlock(LineBlock),
    Directive(Directive),
    DirectiveArgument(DirectiveArgument),
    Target(Target),
    TargetIdentifier(TargetIdentifier),
    InlineTarget(InlineTarget),
    Reference(Reference),
    NamedReference(NamedReference),
    Role(Role),
    RefRole(RefRole),
    Text(Text),
    Literal(Literal),
    Emphasis(Emphasis),
    Strong(Strong),
    Field(Field),
    FieldList(FieldList),
    Transition(Transition),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    lang: Option<String>,
    caption: Option<String>,
    copyable: bool,
    emphasize_lines: Option<Vec<(i32, i32)>>,
    value: String,
    linenos: bool,
    lineno_start: Option<i32>,
    source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Label {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paragraph {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Footnote {
    children: Vec<Node>,
    id: String,
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FootnoteReference {
    children: Vec<Node>, // InlineNode
    id: String,
    refname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubstitutionDefinition {
    children: Vec<Node>, // InlineNode
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubstitutionReference {
    children: Vec<Node>, // InlineNode
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockSubstitutionReference {
    children: Vec<Node>,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    children: Vec<Node>,
    fileid: FileId,

    #[serde(default)]
    options: HashMap<String, bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Heading {
    children: Vec<Node>, // InlineNode
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefinitionListItem {
    children: Vec<Node>,
    term: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefinitionList {
    children: Vec<DefinitionListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItem {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    children: Vec<ListItem>,
    enumtype: ListEnumType,
    startat: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineBlock {
    children: Vec<Line>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Directive {
    children: Vec<Node>,
    domain: String,
    name: String,
    argument: Vec<Node>, // InlineNode

    #[serde(default)]
    options: HashMap<String, bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TocTreeDirectiveEntry {
    title: Option<String>,
    url: Option<String>,
    slug: Option<String>,
    ref_project: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TocTreeDirective {
    directive: Directive,
    entries: Vec<TocTreeDirectiveEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectiveArgument {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    domain: String,
    name: String,
    html_id: Option<String>,

    #[serde(default)]
    options: Option<HashMap<String, bson::Bson>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetIdentifier {
    children: Vec<Node>, // InlineNode
    ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineTarget {
    children: Vec<Node>, // InlineNode
    #[serde(flatten)]
    target: Target,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    children: Vec<Node>, // InlineNode
    refuri: String,

    #[serde(default)]
    refname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedReference {
    refname: String,
    refuri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    children: Vec<Node>, // InlineNode
    domain: String,
    name: String,
    target: String,
    flag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefRole {
    #[serde(flatten)]
    role: Role,

    fileid: Option<(String, String)>,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Text {
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Literal {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Emphasis {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Strong {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    children: Vec<Node>,
    name: String,
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldList {
    children: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transition {}

#[derive(Debug, Serialize, Deserialize)]
pub struct StaticAssetReference {
    checksum: String,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    page_id: String,
    filename: String,
    ast: Root,
    source: String,
    static_assets: Vec<StaticAssetReference>,
}
