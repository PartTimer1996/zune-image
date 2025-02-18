#![cfg(feature = "png")]
//! Represents an png image decoder
use log::{debug, info};
use zune_core::bit_depth::BitDepth;
use zune_core::colorspace::ColorSpace;
use zune_core::result::DecodingResult;
pub use zune_png::*;

use crate::codecs::ImageFormat;
use crate::deinterleave::{deinterleave_u16, deinterleave_u8};
use crate::errors::ImageErrors;
use crate::image::Image;
use crate::metadata::ImageMetadata;
use crate::traits::DecoderTrait;

impl<'a> DecoderTrait<'a> for PngDecoder<'a>
{
    fn decode(&mut self) -> Result<Image, ImageErrors>
    {
        let metadata = self.read_headers()?.unwrap();

        let pixels = self
            .decode()
            .map_err(<error::PngDecodeErrors as Into<ImageErrors>>::into)?;

        let depth = self.get_depth().unwrap();
        let (width, height) = self.get_dimensions().unwrap();
        let colorspace = self.get_colorspace().unwrap();

        let mut image = match pixels
        {
            DecodingResult::U8(data) => Image::from_u8(&data, width, height, colorspace),
            DecodingResult::U16(data) => Image::from_u16(&data, width, height, colorspace),
            _ => unreachable!()
        };
        // metadata
        image.metadata = metadata;

        Ok(image)
    }
    fn get_dimensions(&self) -> Option<(usize, usize)>
    {
        self.get_dimensions()
    }

    fn get_out_colorspace(&self) -> ColorSpace
    {
        self.get_colorspace().unwrap()
    }

    fn get_name(&self) -> &'static str
    {
        "PNG Decoder"
    }

    fn read_headers(&mut self) -> Result<Option<ImageMetadata>, crate::errors::ImageErrors>
    {
        self.decode_headers()
            .map_err(<error::PngDecodeErrors as Into<ImageErrors>>::into)?;

        let (width, height) = self.get_dimensions().unwrap();
        let depth = self.get_depth().unwrap();

        let mut metadata = ImageMetadata {
            format: Some(ImageFormat::PNG),
            colorspace: self.get_colorspace().unwrap(),
            depth: depth,
            width: width,
            height: height,
            default_gamma: self.get_info().unwrap().gamma,
            ..Default::default()
        };
        #[cfg(feature = "metadata")]
        {
            let info = self.get_info().unwrap();
            // see if we have an exif chunk
            if let Some(exif) = info.exif
            {
                metadata.parse_raw_exif(exif)
            }
        }

        Ok(Some(metadata))
    }
}

impl From<zune_png::error::PngDecodeErrors> for ImageErrors
{
    fn from(from: zune_png::error::PngDecodeErrors) -> Self
    {
        let err = format!("png: {from:?}");

        ImageErrors::ImageDecodeErrors(err)
    }
}
