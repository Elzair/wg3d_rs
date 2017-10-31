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
    transform: Matrix4<f32>,
    has_bones: bool,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
) -> Result<Mesh> {
    let mut primitives = Vec::<Primitive>::new();

    for prim in mesh.primitives() {
        let primitive = get_primitive(&prim, transform, has_bones, buffers, textures)?;
        primitives.push(primitive);
    }

    Ok(Mesh {
        name: String::from(name),
        primitives: primitives,
    })
}

