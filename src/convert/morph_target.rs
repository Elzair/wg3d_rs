use std::io::Cursor;
use std::mem::size_of;

use byteorder::{LE, ByteOrder, ReadBytesExt};
use cgmath::Vector3;

use gltf::accessor::{Accessor, DataType, Dimensions};
use gltf::accessor::sparse::{IndexType, Sparse as GltfSparse};
use gltf::mesh::Primitive as GltfPrimitive;
use gltf_importer::Buffers;
use gltf_utils::{AccessorIter, Source};

use super::super::{Result, Error};
use super::ConvertError;

pub struct MorphTarget {
    // Currently all morph targets are `[f32; 3]`.
    positions: Option<Data>,
    normals: Option<Data>,
    tangents: Option<Data>,
}

pub enum Data {
    Full(Vec<Vector3<f32>>),
    Sparse(Vec<SparseDatum>),
}

pub struct SparseDatum {
    pub index: u32,
    pub value: Vector3<f32>,
}

pub fn get<'a>(
    primitive: &'a GltfPrimitive,
    buffers: &'a Buffers,
) -> Result<Vec<MorphTarget>> {
    primitive.morph_targets().map(|morph_target| {
        let positions = get_data(morph_target.positions(), buffers)?;
        let normals = get_data(morph_target.normals(), buffers)?;
        let tangents = get_data(morph_target.tangents(), buffers)?;

        Ok(MorphTarget {
            positions,
            normals,
            tangents,
        })
    }).collect()
}

fn get_data<'a>(
    accessor: Option<Accessor>,
    buffers: &'a Buffers,
) -> Result<Option<Data>> {
    if let Some(access) = accessor {
        // Ensure morph target accessor has the right format.
        // Currently, all morph targets are `[f32; 3]`.
        match access.dimensions() {
            Dimensions::Vec3 => {
                match access.data_type() {
                    DataType::F32 => {},
                    _ => { return Err(Error::Convert(ConvertError::Other)); }
                }
            },
            _ => { return Err(Error::Convert(ConvertError::Other)); }

        }

        let access2 = access.clone();
        if let Some(sparse) = access.sparse() {
            let indices = get_sparse_indices(&sparse, buffers)?;
            let values = get_sparse_values(&sparse, buffers)?;

            Ok(Some(Data::Sparse(indices.into_iter().zip(values.into_iter())
                                 .map(|(index, value)| SparseDatum { index, value })
                                 .collect())))
        } else {
            Ok(Some(Data::Full(FullMorphs(AccessorIter::new(access2, buffers))
                    .map(|data| Vector3::from(data))
                    .collect())))
        }
    } else { Ok(None) }
}

fn get_sparse_indices<'a>(
    sparse: &'a GltfSparse,
    buffers: &'a Buffers,
) -> Result<Vec<u32>> {
    let count = sparse.count() as usize;
    let indices = sparse.indices();
    let index_type = indices.index_type();
    let index_size = match index_type {
        IndexType::U8 => size_of::<u8>(),
        IndexType::U16 => size_of::<u16>(),
        IndexType::U32 => size_of::<u32>(),
    };
    
    let view = indices.view();
    let stride = view.stride().unwrap_or(index_size);
    debug_assert!(stride >= index_size);
    
    let start = view.offset();
    let end = start + stride * (count - 1) + index_size;
    let data = &buffers.source_buffer(&view.buffer())[start .. end];
    let mut cursor = Cursor::new(data);
    
    let mut indices = Vec::<u32>::with_capacity(count);

    #[allow(unused_variables)]
    for i in 0..count {
        let index = match index_type {
            IndexType::U8 => {
                cursor.read_u8()? as u32
            },
            IndexType::U16 => {
                cursor.read_u16::<LE>()? as u32
            },
            IndexType::U32 => {
                cursor.read_u32::<LE>()? as u32
            },
        };
        indices.push(index);
    }

    Ok(indices)
}

fn get_sparse_values<'a>(
    sparse: &'a GltfSparse,
    buffers: &'a Buffers,
) -> Result<Vec<Vector3<f32>>> {
    let count = sparse.count() as usize;
    let values = sparse.values();
    let view = values.view();
    let stride = view.stride().unwrap_or(size_of::<[f32; 3]>());
    debug_assert!(stride >= size_of::<[f32; 3]>());
    
    let start = view.offset();
    let end = start + stride * (count - 1) + size_of::<[f32; 3]>();
    let data = &buffers.source_buffer(&view.buffer())[start .. end];
    let mut cursor = Cursor::new(data);
    
    let mut values = Vec::<Vector3<f32>>::with_capacity(count);

    #[allow(unused_variables)]
    for i in 0..count {
        let x = cursor.read_f32::<LE>()?;
        let y = cursor.read_f32::<LE>()?;
        let z = cursor.read_f32::<LE>()?;
        values.push(Vector3::new(x, y, z));
    }

    Ok(values)
}

/// XYZ vertex positions of type `[f32; 3]`.
#[derive(Clone, Debug)]
struct FullMorphs<'a>(AccessorIter<'a, [f32; 3]>);
impl<'a> ExactSizeIterator for FullMorphs<'a> {}
impl<'a> Iterator for FullMorphs<'a> {
    type Item = [f32; 3];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
