use gltf::mesh::Mesh as GltfMesh;
use gltf_importer::Buffers;

use super::super::Result;
use super::primitive::{Primitive, get as get_primitives};
use super::material::Materials;

pub struct Mesh {
    name: String,
    primitives: Vec<Primitive>,
}

pub fn get<'a>(
    mesh: &'a GltfMesh,
    name: &'a str,
    node_weights: Option<&'a [f32]>,
    has_joints: bool,
    buffers: &'a Buffers,
    materials: &'a Materials,
) -> Result<Mesh> {
    let weights = if let Some(weights) = mesh.weights() {
        Some(weights)
    } else if let Some(weights2) = node_weights {
        Some(weights2)
    } else {
        None
    };

    let primitives = get_primitives(
        mesh.primitives(),
        weights,
        has_joints,
        buffers,
        materials
    )?;

    Ok(Mesh {
        name: String::from(name),
        primitives: primitives,
    })
}

