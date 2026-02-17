use std::path::PathBuf;

use clap::Args;
use slimg_core::{optimize, output_path};

use super::collect_files;

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
}

pub fn run(args: OptimizeArgs) -> anyhow::Result<()> {
    let files = collect_files(&args.input, args.recursive)?;

    if files.is_empty() {
        anyhow::bail!("no image files found in {}", args.input.display());
    }

    for file in &files {
        let original_data = std::fs::read(file)?;
        let original_size = original_data.len() as u64;

        let result = optimize(&original_data, args.quality)?;
        let new_size = result.data.len() as u64;

        let out = if args.overwrite {
            file.clone()
        } else {
            output_path(file, result.format, args.output.as_deref())
        };

        // Only write if new size is smaller, unless overwriting explicitly
        if new_size < original_size || args.overwrite {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            result.save(&out)?;

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
        } else {
            eprintln!(
                "{} -> skipped (optimized size {} >= original {})",
                file.display(),
                new_size,
                original_size,
            );
        }
    }

    Ok(())
}
