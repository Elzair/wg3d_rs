extern crate gltf;
extern crate wg3d;

use std::fs::File;
use std::io;
use std::path::Path;
use gltf::Gltf;

fn main() {
    let file = File::open("testmodels/gltf/model.gltf").ok().unwrap();
    let gltf = Gltf::from_reader(io::BufReader::new(file)).ok().unwrap()
        .validate_minimally().ok().unwrap();

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!(
                "Node {} has {} children",
                node.index(),
                node.children().count(),
            );
        }
    }
}
