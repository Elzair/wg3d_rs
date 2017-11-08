use std::path::Path;

use gltf::image::Data as GltfData;
use gltf::gltf::Textures as GltfTextures;
use gltf::texture::{MinFilter as GltfMinFilter, MagFilter as GltfMagFilter, WrappingMode as GltfWrappingMode};
use gltf_importer::Buffers;
use image::{GenericImage, DynamicImage, load_from_memory as load_image_from_memory, open as open_image};

use super::super::Result;
use super::ConvertError;

pub struct Textures {
    textures: Vec<Texture>,
}

impl Textures {
    pub fn get(&self, index: usize) -> Option<&str> {
        match self.textures.iter().nth(index) {
            Some(texture) => Some(texture.name.as_ref()),
            None => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    name: String,
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
    textures: GltfTextures,
    buffers: &'a Buffers
) -> Result<Textures> {
    let my_textures = textures.map(|texture| {
        let name = texture.name().ok_or(ConvertError::NoName)?;
        let sampler = texture.sampler();
        let mag_filter = match sampler.mag_filter() {
            Some(GltfMagFilter::Linear) => MagFilter::Linear,
            Some(GltfMagFilter::Nearest) => MagFilter::Nearest,
            None => MagFilter::Nearest,
        };
        let min_filter = match sampler.min_filter() {
            Some(GltfMinFilter::Linear) => MinFilter::Linear,
            Some(GltfMinFilter::Nearest) => MinFilter::Nearest,
            Some(GltfMinFilter::LinearMipmapNearest) => MinFilter::LinearMipmapNearest,
            Some(GltfMinFilter::NearestMipmapNearest) => MinFilter::NearestMipmapNearest,
            Some(GltfMinFilter::LinearMipmapLinear) => MinFilter::LinearMipmapLinear,
            Some(GltfMinFilter::NearestMipmapLinear) => MinFilter::NearestMipmapLinear,
            None => MinFilter::Nearest,
        };
        let wrap_s = match sampler.wrap_s() {
            GltfWrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            GltfWrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            GltfWrappingMode::Repeat => WrappingMode::Repeat,
        };
        let wrap_t = match sampler.wrap_t() {
            GltfWrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            GltfWrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            GltfWrappingMode::Repeat => WrappingMode::Repeat,
        };

        // Get contents of image as either byte array or `image::DynamicImage`.
        let img = match texture.source().data() {
            GltfData::View { view, .. } => {
                let contents = buffers.view(&view).ok_or(ConvertError::MissingImageBuffer)?;
                load_image_from_memory(contents)?
            },
            GltfData::Uri{ uri, .. } => {
                let full_path = base_path.to_path_buf().join(uri);
                open_image(full_path)?
            },
        };

        let format = match &img {
            &DynamicImage::ImageLuma8(_) => Format::GrayImage,
            &DynamicImage::ImageLumaA8(_) => Format::GrayAlphaImage,
            &DynamicImage::ImageRgb8(_) => Format::RgbImage,
            &DynamicImage::ImageRgba8(_) => Format::RgbaImage,
        };

        Ok(Texture {
            name: String::from(name),
            mag_filter: mag_filter,
            min_filter: min_filter,
            wrap_s_mode: wrap_s,
            wrap_t_mode: wrap_t,
            width: img.width(),
            height: img.height(),
            format: format,
            contents: img.raw_pixels(),
        })
    }).collect::<Result<Vec<_>>>()?;

    Ok(Textures {
        textures: my_textures,
    })
}

#[cfg(test)]
mod tests {
    // use super::super::load_gltf;
    use super::*;

    // #[test]
    // fn test_convert_buffers_get() {
    //     let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
    //     let parent = path.parent().unwrap();
    //     let gltf = load_gltf(path).unwrap();
    //     let buffers = get_buffers(&parent, &gltf).unwrap();

    //     match get(&parent, &gltf, &buffers) {
    //         Ok(_) => {},
    //         Err(err) => {
    //             println!("{}", err.to_string());
    //             assert!(false);
    //         }
    //     }
    // }
}
