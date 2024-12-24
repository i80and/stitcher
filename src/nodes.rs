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
pub struct Node {
    #[serde(flatten)]
    pub data: NodeData,

    position: Position,
}

impl Node {
    pub fn for_each(&mut self, f: &mut impl FnMut(&mut Node)) {
        f(self);

        for child in self.data.get_children() {
            child.for_each(f);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum NodeData {
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

impl NodeData {
    pub fn get_children(&mut self) -> &mut [Node] {
        match self {
            NodeData::Code(_) => &mut [],
            NodeData::Comment(node) => &mut node.children,
            NodeData::Label(node) => &mut node.children,
            NodeData::Section(node) => &mut node.children,
            NodeData::Paragraph(node) => &mut node.children,
            NodeData::Footnote(node) => &mut node.children,
            NodeData::FootnoteReference(node) => &mut node.children,
            NodeData::SubstitutionDefinition(node) => &mut node.children,
            NodeData::SubstitutionReference(node) => &mut node.children,
            NodeData::Root(node) => &mut node.children,
            NodeData::Heading(node) => &mut node.children,
            NodeData::DefinitionListItem(node) => &mut node.children,
            NodeData::DefinitionList(node) => &mut node.children,
            NodeData::ListItem(node) => &mut node.children,
            NodeData::List(node) => &mut node.children,
            NodeData::Line(node) => &mut node.children,
            NodeData::LineBlock(node) => &mut node.children,
            NodeData::Directive(node) => &mut node.children,
            NodeData::DirectiveArgument(node) => &mut node.children,
            NodeData::Target(node) => &mut node.children,
            NodeData::TargetIdentifier(node) => &mut node.children,
            NodeData::InlineTarget(_) => &mut [],
            NodeData::Reference(node) => &mut node.children,
            NodeData::NamedReference(_) => &mut [],
            NodeData::Role(node) => &mut node.children,
            NodeData::RefRole(_) => &mut [],
            NodeData::Text(_) => &mut [],
            NodeData::Literal(node) => &mut node.children,
            NodeData::Emphasis(node) => &mut node.children,
            NodeData::Strong(node) => &mut node.children,
            NodeData::Field(node) => &mut node.children,
            NodeData::FieldList(node) => &mut node.children,
            NodeData::Transition(_) => &mut [],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    lang: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    copyable: bool,
    emphasize_lines: Option<Vec<(i32, i32)>>,
    value: String,
    linenos: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    lineno_start: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub fileid: FileId,

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
    children: Vec<Node>, // DefinitionListItem
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItem {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    children: Vec<Node>, // ListItem
    enumtype: ListEnumType,

    #[serde(skip_serializing_if = "Option::is_none")]
    startat: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineBlock {
    children: Vec<Node>, // Line
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Directive {
    children: Vec<Node>,
    domain: String,
    name: String,
    argument: Vec<Node>, // InlineNode

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
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
    entries: Vec<Node>, // TocTreeDirectiveEntry
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectiveArgument {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    children: Vec<Node>,
    domain: String,
    name: String,
    html_id: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<HashMap<String, bson::Bson>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetIdentifier {
    children: Vec<Node>, // InlineNode
    ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineTarget {
    #[serde(flatten)]
    target: Target,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    children: Vec<Node>, // InlineNode
    refuri: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fileid: Option<(String, String)>,

    #[serde(skip_serializing_if = "Option::is_none")]
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
    children: Vec<Node>, // Field
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transition {}

#[derive(Debug, Serialize, Deserialize)]
pub struct StaticAssetReference {
    checksum: String,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Facet {
    category: String,
    value: String,
    sub_facets: Option<Vec<Facet>>,
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub page_id: String,
    filename: String,
    pub ast: Node,
    source: String,
    static_assets: Vec<StaticAssetReference>,
    facets: Option<Vec<Facet>>,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::Seek;

    use super::*;

    fn normalize_bson(value: &mut bson::Bson) {
        if let bson::Bson::Document(map) = value {
            let mut sorted_map: bson::Document = bson::Document::new();

            // Collect keys, sort them, and insert into the sorted map.
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                if let Some(mut val) = map.remove(&key) {
                    normalize_bson(&mut val); // Recursively normalize nested objects.
                    sorted_map.insert(key, val);
                }
            }
            *map = sorted_map;
        } else if let bson::Bson::Array(array) = value {
            for item in array.iter_mut() {
                normalize_bson(item); // Recursively normalize array elements.
            }
        }
    }

    /// Ensure that deserializing an example document into a Snooty Document, then deserializing the same
    /// document into a raw Bson tree, results in the same data. This requires normalizing object key
    /// order and sprinkling some annoying #[serde(skip_serializing_if)] attributes around to make sure
    /// our output is identical.
    #[test]
    fn round_trip_identical() {
        let f = std::fs::File::open("test_data/supported-operations.bson").unwrap();
        let mut reader = std::io::BufReader::new(f);
        let mut doc_raw: bson::Bson = bson::from_reader(&mut reader).unwrap();
        normalize_bson(&mut doc_raw);
        reader.seek(std::io::SeekFrom::Start(0)).unwrap();

        let doc1: Document = bson::from_reader(reader).unwrap();
        let mut b = bson::to_bson(&doc1).unwrap();
        normalize_bson(&mut b);

        assert_eq!(doc_raw, b);
    }
}
