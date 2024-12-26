use std::{borrow::Cow, collections::HashMap};

use lazy_static::lazy_static;

use crate::nodes;

lazy_static! {
    static ref PAT_TARGET_PART_SEPARATOR: regex::Regex = regex::Regex::new(r###"[_-]+"###).unwrap();
    static ref PAT_WHITESPACE: regex::Regex = regex::Regex::new(r###"\s+"###).unwrap();
}

/// Normalize targets to allow easy matching against the target
/// database: normalize whitespace.
fn normalize_target(target: &str) -> Cow<'_, str> {
    return PAT_WHITESPACE.replace_all(target, " ");
}

struct LocalDefinition {
    canonical_name: String,
    fileid: nodes::FileId,
    title: Vec<nodes::Node>,
    html5_id: String,
}

pub struct InternalResult {
    result: (String, String),
    canonical_name: String,
    title: Vec<nodes::Node>,
}

pub struct TargetDatabase {
    local_definitions: HashMap<String, Vec<LocalDefinition>>,
}

impl TargetDatabase {
    pub fn new() -> Self {
        Self {
            local_definitions: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Vec<InternalResult> {
        let key = normalize_target(key);
        let mut results: Vec<InternalResult> = vec![];
        let matches = if let Some(matches) = self.local_definitions.get(key.as_ref()) {
            matches
        } else {
            return results;
        };

        for def in matches {
            results.push(InternalResult {
                result: (def.fileid.without_known_suffix(), def.html5_id.to_owned()),
                canonical_name: def.canonical_name.to_owned(),
                title: def.title.to_owned(),
            })
        }

        results
    }

    pub fn define_local_target(
        &mut self,
        domain: &str,
        name: &str,
        targets: &[&str],
        pageid: &nodes::FileId,
        title: &[nodes::Node],
        html5_id: &str,
    ) {
        // If multiple target names are given, prefer placing the one with the most periods
        // into referring RefRole nodes. This is an odd heuristic, but should work for now.
        // e.g. if a RefRole links to "-v", we want it to get normalized to "mongod.-v" if that's
        // what gets resolved.
        let canonical_name = targets
            .iter()
            .max_by_key(|x| x.matches('.').count())
            .unwrap()
            .to_string();

        for target in targets {
            let target = normalize_target(target);
            let key = format!("{domain}:{name}:{target}");
            self.local_definitions
                .entry(key)
                .or_default()
                .push(LocalDefinition {
                    canonical_name: canonical_name.to_owned(),
                    fileid: pageid.to_owned(),
                    title: title.to_owned(),
                    html5_id: html5_id.to_owned(),
                })
        }
    }
}
