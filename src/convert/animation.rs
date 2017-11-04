// use std::io::Cursor;
use std::u16;

// use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Vector3, Quaternion};
use gltf::Gltf;
use gltf::accessor::{Accessor, DataType, Dimensions};
use gltf::animation::{Animation as GltfAnimation, Channel as GltfChannel, InterpolationAlgorithm, Sampler as GltfSampler, TrsProperty};
use gltf_importer::Buffers;
use gltf_utils::{AccessorIter, Denormalize, Source as GltfSource};

use super::super::{Result, Error};
use super::ConvertError;
use super::skin::Skin;

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
        times: Vec<f32>,
        translations: Vec<Vector3<f32>>,
    },
    Rotation {
        joint_index: u16,
        interpolation: Interpolation,
        times: Vec<f32>,
        rotations: Vec<Quaternion<f32>>,
    },
    Scale {
        joint_index: u16,
        interpolation: Interpolation,
        times: Vec<f32>,
        scales: Vec<Vector3<f32>>,
    },
    Weights {
        joint_index: u16,
        interpolation: Interpolation,
        times: Vec<f32>,
        weights: Vec<f32>,
    },
}

fn get_channels<'a>(
    animation: &'a GltfAnimation,
    skins: &'a [Skin],
    buffers: &'a Buffers,
) -> Result<Vec<Channel>> {
    animation.channels().map(|channel| {
        let sampler = channel.sampler();
        let interpolation_method = match sampler.interpolation() {
            InterpolationAlgorithm::CatmullRomSpline => Interpolation::CatmullRom,
            InterpolationAlgorithm::CubicSpline => Interpolation::Cubic,
            InterpolationAlgorithm::Linear => Interpolation::Linear,
            InterpolationAlgorithm::Step => Interpolation::Step,
        };
        let times = Times(AccessorIter::new(sampler.input(), buffers)).collect();

        let target = channel.target();
        let joint_index = get_joint_index(target.node().index(), skins)?;

        match target.path() {
            TrsProperty::Translation => {
                let translations = Translations(
                    AccessorIter::new(sampler.output(), buffers)
                ).map(|trans| Vector3::<f32>::from(trans)
                ).collect::<Vec<_>>();

                Ok(Channel::Translation {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    times: times,
                    translations: translations,
                })
            },
            TrsProperty::Rotation => {
                let rotations = RotationsF32(
                    Rotations::new(sampler.output(), buffers)
                ).map(|rot| Quaternion::<f32>::from(rot)
                ).collect::<Vec<_>>();

                Ok(Channel::Rotation {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    times: times,
                    rotations: rotations,
                })
            },
            TrsProperty::Scale => {
                let scales = Scales(
                    AccessorIter::new(sampler.output(), buffers)
                ).map(|scale| Vector3::<f32>::from(scale)
                ).collect::<Vec<_>>();

                Ok(Channel::Scale {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    times: times,
                    scales: scales,
                })
            },
            TrsProperty::Weights => {
                let weights = WeightsF32(
                    Weights::new(sampler.output(), buffers)
                ).collect::<Vec<_>>();

                Ok(Channel::Weights {
                    joint_index: joint_index,
                    interpolation: interpolation_method,
                    times: times,
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

/// Timestamp of type `f32`.
#[derive(Clone, Debug)]
pub struct Times<'a>(AccessorIter<'a, f32>);

impl<'a> ExactSizeIterator for Times<'a> {}
impl<'a> Iterator for Times<'a> {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// XYZ translations of type `[f32; 3]`.
#[derive(Clone, Debug)]
pub struct Translations<'a>(AccessorIter<'a, [f32; 3]>);

impl<'a> ExactSizeIterator for Translations<'a> {}
impl<'a> Iterator for Translations<'a> {
    type Item = [f32; 3];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// XYZW quaternion rotations of type `[f32; 4]`.
#[derive(Clone, Debug)]
pub struct RotationsF32<'a>(Rotations<'a>);

impl<'a> ExactSizeIterator for RotationsF32<'a> {}
impl<'a> Iterator for RotationsF32<'a> {
    type Item = [f32; 4];
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Rotations::F32(ref mut i) => i.next(),
            Rotations::U8(ref mut i) => i.next().map(|x| x.denormalize()),
            Rotations::I16(ref mut i) => i.next().map(|x| {
                [x[0] as f32 / 32767.0,
                 x[1] as f32 / 32767.0,
                 x[2] as f32 / 32767.0,
                 x[3] as f32 / 32767.0]
            }),
            Rotations::U16(ref mut i) => i.next().map(|x| x.denormalize()),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            Rotations::F32(ref i) => i.size_hint(),
            Rotations::U8(ref i) => i.size_hint(),
            Rotations::I16(ref i) => i.size_hint(),
            Rotations::U16(ref i) => i.size_hint(),
        }
    }
}

/// XYZ scales of type `[f32; 3]`.
#[derive(Clone, Debug)]
pub struct Scales<'a>(AccessorIter<'a, [f32; 3]>);

impl<'a> ExactSizeIterator for Scales<'a> {}
impl<'a> Iterator for Scales<'a> {
    type Item = [f32; 3];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// Morph-target weights of type `f32`.
#[derive(Clone, Debug)]
pub struct WeightsF32<'a>(Weights<'a>);

impl<'a> ExactSizeIterator for WeightsF32<'a> {}
impl<'a> Iterator for WeightsF32<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Weights::F32(ref mut i) => i.next(),
            Weights::U8(ref mut i) => i.next().map(|x| x.denormalize()),
            Weights::I16(ref mut i) => i.next().map(|x| x as f32 / 32767.0),
            Weights::U16(ref mut i) => i.next().map(|x| x.denormalize()),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            Weights::F32(ref i) => i.size_hint(),
            Weights::U8(ref i) => i.size_hint(),
            Weights::I16(ref i) => i.size_hint(),
            Weights::U16(ref i) => i.size_hint(),
        }
    }
}

/// Rotations
#[derive(Clone, Debug)]
enum Rotations<'a> {
    F32(AccessorIter<'a, [f32; 4]>),
    U8(AccessorIter<'a, [u8; 4]>),
    I16(AccessorIter<'a, [i16; 4]>),
    U16(AccessorIter<'a, [u16; 4]>),
}

impl<'a> Rotations<'a> {
    fn new<S: GltfSource>(accessor: Accessor<'a>, source: &'a S) -> Rotations<'a> {
        match accessor.dimensions() {
            Dimensions::Vec4 => {
                match accessor.data_type() {
                    DataType::F32 => {
                        Rotations::F32(AccessorIter::new(accessor, source))
                    },
                    DataType::U8 => {
                        Rotations::U8(AccessorIter::new(accessor, source))
                    },
                    DataType::I16 => {
                        Rotations::I16(AccessorIter::new(accessor, source))
                    },
                    DataType::U16 => {
                        Rotations::U16(AccessorIter::new(accessor, source))
                    },
                    _ => unimplemented!(),
                }
            },
            _ => unimplemented!(),
        }
    }
}

/// Weights
#[derive(Clone, Debug)]
enum Weights<'a> {
    F32(AccessorIter<'a, f32>),
    U8(AccessorIter<'a, u8>),
    I16(AccessorIter<'a, i16>), 
    U16(AccessorIter<'a, u16>),
}

impl<'a> Weights<'a> {
    fn new<S: GltfSource>(accessor: Accessor<'a>, source: &'a S) -> Weights<'a> {
        match accessor.dimensions() {
            Dimensions::Scalar => {
                match accessor.data_type() {
                    DataType::F32 => {
                        Weights::F32(AccessorIter::new(accessor, source))
                    },
                    DataType::U8 => {
                        Weights::U8(AccessorIter::new(accessor, source))
                    },
                    DataType::I16 => {
                        Weights::I16(AccessorIter::new(accessor, source))
                    },
                    DataType::U16 => {
                        Weights::U16(AccessorIter::new(accessor, source))
                    },
                    _ => unimplemented!(),
                }
            },
            _ => unimplemented!(),
        }
    }
}
