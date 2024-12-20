use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use bundle::BundleElementData;
use clap::Parser;

mod bundle;
mod nodes;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Bundles to operate on
    bundles: Vec<PathBuf>,

    /// The path to which to save the stitched bundle
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,
}

fn splice(
    bundles: &mut [bundle::Bundle],
    site_metadata: &bundle::SiteMetadata,
    mut out_bundle: zip::ZipWriter<BufWriter<File>>,
) -> Result<()> {
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

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
                    if let BundleElementData::Asset(asset) = &element.data {
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

                        out_bundle.start_file(format!("assets/{asset_hash_string}"), options)?;
                        out_bundle.write_all(asset)?;
                        continue;
                    }

                    let full_path = element.get_full_bundle_path();
                    let full_path_string = full_path.to_str().unwrap_or_else(|| {
                        panic!("Failed to convert entry name to string: {:?}", full_path)
                    });
                    out_bundle.start_file(full_path_string, options).unwrap();

                    match element.data {
                        BundleElementData::Document(document) => {
                            let serialized = bson::to_vec(&document)?;
                            out_bundle.write_all(&serialized)?;
                        }
                        BundleElementData::Diagnostics(diagnostics) => {
                            let serialized = bson::to_vec(&bundle::Diagnostics { diagnostics })?;
                            out_bundle.write_all(&serialized)?;
                        }
                        BundleElementData::Asset(_) => (), // Already written
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
        for bundle in bundles {
            scope.execute(|| {
                let bundle_ns = PathBuf::from(bundle.metadata.get_namespace());
                for entry in bundle {
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

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let output_file = File::create(cli.output)?;
    let output_writer = BufWriter::new(output_file);
    let output_archive = zip::ZipWriter::new(output_writer);

    let mut bundles = vec![];
    for path in cli.bundles {
        let bundle = bundle::Bundle::open(&path)?;
        bundles.push(bundle);
    }

    let site_metadata = bundle::SiteMetadata::new("mongodb", "main");
    splice(&mut bundles, &site_metadata, output_archive)?;

    Ok(())
}
