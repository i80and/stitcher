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
    pub fn get_namespace(&self) -> String {
        format!("{}/{}", self.project, self.branch)
    }
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
}

pub enum BundleElementData {
    Document(nodes::Document),
    Asset(Vec<u8>),
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
                    eprintln!("Bundle entry {} has a prohibited path", file.name());
                    continue;
                }
            };

            if !file.is_file() {
                continue;
            }

            if filename.starts_with("documents") {
                return Some(
                    bson::from_reader(file)
                        .with_context(|| {
                            format!("Error deserializing BSON: {}", filename.display())
                        })
                        .map(|value| {
                            BundleElement::new(filename, BundleElementData::Document(value))
                        }),
                );
            } else if filename.starts_with("assets") {
                let mut buf: Vec<u8> = vec![];
                if let Err(err) = file
                    .read_to_end(&mut buf)
                    .with_context(|| format!("Error reading asset: {}", filename.display()))
                {
                    return Some(Err(err));
                }

                return Some(Ok(BundleElement::new(
                    filename,
                    BundleElementData::Asset(buf),
                )));
            } else if filename == Path::new("site.bson") || filename.starts_with("diagnostics") {
                continue;
            } else {
                eprintln!("Unexpected bundle entry: {}", filename.display());
            }
        }
    }
}
