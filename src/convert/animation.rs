use std::u16;

use cgmath::{Vector3, Quaternion};
use gltf::gltf::Animations as GltfAnimations;
use gltf::animation::{Animation as GltfAnimation, InterpolationAlgorithm, TrsProperty};
use gltf_importer::Buffers;

use super::super::Result;
use super::ConvertError;
use super::skin::Skins;
use super::util::ChannelIterators;

pub struct Animations {
    animations: Vec<Animation>,
}

pub struct Animation {
    name: String,
    channels: Vec<Channel>,
}

pub fn get<'a>(
    animations: GltfAnimations,
    skins: &'a Skins,
    buffers: &'a Buffers,
) -> Result<Animations> {
    let my_animations = animations.map(|animation| {
        let name = animation.name().ok_or(ConvertError::NoName)?;
        let channels = get_channels(&animation, skins, buffers)?;
        
        Ok(Animation {
            name: String::from(name),
            channels: channels,
        })

    }).collect::<Result<Vec<_>>>()?;

    Ok(Animations {
        animations: my_animations,
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
    skins: &'a Skins,
    buffers: &'a Buffers,
) -> Result<Vec<Channel>> {
    animation.channels().map(|channel| {
        let sampler = channel.sampler();
        let (interpolation_method, times) = match sampler.interpolation() {
            InterpolationAlgorithm::CatmullRomSpline => {
                let mut times = channel.times(buffers).collect::<Vec<_>>();
                // Add stub timestamps for start and end tangents of spline.
                let (first, last) = {
                    (times[0], times[times.len()-1])
                };
                times.push(first);
                times.push(last);

                (Interpolation::CatmullRom, times)
            },
            InterpolationAlgorithm::CubicSpline => {
                let mut times = channel.times(buffers).collect::<Vec<_>>();
                // Add stub timestamps for start and end tangents of spline.
                let (first, last) = {
                    (times[0], times[times.len()-1])
                };
                times.push(first);
                times.push(last);

                (Interpolation::Cubic, times)
            },
            InterpolationAlgorithm::Linear => {
                let times = channel.times(buffers).collect::<Vec<_>>();

                (Interpolation::Linear, times)
            },
            InterpolationAlgorithm::Step => {
                let times = channel.times(buffers).collect::<Vec<_>>();

                (Interpolation::Step, times)
            },
        };

        let target = channel.target();
        let joint_index = skins.get_joint_index(target.node().index())
            .ok_or(ConvertError::InvalidJoint)?;

        match target.path() {
            TrsProperty::Translation => {
                let translations = times.into_iter().zip(channel.translations(
                    buffers
                ).unwrap()).map(|(time_stamp, vector)| {
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
                let rotations = times.into_iter().zip(channel.rotations_f32(
                    buffers
                ).unwrap()).map(|(time_stamp, quaternion)| {
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
                let scales = times.into_iter().zip(channel.scales(
                    buffers
                ).unwrap()).map(|(time_stamp, vector)| {
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
                let weights = times.into_iter().zip(channel.weights_f32(
                    buffers
                ).unwrap()).map(|(time_stamp, scalar)| {
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

pub enum Interpolation {
    CatmullRom,
    Cubic,
    Linear,
    Step,
}
