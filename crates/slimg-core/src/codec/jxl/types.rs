/// JXL encoding configuration.
pub(crate) struct EncodeConfig {
    pub lossless: bool,
    pub distance: f32,
}

impl EncodeConfig {
    pub fn from_quality(quality: u8) -> Self {
        if quality >= 100 {
            return Self {
                lossless: true,
                distance: 0.0,
            };
        }
        let distance =
            unsafe { libjxl_enc_sys::JxlEncoderDistanceFromQuality(quality as f32) };
        Self {
            lossless: false,
            distance,
        }
    }
}
