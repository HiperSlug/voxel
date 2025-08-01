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
    pub path: String, // This path is loaded at runtime
    pub size: UVec2,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Clone for shared ref
pub struct BlockVariant {
    pub display_name: String,
    pub collision_aabbs: Option<Vec<Aabb3d>>,
    pub is_empty: Option<bool>,
    pub block_model: BlockModel,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Clone for shared ref
pub enum BlockModel {
    Cube(BlockModelCube),
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Clone for shared ref
pub struct BlockModelCube {
    pub material_index: usize,
    pub is_translucent: bool,
    pub texture_coords: TextureCoords,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Clone for shared ref
pub struct TextureCoords {
    pub pos_x: UVec2,
    pub neg_x: UVec2,
    pub pos_y: UVec2,
    pub neg_y: UVec2,
    pub pos_z: UVec2,
    pub neg_z: UVec2,
}
