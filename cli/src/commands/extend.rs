use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use rayon::prelude::*;
use slimg_core::{ExtendMode, FillColor, PipelineOptions, convert, decode_file, output_path};

use super::{
    ErrorCollector, FormatArg, collect_files, configure_thread_pool, make_progress_bar,
    parse_size, safe_write,
};

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("mode").required(true).args(["aspect", "size"]))]
pub struct ExtendArgs {
    /// Input file or directory
    pub input: PathBuf,

    /// Aspect ratio: width:height (e.g. 1:1, 16:9)
    #[arg(long, value_parser = super::crop::parse_aspect, conflicts_with = "size")]
    pub aspect: Option<(u32, u32)>,

    /// Target size: WIDTHxHEIGHT (e.g. 1920x1080)
    #[arg(long, value_parser = parse_size, conflicts_with = "aspect")]
    pub size: Option<(u32, u32)>,

    /// Fill color as hex (e.g. '#FFFFFF', '000000'). Default: white.
    #[arg(long, conflicts_with = "transparent")]
    pub color: Option<String>,

    /// Use transparent background (for formats with alpha support)
    #[arg(long, conflicts_with = "color")]
    pub transparent: bool,

    /// Output format (defaults to input format)
    #[arg(short, long)]
    pub format: Option<FormatArg>,

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

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,
}

fn parse_hex_color(s: &str) -> anyhow::Result<[u8; 4]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 && s.len() != 8 {
        anyhow::bail!("expected 6 or 8 hex digits (e.g. 'FF0000' or 'FF0000FF')");
    }
    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;
    let a = if s.len() == 8 {
        u8::from_str_radix(&s[6..8], 16)?
    } else {
        255
    };
    Ok([r, g, b, a])
}

fn build_extend_mode(args: &ExtendArgs) -> anyhow::Result<ExtendMode> {
    match (args.aspect, args.size) {
        (Some((w, h)), None) => Ok(ExtendMode::AspectRatio { width: w, height: h }),
        (None, Some((w, h))) => Ok(ExtendMode::Size { width: w, height: h }),
        _ => anyhow::bail!("specify exactly one of --aspect or --size"),
    }
}

fn build_fill_color(args: &ExtendArgs, format: slimg_core::Format) -> anyhow::Result<FillColor> {
    if args.transparent {
        if format == slimg_core::Format::Jpeg {
            eprintln!(
                "warning: JPEG does not support transparency, using white background"
            );
            return Ok(FillColor::Solid([255, 255, 255, 255]));
        }
        return Ok(FillColor::Transparent);
    }

    match &args.color {
        Some(hex) => Ok(FillColor::Solid(parse_hex_color(hex)?)),
        None => Ok(FillColor::Solid([255, 255, 255, 255])),
    }
}

pub fn run(args: ExtendArgs) -> anyhow::Result<()> {
    let extend_mode = build_extend_mode(&args)?;
    let files = collect_files(&args.input, args.recursive)?;

    if files.is_empty() {
        anyhow::bail!("no image files found in {}", args.input.display());
    }

    configure_thread_pool(args.jobs)?;

    let pb = make_progress_bar(files.len());
    let errors = ErrorCollector::new();

    files.par_iter().for_each(|file| {
        let result: anyhow::Result<()> = (|| {
            let original_size = std::fs::metadata(file)?.len();
            let (image, src_format) =
                decode_file(file).with_context(|| format!("{}", file.display()))?;

            let target_format = args.format.map(|f| f.into_format()).unwrap_or(src_format);

            if !target_format.can_encode() {
                anyhow::bail!("cannot encode to {} format", target_format.extension());
            }

            let fill = build_fill_color(&args, target_format)?;

            let options = PipelineOptions {
                format: target_format,
                quality: args.quality,
                resize: None,
                crop: None,
                extend: Some(extend_mode.clone()),
                fill_color: Some(fill),
            };

            let result =
                convert(&image, &options).with_context(|| format!("{}", file.display()))?;

            let out = output_path(file, target_format, args.output.as_deref());
            safe_write(&out, &result.data, args.overwrite)?;

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
        anyhow::bail!("{fail_count} file(s) failed to extend");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_6_digits() {
        assert_eq!(parse_hex_color("FF0000").unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn parse_hex_color_with_hash() {
        assert_eq!(parse_hex_color("#00FF00").unwrap(), [0, 255, 0, 255]);
    }

    #[test]
    fn parse_hex_color_8_digits() {
        assert_eq!(parse_hex_color("FF000080").unwrap(), [255, 0, 0, 128]);
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert!(parse_hex_color("xyz").is_err());
    }
}
