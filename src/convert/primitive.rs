use cgmath::{Vector4, Matrix4};
use gltf::mesh as gltf_mesh;
use gltf_importer::Buffers;
use gltf_utils::PrimitiveIterators;

use super::super::{Result, Error};
use super::ConvertError;
use super::material::{Material, get as get_material};
use super::texture::Texture;

// TODO: Remove when gltf-utils is fixed.
use std::io;
use std::mem;
use byteorder::{LittleEndian, ReadBytesExt};
use gltf::accessor as gltfacc;

pub struct Primitive {
    material: Material,
    vertex_attributes: VertexAttributes,
    indices: Vec<u32>,
}

pub fn get<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
    has_bones: bool,
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
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords_0: Vec<[f32; 2]>,
    texcoords_1: Option<Vec<[f32; 2]>>,
    tangents: Option<Vec<[f32; 4]>>,
    bones: Option<Bones>,
}

fn get_vertex_attributes<'a>(
    primitive: &'a gltf_mesh::Primitive,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
    has_joints: bool,
) -> Result<VertexAttributes> {
    let positions = get_positions(primitive, buffers)?;
    let normals = get_normals(primitive, buffers)?;
    let texcoords_0 = get_texcoords_0(primitive, buffers)?;
    let texcoords_1 = get_texcoords_1(primitive, buffers);
    let tangents = get_tangents(primitive, buffers);
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
    joints: Vec<[u16; 4]>,
    weights: Vec<[f32; 4]>,
}

fn get_bones<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
    has_joints: bool
) -> Result<Option<Bones>> {
    if has_joints {
        // TODO: Remove when gltf-utils is fixed.
        // let joints = get_joints(primitive, buffers)?;
        let joints = get_joints_work_around(primitive, buffers)?;
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
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 3]>> {
    let iter = primitive.positions(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|pos| {
        // Transform coordinates from gltf to vulkan by rotating 180deg around X-axis.
        let position = Vector4::<f32>::new(pos[0], pos[1], pos[2], 1.0);
        let transform = Matrix4::new(
            1.0,  0.0,  0.0, 0.0,
            0.0, -1.0,  0.0, 0.0,
            0.0, -0.0, -1.0, 0.0,
            0.0,  0.0,  0.0, 1.0,
        );
        let pos2 = transform * position;

        [pos2.x, pos2.y, pos2.z]
    }).collect::<Vec<_>>())
}

fn get_normals<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 3]>> {
    let iter = primitive.normals(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.map(|norm| {
        // Transform coordinates from gltf to vulkan by rotating 180deg around X-axis.
        let normal = Vector4::<f32>::new(norm[0], norm[1], norm[2], 1.0);
        let transform = Matrix4::new(
            1.0,  0.0,  0.0, 0.0,
            0.0, -1.0,  0.0, 0.0,
            0.0, -0.0, -1.0, 0.0,
            0.0,  0.0,  0.0, 1.0,
        );
        let norm2 = transform * normal;

        [norm2.x, norm2.y, norm2.z]
    }).collect::<Vec<_>>())
}

fn get_texcoords_0<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 2]>> {
    let iter = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.collect::<Vec<_>>())
}

fn get_texcoords_1<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Option<Vec<[f32; 2]>> {
    if let Some(iter) = primitive.tex_coords_f32(1, buffers) {
        Some(iter.collect::<Vec<_>>())
    } else { None }

}

fn get_tangents<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Option<Vec<[f32; 4]>> {
    if let Some(iter) = primitive.tangents(buffers) {
        Some(iter.map(|tang| {
            // Transform coordinates from gltf to vulkan by rotating 180deg around X-axis.
            let tangent = Vector4::from(tang);
            let transform = Matrix4::new(
                1.0,  0.0,  0.0, 0.0,
                0.0, -1.0,  0.0, 0.0,
                0.0, -0.0, -1.0, 0.0,
                0.0,  0.0,  0.0, 1.0,
            );
            let tang2 = transform * tangent;

            [tang2.x, tang2.y, tang2.z, tang2.w]
        }).collect::<Vec<_>>())
    } else { None }
}

fn get_joints<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[u16; 4]>> {
    let iter = primitive.joints_u16(0, buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.collect::<Vec<_>>())
}

fn get_weights<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 4]>> {
    let iter = primitive.weights_f32(0, buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.collect::<Vec<_>>())
}

fn get_indices<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<u32>> {
    let iter = primitive.indices_u32(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.collect::<Vec<_>>())
}

// Ugly hack to workaround a bug in gltf-utils 0.9.2 
// TODO: Remove when gltf-utils is fixed.
fn get_joints_work_around<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&gltf_mesh::Semantic::Joints(0))
        .ok_or(ConvertError::MissingAttributes)?;

    let contents = buffers.view(&access.view()).ok_or(ConvertError::Other)?;
    let mut offset = access.offset();

    match access.dimensions() {
        gltfacc::Dimensions::Vec4 => {
            match access.data_type() {
                gltfacc::DataType::U8 => {
                    let size = mem::size_of::<[u8; 4]>();
                    let mut joints = Vec::<[u16; 4]>::with_capacity(access.count());

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &contents[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let j0 = cursor.read_u8()?;
                        let jj0 = j0 as u16;
                        let j1 = cursor.read_u8()?;
                        let jj1 = j1 as u16;
                        let j2 = cursor.read_u8()?;
                        let jj2 = j2 as u16;
                        let j3 = cursor.read_u8()?;
                        let jj3 = j3 as u16;
                        joints.push([jj0, jj1, jj2, jj3]);

                        offset = offset + access.view().stride().unwrap_or(access.size());
                    }

                    Ok(joints)
                },
                gltfacc::DataType::U16 => {
                    let size = mem::size_of::<[u16; 4]>();
                    let mut joints = Vec::<[u16; 4]>::with_capacity(access.count());

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &contents[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let j0 = cursor.read_u16::<LittleEndian>()?;
                        let j1 = cursor.read_u16::<LittleEndian>()?;
                        let j2 = cursor.read_u16::<LittleEndian>()?;
                        let j3 = cursor.read_u16::<LittleEndian>()?;
                        joints.push([j0, j1, j2, j3]);

                        offset = offset + access.view().stride().unwrap_or(access.size());
                    }

                    Ok(joints)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}
