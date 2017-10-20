use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gltf::Gltf;

use super::super::{Result, Error, Wg3dError};

pub type Buffers = HashMap<String, Vec<u8>>;

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
        file.read_to_end(&mut contents)?;

        buffers.insert(buffer.uri().to_string(), contents);
    }

    Ok(buffers)
}
