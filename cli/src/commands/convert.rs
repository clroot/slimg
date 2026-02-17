use std::path::PathBuf;

use clap::Args;
use slimg_core::{PipelineOptions, convert, decode_file, output_path};

use super::FormatArg;
use super::collect_files;

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
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    let target_format = args.format.into_format();
    let files = collect_files(&args.input, args.recursive)?;

    if files.is_empty() {
        anyhow::bail!("no image files found in {}", args.input.display());
    }

    let options = PipelineOptions {
        format: target_format,
        quality: args.quality,
        resize: None,
    };

    for file in &files {
        let original_size = std::fs::metadata(file)?.len();
        let (image, _src_format) = decode_file(file)?;
        let result = convert(&image, &options)?;

        let out = output_path(file, target_format, args.output.as_deref());
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        result.save(&out)?;

        let new_size = result.data.len() as u64;
        let ratio = if original_size > 0 {
            (new_size as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        eprintln!(
            "{} -> {} ({} -> {} bytes, {:.1}%)",
            file.display(),
            out.display(),
            original_size,
            new_size,
            ratio,
        );
    }

    Ok(())
}
