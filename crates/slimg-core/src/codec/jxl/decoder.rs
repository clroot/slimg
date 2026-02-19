use std::ptr;

use libjxl_sys::*;

use crate::error::{Error, Result};

/// Safe wrapper around libjxl decoder.
pub(crate) struct Decoder {
    ptr: *mut JxlDecoder,
}

impl Decoder {
    /// Create a new JXL decoder instance.
    pub fn new() -> Result<Self> {
        let ptr = unsafe { JxlDecoderCreate(ptr::null()) };
        if ptr.is_null() {
            return Err(Error::Decode("failed to create JXL decoder".into()));
        }
        Ok(Self { ptr })
    }

    /// Decode JXL data into RGBA pixels. Returns (width, height, rgba_pixels).
    pub fn decode_to_rgba(&mut self, data: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
        unsafe { JxlDecoderReset(self.ptr) };

        let events = JxlDecoderStatus_JXL_DEC_BASIC_INFO | JxlDecoderStatus_JXL_DEC_FULL_IMAGE;
        let status = unsafe { JxlDecoderSubscribeEvents(self.ptr, events as i32) };
        if status != JxlDecoderStatus_JXL_DEC_SUCCESS {
            return Err(Error::Decode("failed to subscribe decoder events".into()));
        }

        let status = unsafe {
            JxlDecoderSetInput(self.ptr, data.as_ptr(), data.len())
        };
        if status != JxlDecoderStatus_JXL_DEC_SUCCESS {
            return Err(Error::Decode("failed to set decoder input".into()));
        }
        unsafe { JxlDecoderCloseInput(self.ptr) };

        let format = JxlPixelFormat {
            num_channels: 4,
            data_type: JxlDataType_JXL_TYPE_UINT8,
            endianness: JxlEndianness_JXL_NATIVE_ENDIAN,
            align: 0,
        };

        let mut width = 0u32;
        let mut height = 0u32;
        let mut pixels: Vec<u8> = Vec::new();

        loop {
            let status = unsafe { JxlDecoderProcessInput(self.ptr) };

            if status == JxlDecoderStatus_JXL_DEC_BASIC_INFO {
                let mut info: JxlBasicInfo = unsafe { std::mem::zeroed() };
                let s = unsafe { JxlDecoderGetBasicInfo(self.ptr, &mut info) };
                if s != JxlDecoderStatus_JXL_DEC_SUCCESS {
                    return Err(Error::Decode("failed to get basic info".into()));
                }
                width = info.xsize;
                height = info.ysize;

                let mut buf_size: usize = 0;
                let s = unsafe {
                    JxlDecoderImageOutBufferSize(self.ptr, &format, &mut buf_size)
                };
                if s != JxlDecoderStatus_JXL_DEC_SUCCESS {
                    return Err(Error::Decode("failed to get output buffer size".into()));
                }

                pixels.resize(buf_size, 0);
                let s = unsafe {
                    JxlDecoderSetImageOutBuffer(
                        self.ptr,
                        &format,
                        pixels.as_mut_ptr().cast(),
                        pixels.len(),
                    )
                };
                if s != JxlDecoderStatus_JXL_DEC_SUCCESS {
                    return Err(Error::Decode("failed to set output buffer".into()));
                }
            } else if status == JxlDecoderStatus_JXL_DEC_FULL_IMAGE {
                return Ok((width, height, pixels));
            } else if status == JxlDecoderStatus_JXL_DEC_SUCCESS {
                if !pixels.is_empty() {
                    return Ok((width, height, pixels));
                }
                return Err(Error::Decode("decoder finished without producing image".into()));
            } else if status == JxlDecoderStatus_JXL_DEC_ERROR {
                return Err(Error::Decode("JXL decoding failed".into()));
            }
            // JXL_DEC_NEED_MORE_INPUT shouldn't happen since we closed input
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            JxlDecoderDestroy(self.ptr);
        }
    }
}
