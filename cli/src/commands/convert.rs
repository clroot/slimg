use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use rayon::prelude::*;
use slimg_core::{PipelineOptions, convert, decode_file, output_path};

use super::{
    ErrorCollector, FormatArg, collect_files, configure_thread_pool, make_progress_bar, safe_write,
};

#[derive(Debug, Args)]
pub struct ConvertArgs {
    /// Input file or directory
    pub input: PathBuf,

    /// Output format
    #[arg(short, long)]
    pub format: FormatArg,

    /// Encoding quality (0-100)
    #[arg(short, long, default_value_t = 80)]
    pub quality: u8,

    /// Output path (file or directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Process subdirectories recursively
    #[arg(long)]
    pub recursive: bool,

    /// Number of parallel jobs (defaults to CPU count)
    #[arg(short, long)]
    pub jobs: Option<usize>,
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    let target_format = args.format.into_format();
    let files = collect_files(&args.input, args.recursive)?;

    if files.is_empty() {
        anyhow::bail!("no image files found in {}", args.input.display());
    }

    configure_thread_pool(args.jobs)?;

    let options = PipelineOptions {
        format: target_format,
        quality: args.quality,
        resize: None,
        crop: None,
    };

    let pb = make_progress_bar(files.len());
    let errors = ErrorCollector::new();

    files.par_iter().for_each(|file| {
        let result: anyhow::Result<()> = (|| {
            let original_size = std::fs::metadata(file)?.len();
            let (image, _src_format) =
                decode_file(file).with_context(|| format!("{}", file.display()))?;
            let result =
                convert(&image, &options).with_context(|| format!("{}", file.display()))?;

            let out = output_path(file, target_format, args.output.as_deref());
            safe_write(&out, &result.data, false)?;

            let new_size = result.data.len() as u64;
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
        anyhow::bail!("{fail_count} file(s) failed to convert");
    }

    Ok(())
}
