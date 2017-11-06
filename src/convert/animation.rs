// use std::io::Cursor;
use std::u16;

// use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Vector3, Quaternion};
use gltf::Gltf;
use gltf::animation::{Animation as GltfAnimation, InterpolationAlgorithm, TrsProperty};
use gltf_importer::Buffers;

use super::super::{Result, Error};
use super::ConvertError;
use super::skin::Skin;
use super::util::{RotationsF32, Scales, Times, Translations, WeightsF32};

pub struct Animation {
    name: String,
    channels: Vec<Channel>,
}

pub fn get<'a>(
    gltf: &'a Gltf,
    skins: &'a [Skin],
    buffers: &'a Buffers,
) -> Result<Vec<Animation>> {
    gltf.animations().map(|animation| {
        get_animation(&animation, skins, buffers)
    }).collect::<Result<Vec<_>>>()
}

fn get_animation<'a>(
    animation: &'a GltfAnimation,
    skins: &'a [Skin],
    buffers: &'a Buffers,
) -> Result<Animation> {
    let name = animation.name().ok_or(ConvertError::NoName)?;
    let channels = get_channels(animation, skins, buffers)?;
    
    Ok(Animation {
        name: String::from(name),
        channels: channels,
    })
}

pub enum Channel {
    Translation {
        joint_index: u16,
        interpolation: Interpolation,
        translations: Vec<Vector3Data>,
    },
    Rotation {
        joint_index: u16,
        interpolation: Interpolation,
        rotations: Vec<QuaternionData>,
    },
    Scale {
        joint_index: u16,
        interpolation: Interpolation,
        scales: Vec<Vector3Data>,
    },
    Weights {
        joint_index: u16,
        interpolation: Interpolation,
        weights: Vec<ScalarData>,
    },
}

pub struct Vector3Data {
    time_stamp: f32,
    vector: Vector3<f32>,
}

pub struct QuaternionData {
    time_stamp: f32,
    quaternion: Quaternion<f32>,
}

pub struct ScalarData {
    time_stamp: f32,
    scalar: f32,
}

fn get_channels<'a>(
    animation: &'a GltfAnimation,
    skins: &'a [Skin],
    buffers: &'a Buffers,
) -> Result<Vec<Channel>> {
    animation.channels().map(|channel| {
        let sampler = channel.sampler();
        let (interpolation_method, times) = match sampler.interpolation() {
            InterpolationAlgorithm::CatmullRomSpline => {
                let mut times = Times::new(
                    sampler.input(),
                    buffers
                ).collect::<Vec<_>>();
                // Add stub timestamps for start and end tangents of spline.
                let (first, last) = {
                    (times[0], times[times.len()-1])
                };
                times.push(first);
                times.push(last);

                (Interpolation::CatmullRom, times)
            },
            InterpolationAlgorithm::CubicSpline => {
                let mut times = Times::new(
                    sampler.input(),
                    buffers
                ).collect::<Vec<_>>();
                // Add stub timestamps for start and end tangents of spline.
                let (first, last) = {
                    (times[0], times[times.len()-1])
                };
                times.push(first);
                times.push(last);

                (Interpolation::Cubic, times)
            },
            InterpolationAlgorithm::Linear => {
                let times = Times::new(
                    sampler.input(),
                    buffers
                ).collect::<Vec<_>>();

                (Interpolation::Linear, times)
            },
            InterpolationAlgorithm::Step => {
                let times = Times::new(
                    sampler.input(),
                    buffers
                ).collect::<Vec<_>>();

                (Interpolation::Step, times)
            },
        };

        let target = channel.target();
        let joint_index = get_joint_index(target.node().index(), skins)?;

        match target.path() {
            TrsProperty::Translation => {
                let translations = times.into_iter().zip(Translations::new(
                    sampler.output(),
                    buffers
                )).map(|(time_stamp, vector)| {
                    Vector3Data {
                        time_stamp: time_stamp,
                        vector: Vector3::from(vector),
                    }
                }).collect::<Vec<_>>();

                Ok(Channel::Translation {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    translations: translations,
                })
            },
            TrsProperty::Rotation => {
                let rotations = times.into_iter().zip(RotationsF32::new(
                    sampler.output(),
                    buffers
                )).map(|(time_stamp, quaternion)| {
                    QuaternionData {
                        time_stamp: time_stamp,
                        quaternion: Quaternion::from(quaternion),
                    }
                }).collect::<Vec<_>>();

                Ok(Channel::Rotation {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    rotations: rotations,
                })
            },
            TrsProperty::Scale => {
                let scales = times.into_iter().zip(Scales::new(
                    sampler.output(),
                    buffers
                )).map(|(time_stamp, vector)| {
                    Vector3Data {
                        time_stamp: time_stamp,
                        vector: Vector3::from(vector),
                    }
                }).collect::<Vec<_>>();

                Ok(Channel::Scale {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    scales: scales,
                })
            },
            TrsProperty::Weights => {
                let weights = times.into_iter().zip(WeightsF32::new(
                    sampler.output(),
                    buffers
                )).map(|(time_stamp, scalar)| {
                    ScalarData {
                        time_stamp: time_stamp,
                        scalar: scalar,
                    }
                }).collect::<Vec<_>>();

                Ok(Channel::Weights {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    weights: weights,
                })
            },
        }
    }).collect::<Result<Vec<_>>>()
}

fn get_joint_index<'a>(
    node_index: usize,
    skins: &'a [Skin],
) -> Result<u16> {
    for skin in skins.iter() {
        if let Some(joint_index) = skin.get_joint_index(node_index) {
            return Ok(joint_index);
        }
    }

    Err(Error::Convert(ConvertError::InvalidJoint))
}

pub enum Interpolation {
    CatmullRom,
    Cubic,
    Linear,
    Step,
}
