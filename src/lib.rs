extern crate bincode;
extern crate byteorder;
extern crate gltf;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::mem;
use std::path::Path;
use std::result;
use std::slice;
use std::u16;

use byteorder::{LittleEndian, ReadBytesExt};
use gltf::{Gltf, Material};
use gltf::accessor::{DataType, Dimensions};
use gltf::mesh::{Primitive, Semantic};

pub type Buffers = HashMap<String, Vec<u8>>;

pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Gltf> {
    let file = File::open(path)?;
    let gltf = Gltf::from_reader(io::BufReader::new(file))?
    .validate_minimally()?;

    Ok(gltf)
}

pub fn get_buffers<'a>(
    base_path: &'a Path,
    gltf: &'a Gltf
) -> Result<Buffers> {
    let mut buffers = Buffers::new();

    for buffer in gltf.buffers() {
        let full_uri = base_path.join(buffer.uri());
        let mut file = File::open(full_uri)?;
        let metadata = file.metadata()?;

        if metadata.len() != (buffer.length() as u64) {
            return Err(Error::Wg3d(Wg3dError::InvalidBufferLength));
        }
        
        let mut contents = Vec::<u8>::with_capacity(buffer.length()); 
        file.read_to_end(&mut contents).ok().unwrap();

        buffers.insert(buffer.uri().to_string(), contents);
    }

    Ok(buffers)
}

pub fn get_nodes<'a>(
    gltf: &'a Gltf,
    buffers: &'a Buffers
) -> Result<()> {
    for node in gltf.nodes() {
        if let Some(mesh) = node.mesh() {
            // See if the node also has a skin.
            let has_joints = node.skin().is_some();
            
            let _ = get_mesh(gltf, mesh.index(), has_joints, buffers)?;
        }
    }

    Ok(())
}

struct Mesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords_0: Vec<[f32; 2]>,
    texcoords_1: Option<Vec<[f32; 2]>>,
    joints: Option<Vec<[u16; 4]>>,
    weights: Option<Vec<[u16; 4]>>,
}

fn get_mesh<'a>(
    gltf: &'a Gltf,
    index: usize,
    has_joints: bool,
    buffers: &'a Buffers
) -> Result<Vec<Mesh>> {
    let mut meshes = Vec::<Mesh>::new();
    
    for mesh in gltf.meshes() {
        if mesh.index() == index {
            for prim in mesh.primitives() {
                let material = prim.material();

                let positions = get_positions(&prim, buffers)?;
                let normals = get_normals(&prim, buffers)?;
                let texcoords_0 = get_texcoords(&prim, 0, buffers)?;

                let texcoords_1 = if let Some(_) = prim.get(&Semantic::TexCoords(1)) {
                    let res = get_texcoords(&prim, 0, buffers)?;
                    Some(res)
                } else { None };

                let (joints, weights) = if has_joints {
                    let joints = get_joints(&prim, buffers)?;
                    let weights = get_weights(&prim, buffers)?;
                    (Some(joints), Some(weights))
                } else { (None, None) };

                meshes.push(Mesh {
                    positions: positions,
                    normals: normals,
                    texcoords_0: texcoords_0,
                    texcoords_1: texcoords_1,
                    joints: joints,
                    weights: weights,
                });
            }
        }
    }

    Ok(meshes)
}

fn get_positions<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 3]>> {
    let access = primitive.get(&Semantic::Positions)
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec3 => {
            match access.data_type() {
                DataType::F32 => {
                    let size = mem::size_of::<[f32; 3]>();
                    let mut positions = Vec::<[f32; 3]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;
                        positions.push([x, y, z]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(positions)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

fn get_normals<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 3]>> {
    let access = primitive.get(&Semantic::Normals)
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.data_type() {
        DataType::F32 => {
            match access.dimensions() {
                Dimensions::Vec3 => {
                    let size = mem::size_of::<[f32; 3]>();
                    let mut normals = Vec::<[f32; 3]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let x = cursor.read_f32::<LittleEndian>()?;
                        let y = cursor.read_f32::<LittleEndian>()?;
                        let z = cursor.read_f32::<LittleEndian>()?;
                        normals.push([x, y, z]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(normals)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
    }
}

fn get_texcoords<'a>(
    primitive: &'a Primitive,
    index: u32,
    buffers: &'a Buffers
) -> Result<Vec<[f32; 2]>> {
    let access = primitive.get(&Semantic::TexCoords(index))
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.data_type() {
        DataType::F32 => {
            match access.dimensions() {
                Dimensions::Vec2 => {
                    let size = mem::size_of::<[f32; 2]>();
                    let mut tex0 = Vec::<[f32; 2]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let u = cursor.read_f32::<LittleEndian>()?;
                        let v = cursor.read_f32::<LittleEndian>()?;
                        tex0.push([u, v]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(tex0)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
    }
}

fn get_joints<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&Semantic::Joints(0))
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec4 => {
            match access.data_type() {
                DataType::U8 => {
                    let size = mem::size_of::<[u8; 4]>();
                    let mut joints = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let jj0 = cursor.read_u8()?;
                        let j0 = (jj0 as u16) << 8;
                        let jj1 = cursor.read_u8()?;
                        let j1 = (jj1 as u16) << 8;
                        let jj2 = cursor.read_u8()?;
                        let j2 = (jj2 as u16) << 8;
                        let jj3 = cursor.read_u8()?;
                        let j3 = (jj3 as u16) << 8;
                        joints.push([j0, j1, j2, j3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(joints)
                },
                DataType::U16 => {
                    let size = mem::size_of::<[u16; 4]>();
                    let mut joints = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let j0 = cursor.read_u16::<LittleEndian>()?;
                        let j1 = cursor.read_u16::<LittleEndian>()?;
                        let j2 = cursor.read_u16::<LittleEndian>()?;
                        let j3 = cursor.read_u16::<LittleEndian>()?;
                        joints.push([j0, j1, j2, j3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(joints)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

fn get_weights<'a>(
    primitive: &'a Primitive,
    buffers: &'a Buffers
) -> Result<Vec<[u16; 4]>> {
    let access = primitive.get(&Semantic::Joints(0))
        .ok_or(Wg3dError::MissingAttributes)?;
    let view = access.view();
    let buff = view.buffer();

    let mut offset = view.offset() + access.offset();

    match access.dimensions() {
        Dimensions::Vec4 => {
            match access.data_type() {
                DataType::U8 => {
                    let size = mem::size_of::<[u8; 4]>();
                    let mut weights = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let ww0 = cursor.read_u8()?;
                        let w0 = (ww0 as u16) << 8;
                        let ww1 = cursor.read_u8()?;
                        let w1 = (ww1 as u16) << 8;
                        let ww2 = cursor.read_u8()?;
                        let w2 = (ww2 as u16) << 8;
                        let ww3 = cursor.read_u8()?;
                        let w3 = (ww3 as u16) << 8;
                        weights.push([w0 as u16, w1 as u16, w2 as u16, w3 as u16]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(weights)
                },
                DataType::U16 => {
                    let size = mem::size_of::<[u16; 4]>();
                    let mut weights = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let w0 = cursor.read_u16::<LittleEndian>()?;
                        let w1 = cursor.read_u16::<LittleEndian>()?;
                        let w2 = cursor.read_u16::<LittleEndian>()?;
                        let w3 = cursor.read_u16::<LittleEndian>()?;
                        weights.push([w0, w1, w2, w3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(weights)
                },
                DataType::F32 => {
                    let size = mem::size_of::<[f32; 4]>();
                    let mut weights = Vec::<[u16; 4]>::with_capacity(access.count());
                    let inbuf = buffers.get(buff.uri()).unwrap();

                    #[allow(unused_variables)]
                    for i in 0..access.count() {
                        let sl = &inbuf[offset..(offset+size)];
                        let mut cursor = io::Cursor::new(sl);

                        let ww0 = cursor.read_f32::<LittleEndian>()?;
                        let w0 = (ww0 * (u16::max_value() as f32)).round() as u16;
                        let ww1 = cursor.read_f32::<LittleEndian>()?;
                        let w1 = (ww1 * (u16::max_value() as f32)).round() as u16;
                        let ww2 = cursor.read_f32::<LittleEndian>()?;
                        let w2 = (ww2 * (u16::max_value() as f32)).round() as u16;
                        let ww3 = cursor.read_f32::<LittleEndian>()?;
                        let w3 = (ww3 * (u16::max_value() as f32)).round() as u16;
                        weights.push([w0, w1, w2, w3]);

                        offset = offset + view.stride().unwrap_or(access.size());
                    }

                    Ok(weights)
                },
                _ => Err(Error::Wg3d(Wg3dError::UnsupportedDataType)),
            }
        },
        _ => Err(Error::Wg3d(Wg3dError::UnsupportedDimensions)),
    }
}

/// This is the top level Error for this crate.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Gltf(gltf::Error),
    Wg3d(Wg3dError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => err.fmt(fmt),
            &Error::Gltf(ref err) => err.fmt(fmt),
            &Error::Wg3d(ref err) => err.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref err) => err.description(),
            &Error::Gltf(ref err) => err.description(),
            &Error::Wg3d(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::Io(ref err) => err.cause(),
            &Error::Gltf(ref err) => err.cause(),
            &Error::Wg3d(ref err) => err.cause(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<gltf::Error> for Error {
    fn from(err: gltf::Error) -> Error {
        Error::Gltf(err)
    }
}

impl From<Wg3dError> for Error {
    fn from(err: Wg3dError) -> Error {
        Error::Wg3d(err)
    }
}

/// This is the result type.
pub type Result<T> = result::Result<T, Error>;

/// Error container for handling Wg3d
#[derive(Debug)]
pub enum Wg3dError {
    /// Primitive missing required attributes
    MissingAttributes,
    /// Unsupported data type
    UnsupportedDataType,
    /// Unsupported dimensions
    UnsupportedDimensions,
    /// Invalid buffer length
    InvalidBufferLength,
    /// Something weird
    Other,
}

impl fmt::Display for Wg3dError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Wg3dError::MissingAttributes => {
                write!(fmt, "Primitive missing required attributes")
            },
            Wg3dError::UnsupportedDataType => {
                write!(fmt, "Primitive attribute using unsupported data type")
            },
            Wg3dError::UnsupportedDimensions => {
                write!(fmt, "Primitive attribute using unsupported dimensions")
            },
            Wg3dError::InvalidBufferLength => {
                write!(fmt, "Invalid buffer length")
            },
            Wg3dError::Other => {
                write!(fmt, "Something weird happened")
            },
        }
    }
}

impl error::Error for Wg3dError {
    fn description(&self) -> &str {
        static MISSING_ATTRIBUTES: &'static str = "Primitive missing required attributes";
        static UNSUPPORTED_DATA_TYPE: &'static str = "Primitive attribute using unsupported data type";
        static UNSUPPORTED_DIMENSIONS: &'static str = "Primitive attribute using unsupported dimensions";
        static INVALID_BUFFER_LENGTH: &'static str = "Invalid buffer length";
        static OTHER: &'static str = "Something weird happened";

        match *self {
            Wg3dError::MissingAttributes => {
                MISSING_ATTRIBUTES
            },
            Wg3dError::UnsupportedDataType => {
                UNSUPPORTED_DATA_TYPE
            },
            Wg3dError::UnsupportedDimensions => {
                UNSUPPORTED_DIMENSIONS
            },
            Wg3dError::InvalidBufferLength => {
                INVALID_BUFFER_LENGTH
            }
            Wg3dError::Other => {
                OTHER
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        let parent = path.parent().unwrap();
        let gltf = load_gltf(path).ok().unwrap();

        let buffers = get_buffers(parent, &gltf).ok().unwrap();

        match get_nodes(&gltf, &buffers) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
