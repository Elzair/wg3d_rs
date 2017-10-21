use std::collections::HashMap;
use std::path::Path;

use gltf::{Gltf, texture};
use gltf::image as gltf_image;
use image::{self, GenericImage};

use super::super::{Result, Error};
use super::ConvertError;
use super::buffer::Buffers;

pub type Textures = HashMap<String, Texture>;

#[derive(Clone, Debug)]
pub struct Texture {
    mag_filter: MagFilter,
    min_filter: MinFilter,
    wrap_s_mode: WrappingMode,
    wrap_t_mode: WrappingMode,
    width: u32,
    height: u32,
    format: Format,
    contents: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum MagFilter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug)]
pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

#[derive(Clone, Copy, Debug)]
pub enum WrappingMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

#[derive(Clone, Copy, Debug)]
pub enum Format {
    GrayImage,
    GrayAlphaImage,
    RgbImage,
    RgbaImage,
}

pub fn get<'a>(
    base_path: &'a Path,
    gltf: &'a Gltf,
    buffers: &'a Buffers
) -> Result<Textures> {
    let mut textures = Textures::new();
    
    for texture in gltf.textures() {
        let sampler = texture.sampler();
        let mag_filter = match sampler.mag_filter() {
            Some(texture::MagFilter::Linear) => MagFilter::Linear,
            Some(texture::MagFilter::Nearest) => MagFilter::Nearest,
            None => MagFilter::Nearest,
        };
        let min_filter = match sampler.min_filter() {
            Some(texture::MinFilter::Linear) => MinFilter::Linear,
            Some(texture::MinFilter::Nearest) => MinFilter::Nearest,
            Some(texture::MinFilter::LinearMipmapNearest) => MinFilter::LinearMipmapNearest,
            Some(texture::MinFilter::NearestMipmapNearest) => MinFilter::NearestMipmapNearest,
            Some(texture::MinFilter::LinearMipmapLinear) => MinFilter::LinearMipmapLinear,
            Some(texture::MinFilter::NearestMipmapLinear) => MinFilter::NearestMipmapLinear,
            None => MinFilter::Nearest,
        };
        let wrap_s = match sampler.wrap_s() {
            texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            texture::WrappingMode::Repeat => WrappingMode::Repeat,
        };
        let wrap_t = match sampler.wrap_t() {
            texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            texture::WrappingMode::Repeat => WrappingMode::Repeat,
        };

        // Get contents of image as either byte array or `image::DynamicImage`.
        let (uri, img) = match texture.source().data() {
            gltf_image::Data::View { view, .. } => {
                let offset = view.offset();
                let length = view.length();
                let buffer = view.buffer();
                let uri = buffer.uri().to_string();

                if let Some(arr) = buffers.get(&uri) {
                    let sl = &arr[offset..(offset+length)];
                    
                    let img = image::load_from_memory(sl)?;

                    (uri, img)
                } else {
                    return Err(Error::Convert(ConvertError::MissingImageBuffer));
                }
            },
            gltf_image::Data::Uri{ uri, .. } => {
                let uri_copy = uri.to_string();
                let full_path = base_path.to_path_buf().join(uri);
                let img = image::open(full_path)?;

                (uri_copy, img )
            },
        };

        let format = match &img {
            &image::DynamicImage::ImageLuma8(_) => Format::GrayImage,
            &image::DynamicImage::ImageLumaA8(_) => Format::GrayAlphaImage,
            &image::DynamicImage::ImageRgb8(_) => Format::RgbImage,
            &image::DynamicImage::ImageRgba8(_) => Format::RgbaImage,
        };

        match textures.insert(
            uri,
            Texture {
                mag_filter: mag_filter,
                min_filter: min_filter,
                wrap_s_mode: wrap_s,
                wrap_t_mode: wrap_t,
                width: img.width(),
                height: img.height(),
                format: format,
                contents: img.raw_pixels(),
            }
        ) {
            Some(returned_value) => {
                return Err(Error::Convert(ConvertError::MultipleTexturesInBuffer));
            },
            None => {},
        }
    }

    Ok(textures)
}

#[cfg(test)]
mod tests {
    use super::super::load_gltf;
    use super::super::buffer::get as get_buffers;
    use super::*;

    #[test]
    fn test_convert_buffers_get() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        let parent = path.parent().unwrap();
        let gltf = load_gltf(path).unwrap();
        let buffers = get_buffers(&parent, &gltf).unwrap();

        match get(&parent, &gltf, &buffers) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
