extern crate bincode;
extern crate gltf;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::result;

use gltf::Gltf;

pub fn test() -> Result<()> {
    let file = File::open("testmodels/gltf2/Box/Box.gltf")?;
    let gltf = Gltf::from_reader(io::BufReader::new(file))?
        .validate_minimally()?;

    println!("Got here!");

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!(
                "Node {} has {} children",
                node.index(),
                node.children().count(),
            );
        }
    }

    Ok(())
}

/// This is the top level Error for this crate.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Gltf(gltf::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => err.fmt(fmt),
            &Error::Gltf(ref err) => err.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref err) => err.description(),
            &Error::Gltf(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::Io(ref err) => err.cause(),
            &Error::Gltf(ref err) => err.cause(),
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

/// This is the result type.
pub type Result<T> = result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        match test() {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
