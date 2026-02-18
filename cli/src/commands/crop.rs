use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use rayon::prelude::*;
use slimg_core::{CropMode, PipelineOptions, convert, decode_file, output_path};

use super::{
    ErrorCollector, FormatArg, collect_files, configure_thread_pool, make_progress_bar, safe_write,
};

#[derive(Debug, Args)]
pub struct CropArgs {
    /// Input file or directory
    pub input: PathBuf,

    /// Crop region: x,y,width,height (e.g. 100,50,800,600)
    #[arg(long, value_parser = parse_region, conflicts_with = "aspect")]
    pub region: Option<(u32, u32, u32, u32)>,

    /// Aspect ratio: width:height (e.g. 16:9, 1:1)
    #[arg(long, value_parser = parse_aspect, conflicts_with = "region")]
    pub aspect: Option<(u32, u32)>,

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

fn parse_region(s: &str) -> Result<(u32, u32, u32, u32), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err("expected format: x,y,width,height (e.g. 100,50,800,600)".to_string());
    }
    let nums: Vec<u32> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            p.trim()
                .parse::<u32>()
                .map_err(|_| format!("invalid number at position {}: '{}'", i + 1, p))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((nums[0], nums[1], nums[2], nums[3]))
}

pub(crate) fn parse_aspect(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("expected format: width:height (e.g. 16:9, 1:1)".to_string());
    }
    let w: u32 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("invalid width: '{}'", parts[0]))?;
    let h: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("invalid height: '{}'", parts[1]))?;
    if w == 0 || h == 0 {
        return Err("aspect ratio values must be non-zero".to_string());
    }
    Ok((w, h))
}

fn build_crop_mode(args: &CropArgs) -> anyhow::Result<CropMode> {
    match (args.region, args.aspect) {
        (Some((x, y, w, h)), None) => Ok(CropMode::Region {
            x,
            y,
            width: w,
            height: h,
        }),
        (None, Some((w, h))) => Ok(CropMode::AspectRatio { width: w, height: h }),
        _ => anyhow::bail!("specify exactly one of --region or --aspect"),
    }
}

pub fn run(args: CropArgs) -> anyhow::Result<()> {
    let crop_mode = build_crop_mode(&args)?;
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

            let options = PipelineOptions {
                format: target_format,
                quality: args.quality,
                resize: None,
                crop: Some(crop_mode.clone()),
                extend: None,
                fill_color: None,
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
        anyhow::bail!("{fail_count} file(s) failed to crop");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_region_valid() {
        assert_eq!(parse_region("100,50,800,600"), Ok((100, 50, 800, 600)));
    }

    #[test]
    fn parse_region_with_spaces() {
        assert_eq!(parse_region("100, 50, 800, 600"), Ok((100, 50, 800, 600)));
    }

    #[test]
    fn parse_region_wrong_count() {
        assert!(parse_region("100,50,800").is_err());
    }

    #[test]
    fn parse_region_invalid_number() {
        assert!(parse_region("abc,50,800,600").is_err());
    }

    #[test]
    fn parse_aspect_valid() {
        assert_eq!(parse_aspect("16:9"), Ok((16, 9)));
    }

    #[test]
    fn parse_aspect_square() {
        assert_eq!(parse_aspect("1:1"), Ok((1, 1)));
    }

    #[test]
    fn parse_aspect_wrong_format() {
        assert!(parse_aspect("16-9").is_err());
    }

    #[test]
    fn parse_aspect_zero() {
        assert!(parse_aspect("0:9").is_err());
    }
}
