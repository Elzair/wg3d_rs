use std::io;

use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Matrix4, SquareMatrix};
use gltf::accessor::{DataType, Dimensions};
use gltf::scene::Node as GltfNode;
use gltf::skin::Skin as GltfSkin;
use gltf_importer::Buffers;
use itertools::multizip;

use super::super::{Result, Error};
use super::ConvertError;

pub struct Skin {
    root: usize,
    joints: Vec<Joint>,
}

pub fn get<'a>(
    skin: Option<GltfSkin>,
    buffers: &'a Buffers,
) -> Result<Option<Skin>> {
    if skin.is_none() {
        return Ok(None);
    }
    let sk = skin.unwrap();

    let root = {
        let root_node_index = sk.skeleton().ok_or(ConvertError::NoSkeleton)?.index();
        sk.joints().find(|ref joint| joint.index() == root_node_index).unwrap().index()
    };

    let joints = get_joints(&sk, buffers)?;

    Ok(Some(Skin {
        root: root,
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

fn get_children_indices<'a>(
    skin: &'a GltfSkin,
) -> Vec<Vec<usize>> {
    let len = skin.joints().count();
    let mapping = skin.joints().map(|joint| joint.index())
        .zip((0..len).into_iter())
        .collect::<Vec<_>>();

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

                                // TODO: Determine if we need to multiply this with
                                // a rotation matrix of 180 degrees around X axis.
                                ibms.push(Matrix4::new(
                                    c0r0, c0r1, c0r2, c0r3,
                                    c1r0, c1r1, c1r2, c1r3,
                                    c2r0, c2r1, c2r2, c2r3,
                                    c3r0, c3r1, c3r2, c3r3,
                                ));
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

// Retrieve indices of the parent nodes of joint nodes
fn get_parents<'a>(
    root: &'a GltfNode,
    indices: &'a Vec<usize>,
) -> Vec<usize> {
    // Get indices from nodes array.
    let mut pairs = Vec::<(usize, usize)>::new();
    pairs.push((root.index(), usize::max_value()));
    get_node(root, &mut pairs);

    // Convert indices from nodes array to joints array (to match up with JOINTS_0 data).
    let mut parents = Vec::<usize>::with_capacity(pairs.len());

    let mut j = 0;

    for index in indices {
        for (index2, parent) in pairs.clone() {
            if *index == index2 {
                if parent == usize::max_value() {
                    parents.push(usize::max_value());
                } else {
                    // Find index of parent in joints array.
                    j = 0;

                    for parent2 in indices {
                        if parent == *parent2 {
                            parents.push(j);
                        }

                        j += 1;
                    }
                }
            }
        }
    }

    parents
}

// Recursive helper function
fn get_node<'a>(
    node: &'a GltfNode,
    pair: &'a mut Vec<(usize, usize)>,
) {
    for child in node.children() {
        pair.push((child.index(), node.index()));
        get_node(&child, pair);
    }
}
