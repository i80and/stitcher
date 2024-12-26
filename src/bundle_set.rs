use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::analyzer::{self, Analyzer};
use crate::bundle;
use crate::target_database;

pub struct BundleSet {
    pub bundles: Vec<Mutex<bundle::Bundle>>,
}

impl BundleSet {
    pub fn new(bundles: impl Iterator<Item = bundle::Bundle>) -> Self {
        Self {
            bundles: bundles.map(Mutex::new).collect(),
        }
    }
    pub fn splice(
        &self,
        site_metadata: &bundle::SiteMetadata,
        mut out_bundle: zip::ZipWriter<BufWriter<File>>,
    ) -> anyhow::Result<()> {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        out_bundle.start_file("site.bson", options)?;
        out_bundle.write_all(&bson::to_vec(&site_metadata)?)?;

        // Avoid writing any asset more than once, so store the unique hash of each and skip dups
        let stored_assets: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        let (tx, rx) = crossbeam_channel::bounded::<Option<bundle::BundleElement>>(10);

        let n_cpus = std::thread::available_parallelism()?.get();
        assert!(n_cpus >= 1_usize);
        log::debug!("Splicing with {} threads", n_cpus);
        let mut pool = scoped_threadpool::Pool::new(u32::try_from(n_cpus)?);

        let thread = std::thread::spawn(move || -> anyhow::Result<()> {
            loop {
                let packet = rx.recv().unwrap();

                match packet {
                    Some(element) => {
                        // If this asset has already been stored, skip it
                        if let bundle::BundleElementData::Asset(asset) = &element.data {
                            let asset_hash = element.name.file_name().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Bundle element is missing a filename: ${:?}",
                                    element.name
                                )
                            })?;
                            let asset_hash_string = asset_hash.to_str().unwrap();
                            let mut guard = stored_assets.lock().unwrap();
                            if !guard.insert(asset_hash_string.to_owned()) {
                                // This asset was already stored
                                continue;
                            }

                            out_bundle
                                .start_file(format!("assets/{asset_hash_string}"), options)?;
                            out_bundle.write_all(asset)?;
                            continue;
                        }

                        let full_path = element.get_full_bundle_path();
                        let full_path_string = full_path.to_str().unwrap_or_else(|| {
                            panic!("Failed to convert entry name to string: {:?}", full_path)
                        });
                        out_bundle.start_file(full_path_string, options).unwrap();

                        match element.data {
                            bundle::BundleElementData::Document(document) => {
                                let serialized = bson::to_vec(&document)?;
                                out_bundle.write_all(&serialized)?;
                            }
                            bundle::BundleElementData::Diagnostics(diagnostics) => {
                                let serialized =
                                    bson::to_vec(&bundle::Diagnostics { diagnostics })?;
                                out_bundle.write_all(&serialized)?;
                            }
                            bundle::BundleElementData::Asset(_) => (), // Already written
                        }
                    }
                    None => {
                        out_bundle.finish().unwrap();
                        return Ok(());
                    }
                }
            }
        });

        pool.scoped(|scope| {
            // Chunk our input into a thread pool at bundle granularity
            for bundle in &self.bundles {
                scope.execute(|| {
                    let mut bundle = bundle.lock().unwrap();
                    let bundle_ns = PathBuf::from(bundle.metadata.get_namespace());
                    for entry in bundle.into_iter() {
                        let mut entry = entry.unwrap();
                        entry.migrate(&bundle_ns);
                        tx.send(Some(entry)).unwrap();
                    }
                });
            }
        });

        tx.send(None)?;
        thread.join().unwrap()?;

        Ok(())
    }

    pub fn link(&mut self) -> anyhow::Result<()> {
        let n_cpus = std::thread::available_parallelism()?.get();
        let mut pool = scoped_threadpool::Pool::new(u32::try_from(n_cpus)?);

        let db = Mutex::new(target_database::TargetDatabase::new());

        pool.scoped(|scope| {
            for bundle in &self.bundles {
                scope.execute(|| {
                    let mut target_analyzer = analyzer::TargetPass1::new(&db);
                    let mut bundle = bundle.lock().unwrap();
                    for entry in bundle.into_iter() {
                        let entry = entry.unwrap();
                        if let bundle::BundleElementData::Document(mut doc) = entry.data {
                            target_analyzer.enter_page(&doc);
                            doc.ast.for_each(&mut target_analyzer);
                            target_analyzer.exit_page(&doc);
                        }
                    }
                });
            }
        });
        Ok(())
    }
}
