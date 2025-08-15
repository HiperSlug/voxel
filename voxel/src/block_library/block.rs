use bevy::{math::bounding::Aabb3d, prelude::*};
use math::PerSignedAxis;
use std::collections::HashMap;

use super::intermediate::IntermediateBlock;

#[derive(Debug, Clone)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: PerSignedAxis<usize>,
}

impl Block {
    pub fn from_intermediate(
        intermediate: &IntermediateBlock,
        texture_context: &HashMap<String, usize>,
    ) -> Option<Self> {
        let IntermediateBlock {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        } = intermediate.clone();

        let opt_textures = textures.0.map(|k| texture_context.get(&k).copied());
        if !opt_textures.iter().all(|opt| opt.is_some()) {
            return None;
        }
        let textures = opt_textures.map(|opt| opt.unwrap()).into();

        Some(Self {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        })
    }
}
