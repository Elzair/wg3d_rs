extern crate bincode;
extern crate byteorder;
extern crate cgmath;
extern crate float_cmp;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;
extern crate image;
extern crate itertools;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::error;
use std::fmt;
use std::io;
use std::result;

pub mod convert;

/// This is the top level Error for this crate.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Gltf(gltf::Error),
    GltfImport(gltf_importer::Error),
    Image(image::ImageError),
    Convert(convert::ConvertError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => err.fmt(fmt),
            &Error::Gltf(ref err) => err.fmt(fmt),
            &Error::GltfImport(ref err) => err.fmt(fmt),
            &Error::Image(ref err) => err.fmt(fmt),
            &Error::Convert(ref err) => err.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref err) => err.description(),
            &Error::Gltf(ref err) => err.description(),
            &Error::GltfImport(ref err) => err.description(),
            &Error::Image(ref err) => err.description(),
            &Error::Convert(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::Io(ref err) => err.cause(),
            &Error::Gltf(ref err) => err.cause(),
            &Error::GltfImport(ref err) => err.cause(),
            &Error::Image(ref err) => err.cause(),
            &Error::Convert(ref err) => err.cause(),
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

impl From<gltf_importer::Error> for Error {
    fn from(err: gltf_importer::Error) -> Error {
        Error::GltfImport(err)
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Error {
        Error::Image(err)
    }
}

impl From<convert::ConvertError> for Error {
    fn from(err: convert::ConvertError) -> Error {
        Error::Convert(err)
    }
}

/// This is the result type.
pub type Result<T> = result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
    }
}
