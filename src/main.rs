use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod bundle;

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
    mut out_bundle: zip::ZipWriter<BufWriter<File>>,
) -> Result<()> {
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let (tx, rx) = crossbeam_channel::bounded(10);

    let n_cpus = std::thread::available_parallelism()?.get();
    assert!(n_cpus >= 1_usize);
    log::debug!("Splicing with {} threads", n_cpus);
    let mut pool = scoped_threadpool::Pool::new(u32::try_from(n_cpus)?);

    std::thread::spawn(move || loop {
        let packet = rx.recv().unwrap();

        match packet {
            Some(name) => out_bundle.start_file(name, options).unwrap(),
            None => {
                out_bundle.finish().unwrap();
                return;
            }
        }
    });

    pool.scoped(|scope| {
        // Chunk our input into a thread pool at bundle granularity
        for bundle in bundles {
            scope.execute(|| {
                let bundle_ns = bundle.metadata.get_namespace();
                for entry in bundle {
                    let entry = entry.unwrap();
                    let name = entry
                        .name
                        .to_str()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Failed to convert entry name to string: {:?}",
                                entry.name
                            )
                        })
                        .unwrap();
                    let name = format!("{}/{}", bundle_ns, name);
                    tx.send(Some(name)).unwrap();
                }
            });
        }
    });

    tx.send(None).unwrap();

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

    splice(&mut bundles, output_archive)?;

    Ok(())
}
