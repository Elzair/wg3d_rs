use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use gltf::{Gltf, texture};
use gltf::image as gltf_image;
use image::{self, DynamicImage};

use super::super::{Result, Error, Wg3dError};
use super::buffer::Buffers;

pub type TexturesInfo = HashMap<String, TextureInfo>;

pub struct TextureInfo {
    mag_filter: MagFilter,
    min_filter: MinFilter,
    wrap_s_mode: WrappingMode,
    wrap_t_mode: WrappingMode,
    contents: TextureContents,
}

pub enum MagFilter {
    Nearest,
    Linear,
}

pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

pub enum WrappingMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

pub enum TextureContents {
    DynamicImage(DynamicImage),
    Array(Vec<u8>),
}

pub fn get<'a>(
    base_path: &'a Path,
    gltf: &'a Gltf,
    buffers: &'a Buffers
) -> Result<TexturesInfo> {
    let mut textures = TexturesInfo::new();
    
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
        let (uri, contents) = match texture.source().data() {
            gltf_image::Data::View { view, mime_type } => {
                let offset = view.offset();
                let length = view.length();
                let buffer = view.buffer();
                let uri = buffer.uri().to_string();

                if let Some(arr) = buffers.get(&uri) {
                    let sl = &arr[offset..(offset+length)];
                    
                    let contents = image::load_from_memory(sl)?;

                    (uri, TextureContents::DynamicImage(contents))
                } else {
                    return Err(Error::Wg3d(Wg3dError::MissingImageBuffer));
                }
            },
            gltf_image::Data::Uri{ uri, mime_type } => {
                let uri_copy = uri.to_string();
                let full_path = base_path.to_path_buf().join(uri);
                let contents = image::open(full_path)?;

                (uri_copy, TextureContents::DynamicImage(contents))
            },
        };

        textures.insert(
            uri,
            TextureInfo {
                mag_filter: mag_filter,
                min_filter: min_filter,
                wrap_s_mode: wrap_s,
                wrap_t_mode: wrap_t,
                contents: contents,
            }
        );
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
