use std::path::Path;
use std::io;
use std::io::Read;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt};
use gltf::{Gltf, material};
use gltf::accessor::{DataType, Dimensions};
use gltf::mesh::{Primitive, Semantic};

use super::super::{Result, Error, Wg3dError};
use super::buffer::Buffers;

pub struct Mesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords_0: Vec<[f32; 2]>,
    texcoords_1: Option<Vec<[f32; 2]>>,
    tangents: Option<Vec<[f32; 4]>>,
    joints: Option<Vec<[u16; 4]>>,
    weights: Option<Vec<[u16; 4]>>,
}

pub fn get_mesh<'a>(
    gltf: &'a Gltf,
    index: usize,
    has_joints: bool,
    buffers: &'a Buffers
) -> Result<Vec<Mesh>> {
    let mut meshes = Vec::<Mesh>::new();
    
    for mesh in gltf.meshes() {
        if mesh.index() == index {
            for prim in mesh.primitives() {
                let material = get_material(&prim);

                let positions = get_positions(&prim, buffers)?;
                let normals = get_normals(&prim, buffers)?;
                let texcoords_0 = get_texcoords(&prim, 0, buffers)?;

                let texcoords_1 = if let Some(_) = prim.get(&Semantic::TexCoords(1)) {
                    let res = get_texcoords(&prim, 0, buffers)?;
                    Some(res)
                } else { None };

                // Retrieve tangents (if they exist) and also compute bitangents.
                let tangents = if let Some(_) = prim.get(&Semantic::Tangents) {
                    let res = get_tangents(&prim, buffers)?;
                    Some(res)
                } else { None };

                let (joints, weights) = if has_joints {
                    let joints = get_joints(&prim, buffers)?;
                    let weights = get_weights(&prim, buffers)?;
                    (Some(joints), Some(weights))
                } else { (None, None) };

                meshes.push(Mesh {
                    positions: positions,
                    normals: normals,
                    texcoords_0: texcoords_0,
                    texcoords_1: texcoords_1,
                    tangents: tangents,
                    joints: joints,
                    weights: weights,
                });
            }
        }
    }

    Ok(meshes)
}

// Material Methods

enum AlphaMode {
    Blend,
    Mask,
    Opaque,
}

fn get_material<'a>(
    primitive: &'a Primitive,
) -> Result<()> {
    let material = primitive.material();

    let alpha_cutoff = material.alpha_cutoff();
    let alpha_mode = match material.alpha_mode() {
        material::AlphaMode::Blend => AlphaMode::Blend,
        material::AlphaMode::Mask => AlphaMode::Mask,
        material::AlphaMode::Opaque => AlphaMode::Opaque,
    };
    let double_sided = material.double_sided();
    
    Ok(())
}

struct NormalTexture {
    scale: f32,
    tex_corrd: u32,
    contents: Vec<u8>,
}

fn get_normal_texture<'a>(
    material: &'a material::Material,
) -> Result<Option<()>> {
    let normal_texture = match material.normal_texture() {
        Some(tex) => tex,
        None => { return Ok(None); },
    };

    let scale = normal_texture.scale();
    let tex_coord = normal_texture.tex_coord();
    let texture = normal_texture.texture();
    

    Ok(Some(()))
}

// Vertex Attribute Methods

fn get_positions<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 3]>> {
    let access = primitive.get(&Semantic::Positions)
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec3 => {
            match access.data_type() {
                DataType::F32 => {
                    let size = mem::size_of::<[f32; 3]>();
                    let mut positions = Vec::<[f32; 3]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;
                        positions.push([x, y, z]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(positions)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

fn get_normals<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 3]>> {
    let access = primitive.get(&Semantic::Normals)
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.data_type() {
        DataType::F32 => {
            match access.dimensions() {
                Dimensions::Vec3 => {
                    let size = mem::size_of::<[f32; 3]>();
                    let mut normals = Vec::<[f32; 3]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;
                        normals.push([x, y, z]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(normals)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
    }
}

fn get_texcoords<'a>(
    primitive: &'a Primitive,
    index: u32,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 2]>> {
    let access = primitive.get(&Semantic::TexCoords(index))
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.data_type() {
        DataType::F32 => {
            match access.dimensions() {
                Dimensions::Vec2 => {
                    let size = mem::size_of::<[f32; 2]>();
                    let mut tex0 = Vec::<[f32; 2]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let u = cursor.read_f32::<LittleEndian>()?;
                        let v = cursor.read_f32::<LittleEndian>()?;
                        tex0.push([u, v]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(tex0)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
    }
}

fn get_tangents<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 4]>> {
    let access = primitive.get(&Semantic::Tangents)
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec4 => {
            match access.data_type() {
                DataType::F32 => {
                    let size = mem::size_of::<[f32; 4]>();
                    let mut tangents = Vec::<[f32; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;
                        let w = cursor.read_f32::<LittleEndian>()?;
                        tangents.push([x, y, z, w]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(tangents)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

fn get_joints<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&Semantic::Joints(0))
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec4 => {
            match access.data_type() {
                DataType::U8 => {
                    let size = mem::size_of::<[u8; 4]>();
                    let mut joints = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let jj0 = cursor.read_u8()?;
                        let j0 = (jj0 as u16) << 8;
                        let jj1 = cursor.read_u8()?;
                        let j1 = (jj1 as u16) << 8;
                        let jj2 = cursor.read_u8()?;
                        let j2 = (jj2 as u16) << 8;
                        let jj3 = cursor.read_u8()?;
                        let j3 = (jj3 as u16) << 8;
                        joints.push([j0, j1, j2, j3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(joints)
                },
                DataType::U16 => {
                    let size = mem::size_of::<[u16; 4]>();
                    let mut joints = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let j0 = cursor.read_u16::<LittleEndian>()?;
                        let j1 = cursor.read_u16::<LittleEndian>()?;
                        let j2 = cursor.read_u16::<LittleEndian>()?;
                        let j3 = cursor.read_u16::<LittleEndian>()?;
                        joints.push([j0, j1, j2, j3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(joints)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

fn get_weights<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&Semantic::Joints(0))
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec4 => {
            match access.data_type() {
                DataType::U8 => {
                    let size = mem::size_of::<[u8; 4]>();
                    let mut weights = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let ww0 = cursor.read_u8()?;
                        let w0 = (ww0 as u16) << 8;
                        let ww1 = cursor.read_u8()?;
                        let w1 = (ww1 as u16) << 8;
                        let ww2 = cursor.read_u8()?;
                        let w2 = (ww2 as u16) << 8;
                        let ww3 = cursor.read_u8()?;
                        let w3 = (ww3 as u16) << 8;
                        weights.push([w0 as u16, w1 as u16, w2 as u16, w3 as u16]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(weights)
                },
                DataType::U16 => {
                    let size = mem::size_of::<[u16; 4]>();
                    let mut weights = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let w0 = cursor.read_u16::<LittleEndian>()?;
                        let w1 = cursor.read_u16::<LittleEndian>()?;
                        let w2 = cursor.read_u16::<LittleEndian>()?;
                        let w3 = cursor.read_u16::<LittleEndian>()?;
                        weights.push([w0, w1, w2, w3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(weights)
                },
                DataType::F32 => {
                    let size = mem::size_of::<[f32; 4]>();
                    let mut weights = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let ww0 = cursor.read_f32::<LittleEndian>()?;
                        let w0 = (ww0 * (u16::max_value() as f32)).round() as u16;
                        let ww1 = cursor.read_f32::<LittleEndian>()?;
                        let w1 = (ww1 * (u16::max_value() as f32)).round() as u16;
                        let ww2 = cursor.read_f32::<LittleEndian>()?;
                        let w2 = (ww2 * (u16::max_value() as f32)).round() as u16;
                        let ww3 = cursor.read_f32::<LittleEndian>()?;
                        let w3 = (ww3 * (u16::max_value() as f32)).round() as u16;
                        weights.push([w0, w1, w2, w3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(weights)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

