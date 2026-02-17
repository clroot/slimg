use std::path::Path;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Avif,
    Jxl,
    Qoi,
}

impl Format {
    /// Detect format from a file extension (case-insensitive).
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::WebP),
            "avif" => Some(Self::Avif),
            "jxl" => Some(Self::Jxl),
            "qoi" => Some(Self::Qoi),
            _ => None,
        }
    }

    /// Detect format from magic bytes at the start of the file data.
    pub fn from_magic_bytes(data: &[u8]) -> Option<Self> {
        if data.len() >= 3 && data[..3] == [0xFF, 0xD8, 0xFF] {
            return Some(Self::Jpeg);
        }
        if data.len() >= 4 && data[..4] == [0x89, 0x50, 0x4E, 0x47] {
            return Some(Self::Png);
        }
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return Some(Self::WebP);
        }
        if data.len() >= 12 && &data[4..8] == b"ftyp" {
            let brand = &data[8..12];
            if brand.starts_with(b"avif") || brand.starts_with(b"avis") {
                return Some(Self::Avif);
            }
        }
        // JXL: bare codestream starts with [0xFF, 0x0A]
        if data.len() >= 2 && data[..2] == [0xFF, 0x0A] {
            return Some(Self::Jxl);
        }
        // JXL: container format starts with [0x00, 0x00, 0x00, 0x0C] + "JXL " at bytes 4-7
        if data.len() >= 8 && data[..4] == [0x00, 0x00, 0x00, 0x0C] && &data[4..8] == b"JXL " {
            return Some(Self::Jxl);
        }
        if data.len() >= 4 && &data[..4] == b"qoif" {
            return Some(Self::Qoi);
        }
        None
    }

    /// Return the canonical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Avif => "avif",
            Self::Jxl => "jxl",
            Self::Qoi => "qoi",
        }
    }

    /// Whether encoding is supported for this format.
    ///
    /// Returns `false` only for JXL due to GPL license restrictions
    /// in the reference encoder.
    pub fn can_encode(&self) -> bool {
        !matches!(self, Self::Jxl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── Extension detection ─────────────────────────────────────

    #[test]
    fn extension_jpg() {
        assert_eq!(
            Format::from_extension(Path::new("photo.jpg")),
            Some(Format::Jpeg)
        );
    }

    #[test]
    fn extension_jpeg() {
        assert_eq!(
            Format::from_extension(Path::new("photo.jpeg")),
            Some(Format::Jpeg)
        );
    }

    #[test]
    fn extension_jpg_uppercase() {
        assert_eq!(
            Format::from_extension(Path::new("photo.JPG")),
            Some(Format::Jpeg)
        );
    }

    #[test]
    fn extension_jpeg_mixed_case() {
        assert_eq!(
            Format::from_extension(Path::new("photo.JpEg")),
            Some(Format::Jpeg)
        );
    }

    #[test]
    fn extension_png() {
        assert_eq!(
            Format::from_extension(Path::new("image.png")),
            Some(Format::Png)
        );
    }

    #[test]
    fn extension_png_uppercase() {
        assert_eq!(
            Format::from_extension(Path::new("image.PNG")),
            Some(Format::Png)
        );
    }

    #[test]
    fn extension_webp() {
        assert_eq!(
            Format::from_extension(Path::new("image.webp")),
            Some(Format::WebP)
        );
    }

    #[test]
    fn extension_avif() {
        assert_eq!(
            Format::from_extension(Path::new("image.avif")),
            Some(Format::Avif)
        );
    }

    #[test]
    fn extension_jxl() {
        assert_eq!(
            Format::from_extension(Path::new("image.jxl")),
            Some(Format::Jxl)
        );
    }

    #[test]
    fn extension_qoi() {
        assert_eq!(
            Format::from_extension(Path::new("image.qoi")),
            Some(Format::Qoi)
        );
    }

    #[test]
    fn extension_unknown() {
        assert_eq!(Format::from_extension(Path::new("file.bmp")), None);
    }

    #[test]
    fn extension_none() {
        assert_eq!(Format::from_extension(Path::new("noext")), None);
    }

    #[test]
    fn extension_empty_path() {
        assert_eq!(Format::from_extension(&PathBuf::new()), None);
    }

    // ── Magic byte detection ────────────────────────────────────

    #[test]
    fn magic_jpeg() {
        let data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00];
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Jpeg));
    }

    #[test]
    fn magic_png() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Png));
    }

    #[test]
    fn magic_webp() {
        let mut data = Vec::new();
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // file size placeholder
        data.extend_from_slice(b"WEBP");
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::WebP));
    }

    #[test]
    fn magic_avif_avif_brand() {
        let mut data = vec![0x00, 0x00, 0x00, 0x20]; // box size
        data.extend_from_slice(b"ftyp");
        data.extend_from_slice(b"avif");
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Avif));
    }

    #[test]
    fn magic_avif_avis_brand() {
        let mut data = vec![0x00, 0x00, 0x00, 0x20]; // box size
        data.extend_from_slice(b"ftyp");
        data.extend_from_slice(b"avis");
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Avif));
    }

    #[test]
    fn magic_jxl_bare_codestream() {
        let data = [0xFF, 0x0A, 0x00, 0x00];
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Jxl));
    }

    #[test]
    fn magic_jxl_container() {
        let mut data = vec![0x00, 0x00, 0x00, 0x0C];
        data.extend_from_slice(b"JXL ");
        data.extend_from_slice(&[0x0D, 0x0A, 0x87, 0x0A]);
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Jxl));
    }

    #[test]
    fn magic_qoi() {
        let mut data = Vec::new();
        data.extend_from_slice(b"qoif");
        data.extend_from_slice(&[0x00; 10]);
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Qoi));
    }

    #[test]
    fn magic_unknown() {
        let data = [0x00, 0x00, 0x00, 0x00];
        assert_eq!(Format::from_magic_bytes(&data), None);
    }

    #[test]
    fn magic_empty() {
        assert_eq!(Format::from_magic_bytes(&[]), None);
    }

    #[test]
    fn magic_too_short_for_jpeg() {
        assert_eq!(Format::from_magic_bytes(&[0xFF, 0xD8]), None);
    }

    // ── extension() ─────────────────────────────────────────────

    #[test]
    fn extension_string() {
        assert_eq!(Format::Jpeg.extension(), "jpg");
        assert_eq!(Format::Png.extension(), "png");
        assert_eq!(Format::WebP.extension(), "webp");
        assert_eq!(Format::Avif.extension(), "avif");
        assert_eq!(Format::Jxl.extension(), "jxl");
        assert_eq!(Format::Qoi.extension(), "qoi");
    }

    // ── can_encode ──────────────────────────────────────────────

    #[test]
    fn can_encode_jxl_is_false() {
        assert!(!Format::Jxl.can_encode());
    }

    #[test]
    fn can_encode_all_others_true() {
        assert!(Format::Jpeg.can_encode());
        assert!(Format::Png.can_encode());
        assert!(Format::WebP.can_encode());
        assert!(Format::Avif.can_encode());
        assert!(Format::Qoi.can_encode());
    }
}
