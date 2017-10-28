use std::path::Path;
use std::io;
use std::io::Read;
use std::mem;

use cgmath::Matrix4;
use gltf::Gltf;
use gltf::mesh::Mesh as GltfMesh;
use gltf::skin::Skin as GltfSkin;
use gltf_importer::Buffers;

use super::super::{Result, Error};
use super::ConvertError;
use super::primitive::{Primitive, get as get_primitive};
use super::skin::{Skin, get as get_skin};
use super::texture::Texture;

pub struct Mesh {
    skin: Option<Skin>,
    primitives: Vec<Primitive>,
}

pub fn get<'a>(
    mesh: &'a GltfMesh,
    skin: Option<GltfSkin>,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
) -> Result<Mesh> {
    let skin = get_skin(skin, buffers)?;
    let has_bones = skin.is_some();
    
    let mut primitives = Vec::<Primitive>::new();
    
    for prim in mesh.primitives() {
        let primitive = get_primitive(&prim, transform, buffers, textures, has_bones)?;
        primitives.push(primitive);
    }

    Ok(Mesh {
        skin: skin,
        primitives: primitives,
    })
}

