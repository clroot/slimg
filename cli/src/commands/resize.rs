use std::path::PathBuf;

use clap::Args;
use slimg_core::{PipelineOptions, ResizeMode, convert, decode_file, output_path};

use super::FormatArg;

#[derive(Debug, Args)]
pub struct ResizeArgs {
    /// Input file
    pub input: PathBuf,

    /// Target width in pixels
    #[arg(long)]
    pub width: Option<u32>,

    /// Target height in pixels
    #[arg(long)]
    pub height: Option<u32>,

    /// Scale factor (e.g. 0.5 for half size)
    #[arg(long)]
    pub scale: Option<f64>,

    /// Output format (defaults to input format)
    #[arg(short, long)]
    pub format: Option<FormatArg>,

    /// Encoding quality (0-100)
    #[arg(short, long, default_value_t = 80)]
    pub quality: u8,

    /// Output path
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub fn run(args: ResizeArgs) -> anyhow::Result<()> {
    let resize_mode = match (args.width, args.height, args.scale) {
        (Some(w), Some(h), None) => ResizeMode::Fit(w, h),
        (Some(w), None, None) => ResizeMode::Width(w),
        (None, Some(h), None) => ResizeMode::Height(h),
        (None, None, Some(s)) => ResizeMode::Scale(s),
        (None, None, None) => {
            anyhow::bail!("specify at least one of --width, --height, or --scale");
        }
        _ => {
            anyhow::bail!("--scale cannot be combined with --width or --height");
        }
    };

    let original_size = std::fs::metadata(&args.input)?.len();
    let (image, src_format) = decode_file(&args.input)?;

    let target_format = args.format.map(|f| f.into_format()).unwrap_or(src_format);

    if !target_format.can_encode() {
        anyhow::bail!("cannot encode to {} format", target_format.extension());
    }

    let options = PipelineOptions {
        format: target_format,
        quality: args.quality,
        resize: Some(resize_mode),
        crop: None,
    };

    let result = convert(&image, &options)?;

    let out = output_path(&args.input, target_format, args.output.as_deref());
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
        args.input.display(),
        out.display(),
        original_size,
        new_size,
        ratio,
    );

    Ok(())
}
