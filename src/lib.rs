use failure::Fail;
use heif_sys::*;
use std::ffi;
use std::mem;
use std::ptr;

mod test;

#[derive(Debug, Fail)]
#[repr(C)]
pub enum HeifError {
    #[fail(display = "Input does not exist")]
    InputDoesNotExist = 1,
    #[fail(display = "Invalid Input")]
    InvalidInput = 2,
    #[fail(display = "Unsupported File Type")]
    UnsupportedFiletype = 3,
    #[fail(display = "Unsupported Feature")]
    UnsupportedFeature = 4,
    #[fail(display = "Usage Error")]
    UsageHeifError = 5,
    #[fail(display = "Memory Allocation Error")]
    MemoryAllocationHeifError = 6,
    #[fail(display = "Decoder Plugin Error")]
    DecoderPluginHeifError = 7,
    #[fail(display = "Encoder Plugin Error")]
    EncoderPluginHeifError = 8,
    #[fail(display = "Encoding Error")]
    EncodingHeifError = 9,
    #[fail(display = "Failed to create context")]
    ContexCreateFailed,
    #[fail(display = "Unknown Error")]
    Unknown,
}

pub fn err_message(err: heif_error) -> String {
    unsafe { ffi::CStr::from_ptr(err.message) }
        .to_str()
        .unwrap()
        .to_owned()
}
pub fn err_result(err: heif_error) -> Result<(), HeifError> {
    if err.code == 0 {
        Ok(())
    } else if err.code > 0 && err.code < 10 {
        Err(unsafe { mem::transmute(err.code) })
    } else {
        Err(HeifError::Unknown)
    }
}

/*
    ContexCreateFailed,
    HeifFileReadFailed(String),
    ImageHandleAcquireFailed,
    ImageCreateFailed,
    ImageDecode(String),
    GetEncoderFailed,
    SetLossQuality,
}*/

#[derive(Debug)]
#[repr(C)]
pub enum ColorSpace {
    Undefined = 99,
    YCbCr = 0,
    Rgb = 1,
    Monochrome = 2,
}

#[derive(Debug)]
#[repr(C)]
pub enum Chroma {
    Undefined = 99,
    Monochrome = 0,
    C420 = 1,
    C422 = 2,
    C444 = 3,
    InterleavedRgb = 10,
    InterleavedRgba = 11,
}

#[derive(Debug)]
#[repr(C)]
pub enum Channel {
    Y = 0,
    Cb = 1,
    Cr = 2,
    R = 3,
    G = 4,
    B = 5,
    Alpha = 6,
    Interleaved = 10,
}

#[derive(Debug)]
pub struct DecodeOptions {
    inner: *mut heif_decoding_options,
}

impl DecodeOptions {
    pub fn new() -> DecodeOptions {
        DecodeOptions {
            inner: unsafe { heif_decoding_options_alloc() },
        }
    }
}

impl Drop for DecodeOptions {
    fn drop(&mut self) {
        unsafe { heif_decoding_options_free(self.inner) };
    }
}

pub struct ImageHandle {
    inner: *mut heif_image_handle,
}

impl ImageHandle {
    pub fn width(&self) -> u32 {
        unsafe { heif_image_handle_get_width(self.inner) as _ }
    }

    pub fn height(&self) -> u32 {
        unsafe { heif_image_handle_get_width(self.inner) as _ }
    }

    pub fn has_alpha_channel(&self) -> bool {
        unsafe {
            if heif_image_handle_has_alpha_channel(self.inner) == 0 {
                false
            } else {
                true
            }
        }
    }

    pub fn decode(&self, options: &DecodeOptions) -> Result<Image, HeifError> {
        unsafe {
            let mut image = Box::new(mem::uninitialized());
            let err = heif_decode_image(
                self.inner,
                &mut *image,
                heif_colorspace_heif_colorspace_undefined, // encoder->colorspace(has_alpha),
                heif_chroma_heif_chroma_undefined,         //encoder->chroma(has_alpha),
                options.inner,
            );
            err_result(err)?;
            Ok(Image { inner: *image })
        }
    }
}

pub struct Context {
    inner: *mut heif_context,
}

impl Context {
    pub fn new() -> Result<Context, HeifError> {
        let ctx = unsafe { heif_context_alloc() };
        if ctx == ptr::null_mut() {
            Err(HeifError::ContexCreateFailed)
        } else {
            Ok(Context { inner: ctx })
        }
    }

    pub fn read_from_bytes(&self, bytes: &[u8]) -> Result<(), HeifError> {
        let err = unsafe {
            heif_context_read_from_memory_without_copy(
                self.inner,
                bytes.as_ptr() as _,
                bytes.len(),
                ptr::null(),
            )
        };
        err_result(err)
    }

    pub fn read_from_file(&mut self, name: &str) -> Result<(), HeifError> {
        let c_name = ffi::CString::new(name).unwrap();
        let err = unsafe { heif_context_read_from_file(self.inner, c_name.as_ptr(), ptr::null()) };
        err_result(err)
    }

    pub fn write_to_file(&self, name: &str) {
        unsafe {
            let c_name = ffi::CString::new(name).unwrap();
            heif_context_write_to_file(self.inner, c_name.as_ptr());
        }
    }

    pub fn get_number_of_top_level_images(&mut self) -> usize {
        unsafe { heif_context_get_number_of_top_level_images(self.inner) as _ }
    }

    pub fn get_primary_image_handle(&self) -> Result<ImageHandle, HeifError> {
        unsafe {
            let mut handle = Box::new(mem::uninitialized());
            //       let handle = mem::uninitialized();
            let err = heif_context_get_primary_image_handle(self.inner, &mut *handle);
            err_result(err)?;
            Ok(ImageHandle { inner: *handle })
        }
    }

    pub fn get_encoder_for_format(&mut self) -> Result<Encoder, HeifError> {
        let mut encoder = Box::new(unsafe { mem::uninitialized() });
        let err = unsafe {
            heif_context_get_encoder_for_format(
                self.inner,
                heif_compression_format_heif_compression_HEVC,
                &mut *encoder,
            )
        };
        err_result(err)?;
        Ok(Encoder { inner: *encoder })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { heif_context_free(self.inner) };
    }
}

pub struct Encoder {
    inner: *mut heif_encoder,
}

impl Encoder {
    pub fn set_lossy_quality(&mut self, value: usize) -> Result<(), HeifError> {
        let err = unsafe { heif_encoder_set_lossy_quality(self.inner, value as _) };
        err_result(err)
    }
}

pub struct Image {
    inner: *mut heif_image,
}

impl Image {
    pub fn new(width: u32, height: u32, colorspace: u32, chroma: u32) -> Result<Image, HeifError> {
        let mut image = Image {
            inner: unsafe { mem::uninitialized() },
        };
        let err = unsafe {
            heif_image_create(
                width as _,
                height as _,
                colorspace,
                chroma,
                &mut image.inner,
            )
        };
        err_result(err)?;
        Ok(image)
    }

    pub fn get_plane(&self, channel: Channel) -> (&mut [u8], u32) {
        unsafe {
            let mut stride: i32 = 1;
            let data = heif_image_get_plane(self.inner, channel as _, &mut stride as _);
            let height = self.height() as usize;
            let size = height * (stride as usize);
            use std::slice;
            let bytes = slice::from_raw_parts_mut(data, size);
            (bytes, stride as _)
        }
    }

    pub fn width(&self) -> u32 {
        unsafe { heif_image_get_width(self.inner, heif_channel_heif_channel_Y) as _ }
    }

    pub fn height(&self) -> u32 {
        unsafe { heif_image_get_height(self.inner, heif_channel_heif_channel_Y) as _ }
    }

    pub fn get_chroma_format(&self) -> Chroma {
        unsafe { mem::transmute(heif_image_get_chroma_format(self.inner)) }
    }

    pub fn get_color_space(&self) -> ColorSpace {
        unsafe { mem::transmute(heif_image_get_colorspace(self.inner)) }
    }
}
