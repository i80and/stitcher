use std::{collections::HashMap, path::PathBuf};

use crate::analyzer::{self, FileIdStack};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref PAT_FILE_EXTENSIONS: regex::Regex =
        regex::Regex::new(r###"\.((txt)|(rst)|(yaml)|(ast))$"###).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceInfo {
    line: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    start: SourceInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl FileId {
    pub fn without_known_suffix(&self) -> String {
        let new_filename = PAT_FILE_EXTENSIONS.replace_all(
            self.path
                .file_name()
                .expect("Filename required to remove suffix")
                .to_str()
                .expect("Filename must be expressible as a string to remove suffix"),
            "",
        );
        let fileid: FileId = self.path.with_file_name(new_filename.as_ref()).into();
        fileid.as_posix()
    }

    pub fn as_posix(&self) -> String {
        self.path
            .components()
            .map(|part| match part {
                std::path::Component::Prefix(_) => {
                    panic!("prefix component cannot be part of FileIds")
                }
                std::path::Component::RootDir => "",
                std::path::Component::CurDir => ".",
                std::path::Component::ParentDir => "..",
                std::path::Component::Normal(part) => part.to_str().unwrap(),
            })
            .collect::<Vec<&str>>()
            .join("/")
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    #[serde(flatten)]
    pub data: NodeData,

    position: Position,
}

impl Node {
    pub fn for_each(&mut self, f: &mut impl FnMut(&mut Node)) {
        let mut analyzer = analyzer::SimpleAnalyzer::new(f);
        self.run_analyzer(&mut analyzer);
    }

    pub fn run_analyzer(&mut self, analyzer: &mut impl analyzer::Analyzer) {
        self.run_analyzer_inner(&mut analyzer::FileIdStack::new(), analyzer)
    }

    fn run_analyzer_inner(
        &mut self,
        fileid_stack: &mut FileIdStack,
        analyzer: &mut impl analyzer::Analyzer,
    ) {
        let need_to_pop = if let NodeData::Root(root_node) = &self.data {
            fileid_stack.push(&root_node.fileid);
            true
        } else {
            false
        };

        analyzer.enter_node(fileid_stack, self);

        for child in self.data.get_children() {
            child.run_analyzer_inner(fileid_stack, analyzer);
        }

        analyzer.exit_node(fileid_stack, self);

        if need_to_pop {
            fileid_stack.pop();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            NodeData::Comment(n) => &mut n.children,
            NodeData::Label(n) => &mut n.children,
            NodeData::Section(n) => &mut n.children,
            NodeData::Paragraph(n) => &mut n.children,
            NodeData::Footnote(n) => &mut n.children,
            NodeData::FootnoteReference(n) => &mut n.children,
            NodeData::SubstitutionDefinition(n) => &mut n.children,
            NodeData::SubstitutionReference(n) => &mut n.children,
            NodeData::Root(n) => &mut n.children,
            NodeData::Heading(n) => &mut n.children,
            NodeData::DefinitionListItem(n) => &mut n.children,
            NodeData::DefinitionList(n) => &mut n.children,
            NodeData::ListItem(n) => &mut n.children,
            NodeData::List(n) => &mut n.children,
            NodeData::Line(n) => &mut n.children,
            NodeData::LineBlock(n) => &mut n.children,
            NodeData::Directive(n) => &mut n.children,
            NodeData::DirectiveArgument(n) => &mut n.children,
            NodeData::Target(n) => &mut n.children,
            NodeData::TargetIdentifier(n) => &mut n.children,
            NodeData::InlineTarget(_) => &mut [],
            NodeData::Reference(n) => &mut n.children,
            NodeData::NamedReference(_) => &mut [],
            NodeData::Role(n) => &mut n.children,
            NodeData::RefRole(_) => &mut [],
            NodeData::Text(_) => &mut [],
            NodeData::Literal(n) => &mut n.children,
            NodeData::Emphasis(n) => &mut n.children,
            NodeData::Strong(n) => &mut n.children,
            NodeData::Field(n) => &mut n.children,
            NodeData::FieldList(n) => &mut n.children,
            NodeData::Transition(_) => &mut [],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Label {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paragraph {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Footnote {
    children: Vec<Node>,
    id: String,
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FootnoteReference {
    children: Vec<Node>, // InlineNode
    id: String,
    refname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubstitutionDefinition {
    children: Vec<Node>, // InlineNode
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubstitutionReference {
    children: Vec<Node>, // InlineNode
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockSubstitutionReference {
    children: Vec<Node>,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Root {
    children: Vec<Node>,
    pub fileid: FileId,

    #[serde(default)]
    options: HashMap<String, bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Heading {
    children: Vec<Node>, // InlineNode
    id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefinitionListItem {
    children: Vec<Node>,
    term: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefinitionList {
    children: Vec<Node>, // DefinitionListItem
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListItem {
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List {
    children: Vec<Node>, // ListItem
    enumtype: ListEnumType,

    #[serde(skip_serializing_if = "Option::is_none")]
    startat: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Line {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineBlock {
    children: Vec<Node>, // Line
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Directive {
    children: Vec<Node>,
    domain: String,
    name: String,
    argument: Vec<Node>, // InlineNode

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    options: HashMap<String, bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TocTreeDirectiveEntry {
    title: Option<String>,
    url: Option<String>,
    slug: Option<String>,
    ref_project: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TocTreeDirective {
    directive: Directive,
    entries: Vec<Node>, // TocTreeDirectiveEntry
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectiveArgument {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Target {
    pub children: Vec<Node>,
    pub domain: String,
    pub name: String,
    pub html_id: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<HashMap<String, bson::Bson>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetIdentifier {
    pub children: Vec<Node>, // InlineNode
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineTarget {
    #[serde(flatten)]
    target: Target,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reference {
    children: Vec<Node>, // InlineNode
    refuri: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    refname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamedReference {
    refname: String,
    refuri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
    children: Vec<Node>, // InlineNode
    domain: String,
    name: String,
    target: String,
    flag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefRole {
    #[serde(flatten)]
    role: Role,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fileid: Option<(String, String)>,

    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Text {
    value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Literal {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Emphasis {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Strong {
    children: Vec<Node>, // InlineNode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    children: Vec<Node>,
    name: String,
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldList {
    children: Vec<Node>, // Field
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transition {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticAssetReference {
    checksum: String,
    key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Facet {
    category: String,
    value: String,
    sub_facets: Option<Vec<Facet>>,
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub page_id: String,
    pub filename: FileId,
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

    #[test]
    fn test_without_known_suffix() {
        let fileid: FileId = PathBuf::from("foo/bar.txt").into();
        assert_eq!(fileid.without_known_suffix(), "foo/bar");

        let fileid: FileId = PathBuf::from("foo/bar.png").into();
        assert_eq!(fileid.without_known_suffix(), "foo/bar.png")
    }

    #[test]
    fn test_as_posix() {
        let fileid: FileId = PathBuf::from("foo/bar").into();
        assert_eq!(fileid.as_posix(), "foo/bar");

        let fileid: FileId = PathBuf::from("/foo/bar").into();
        assert_eq!(fileid.as_posix(), "/foo/bar");

        let fileid: FileId = PathBuf::from("/foo/../bar").into();
        assert_eq!(fileid.as_posix(), "/foo/../bar");

        let fileid: FileId = PathBuf::from("/foo/./bar").into();
        assert_eq!(fileid.as_posix(), "/foo/bar");
    }
}
