use cgmath::{Vector2, Vector3, Vector4, Matrix4};
use gltf::mesh as gltf_mesh;
use gltf_importer::Buffers;
use gltf_utils::PrimitiveIterators;

use super::super::Result;
use super::ConvertError;
use super::material::{Material, get as get_material};
use super::texture::Texture;

pub struct Primitive {
    material: Material,
    vertex_attributes: VertexAttributes,
    indices: Vec<u32>,
}

pub fn get<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    has_bones: bool,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
) -> Result<Primitive> {
    let material = get_material(primitive, textures)?;
    let vertex_attributes = get_vertex_attributes(
        primitive,
        transform,
        buffers,
        has_bones
    )?;
    let indices = get_indices(primitive, buffers)?;

    Ok(Primitive {
        material: material,
        vertex_attributes: vertex_attributes,
        indices: indices,
    })
}

pub struct VertexAttributes {
    positions: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    texcoords_0: Vec<Vector2<f32>>,
    texcoords_1: Option<Vec<Vector2<f32>>>,
    tangents: Option<Vec<Vector4<f32>>>,
    bones: Option<Bones>,
}

fn get_vertex_attributes<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
    has_joints: bool,
) -> Result<VertexAttributes> {
    let positions = get_positions(primitive, transform, buffers)?;
    let normals = get_normals(primitive, transform, buffers)?;
    let texcoords_0 = get_texcoords_0(primitive, buffers)?;
    let texcoords_1 = get_texcoords_1(primitive, buffers);
    let tangents = get_tangents(primitive, transform, buffers);
    let bones = get_bones(primitive, buffers, has_joints)?;

    Ok(VertexAttributes {
        positions: positions,
        normals: normals,
        texcoords_0: texcoords_0,
        texcoords_1: texcoords_1,
        tangents: tangents,
        bones: bones,
    })
}

pub struct Bones {
    joints: Vec<Vector4<u16>>,
    weights: Vec<Vector4<f32>>,
}

fn get_bones<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
    has_joints: bool
) -> Result<Option<Bones>> {
    if has_joints {
        let joints = get_joints(primitive, buffers)?;
        let weights = get_weights(primitive, buffers)?;

        Ok(Some(Bones {
            joints: joints,
            weights: weights,
        }))
    } else {
        Ok(None)
    }
}

fn get_positions<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
) -> Result<Vec<Vector3<f32>>> {
    let iter = primitive.positions(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|pos| {
        let position = transform * Vector4::<f32>::new(pos[0], pos[1], pos[2], 1.0);
        position.truncate()
    }).collect::<Vec<_>>())
}

fn get_normals<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
) -> Result<Vec<Vector3<f32>>> {
    let iter = primitive.normals(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|norm| {
        let normal = transform * Vector4::<f32>::new(norm[0], norm[1], norm[2], 1.0);
        normal.truncate()
    }).collect::<Vec<_>>())
}

fn get_texcoords_0<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<Vector2<f32>>> {
    let iter = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|tx0| {
        Vector2::<f32>::from(tx0)
    }).collect::<Vec<_>>())
}

fn get_texcoords_1<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Option<Vec<Vector2<f32>>> {
    if let Some(iter) = primitive.tex_coords_f32(1, buffers) {
        Some(iter.map(|tx1| {
            Vector2::from(tx1)
        }).collect::<Vec<_>>())
    } else { None }

}

fn get_tangents<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
) -> Option<Vec<Vector4<f32>>> {
    if let Some(iter) = primitive.tangents(buffers) {
        Some(iter.map(|tang| {
            transform * Vector4::from(tang)
        }).collect::<Vec<_>>())
    } else { None }
}

fn get_joints<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<Vector4<u16>>> {
    let iter = primitive.joints_u16(0, buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|joint| {
        Vector4::from(joint)
    }).collect::<Vec<_>>())
}

fn get_weights<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<Vector4<f32>>> {
    let iter = primitive.weights_f32(0, buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|weight| {
        Vector4::from(weight)
    }).collect::<Vec<_>>())
}

fn get_indices<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<u32>> {
    let iter = primitive.indices_u32(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.collect::<Vec<_>>())
}
