use std::cell::Cell;

use gltf::scene::Node as GltfNode;
use gltf::skin::Skin as GltfSkin;

use super::super::{Result, Error};
use super::buffer::Buffers;
use super::ConvertError;

pub type Skin = Vec<Joint>;

pub fn get<'a>(
    skin: Option<GltfSkin>,
    buffers: &'a Buffers,
) -> Result<Option<Skin>> {
    if skin.is_none() {
        return Ok(None);
    }

    let joints = get_joints(&skin.unwrap())?;

    Ok(Some(joints))
}

pub struct Joint {
    parent: usize,
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: f32,
}

fn get_joints<'a>(
    skin: &'a GltfSkin,
) -> Result<Vec<Joint>> {
    // Get joint indices.
    let skeleton = skin.skeleton();
    
    if skeleton.is_none() {
        return Err(Error::Convert(ConvertError::NoSkeleton));
    }

    let indices = skin.joints().map(|joint| {
        joint.index()
    }).collect::<Vec<_>>();
    let parents = get_parents(&skeleton.unwrap(), &indices);
    let joints = parents.iter().zip(skin.joints()).map(|(parent, joint)| {
        let (translation, rotation, scale) = joint.transform().decomposed();

        Joint {
            parent: *parent,
            translation: translation,
            rotation: rotation,
            scale: scale[0], // WG3D only supports uniform scaling.
        }
    }).collect::<Vec<_>>();

    Ok(joints)
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
