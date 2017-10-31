use std::env::current_dir;
use std::error;
use std::fmt;
use std::path::Path;

use cgmath::{Matrix4, SquareMatrix};
use gltf::{Scene, Node};
use gltf_importer::{import, Buffers};

use super::Result;

pub mod material;
pub mod mesh;
pub mod primitive;
pub mod skin;
pub mod texture;

use self::mesh::{Mesh, get as get_mesh};
use self::skin::{Skin, get as get_skins};
use self::texture::{Texture, get as get_textures};

pub struct Model {
    mesh: Mesh,
}

pub fn get<P: AsRef<Path>>(path: P) -> Result<Vec<Model>> {
    // Read in all relevant data.
    let cwd = current_dir()?;
    let parent = path.as_ref().parent().unwrap_or(&cwd);
    let (gltf, buffers) = import(&path)?;
    let textures = get_textures(&parent, &gltf, &buffers)?;

    // Retrieve default scene from gltf.
    let scene = gltf.default_scene().ok_or(ConvertError::NoDefaultScene)?;

    // Retrieve skins.
    let skins = get_skins(&gltf, &buffers)?;

    // Retrieve models.
    let models = get_models(&scene, &buffers, &textures)?;

    Ok(models)
}

pub fn get_models<'a>(
    scene: &'a Scene,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
) -> Result<Vec<Model>> {
    let mut models = Vec::<Model>::new();

    for root_node in scene.nodes() {
        get_models_helper(
            &root_node,
            Matrix4::identity(),
            &mut models,
            buffers,
            textures
        )?;
    }

    Ok(models)
}

fn get_models_helper<'a>(
    node: &'a Node,
    current_transform: Matrix4<f32>,
    models: &'a mut Vec<Model>,
    buffers: &'a Buffers,
    textures: &'a Vec<texture::Texture>,
) -> Result<()> {
    // Compute current transform.
    let local_transform = Matrix4::from(node.transform().matrix());
    let transform = current_transform * local_transform;

    // Add model if mesh is present.
    if let Some(mesh) = node.mesh() {
        let name = node.name().ok_or(ConvertError::NoName)?;
        let has_bones = node.skin().is_some();
        let mesh = get_mesh(&mesh, name, transform, has_bones, buffers, textures)?;
        models.push(Model { mesh: mesh });
    }
    
    // Try to find models in child nodes.
    for node in node.children() {
        get_models_helper(&node, transform, models, buffers, textures)?;
    }

    Ok(())
}

/// Error container for handling Wg3d
#[derive(Debug)]
pub enum ConvertError {
    /// Primitive missing required attributes
    MissingAttributes,
    /// Unsupported data type
    UnsupportedDataType,
    /// Unsupported dimensions
    UnsupportedDimensions,
    /// Invalid buffer length
    InvalidBufferLength,
    /// Image buffer not present
    MissingImageBuffer,
    /// Multiple textures share binary buffer
    MultipleTexturesInBuffer,
    /// No specified root node of skeleton for a skin
    NoSkeleton,
    /// No default scene present
    NoDefaultScene,
    /// No name for a mesh, skin, or animation
    NoName,
    /// Something weird
    Other,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConvertError::MissingAttributes => {
                write!(fmt, "Primitive missing required attributes")
            },
            ConvertError::UnsupportedDataType => {
                write!(fmt, "Primitive attribute using unsupported data type")
            },
            ConvertError::UnsupportedDimensions => {
                write!(fmt, "Primitive attribute using unsupported dimensions")
            },
            ConvertError::InvalidBufferLength => {
                write!(fmt, "Invalid buffer length")
            },
            ConvertError::MissingImageBuffer => {
                write!(fmt, "Missing image buffer")
            },
            ConvertError::MultipleTexturesInBuffer => {
                write!(fmt, "Multiple textures share binary buffer")
            },
            ConvertError::NoSkeleton => {
                write!(fmt, "No specified root node of skeleton for a skin")
            },
            ConvertError::NoDefaultScene => {
                write!(fmt, "No default scene present")
            },
            ConvertError::NoName => {
                write!(fmt, "No name for a mesh, skin, or animation")
            },
            ConvertError::Other => {
                write!(fmt, "Something weird happened")
            },
        }
    }
}

impl error::Error for ConvertError {
    fn description(&self) -> &str {
        static MISSING_ATTRIBUTES: &'static str = "Primitive missing required attributes";
        static UNSUPPORTED_DATA_TYPE: &'static str = "Primitive attribute using unsupported data type";
        static UNSUPPORTED_DIMENSIONS: &'static str = "Primitive attribute using unsupported dimensions";
        static INVALID_BUFFER_LENGTH: &'static str = "Invalid buffer length";
        static MISSING_IMAGE_BUFFER: &'static str = "Missing image buffer";
        static MULTIPLE_TEXTURES_IN_BUFFER: &'static str = "Multiple textures share binary buffer";
        static NO_SKELETON: &'static str = "No specified root node of skeleton for a skin";
        static NO_DEFAULT_SCENE: &'static str = "No default scene present";
        static NO_NAME: &'static str = "No name for a mesh, skin, or animation";
        static OTHER: &'static str = "Something weird happened";

        match *self {
            ConvertError::MissingAttributes => {
                MISSING_ATTRIBUTES
            },
            ConvertError::UnsupportedDataType => {
                UNSUPPORTED_DATA_TYPE
            },
            ConvertError::UnsupportedDimensions => {
                UNSUPPORTED_DIMENSIONS
            },
            ConvertError::InvalidBufferLength => {
                INVALID_BUFFER_LENGTH
            },
            ConvertError::MissingImageBuffer => {
                MISSING_IMAGE_BUFFER
            },
            ConvertError::MultipleTexturesInBuffer => {
                MULTIPLE_TEXTURES_IN_BUFFER
            },
            ConvertError::NoSkeleton => {
                NO_SKELETON
            },
            ConvertError::NoDefaultScene => {
                NO_DEFAULT_SCENE
            },
            ConvertError::NoName => {
                NO_NAME
            },
            ConvertError::Other => {
                OTHER
            },
        }
    }

    fn cause(&self) -> Option<&error::Error> { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        match get(path) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
