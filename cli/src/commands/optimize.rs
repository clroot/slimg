use std::path::PathBuf;

use clap::Args;
use rayon::prelude::*;
use slimg_core::{optimize, output_path};

use super::{ErrorCollector, collect_files, configure_thread_pool, make_progress_bar, safe_write};

#[derive(Debug, Args)]
pub struct OptimizeArgs {
    /// Input file or directory
    pub input: PathBuf,

    /// Encoding quality (0-100)
    #[arg(short, long, default_value_t = 80)]
    pub quality: u8,

    /// Output path (file or directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overwrite original file
    #[arg(long)]
    pub overwrite: bool,

    /// Process subdirectories recursively
    #[arg(long)]
    pub recursive: bool,

    /// Number of parallel jobs (defaults to CPU count)
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

pub fn run(args: OptimizeArgs) -> anyhow::Result<()> {
    let files = collect_files(&args.input, args.recursive)?;

    if files.is_empty() {
        anyhow::bail!("no image files found in {}", args.input.display());
    }

    configure_thread_pool(args.jobs)?;

    let pb = make_progress_bar(files.len());
    let errors = ErrorCollector::new();

    files.par_iter().for_each(|file| {
        let result: anyhow::Result<()> = (|| {
            let original_data = std::fs::read(file)?;
            let original_size = original_data.len() as u64;

            let result = optimize(&original_data, args.quality)?;
            let new_size = result.data.len() as u64;

            let out = if args.overwrite {
                file.clone()
            } else {
                output_path(file, result.format, args.output.as_deref())
            };

            if new_size < original_size || args.overwrite {
                safe_write(&out, &result.data, args.overwrite)?;

                let ratio = if original_size > 0 {
                    (new_size as f64 / original_size as f64) * 100.0
                } else {
                    0.0
                };

                pb.println(format!(
                    "{} -> {} ({} -> {} bytes, {:.1}%)",
                    file.display(),
                    out.display(),
                    original_size,
                    new_size,
                    ratio,
                ));
            } else {
                pb.println(format!(
                    "{} -> skipped (optimized size {} >= original {})",
                    file.display(),
                    new_size,
                    original_size,
                ));
            }

            Ok(())
        })();

        if let Err(e) = result {
            errors.push(file, &e);
        }
        pb.inc(1);
    });

    let fail_count = errors.summarize(&pb);
    pb.finish_and_clear();

    if fail_count > 0 {
        anyhow::bail!("{fail_count} file(s) failed to optimize");
    }

    Ok(())
}
