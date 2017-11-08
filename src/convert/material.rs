use gltf::gltf::Materials as GltfMaterials;
use gltf::material::{AlphaMode as GltfAlphaMode, Material as GltfMaterial};

use super::super::Result;
use super::ConvertError;
use super::texture::Textures;

pub struct Materials {
    materials: Vec<Material>,
}

impl Materials {
    pub fn get(&self, index: usize) -> Option<&str> {
        match self.materials.iter().nth(index) {
            Some(material) => Some(material.name.as_ref()),
            None => None,
        }
    }
}

pub struct Material {
    name: String,
    alpha_cutoff: f32,
    alpha_mode: AlphaMode,
    double_sided: bool,
    base_color: BaseColor,
    metal_roughness: MetallicRoughness,
    normal_map: Option<NormalMap>,
    occlusion_map: Option<OcclusionMap>,
    emission_map: Option<EmissionMap>,
}

pub enum AlphaMode {
    Blend,
    Mask,
    Opaque,
}

pub fn get<'a>(
    materials: GltfMaterials,
    textures: &'a Textures, 
) -> Result<Materials> {
    let my_materials = materials.map(|material| {
        let name = material.name().ok_or(ConvertError::NoName)?;
        let alpha_cutoff = material.alpha_cutoff();
        let alpha_mode = match material.alpha_mode() {
            GltfAlphaMode::Blend => AlphaMode::Blend,
            GltfAlphaMode::Mask => AlphaMode::Mask,
            GltfAlphaMode::Opaque => AlphaMode::Opaque,
        };
        let double_sided = material.double_sided();

        let base_color = get_base_color(&material, textures)?;
        let metal_roughness = get_metallic_roughness(&material, textures)?;
        let normal_map = get_normal_map(&material, textures)?;
        let occlusion_map = get_occlusion_map(&material, textures)?;
        let emission_map = get_emission_map(&material, textures)?;
        
        Ok(Material {
            name: name.to_owned(),
            alpha_cutoff: alpha_cutoff,
            alpha_mode: alpha_mode,
            double_sided: double_sided,
            base_color: base_color,
            metal_roughness: metal_roughness,
            normal_map: normal_map,
            occlusion_map: occlusion_map,
            emission_map: emission_map,
        })
    }).collect::<Result<Vec<_>>>()?;

    Ok(Materials {
        materials: my_materials,
    })
}

pub enum BaseColor {
    Factor([f32; 4]),
    Texture {
        tex_coord: u32,
        name: String,
    },
}

fn get_base_color<'a>(
    material: &'a GltfMaterial,
    textures: &'a Textures, 
) -> Result<BaseColor> {
    let pbr = material.pbr_metallic_roughness();

    match pbr.base_color_texture() {
        Some(tex) => {
            let tex_coord = tex.tex_coord();
            let name = textures.get(tex.texture().index())
                .ok_or(ConvertError::MissingImageBuffer)?;
            
            Ok(BaseColor::Texture {
                tex_coord: tex_coord,
                name: name.to_owned(),
            })
        },
        None => Ok(BaseColor::Factor(pbr.base_color_factor())),
    }
}

pub enum MetallicRoughness {
    Factor {
        metallicity: f32,
        roughness: f32,
    },
    Texture {
        tex_coord: u32,
        name: String,
    },
}

fn get_metallic_roughness<'a>(
    material: &'a GltfMaterial,
    textures: &'a Textures, 
) -> Result<MetallicRoughness> {
    let pbr = material.pbr_metallic_roughness();

    match pbr.metallic_roughness_texture() {
        Some(tex) => {
            let tex_coord = tex.tex_coord();
            let name = textures.get(tex.texture().index())
                .ok_or(ConvertError::MissingImageBuffer)?;
            
            Ok(MetallicRoughness::Texture {
                tex_coord: tex_coord,
                name: name.to_owned(),
            })
        },
        None => Ok(MetallicRoughness::Factor {
            metallicity: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
        }),
    }
}

pub struct NormalMap {
    scale: f32,
    tex_coord: u32,
    name: String,
}

fn get_normal_map<'a>(
    material: &'a GltfMaterial,
    textures: &'a Textures, 
) -> Result<Option<NormalMap>> {
    match material.normal_texture() {
        Some(tex) => {
            let scale = tex.scale();
            let tex_coord = tex.tex_coord();
            let name = textures.get(tex.texture().index())
                .ok_or(ConvertError::MissingImageBuffer)?;

            Ok(Some(NormalMap {
                scale: scale,
                tex_coord: tex_coord,
                name: name.to_owned(),
            }))
        },
        None => { Ok(None) },
    }
}

pub struct OcclusionMap {
    strength: f32,
    tex_coord: u32,
    name: String,
}

fn get_occlusion_map<'a>(
    material: &'a GltfMaterial,
    textures: &'a Textures, 
) -> Result<Option<OcclusionMap>> {
    match material.occlusion_texture() {
        Some(tex) => {
            let strength = tex.strength();
            let tex_coord = tex.tex_coord();
            let name = textures.get(tex.texture().index())
                .ok_or(ConvertError::MissingImageBuffer)?;

            Ok(Some(OcclusionMap {
                strength: strength,
                tex_coord: tex_coord,
                name: name.to_owned(),
            }))
        },
        None => { Ok(None) },
    }
}

pub enum EmissionMap {
    Factor([f32; 3]),
    Texture {
        tex_coord: u32,
        name: String,
    },
}

fn get_emission_map<'a>(
    material: &'a GltfMaterial,
    textures: &'a Textures, 
) -> Result<Option<EmissionMap>> {
    match material.emissive_texture() {
        Some(tex) => {
            let tex_coord = tex.tex_coord();
            let name = textures.get(tex.texture().index())
                .ok_or(ConvertError::MissingImageBuffer)?;
            
            Ok(Some(EmissionMap::Texture {
                tex_coord: tex_coord,
                name: name.to_owned(),
            }))
        },
        None => {
            let emissive_factor = material.emissive_factor();
            if emissive_factor == [0.0_f32, 0.0_f32, 0.0_f32] {
                Ok(None)
            } else {
                Ok(Some(EmissionMap::Factor(emissive_factor)))
            }
        },
    }
}
