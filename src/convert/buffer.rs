use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gltf::Gltf;

use super::super::{Result, Error};
use super::ConvertError;

pub type Buffers = HashMap<String, Vec<u8>>;

pub fn get<'a>(
    base_path: &'a Path,
    gltf: &'a Gltf
) -> Result<Buffers> {
    let mut buffers = Buffers::new();

    for buffer in gltf.buffers() {
        let full_uri = base_path.join(buffer.uri());
        let mut file = File::open(full_uri)?;
        let metadata = file.metadata()?;

        if metadata.len() != (buffer.length() as u64) {
            return Err(Error::Convert(ConvertError::InvalidBufferLength));
        }
        
        let mut contents = Vec::<u8>::with_capacity(buffer.length()); 
        file.read_to_end(&mut contents)?;

        buffers.insert(buffer.uri().to_string(), contents);
    }

    Ok(buffers)
}

#[cfg(test)]
mod tests {
    use super::super::load_gltf;
    use super::*;

    #[test]
    fn test_convert_buffers_get() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        let parent = path.parent().unwrap();
        let gltf = load_gltf(path).unwrap();

        match get(&parent, &gltf) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
