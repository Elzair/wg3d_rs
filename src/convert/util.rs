use gltf::accessor::{Accessor, DataType, Dimensions};
use gltf_utils::{AccessorIter, Denormalize, Source};

/// Inverse Bind Matrices iterator
#[derive(Clone, Debug)]
pub struct InverseBindMatrices<'a> {
    inner: AccessorIter<'a, [[f32; 4]; 4]>,
}

impl<'a> InverseBindMatrices<'a> {
    pub fn new<S: Source>(
        accessor: Accessor<'a>,
        source: &'a S
    ) -> Self {
        Self {
            inner: AccessorIter::<'a, [[f32; 4]; 4]>::new(accessor, source),
        }
    }
}

impl<'a> ExactSizeIterator for InverseBindMatrices<'a> {}
impl<'a> Iterator for InverseBindMatrices<'a> {
    type Item = [[f32; 4]; 4];
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Timestamp of type `f32`.
#[derive(Clone, Debug)]
pub struct Times<'a> {
    inner: AccessorIter<'a, f32>,
}

impl<'a> Times<'a> {
    pub fn new<S: Source>(
        accessor: Accessor<'a>,
        source: &'a S
    ) -> Self {
        Self {
            inner: AccessorIter::<'a, f32>::new(accessor, source),
        }
    }
}

impl<'a> ExactSizeIterator for Times<'a> {}
impl<'a> Iterator for Times<'a> {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// XYZ translations of type `[f32; 3]`.
#[derive(Clone, Debug)]
pub struct Translations<'a> {
    inner: AccessorIter<'a, [f32; 3]>
}

impl<'a> Translations<'a> {
    pub fn new<S: Source>(
        accessor: Accessor<'a>,
        source: &'a S
    ) -> Self {
        Self {
            inner: AccessorIter::<'a, [f32; 3]>::new(accessor, source),
        }
    }
}

impl<'a> ExactSizeIterator for Translations<'a> {}

impl<'a> Iterator for Translations<'a> {
    type Item = [f32; 3];
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// XYZW quaternion rotations of type `[f32; 4]`.
#[derive(Clone, Debug)]
pub struct RotationsF32<'a> {
    inner: Rotations<'a>,
}

impl<'a> RotationsF32<'a> {
    pub fn new<S: Source>(
        accessor: Accessor<'a>,
        source: &'a S
    ) -> Self {
        Self {
            inner: Rotations::new(accessor, source),
        }
    }
}

impl<'a> ExactSizeIterator for RotationsF32<'a> {}

impl<'a> Iterator for RotationsF32<'a> {
    type Item = [f32; 4];
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
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
        match self.inner {
            Rotations::F32(ref i) => i.size_hint(),
            Rotations::U8(ref i) => i.size_hint(),
            Rotations::I16(ref i) => i.size_hint(),
            Rotations::U16(ref i) => i.size_hint(),
        }
    }
}

/// XYZ scales of type `[f32; 3]`.
#[derive(Clone, Debug)]
pub struct Scales<'a> {
    inner: AccessorIter<'a, [f32; 3]>,
}

impl<'a> Scales<'a> {
    pub fn new<S: Source>(
        accessor: Accessor<'a>,
        source: &'a S
    ) -> Self {
        Self {
            inner: AccessorIter::<'a, [f32; 3]>::new(accessor, source),
        }
    }
}

impl<'a> ExactSizeIterator for Scales<'a> {}

impl<'a> Iterator for Scales<'a> {
    type Item = [f32; 3];

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Morph-target weights of type `f32`.
#[derive(Clone, Debug)]
pub struct WeightsF32<'a> {
    inner: Weights<'a>,
}

impl<'a> WeightsF32<'a> {
    pub fn new<S: Source>(
        accessor: Accessor<'a>,
        source: &'a S
    ) -> Self {
        Self {
            inner: Weights::new(accessor, source),
        }
    }
}

impl<'a> ExactSizeIterator for WeightsF32<'a> {}

impl<'a> Iterator for WeightsF32<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            Weights::F32(ref mut i) => i.next(),
            Weights::U8(ref mut i) => i.next().map(|x| x.denormalize()),
            Weights::I16(ref mut i) => i.next().map(|x| x as f32 / 32767.0),
            Weights::U16(ref mut i) => i.next().map(|x| x.denormalize()),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner {
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
