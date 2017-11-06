use std::env::current_dir;
use std::error;
use std::fmt;
use std::path::Path;

use cgmath::{Matrix4, SquareMatrix};
use gltf::{Scene, Node};
use gltf_importer::{import, Buffers};

use super::Result;

pub mod animation;
pub mod material;
pub mod mesh;
pub mod primitive;
pub mod skin;
mod util;
pub mod texture;

use self::animation::{Animation, get as get_animations};
use self::mesh::{Mesh, get as get_mesh};
use self::skin::{Skin, get as get_skins};
use self::texture::{Texture, get as get_textures};

pub struct Model {
    mesh: Mesh,
}

pub fn get<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<Model>> {
    // Read in all relevant data.
    let cwd = current_dir()?;
    let parent = path.as_ref().parent().unwrap_or(&cwd);
    let (gltf, buffers) = import(&path)?;
    let textures = get_textures(&parent, &gltf, &buffers)?;

    // Retrieve default scene from gltf.
    let scene = gltf.default_scene().ok_or(ConvertError::NoDefaultScene)?;

    // Retrieve skins.
    let skins = get_skins(&gltf, &buffers)?;

    // Retrieve animations.
    let animations = get_animations(&gltf, &skins, &buffers)?;

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
            &mut models,
            buffers,
            textures
        )?;
    }

    Ok(models)
}

fn get_models_helper<'a>(
    node: &'a Node,
    models: &'a mut Vec<Model>,
    buffers: &'a Buffers,
    textures: &'a Vec<texture::Texture>,
) -> Result<()> {
    // Add model if mesh is present.
    if let Some(mesh) = node.mesh() {
        let name = node.name().ok_or(ConvertError::NoName)?;
        let weights = node.weights();
        let has_bones = node.skin().is_some();
        let mesh = get_mesh(&mesh, name, weights, has_bones, buffers, textures)?;
        models.push(Model { mesh: mesh });
    }
    
    // Try to find models in child nodes.
    for node in node.children() {
        get_models_helper(&node, models, buffers, textures)?;
    }

    Ok(())
}

/// Error container for handling Wg3d
#[derive(Debug)]
pub enum ConvertError {
    /// Primitive missing required attributes
    MissingAttributes,
    /// Image buffer not present
    MissingImageBuffer,
    /// No specified root node of skeleton for a skin
    NoSkeleton,
    /// No default scene present
    NoDefaultScene,
    /// No name for a mesh, skin, or animation
    NoName,
    /// Invalid skeleton joint index
    InvalidJoint,
    /// Too many joints
    TooManyJoints,
    /// Something weird
    Other,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConvertError::MissingAttributes => {
                write!(fmt, "Primitive missing required attributes")
            },
            ConvertError::MissingImageBuffer => {
                write!(fmt, "Missing image buffer")
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
            ConvertError::InvalidJoint => {
                write!(fmt, "Invalid skeleton joint index")
            },
            ConvertError::TooManyJoints => {
                write!(fmt, "Too many joints")
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
        static MISSING_IMAGE_BUFFER: &'static str = "Missing image buffer";
        static NO_SKELETON: &'static str = "No specified root node of skeleton for a skin";
        static NO_DEFAULT_SCENE: &'static str = "No default scene present";
        static NO_NAME: &'static str = "No name for a mesh, skin, or animation";
        static INVALID_JOINT: &'static str = "Invalid skeleton joint index";
        static TOO_MANY_JOINTS: &'static str = "Too many joints";
        static OTHER: &'static str = "Something weird happened";

        match *self {
            ConvertError::MissingAttributes => {
                MISSING_ATTRIBUTES
            },
            ConvertError::MissingImageBuffer => {
                MISSING_IMAGE_BUFFER
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
            ConvertError::InvalidJoint => {
                INVALID_JOINT
            },
            ConvertError::TooManyJoints => {
                TOO_MANY_JOINTS
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
