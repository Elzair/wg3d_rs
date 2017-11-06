// use byteorder::{LE, ByteOrder};
use gltf;
use gltf::accessor::{Accessor, DataType, Dimensions};
use gltf_utils::{AccessorIter, Denormalize, Source};

/// Extra methods for working with `gltf::Skin`.
pub trait SkinIterators<'a> {
    /// Visits the `inverseBindMatrices` of the skin.
    fn ibms<S: Source>(&'a self, source: &'a S) -> Option<InverseBindMatrices<'a>>;
}

impl<'a> SkinIterators<'a> for gltf::Skin<'a> {
    fn ibms<S: Source>(&'a self, source: &'a S) -> Option<InverseBindMatrices<'a>> {
        self.inverse_bind_matrices().map(|accessor| InverseBindMatrices(AccessorIter::new(accessor, source)))
    }
}

/// Inverse Bind Matrices iterator.
#[derive(Clone, Debug)]
pub struct InverseBindMatrices<'a>(AccessorIter<'a, [[f32; 4]; 4]>);

impl<'a> ExactSizeIterator for InverseBindMatrices<'a> {}
impl<'a> Iterator for InverseBindMatrices<'a> {
    type Item = [[f32; 4]; 4];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// Extra methods for working with `gltf::animation::Channel`.
pub trait ChannelIterators<'a> {
    /// Visits the input samples of a channel.
    fn times<S: Source>(&'a self, source: &'a S) -> Times<'a>;

    /// Visits the translation samples of a channel.
    fn translations<S: Source>(&'a self, source: &'a S) -> Option<Translations<'a>>;

    /// Visits the translation samples of a channel.
    fn rotations_f32<S: Source>(&'a self, source: &'a S) -> Option<RotationsF32<'a>>;

    /// Visits the translation samples of a channel.
    fn scales<S: Source>(&'a self, source: &'a S) -> Option<Scales<'a>>;

    /// Visits the translation samples of a channel.
    fn weights_f32<S: Source>(&'a self, source: &'a S) -> Option<WeightsF32<'a>>;
}

impl<'a> ChannelIterators<'a> for gltf::animation::Channel<'a> {
    fn times<S: Source>(&'a self, source: &'a S) -> Times<'a> {
        Times(AccessorIter::new(self.sampler().input(), source))
    }
 
    /// Visits the translation samples of a channel.
    fn translations<S: Source>(&'a self, source: &'a S) -> Option<Translations<'a>> {
        match self.target().path() {
            gltf::animation::TrsProperty::Translation => {
                Some(Translations(AccessorIter::new(self.sampler().output(), source)))
            },
            _ => None,
        }
    }

    /// Visits the translation samples of a channel.
    fn rotations_f32<S: Source>(&'a self, source: &'a S) -> Option<RotationsF32<'a>> {
        match self.target().path() {
            gltf::animation::TrsProperty::Rotation => {
                Some(RotationsF32(Rotations::new(self.sampler().output(), source)))
            },
            _ => None,
        }
    }

    /// Visits the translation samples of a channel.
    fn scales<S: Source>(&'a self, source: &'a S) -> Option<Scales<'a>> {
        match self.target().path() {
            gltf::animation::TrsProperty::Scale => {
                Some(Scales(AccessorIter::new(self.sampler().output(), source)))
            },
            _ => None,
        }
    }

    /// Visits the translation samples of a channel.
    fn weights_f32<S: Source>(&'a self, source: &'a S) -> Option<WeightsF32<'a>> {
        match self.target().path() {
            gltf::animation::TrsProperty::Weights => {
                Some(WeightsF32(Weights::new(self.sampler().output(), source)))
            },
            _ => None,
        }
    }
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
    fn new<S: Source>(accessor: Accessor<'a>, source: &'a S) -> Rotations<'a> {
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
    fn new<S: Source>(accessor: Accessor<'a>, source: &'a S) -> Weights<'a> {
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


