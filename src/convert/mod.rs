use std::env::current_dir;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

use gltf::Gltf;
use gltfimp::import;

use super::Result;

pub mod buffer;
pub mod material;
pub mod mesh;
pub mod primitive;
pub mod skin;
pub mod texture;

// pub fn get_nodes<P: AsRef<Path>>(
//     path: P
// ) -> Result<()> {
//     let cwd = current_dir()?;
//     let parent = path.as_ref().parent().unwrap_or(&cwd);
//     let gltf = load_gltf(path.as_ref())?;
//     let buffers = buffer::get(&parent, &gltf)?;
//     let textures = texture::get(&parent, &gltf, &buffers)?;

//     for node in gltf.nodes() {
//         if let Some(mesh) = node.mesh() {
//             let _ = mesh::get(&mesh, node.skin(), &buffers, &textures)?;
//         }
//     }

//     Ok(())
// }

// fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Gltf> {
//     let file = File::open(path)?;
//     let gltf = Gltf::from_reader(io::BufReader::new(file))?
//     .validate_minimally()?;

//     Ok(gltf)
// }

pub fn get<P: AsRef<Path>>(path: P) -> Result<()> {
    let cwd = current_dir()?;
    let parent = path.as_ref().parent().unwrap_or(&cwd);
    let (gltf, buffers) = import(&path)?;
    let textures = texture::get(&parent, &gltf, &buffers)?;

    Ok(())
}

/// Error container for handling Wg3d
#[derive(Debug)]
pub enum ConvertError {
    /// Primitive missing required attributes
    MissingAttributes,
    /// Unsupported data type
    UnsupportedDataType,
    /// Unsupported dimensions
    UnsupportedDimensions,
    /// Invalid buffer length
    InvalidBufferLength,
    /// Image buffer not present
    MissingImageBuffer,
    /// Multiple textures share binary buffer
    MultipleTexturesInBuffer,
    /// No specified root node of skeleton for a skin
    NoSkeleton,
    /// Something weird
    Other,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConvertError::MissingAttributes => {
                write!(fmt, "Primitive missing required attributes")
            },
            ConvertError::UnsupportedDataType => {
                write!(fmt, "Primitive attribute using unsupported data type")
            },
            ConvertError::UnsupportedDimensions => {
                write!(fmt, "Primitive attribute using unsupported dimensions")
            },
            ConvertError::InvalidBufferLength => {
                write!(fmt, "Invalid buffer length")
            },
            ConvertError::MissingImageBuffer => {
                write!(fmt, "Missing image buffer")
            },
            ConvertError::MultipleTexturesInBuffer => {
                write!(fmt, "Multiple textures share binary buffer")
            },
            ConvertError::NoSkeleton => {
                write!(fmt, "No specified root node of skeleton for a skin")
            },
            ConvertError::Other => {
                write!(fmt, "Something weird happened")
            },
        }
    }
}

impl error::Error for ConvertError {
    fn description(&self) -> &str {
        static MISSING_ATTRIBUTES: &'static str = "Primitive missing required attributes";
        static UNSUPPORTED_DATA_TYPE: &'static str = "Primitive attribute using unsupported data type";
        static UNSUPPORTED_DIMENSIONS: &'static str = "Primitive attribute using unsupported dimensions";
        static INVALID_BUFFER_LENGTH: &'static str = "Invalid buffer length";
        static MISSING_IMAGE_BUFFER: &'static str = "Missing image buffer";
        static MULTIPLE_TEXTURES_IN_BUFFER: &'static str = "Multiple textures share binary buffer";
        static NO_SKELETON: &'static str = "No specified root node of skeleton for a skin";
        static OTHER: &'static str = "Something weird happened";

        match *self {
            ConvertError::MissingAttributes => {
                MISSING_ATTRIBUTES
            },
            ConvertError::UnsupportedDataType => {
                UNSUPPORTED_DATA_TYPE
            },
            ConvertError::UnsupportedDimensions => {
                UNSUPPORTED_DIMENSIONS
            },
            ConvertError::InvalidBufferLength => {
                INVALID_BUFFER_LENGTH
            },
            ConvertError::MissingImageBuffer => {
                MISSING_IMAGE_BUFFER
            },
            ConvertError::MultipleTexturesInBuffer => {
                MULTIPLE_TEXTURES_IN_BUFFER
            },
            ConvertError::NoSkeleton => {
                NO_SKELETON
            },
            ConvertError::Other => {
                OTHER
            },
        }
    }

    fn cause(&self) -> Option<&error::Error> { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        if let Ok(_) = get(path) {
        } else {
            assert!(false);
        }
    }

    // fn get_gltf(path: &Path) -> Gltf {
    //     let file = File::open(path).unwrap();
    //     Gltf::from_reader(io::BufReader::new(file)).unwrap()
    //         .validate_minimally().unwrap()
    // }
    
    // #[test]
    // fn test_load_gltf() {
    //     let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");

    //     if let Ok(_) = load_gltf(path) {
    //     } else {
    //         assert!(false);
    //     }
    // }
    
    // #[test]
    // fn test_get_nodes() {
    //     let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
    //     let parent = path.parent().unwrap();
    //     let gltf = get_gltf(&path);

    //     match get_nodes(path) {
    //         Ok(_) => {},
    //         Err(err) => {
    //             println!("{}", err.to_string());
    //             assert!(false);
    //         }
    //     }
    // }
}
