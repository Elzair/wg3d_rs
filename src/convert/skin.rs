use gltf::skin::Skin as GltfSkin;

use super::super::{Result, Error};
use super::buffer::Buffers;

pub struct Skin {
}

pub fn get<'a>(
    skin: Option<GltfSkin>,
    buffers: &'a Buffers,
) -> Result<Option<Skin>> {
    Ok(Some(Skin {
    }))
}
