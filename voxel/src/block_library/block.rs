use bevy::{math::bounding::Aabb3d, prelude::*};
use math::prelude::*;
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

        let textures = PerSignedAxis::try_from_fn(|s| {
            texture_context.get(textures.get(s)).copied()
        })?;

        Some(Self {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        })
    }
}
