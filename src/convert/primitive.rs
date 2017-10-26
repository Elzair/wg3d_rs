use std::io;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Vector4, Matrix4};
use gltf::accessor::{DataType, Dimensions};
use gltf::mesh::Semantic;
use gltf::mesh as gltf_mesh;

use super::super::{Result, Error};
use super::ConvertError;
use super::buffer::Buffers;
use super::material::{Material, get as get_material};
use super::texture::Textures;

pub struct Primitive {
    vertex_attributes: VertexAttributes,
    indices: Option<Vec<u32>>,
    material: Material,
}

pub fn get<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
    textures: &'a Textures,
    has_bones: bool,
) -> Result<Primitive> {
    let vertex_attributes = get_vertex_attributes(primitive, buffers, has_bones)?;
    let indices = get_indices(primitive, buffers)?;
    let material = get_material(primitive, textures)?;

    Ok(Primitive {
        vertex_attributes: vertex_attributes,
        indices: indices,
        material: material,
    })
}

pub struct VertexAttributes {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords_0: Vec<[u16; 2]>,
    texcoords_1: Option<Vec<[u16; 2]>>,
    tangents: Option<Vec<[f32; 4]>>,
    bones: Option<Bones>,
}

fn get_vertex_attributes<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
    has_joints: bool,
) -> Result<VertexAttributes> {
    let positions = get_positions(primitive, buffers)?;
    let normals = get_normals(primitive, buffers)?;
    let texcoords_0 = get_texcoords(primitive, 0, buffers)?;

    let texcoords_1 = if let Some(_) = primitive.get(&Semantic::TexCoords(1)) {
        let res = get_texcoords(primitive, 0, buffers)?;
        Some(res)
    } else { None };

    // Retrieve tangents (if they exist) and also compute bitangents.
    let tangents = if let Some(_) = primitive.get(&Semantic::Tangents) {
        let res = get_tangents(primitive, buffers)?;
        Some(res)
    } else { None };

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
    weights: Vec<[u16; 4]>,
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

fn get_indices<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Option<Vec<u32>>> {
    let tmp = primitive.indices();

    if tmp.is_none() {
        return Ok(None);
    }

    let access = tmp.unwrap();
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Scalar => {
            match access.data_type() {
                DataType::U8 => {
                    let size = mem::size_of::<u8>();
                    let mut indices = Vec::<u32>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let idx = cursor.read_u8()?;
                        indices.push(idx as u32);

                        offset = offset + size;
                    }

                    Ok(Some(indices))
                },
                DataType::U16 => {
                    let size = mem::size_of::<u16>();
                    let mut indices = Vec::<u32>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let idx = cursor.read_u16::<LittleEndian>()?;
                        indices.push(idx as u32);

                        offset = offset + size;
                    }

                    Ok(Some(indices))
                },
                DataType::U32 => {
                    let size = mem::size_of::<u32>();
                    let mut indices = Vec::<u32>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let idx = cursor.read_u32::<LittleEndian>()?;
                        indices.push(idx);

                        offset = offset + size;
                    }

                    Ok(Some(indices))
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

// Vertex Attribute Methods

fn get_positions<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 3]>> {
    let access = primitive.get(&Semantic::Positions)
        .ok_or(ConvertError::MissingAttributes)?;
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
                        // Transform coordinates from gltf coordinate space to vulkan coordinate space.
                        let position = Vector4::<f32>::new(x, y, z, 1.0);
                        let rotation_matrix = Matrix4::new(
                            1.0,  0.0,  0.0, 0.0,
                            0.0, -1.0, -0.0, 0.0,
                            0.0,  0.0, -1.0, 0.0,
                            0.0,  0.0,  0.0, 1.0
                        );
                        let pos = rotation_matrix * position;
                        positions.push([pos.x, pos.y, pos.z]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(positions)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_normals<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 3]>> {
    let access = primitive.get(&Semantic::Normals)
        .ok_or(ConvertError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec3=> {
            match access.data_type() {
                DataType::F32  => {
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
                        // Transform coordinates from gltf coordinate space to vulkan coordinate space.
                        let normal = Vector4::<f32>::new(x, y, z, 1.0);
                        let rotation_matrix = Matrix4::new(
                            1.0,  0.0,  0.0, 0.0,
                            0.0, -1.0, -0.0, 0.0,
                            0.0,  0.0, -1.0, 0.0,
                            0.0,  0.0,  0.0, 1.0
                        );
                        let norm = rotation_matrix * normal;
                        normals.push([norm.x, norm.y, norm.z]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(normals)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
    }
}

fn get_tangents<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[f32; 4]>> {
    let access = primitive.get(&Semantic::Tangents)
        .ok_or(ConvertError::MissingAttributes)?;
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
                        // Transform coordinates from gltf coordinate space to vulkan coordinate space.
                        let tangent = Vector4::<f32>::new(x, y, z, w);
                        let rotation_matrix = Matrix4::new(
                            1.0,  0.0,  0.0, 0.0,
                            0.0, -1.0, -0.0, 0.0,
                            0.0,  0.0, -1.0, 0.0,
                            0.0,  0.0,  0.0, 1.0
                        );
                        let tang = rotation_matrix * tangent ;
                        tangents.push([tang.x, tang.y, tang.z, tang.w]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(tangents)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_texcoords<'a>(
    primitive: &'a gltf_mesh::Primitive,
    index: u32,
    buffers: &'a Buffers,
) -> Result<Vec<[u16; 2]>> {
    let access = primitive.get(&Semantic::TexCoords(index))
        .ok_or(ConvertError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec2 => {
            match access.data_type() {
                DataType::U8 => {
                    let size = mem::size_of::<[u8; 2]>();
                    let mut coords = Vec::<[u16; 2]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let s = cursor.read_u8()?;
                        let ss = (s as u16) << 8;
                        let t = cursor.read_u8()?;
                        let tt = (t as u16) << 8;
                        // Flip t to transfrom from gltf coordinate space to vulkan coordinate space.
                        let ttt = u16::max_value() - tt;
                        coords.push([ss, ttt]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(coords)
                },
                DataType::U16 => {
                    let size = mem::size_of::<[u16; 2]>();
                    let mut coords = Vec::<[u16; 2]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let s = cursor.read_u16::<LittleEndian>()?;
                        let t = cursor.read_u16::<LittleEndian>()?;
                        // Flip t to transfrom from gltf coordinate space to vulkan coordinate space.
                        let tt = u16::max_value() - t;
                        coords.push([s, tt]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(coords)
                },
                DataType::F32 => {
                    let size = mem::size_of::<[f32; 2]>();
                    let mut coords = Vec::<[u16; 2]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let s = cursor.read_f32::<LittleEndian>()?;
                        let ss = (s * (u16::max_value() as f32)).round() as u16;
                        let t = cursor.read_f32::<LittleEndian>()?;
                        let tt = (t * (u16::max_value() as f32)).round() as u16;
                        // Flip t to transfrom from gltf coordinate space to vulkan coordinate space.
                        let ttt = u16::max_value() - tt;
                        coords.push([ss, ttt]);
                        
                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(coords)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
    }
}

fn get_joints<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&Semantic::Joints(0))
        .ok_or(ConvertError::MissingAttributes)?;
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

                        let j0 = cursor.read_u8()?;
                        let jj0 = j0 as u16;
                        let j1 = cursor.read_u8()?;
                        let jj1 = j1 as u16;
                        let j2 = cursor.read_u8()?;
                        let jj2 = j2 as u16;
                        let j3 = cursor.read_u8()?;
                        let jj3 = j3 as u16;
                        joints.push([jj0, jj1, jj2, jj3]);

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
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_weights<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&Semantic::Joints(0))
        .ok_or(ConvertError::MissingAttributes)?;
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
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

