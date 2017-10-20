use std::env::current_dir;
use std::fs::File;
use std::io;
use std::path::Path;

use gltf::Gltf;

use super::Result;

pub mod buffer;
pub mod mesh;
pub mod texture;

pub fn get_nodes<P: AsRef<Path>>(
    path: P
) -> Result<()> {
    let cwd = current_dir()?;
    let parent = path.as_ref().parent().unwrap_or(&cwd);
    let gltf = load_gltf(path.as_ref())?;
    let buffers = buffer::get(&parent, &gltf)?;

    for node in gltf.nodes() {
        if let Some(mesh) = node.mesh() {
            // See if the node also has a skin.
            let has_joints = node.skin().is_some();
            
            let _ = mesh::get_mesh(&gltf, mesh.index(), has_joints, &buffers)?;
        }
    }

    Ok(())
}

fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Gltf> {
    let file = File::open(path)?;
    let gltf = Gltf::from_reader(io::BufReader::new(file))?
    .validate_minimally()?;

    Ok(gltf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_gltf(path: &Path) -> Gltf {
        let file = File::open(path).unwrap();
        Gltf::from_reader(io::BufReader::new(file)).unwrap()
            .validate_minimally().unwrap()
    }
    
    #[test]
    fn test_load_gltf() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");

        if let Ok(_) = load_gltf(path) {
        } else {
            assert!(false);
        }
    }
    
    #[test]
    fn test_get_nodes() {
        let path = Path::new("testmodels/gltf2/Monster/Monster.gltf");
        let parent = path.parent().unwrap();
        let gltf = get_gltf(&path);

        match get_nodes(path) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err.to_string());
                assert!(false);
            }
        }
    }
}
