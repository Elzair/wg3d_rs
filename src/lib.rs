extern crate bincode;
extern crate byteorder;
extern crate gltf;
extern crate image;
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
use gltf::image as gltf_image;
use gltf::material;
use gltf::mesh::{Primitive, Semantic};
use gltf::texture;

mod convert;
use convert::buffer::Buffers;

/// This is the top level Error for this crate.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Gltf(gltf::Error),
    Image(image::ImageError),
    Wg3d(Wg3dError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => err.fmt(fmt),
            &Error::Gltf(ref err) => err.fmt(fmt),
            &Error::Image(ref err) => err.fmt(fmt),
            &Error::Wg3d(ref err) => err.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref err) => err.description(),
            &Error::Gltf(ref err) => err.description(),
            &Error::Image(ref err) => err.description(),
            &Error::Wg3d(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::Io(ref err) => err.cause(),
            &Error::Gltf(ref err) => err.cause(),
            &Error::Image(ref err) => err.cause(),
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

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Error {
        Error::Image(err)
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
    MissingImageBuffer,
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
            Wg3dError::MissingImageBuffer => {
                write!(fmt, "Missing image buffer")
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
        static MISSING_IMAGE_BUFFER: &'static str = "Missing image buffer";
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
            Wg3dError::MissingImageBuffer => {
                MISSING_IMAGE_BUFFER
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
    }
}
