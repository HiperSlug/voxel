use bevy::math::{UVec2, bounding::Aabb3d};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockLibrary {
    pub materials: Vec<Material>,
    pub blocks: HashMap<String, BlockVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub path: String,
    pub size: UVec2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockVariant {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_transparent: bool,
    pub block_model: BlockModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockModel {
    Empty,
    Cube(BlockModelCube),
    Mesh(BlockModelMesh),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockModelCube {
    pub material_index: usize,
    pub texture_coords: TextureCoords,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextureCoords {
    pub pos_x: UVec2,
    pub neg_x: UVec2,
    pub pos_y: UVec2,
    pub neg_y: UVec2,
    pub pos_z: UVec2,
    pub neg_z: UVec2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockModelMesh {
    pub path: String,
    pub material_index: usize,
}
