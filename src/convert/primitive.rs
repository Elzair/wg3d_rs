use cgmath::{Vector2, Vector3, Vector4};
use gltf::mesh::{Primitive as GltfPrimitive, Primitives as GltfPrimitives};
use gltf_importer::Buffers;
use gltf_utils::PrimitiveIterators;
use itertools::multizip;

use super::super::{Result, Error};
use super::ConvertError;
use super::material::Materials;
use super::morph_target::{MorphTarget, get as get_morph_targets};
use super::texture::Texture;

pub struct Primitive {
    material: String,
    attributes: Attributes,
    indices: Vec<u32>,
}

pub fn get<'a>(
    primitives: GltfPrimitives,
    weights: Option<&'a [f32]>,
    has_joints: bool,
    buffers: &'a Buffers,
    materials: &'a Materials,
) -> Result<Vec<Primitive>> {
    primitives.map(|primitive| {
        // let morph_targets = get_morph_targets(primitive, buffers)?;

        // The default material is not supported.
        let material_index = match primitive.material().index() {
            Some(index) => index,
            None => { return Err(Error::Convert(ConvertError::NoMaterial)); },
        };
        let material = materials.get(material_index)
            .ok_or(ConvertError::Other)?;
        let attributes = get_attributes(
            &primitive,
            has_joints,
            buffers,
        )?;
        let indices = get_indices(&primitive, buffers)?;

        Ok(Primitive {
            material: material.to_owned(),
            attributes: attributes,
            indices: indices,
        })
    }).collect()
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
    primitive: &'a GltfPrimitive,
    has_joints: bool,
    buffers: &'a Buffers,
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

fn get_indices<'a>(
    primitive: &'a GltfPrimitive,
    buffers: &'a Buffers,
) -> Result<Vec<u32>> {
    let iter = primitive.indices_u32(buffers).ok_or(ConvertError::MissingAttributes)?;

    Ok(iter.collect::<Vec<_>>())
}
