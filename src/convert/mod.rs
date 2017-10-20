use std::fs::File;
use std::io;
use std::path::Path;

use gltf::Gltf;

use super::Result;

pub mod buffer;
pub mod mesh;
pub mod texture;

pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Gltf> {
    let file = File::open(path)?;
    let gltf = Gltf::from_reader(io::BufReader::new(file))?
    .validate_minimally()?;

    Ok(gltf)
}

pub fn get_nodes<'a>(
    gltf: &'a Gltf,
    buffers: &'a buffer::Buffers
) -> Result<()> {
    for node in gltf.nodes() {
        if let Some(mesh) = node.mesh() {
            // See if the node also has a skin.
            let has_joints = node.skin().is_some();
            
            let _ = mesh::get_mesh(gltf, mesh.index(), has_joints, buffers)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_gltf() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        let parent = path.parent().unwrap();
        let gltf = load_gltf(path).ok().unwrap();

        let buffers = buffer::get_buffers(parent, &gltf).ok().unwrap();

        match get_nodes(&gltf, &buffers) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
