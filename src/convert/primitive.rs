use cgmath::{Vector2, Vector3, Vector4, Matrix4};
use gltf::mesh as gltf_mesh;
use gltf_importer::Buffers;
use gltf_utils::PrimitiveIterators;
use itertools::multizip;

use super::super::{Result, Error};
use super::ConvertError;
use super::material::{Material, get as get_material};
use super::texture::Texture;

pub struct Primitive {
    material: Material,
    attributes: Attributes,
    indices: Vec<u32>,
}

pub fn get<'a>(
    primitive: &'a gltf_mesh::Primitive,
    has_bones: bool,
    buffers: &'a Buffers,
    textures: &'a Vec<Texture>,
) -> Result<Primitive> {
    let material = get_material(primitive, textures)?;
    let attributes = get_attributes(
        primitive,
        buffers,
        has_bones
    )?;
    let indices = get_indices(primitive, buffers)?;

    Ok(Primitive {
        material: material,
        attributes: attributes,
        indices: indices,
    })
}

pub enum Attributes {
    NoTex1NoTangentNoBones(Vec<VertexNoTex1NoTangentNoBones>),
    NoTex1NoTangentBones(Vec<VertexNoTex1NoTangentBones>),
    NoTex1TangentNoBones(Vec<VertexNoTex1TangentNoBones>),
    NoTex1TangentBones(Vec<VertexNoTex1TangentBones>),
    Tex1NoTangentNoBones(Vec<VertexTex1NoTangentNoBones>),
    Tex1NoTangentBones(Vec<VertexTex1NoTangentBones>),
    Tex1TangentNoBones(Vec<VertexTex1TangentNoBones>),
    Tex1TangentBones(Vec<VertexTex1TangentBones>),
}

fn get_attributes<'a>(
    primitive: &'a gltf_mesh::Primitive,
    buffers: &'a Buffers,
    has_joints: bool,
) -> Result<Attributes> {
    // Common iterators and their number of elements
    let pos_num = primitive.positions(buffers).ok_or(ConvertError::MissingAttributes)?.count();
    let pos_it = primitive.positions(buffers).ok_or(ConvertError::MissingAttributes)?;
    let nor_num = primitive.normals(buffers).ok_or(ConvertError::MissingAttributes)?.count();
    let nor_it = primitive.normals(buffers).ok_or(ConvertError::MissingAttributes)?;
    let tx0_num = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::MissingAttributes)?.count();
    let tx0_it = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?;

    let has_tangents = primitive.tangents(buffers).is_some();
    let has_texcoords_1 = primitive.tex_coords_f32(1, buffers).is_some();

    if has_texcoords_1 && has_tangents && has_joints {
        let tx1_num = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let tx1_it = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?;
        let tan_num = primitive.tangents(buffers).ok_or(ConvertError::Other)?.count();
        let tan_it = primitive.tangents(buffers).ok_or(ConvertError::Other)?;
        let id0_num = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?.count();
        let id0_it = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?;
        let wt0_num = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let wt0_it = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?;

        // Test all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == tx1_num && tx1_num == tan_num && tan_num == id0_num && id0_num == wt0_num {
            
            Ok(Attributes::Tex1TangentBones(multizip((pos_it, nor_it, tx0_it, tx1_it, tan_it, id0_it, wt0_it))
               .map(|(pos, norm, tx0, tx1, tang, ids, wts)| {
                   VertexTex1TangentBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       texcoord1: Vector2::<f32>::from(tx1),
                       tangent: Vector4::<f32>::from(tang),
                       joints: Vector4::<u16>::from(ids),
                       weights: Vector4::<f32>::from(wts),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }
    } else if has_texcoords_1 && has_tangents && !has_joints {
        let tx1_num = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let tx1_it = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?;
        let tan_num = primitive.tangents(buffers).ok_or(ConvertError::Other)?.count();
        let tan_it = primitive.tangents(buffers).ok_or(ConvertError::Other)?;

        // Test all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == tx1_num && tx1_num == tan_num {
            
            Ok(Attributes::Tex1TangentNoBones(multizip((pos_it, nor_it, tx0_it, tx1_it, tan_it))
               .map(|(pos, norm, tx0, tx1, tang)| {
                   VertexTex1TangentNoBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       texcoord1: Vector2::<f32>::from(tx1),
                       tangent: Vector4::<f32>::from(tang),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }
    } else if has_texcoords_1 && !has_tangents && has_joints {
        let tx1_num = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let tx1_it = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?;
        let id0_num = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?.count();
        let id0_it = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?;
        let wt0_num = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let wt0_it = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?;
        
        // Ensure all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == tx1_num && tx1_num == id0_num && id0_num == wt0_num {
            
            Ok(Attributes::Tex1NoTangentBones(multizip((pos_it, nor_it, tx0_it, tx1_it, id0_it, wt0_it))
               .map(|(pos, norm, tx0, tx1, ids, wts)| {
                   VertexTex1NoTangentBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       texcoord1: Vector2::<f32>::from(tx1),
                       joints: Vector4::<u16>::from(ids),
                       weights: Vector4::<f32>::from(wts),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }
    } else if has_texcoords_1 && !has_tangents && !has_joints {
        let tx1_num = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let tx1_it = primitive.tex_coords_f32(0, buffers).ok_or(ConvertError::Other)?;
        
        // Ensure all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == tx1_num {
            
            Ok(Attributes::Tex1NoTangentNoBones(multizip((pos_it, nor_it, tx0_it, tx1_it))
               .map(|(pos, norm, tx0, tx1)| {
                   VertexTex1NoTangentNoBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       texcoord1: Vector2::<f32>::from(tx1),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }

    } else if !has_texcoords_1 && has_tangents && has_joints {
        let tan_num = primitive.tangents(buffers).ok_or(ConvertError::Other)?.count();
        let tan_it = primitive.tangents(buffers).ok_or(ConvertError::Other)?;
        let id0_num = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?.count();
        let id0_it = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?;
        let wt0_num = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let wt0_it = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?;
        
        // Ensure all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == tan_num && tan_num == id0_num && id0_num == wt0_num {
            
            Ok(Attributes::NoTex1TangentBones(multizip((pos_it, nor_it, tx0_it, tan_it, id0_it, wt0_it))
               .map(|(pos, norm, tx0, tang, ids, wts)| {
                   VertexNoTex1TangentBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       tangent: Vector4::<f32>::from(tang),
                       joints: Vector4::<u16>::from(ids),
                       weights: Vector4::<f32>::from(wts),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }

    } else if !has_texcoords_1 && has_tangents && !has_joints {
        let tan_num = primitive.tangents(buffers).ok_or(ConvertError::Other)?.count();
        let tan_it = primitive.tangents(buffers).ok_or(ConvertError::Other)?;
        
        // Ensure all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == tan_num {
            
            Ok(Attributes::NoTex1TangentNoBones(multizip((pos_it, nor_it, tx0_it, tan_it))
               .map(|(pos, norm, tx0, tang)| {
                   VertexNoTex1TangentNoBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       tangent: Vector4::<f32>::from(tang),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }

    } else if !has_texcoords_1 && !has_tangents && has_joints {
        let id0_num = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?.count();
        let id0_it = primitive.joints_u16(0, buffers).ok_or(ConvertError::Other)?;
        let wt0_num = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?.count();
        let wt0_it = primitive.weights_f32(0, buffers).ok_or(ConvertError::Other)?;
        
        // Ensure all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num && tx0_num == id0_num && id0_num == wt0_num {
            
            Ok(Attributes::NoTex1NoTangentBones(multizip((pos_it, nor_it, tx0_it, id0_it, wt0_it))
               .map(|(pos, norm, tx0, ids, wts)| {
                   VertexNoTex1NoTangentBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                       joints: Vector4::<u16>::from(ids),
                       weights: Vector4::<f32>::from(wts),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }
    } else {
        // Test all vertex attributes have the same number of elements.
        if pos_num == nor_num && nor_num == tx0_num {
            
            Ok(Attributes::NoTex1NoTangentNoBones(multizip((pos_it, nor_it, tx0_it))
               .map(|(pos, norm, tx0)| {
                   VertexNoTex1NoTangentNoBones {
                       position: Vector3::<f32>::from(pos),
                       normal: Vector3::<f32>::from(norm),
                       texcoord0: Vector2::<f32>::from(tx0),
                   }
               }).collect()))
        } else {
            Err(Error::Convert(ConvertError::Other))
        }
    }
}

pub struct VertexNoTex1NoTangentNoBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
}

pub struct VertexNoTex1NoTangentBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    joints: Vector4<u16>,
    weights: Vector4<f32>,
}

pub struct VertexNoTex1TangentNoBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    tangent: Vector4<f32>,
}

pub struct VertexNoTex1TangentBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    tangent: Vector4<f32>,
    joints: Vector4<u16>,
    weights: Vector4<f32>,
}

pub struct VertexTex1NoTangentNoBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    texcoord1: Vector2<f32>,
}

pub struct VertexTex1NoTangentBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    texcoord1: Vector2<f32>,
    joints: Vector4<u16>,
    weights: Vector4<f32>,
}

pub struct VertexTex1TangentNoBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    texcoord1: Vector2<f32>,
    tangent: Vector4<f32>,
}

pub struct VertexTex1TangentBones {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    texcoord0: Vector2<f32>,
    texcoord1: Vector2<f32>,
    tangent: Vector4<f32>,
    joints: Vector4<u16>,
    weights: Vector4<f32>,
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
