pub mod convert;
pub mod optimize;
pub mod resize;

use std::path::{Path, PathBuf};

use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use slimg_core::Format;

/// Image format argument for CLI (excludes JXL which cannot be encoded).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Jpeg,
    Png,
    Webp,
    Avif,
    Qoi,
}

impl FormatArg {
    pub fn into_format(self) -> Format {
        match self {
            Self::Jpeg => Format::Jpeg,
            Self::Png => Format::Png,
            Self::Webp => Format::WebP,
            Self::Avif => Format::Avif,
            Self::Qoi => Format::Qoi,
        }
    }
}

/// Known image file extensions that slimg can process.
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif", "jxl", "qoi"];

/// Collect image files from a path.
///
/// - If `path` is a file, returns it as a single-element vec.
/// - If `path` is a directory, lists entries (recursively if `recursive` is true),
///   filters by image extensions, and returns sorted results.
pub(crate) fn collect_files(path: &Path, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    if !path.is_dir() {
        anyhow::bail!("{} is not a file or directory", path.display());
    }

    let mut files = Vec::new();
    collect_dir(path, recursive, &mut files)?;
    files.sort();
    Ok(files)
}

/// Configure rayon's global thread pool.
/// When `jobs` is `None`, rayon defaults to the number of logical CPUs.
pub(crate) fn configure_thread_pool(jobs: Option<usize>) -> anyhow::Result<()> {
    if let Some(n) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| anyhow::anyhow!("failed to configure thread pool: {e}"))?;
    }
    Ok(())
}

/// Create a progress bar for batch processing.
/// Returns a hidden bar when processing a single file.
pub(crate) fn make_progress_bar(total: usize) -> ProgressBar {
    if total <= 1 {
        return ProgressBar::hidden();
    }

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:30}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}

fn collect_dir(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && recursive {
            collect_dir(&path, recursive, out)?;
        } else if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        {
            out.push(path);
        }
    }
    Ok(())
}
