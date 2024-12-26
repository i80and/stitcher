use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::nodes;
use crate::target_database;

use lazy_static::lazy_static;

lazy_static! {
    static ref PAT_INVALID_ID_CHARACTERS: regex::Regex =
        regex::Regex::new(r###"[^\w_\.\-]"###).unwrap();
}

/// Turn an ID into a valid HTML5 element ID.
fn make_html5_id(orig: &str) -> Cow<'_, str> {
    let clean_id = PAT_INVALID_ID_CHARACTERS.replace_all(orig, "-");
    if clean_id.is_empty() {
        return Cow::from("unnamed");
    }

    clean_id
}

pub trait Analyzer {
    fn enter_page(&mut self, _page: &nodes::Document) {}
    fn exit_page(&mut self, _page: &nodes::Document) {}

    fn enter_node(&mut self, _node: &mut nodes::Node) {}
    fn exit_node(&mut self, _node: &mut nodes::Node) {}
}

pub struct SimpleAnalyzer<'a> {
    f: &'a mut dyn FnMut(&mut nodes::Node),
}

impl<'a> SimpleAnalyzer<'a> {
    pub fn new(f: &'a mut dyn FnMut(&mut nodes::Node)) -> Self {
        Self { f }
    }
}

impl<'a> Analyzer for SimpleAnalyzer<'a> {
    fn enter_node(&mut self, node: &mut nodes::Node) {
        (self.f)(node);
    }
}

pub struct TargetPass1<'a> {
    db: &'a Mutex<target_database::TargetDatabase>,
    target_counter: HashMap<String, u32>,
    page: Option<nodes::FileId>,
}

impl<'a> TargetPass1<'a> {
    pub fn new(db: &'a Mutex<target_database::TargetDatabase>) -> Self {
        Self {
            db,
            target_counter: HashMap::new(),
            page: None,
        }
    }
}

impl<'a> Analyzer for TargetPass1<'a> {
    fn enter_page(&mut self, page: &nodes::Document) {
        self.page = Some(page.filename.to_owned());
    }

    fn enter_node(&mut self, node: &mut nodes::Node) {
        if let nodes::NodeData::Target(ref mut target) = node.data {
            // Frankly, this is silly. We just pick the longest identifier. This is arbitrary,
            // and we can consider this behavior implementation-defined to be changed later if needed.
            // It just needs to be something consistent.
            let identifiers: Vec<&nodes::TargetIdentifier> = target
                .children
                .iter()
                .filter_map(|child: &nodes::Node| {
                    if let nodes::NodeData::TargetIdentifier(ref target_identifier) = child.data {
                        Some(target_identifier)
                    } else {
                        None
                    }
                })
                .collect();

            let candidates: Vec<&String> = identifiers
                .iter()
                .filter_map(|node| node.ids.iter().max_by_key(|id| id.len()))
                .collect();

            let chosen_id =
                if let Some(id) = candidates.iter().max_by_key(|candidate| candidate.len()) {
                    id
                } else {
                    return;
                };
            let mut chosen_html_id = format!(
                "{}-{}-{}",
                &target.domain,
                target.name,
                make_html5_id(chosen_id)
            );

            // Disambiguate duplicate IDs, should they occur.
            let counter = *self
                .target_counter
                .entry(chosen_html_id.to_owned())
                .or_insert(0);
            if counter > 0 {
                chosen_html_id += &format!("-{}", counter);
            }
            self.target_counter
                .entry(chosen_html_id.to_owned())
                .and_modify(|c| *c += 1);
            target.html_id = Some(chosen_html_id.to_owned());

            let mut db = self.db.lock().unwrap();
            for target_identifier in identifiers {
                let title = if target_identifier.children.is_empty() {
                    vec![]
                } else {
                    target_identifier.children.to_owned()
                };

                let target_ids: Vec<&str> =
                    target_identifier.ids.iter().map(|x| x.as_ref()).collect();
                db.define_local_target(
                    &target.domain,
                    &target.name,
                    &target_ids,
                    self.page.as_ref().unwrap(),
                    &title,
                    &chosen_html_id,
                );
            }
        }
    }
}
