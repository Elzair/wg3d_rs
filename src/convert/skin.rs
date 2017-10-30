use std::io;

use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Matrix4, SquareMatrix};
use gltf::accessor::{DataType, Dimensions};
use gltf::skin::Skin as GltfSkin;
use gltf_importer::Buffers;
use itertools::multizip;

use super::super::{Result, Error};
use super::ConvertError;

pub struct Skin {
    root_index: usize,
    transform: Matrix4<f32>,
    joints: Vec<Joint>,
}

pub fn get<'a>(
    skin: Option<GltfSkin>,
    transform: Matrix4<f32>,
    buffers: &'a Buffers,
) -> Result<Option<Skin>> {
    if skin.is_none() {
        return Ok(None);
    }

    let sk = skin.unwrap();
    let root_index = get_root_index(&sk)?;
    let joints = get_joints(&sk, buffers)?;

    Ok(Some(Skin {
        root_index: root_index,
        transform: transform,
        joints: joints,
    }))
}

pub struct Joint {
    local_transform: Matrix4<f32>,
    inverse_bind_matrix: Matrix4<f32>,
    children: Vec<usize>,
}

fn get_joints<'a>(
    skin: &'a GltfSkin,
    buffers: &'a Buffers,
) -> Result<Vec<Joint>> {
    let transforms = skin.joints().map(|joint| {
        Matrix4::<f32>::from(joint.transform().matrix())
    }).collect::<Vec<_>>();
    let inverse_bind_matrices = get_inverse_bind_matrices(&skin, buffers)?;
    let child_indices = get_children_indices(skin);

    Ok(multizip((transforms, inverse_bind_matrices, child_indices))
        .map(|(transform, ibm, children)| {
            Joint {
                local_transform: transform,
                inverse_bind_matrix: ibm,
                children: children,
            }
        }).collect::<Vec<_>>())
}

fn get_root_index<'a>(
    skin: &'a GltfSkin,
) -> Result<usize> {
    // Get index in nodes array of root joint node.
    let root_node_index = skin.skeleton().ok_or(ConvertError::NoSkeleton)?.index();

    // Get mapping of `nodes` indices to `joints` indices.
    let len = skin.joints().count();
    let mapping = skin.joints().map(|joint| joint.index())
        .zip((0..len).into_iter())
        .collect::<Vec<_>>();

    let root_index = mapping.iter().find(|&&m| m.0 == root_node_index)
        .ok_or(ConvertError::NoSkeleton)?.1;
    Ok(root_index)
}

fn get_children_indices<'a>(
    skin: &'a GltfSkin,
) -> Vec<Vec<usize>> {
    // Get mapping of `nodes` indices to `joints` indices.
    let len = skin.joints().count();
    let mapping = skin.joints().map(|joint| joint.index())
        .zip((0..len).into_iter())
        .collect::<Vec<_>>();

    // Find `joints` indices for all child joints.
    skin.joints().map(|joint| {
        joint.children().map(|child| {
            mapping.iter().find(|&&m| m.0 == child.index()).unwrap().1
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>()
}

fn get_inverse_bind_matrices<'a>(
    skin: &'a GltfSkin,
    buffers: &'a Buffers,
) -> Result<Vec<Matrix4<f32>>> {
    match skin.inverse_bind_matrices() {
        Some(access) => {
            let contents = buffers.view(&access.view()).ok_or(ConvertError::Other)?;
            let mut offset = access.offset();

            match access.dimensions() {
                Dimensions::Mat4 => {
                    match access.data_type() {
                        DataType::F32 => {
                            let mut ibms = Vec::<Matrix4<f32>>::with_capacity(access.count());

                            #[allow(unused_variables)]
                            for i in 0..access.count() {
                                let sl = &contents[offset..(offset + access.size())];
                                let mut cursor = io::Cursor::new(sl);

                                let c0r0 = cursor.read_f32::<LittleEndian>()?;
                                let c0r1 = cursor.read_f32::<LittleEndian>()?;
                                let c0r2 = cursor.read_f32::<LittleEndian>()?;
                                let c0r3 = cursor.read_f32::<LittleEndian>()?;
                                let c1r0 = cursor.read_f32::<LittleEndian>()?;
                                let c1r1 = cursor.read_f32::<LittleEndian>()?;
                                let c1r2 = cursor.read_f32::<LittleEndian>()?;
                                let c1r3 = cursor.read_f32::<LittleEndian>()?;
                                let c2r0 = cursor.read_f32::<LittleEndian>()?;
                                let c2r1 = cursor.read_f32::<LittleEndian>()?;
                                let c2r2 = cursor.read_f32::<LittleEndian>()?;
                                let c2r3 = cursor.read_f32::<LittleEndian>()?;
                                let c3r0 = cursor.read_f32::<LittleEndian>()?;
                                let c3r1 = cursor.read_f32::<LittleEndian>()?;
                                let c3r2 = cursor.read_f32::<LittleEndian>()?;
                                let c3r3 = cursor.read_f32::<LittleEndian>()?;

                                ibms.push(Matrix4::new(
                                    c0r0, c0r1, c0r2, c0r3,
                                    c1r0, c1r1, c1r2, c1r3,
                                    c2r0, c2r1, c2r2, c2r3,
                                    c3r0, c3r1, c3r2, c3r3,
                                ));

                                offset = offset + access.view().stride().unwrap_or(access.size());
                            }

                            Ok(ibms)
                        },
                        _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
                    }
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
            }
        },
        None => {
            Ok(skin.joints().map(|_| Matrix4::<f32>::identity())
               .collect::<Vec<_>>())
        },
    }
}
