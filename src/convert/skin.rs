use std::u16;
use std::usize;

use cgmath::{Matrix4, SquareMatrix};
use gltf::Gltf;
use gltf::skin::Skin as GltfSkin;
use gltf_importer::Buffers;
use itertools::multizip;

use super::super::{Result, Error};
use super::ConvertError;
use super::util::InverseBindMatrices;

pub struct Skin {
    name: String,
    root_index: u16,
    joints: Vec<Joint>,
}

impl Skin {
    pub fn get_joint_index(&self, node_index: usize) -> Option<u16> {
        let mut index = 0_u16;
        
        for joint in self.joints.iter() {
            if joint.old_index == node_index {
                return Some(index);
            }

            index += 1;
        }

        None
    }
}

pub fn get<'a>(
    gltf: &'a Gltf,
    buffers: &'a Buffers,
) -> Result<Vec<Skin>> {
    gltf.skins().map(|skin| {
        get_skin(&skin, buffers)
    }).collect()
}

fn get_skin<'a>(
    skin: &'a GltfSkin,
    buffers: &'a Buffers,
) -> Result<Skin> {
    let name = skin.name().ok_or(ConvertError::NoName)?;
    let root_index = get_root_index(skin)?;
    let joints = get_joints(skin, buffers)?;

    Ok(Skin {
        name: String::from(name),
        root_index: root_index,
        joints: joints,
    })
}

fn get_root_index<'a>(
    skin: &'a GltfSkin,
) -> Result<u16> {
    // Get index in nodes array of root joint node.
    let root_node_index = skin.skeleton().ok_or(ConvertError::NoSkeleton)?.index();

    // Get mapping of `nodes` indices to `joints` indices.
    let len = skin.joints().count();
    let mapping = skin.joints().map(|joint| joint.index())
        .zip((0..len).into_iter())
        .collect::<Vec<_>>();

    let root_index = mapping.iter().find(|&&m| m.0 == root_node_index)
        .ok_or(ConvertError::NoSkeleton)?.1;

    if root_index >= u16::MAX as usize {
        Err(Error::Convert(ConvertError::TooManyJoints))
    }
    else {
        Ok(root_index as u16)
    }
}

pub struct Joint {
    name: String,
    local_transform: Matrix4<f32>,
    inverse_bind_matrix: Matrix4<f32>,
    parent: u16,
    old_index: usize,
}

fn get_joints<'a>(
    skin: &'a GltfSkin,
    buffers: &'a Buffers,
) -> Result<Vec<Joint>> {
    let names = get_joint_names(skin)?;
    let transforms = skin.joints().map(|joint| {
        Matrix4::<f32>::from(joint.transform().matrix())
    }).collect::<Vec<_>>();
    let inverse_bind_matrices = get_inverse_bind_matrices(&skin, buffers);
    let parent_indices = get_parent_indices(skin)?;
    let old_indices = skin.joints().map(|joint| joint.index()).collect::<Vec<_>>();

    Ok(multizip((names, transforms, inverse_bind_matrices, parent_indices, old_indices))
        .map(|(name, transform, inverse_bind_matrix, parent, old_index)| {
            Joint {
                name: name,
                local_transform: transform,
                inverse_bind_matrix: inverse_bind_matrix,
                parent: parent,
                old_index: old_index,
            }
        }).collect())
}

fn get_joint_names<'a>(
    skin: &'a GltfSkin,
) -> Result<Vec<String>> {
    skin.joints().map(|joint| {
        let name = joint.name().ok_or(ConvertError::NoName)?;
        Ok(String::from(name))
    }).collect()
}

fn get_parent_indices<'a>(
    skin: &'a GltfSkin,
) -> Result<Vec<u16>> {
    // Get mapping of `nodes` indices to `joints` indices.
    let len = skin.joints().count();
    let mapping1 = skin.joints().map(|joint| joint.index())
        .zip((0..len).into_iter())
        .collect::<Vec<_>>();

    // Find `joints` indices for all child joints.
    let mapping2 = skin.joints().map(|joint| {
        joint.children().map(|child| {
            mapping1.iter().find(|&&(nidx, _)| nidx == child.index()).unwrap().1
        }).collect::<Vec<_>>()
    }).zip((0..len).into_iter()).collect::<Vec<_>>();

    // Find `joints` indices for all parents of joints.
    let parent1 = skin.joints().map(|joint| {
        let my_index = mapping1.iter().find(|&&(nidx, _)| nidx == joint.index()).unwrap().1;

        match mapping2.iter().find(|&&(ref children, _)| {
            children.iter().find(|&&child| child == my_index).is_some()
        }) {
            Some(&(_, parent)) => parent,
            None => u16::MAX as usize
        }
    }).collect::<Vec<_>>();

    parent1.into_iter().map(|parent| {
        if parent > u16::MAX as usize {
            Err(Error::Convert(ConvertError::TooManyJoints))
        } else {
            Ok(parent as u16)
        }
    }).collect::<Result<Vec<_>>>()
}


fn get_inverse_bind_matrices<'a>(
    skin: &'a GltfSkin,
    buffers: &'a Buffers,
) -> Vec<Matrix4<f32>> {
    match skin.inverse_bind_matrices() {
        Some(accessor) => {
            InverseBindMatrices::new(accessor, buffers)
                .map(|matrix| {
                    Matrix4::from(matrix)
                }).collect()
        },
        None => {
            skin.joints().map(|_| Matrix4::identity())
                .collect()
        },
    }
}
