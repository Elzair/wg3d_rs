use cgmath::Matrix4;
use gltf::mesh::Mesh as GltfMesh;
use gltf_importer::Buffers;

use super::super::Result;
use super::primitive::{Primitive, get as get_primitive};
use super::texture::Texture;

pub struct Mesh {
    name: String,
    primitives: Vec<Primitive>,
}

pub fn get<'a>(
    mesh: &'a GltfMesh,
    name: &'a str,
    node_weights: Option<&'a [f32]>,
    has_bones: bool,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
) -> Result<Mesh> {
    let weights = match mesh.weights() {
        Some(weights) => Some(weights),
        None => node_weights,
    };

    let primitives = mesh.primitives().map(|prim| {
        let primitive = get_primitive(&prim, weights, has_bones, buffers, textures)?;
        Ok(primitive)
    }).collect::<Result<Vec<_>>>()?;

    Ok(Mesh {
        name: String::from(name),
        primitives: primitives,
    })
}

