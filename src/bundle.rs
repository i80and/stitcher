use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::nodes;

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteMetadata {
    project: String,
    branch: String,
}

impl SiteMetadata {
    pub fn new(project: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            branch: branch.into(),
        }
    }

    pub fn get_namespace(&self) -> String {
        format!("{}/{}", self.project, self.branch)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    severity: String,
    start: i32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

pub struct BundleIntoIterator<'a> {
    bundle: &'a mut Bundle,
    index: usize,
}

pub struct BundleElement {
    pub name: PathBuf,
    pub data: BundleElementData,
}

impl BundleElement {
    pub fn new(name: PathBuf, data: BundleElementData) -> Self {
        Self { name, data }
    }

    pub fn get_full_bundle_path(&self) -> PathBuf {
        self.data.get_path_component().join(&self.name)
    }

    pub fn migrate(&mut self, namespace: &Path) {
        self.name = namespace.join(&self.name);
        if let BundleElementData::Document(document) = &self.data {}
    }
}

pub enum BundleElementData {
    Document(nodes::Document),
    Asset(Vec<u8>),
    Diagnostics(Vec<Diagnostic>),
}

impl BundleElementData {
    pub fn get_path_component(&self) -> &'static Path {
        Path::new(match &self {
            BundleElementData::Document(_) => "documents",
            BundleElementData::Asset(_) => "assets",
            BundleElementData::Diagnostics(_) => "diagnostics",
        })
    }
}

pub struct Bundle {
    pub metadata: SiteMetadata,
    archive: zip::ZipArchive<BufReader<File>>,
}

impl<'a> IntoIterator for &'a mut Bundle {
    type Item = anyhow::Result<BundleElement>;
    type IntoIter = BundleIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BundleIntoIterator {
            bundle: self,
            index: 0,
        }
    }
}

impl Bundle {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        let metadata = bson::from_reader(archive.by_name("site.bson")?)?;

        Ok(Bundle { metadata, archive })
    }
}

impl<'a> Iterator for BundleIntoIterator<'a> {
    type Item = anyhow::Result<BundleElement>;

    fn next(&mut self) -> Option<anyhow::Result<BundleElement>> {
        loop {
            let idx = self.index;

            if idx >= self.bundle.archive.len() {
                return None;
            }

            self.index += 1;

            let mut file = self.bundle.archive.by_index(idx).unwrap();
            let filename = match file.enclosed_name() {
                Some(path) => path,
                None => {
                    log::warn!("Bundle entry {} has a prohibited path", file.name());
                    continue;
                }
            };

            if !file.is_file() {
                continue;
            }

            // Split our filename into the prefix and the remainder; e.g. "documents" and "foo/bar.bson"
            let mut components_iter = filename.components();
            let first_component = components_iter.next().unwrap();
            let filename_prefix: &Path = first_component.as_ref();
            let filename_without_prefix: PathBuf = components_iter.collect();

            if filename_prefix == Path::new("documents") {
                return Some(
                    bson::from_reader(file)
                        .with_context(|| {
                            format!("Error deserializing document BSON: {}", filename.display())
                        })
                        .map(|value| {
                            BundleElement::new(
                                filename_without_prefix,
                                BundleElementData::Document(value),
                            )
                        }),
                );
            } else if filename_prefix == Path::new("assets") {
                let mut buf: Vec<u8> = vec![];
                if let Err(err) = file
                    .read_to_end(&mut buf)
                    .with_context(|| format!("Error reading asset: {}", filename.display()))
                {
                    return Some(Err(err));
                }

                return Some(Ok(BundleElement::new(
                    filename_without_prefix,
                    BundleElementData::Asset(buf),
                )));
            } else if filename_prefix == Path::new("diagnostics") {
                return Some(
                    bson::from_reader(file)
                        .with_context(|| {
                            format!(
                                "Error deserializing diagnostic BSON: {}",
                                filename.display()
                            )
                        })
                        .map(|value: Diagnostics| {
                            BundleElement::new(
                                filename_without_prefix,
                                BundleElementData::Diagnostics(value.diagnostics),
                            )
                        }),
                );
            } else if filename == Path::new("site.bson") {
                continue;
            } else {
                log::warn!("Unexpected bundle entry: {}", filename.display());
            }
        }
    }
}
