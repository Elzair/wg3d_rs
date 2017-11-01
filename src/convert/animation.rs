use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Vector3, Quaternion};
use float_cmp::ApproxEqUlps;
use gltf::Gltf;
use gltf::accessor::{Accessor, DataType, Dimensions};
use gltf::animation::{Animation as GltfAnimation, Channel as GltfChannel, Sampler as GltfSampler, TrsProperty};
use gltf_importer::Buffers;

use super::super::{Result, Error};
use super::ConvertError;
use super::skin::Skin;

pub struct Animation {
    name: String,
    channels: Vec<Channel>,
}

pub fn get<'a>(
    gltf: &'a Gltf,
    skin: &'a Skin,
    buffers: &'a Buffers,
) -> Result<Vec<Animation>> {
    gltf.animations().map(|animation| {
        get_animation(&animation, skin, buffers)
    }).collect::<Result<Vec<_>>>()
}

fn get_animation<'a>(
    animation: &'a GltfAnimation,
    skin: &'a Skin,
    buffers: &'a Buffers,
) -> Result<Animation> {
    let name = animation.name().ok_or(ConvertError::NoName)?;
    let channels = animation.channels().map(|channel| {
        get_channel(&channel, skin, buffers)
    }).collect::<Result<Vec<_>>>()?;
    
    Ok(Animation {
        name: String::from(name),
        channels: channels,
    })
}

pub struct Channel {
    joint_index: usize,
    property: Property,
}

fn get_channel<'a>(
    channel: &'a GltfChannel,
    skin: &'a Skin,
    buffers: &'a Buffers,
) -> Result<Channel> {
    let path = channel.target().path();
    let joint_index = skin.get_joint_index(
        channel.target().node().index()
    ).ok_or(ConvertError::InvalidJoint)?;
    let property = get_property(path, &channel.sampler(), buffers)?;

    Ok(Channel {
        joint_index: joint_index,
        property: property,
    })
}

pub enum Property {
    Translation(Vec<(f32, Vector3<f32>)>),
    Rotation(Vec<(f32, Quaternion<f32>)>),
    Scale(Vec<(f32, Vector3<f32>)>),
    Weight(Vec<(f32, f32)>),
}

fn get_property<'a>(
    path: TrsProperty,
    sampler: &'a GltfSampler,
    buffers: &'a Buffers,
) -> Result<Property> {
    let times = get_times(&sampler.input(), buffers)?;
    match path {
        TrsProperty::Translation => {
            let translations = get_translations(&sampler.output(), buffers)?;
            Ok(Property::Translation(
                times.into_iter().zip(translations.into_iter()).collect()
            ))
        },
        TrsProperty::Rotation => {
            let rotations = get_rotations(&sampler.output(), buffers)?;
            Ok(Property::Rotation(
                times.into_iter().zip(rotations.into_iter()).collect()
            ))
        },
        TrsProperty::Scale => {
            let scales = get_scales(&sampler.output(), buffers)?;
            Ok(Property::Scale(
                times.into_iter().zip(scales.into_iter()).collect()
            ))
        },
        TrsProperty::Weights => {
            let weights = get_weights(&sampler.output(), buffers)?;
            Ok(Property::Weight(
                times.into_iter().zip(weights.into_iter()).collect()
            ))
        },
    }
}

fn get_translations<'a>(
    accessor: &'a Accessor,
    buffers: &'a Buffers,
) -> Result<Vec<Vector3<f32>>> {
    match accessor.dimensions() {
        Dimensions::Vec3 => {
            match accessor.data_type() {
                DataType::F32 => {
                    let contents = buffers.view(&accessor.view())
                        .ok_or(ConvertError::Other)?;
                    let mut translations = Vec::<Vector3<f32>>::with_capacity(accessor.count());
                    let mut offset = accessor.offset();

                    #[allow(unused_variables)]
                    for i in 0..accessor.count() {
                        let sl = &contents[offset..(offset + accessor.size())];
                        let mut cursor = Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;

                        translations.push(Vector3::new(x, y, z));

                        offset = offset + accessor.view().stride().unwrap_or(accessor.size());
                    }

                    Ok(translations)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        }
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_rotations<'a>(
    accessor: &'a Accessor,
    buffers: &'a Buffers,
) -> Result<Vec<Quaternion<f32>>> {
    match accessor.dimensions() {
        Dimensions::Vec4 => {
            match accessor.data_type() {
                DataType::F32 => {
                    let contents = buffers.view(&accessor.view())
                        .ok_or(ConvertError::Other)?;
                    let mut rotations = Vec::<Quaternion<f32>>::with_capacity(accessor.count());
                    let mut offset = accessor.offset();

                    #[allow(unused_variables)]
                    for i in 0..accessor.count() {
                        let sl = &contents[offset..(offset + accessor.size())];
                        let mut cursor = Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;
                        let w = cursor.read_f32::<LittleEndian>()?;

                        rotations.push(Quaternion::new(w, x, y, z));

                        offset = offset + accessor.view().stride().unwrap_or(accessor.size());
                    }

                    Ok(rotations)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        }
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_scales<'a>(
    accessor: &'a Accessor,
    buffers: &'a Buffers,
) -> Result<Vec<Vector3<f32>>> {
    match accessor.dimensions() {
        Dimensions::Vec3 => {
            match accessor.data_type() {
                DataType::F32 => {
                    let contents = buffers.view(&accessor.view())
                        .ok_or(ConvertError::Other)?;
                    let mut translations = Vec::<Vector3<f32>>::with_capacity(accessor.count());
                    let mut offset = accessor.offset();

                    #[allow(unused_variables)]
                    for i in 0..accessor.count() {
                        let sl = &contents[offset..(offset + accessor.size())];
                        let mut cursor = Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;

                        // Return an error if scales are non-uniform.
                        if !x.approx_eq_ulps(&y, 2) || !y.approx_eq_ulps(&z, 2) || !z.approx_eq_ulps(&x, 2) {
                            return Err(Error::Convert(ConvertError::NonUniformScaling));

                        } 

                        translations.push(Vector3::new(x, y, z));

                        offset = offset + accessor.view().stride().unwrap_or(accessor.size());
                    }

                    Ok(translations)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        }
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_weights<'a>(
    accessor: &'a Accessor,
    buffers: &'a Buffers,
) -> Result<Vec<f32>> {
    match accessor.dimensions() {
        Dimensions::Scalar => {
            match accessor.data_type() {
                DataType::F32 => {
                    let contents = buffers.view(&accessor.view())
                        .ok_or(ConvertError::Other)?;
                    let mut weights = Vec::<f32>::with_capacity(accessor.count());
                    let mut offset = accessor.offset();

                    #[allow(unused_variables)]
                    for i in 0..accessor.count() {
                        let sl = &contents[offset..(offset + accessor.size())];
                        let mut cursor = Cursor::new(sl);

                        let weight = cursor.read_f32::<LittleEndian>()?;
                        weights.push(weight);

                        offset = offset + accessor.view().stride().unwrap_or(accessor.size());
                    }

                    Ok(weights)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        }
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}

fn get_times<'a>(
    accessor: &'a Accessor,
    buffers: &'a Buffers,
) -> Result<Vec<f32>> {
    match accessor.dimensions() {
        Dimensions::Scalar => {
            match accessor.data_type() {
                DataType::F32 => {
                    let contents = buffers.view(&accessor.view())
                        .ok_or(ConvertError::Other)?;
                    let mut times = Vec::<f32>::with_capacity(accessor.count());
                    let mut offset = accessor.offset();

                    #[allow(unused_variables)]
                    for i in 0..accessor.count() {
                        let sl = &contents[offset..(offset + accessor.size())];
                        let mut cursor = Cursor::new(sl);

                        let time = cursor.read_f32::<LittleEndian>()?;
                        times.push(time);

                        offset = offset + accessor.view().stride().unwrap_or(accessor.size());
                    }

                    Ok(times)
                },
                _ => Err(Error::Convert(ConvertError::UnsupportedDataType)),
            }
        }
        _ => Err(Error::Convert(ConvertError::UnsupportedDimensions)),
    }
}
