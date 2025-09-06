use crate::math::signed_axis::SignedAxisMap;
use bevy::{math::bounding::Aabb3d, prelude::*};
use std::collections::HashMap;

use super::intermediate::IntermediateBlock;

#[derive(Debug, Clone)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: SignedAxisMap<usize>,
}

impl Block {
    pub fn from_intermediate(
        intermediate: &IntermediateBlock,
        texture_name_to_index: &HashMap<String, usize>,
    ) -> Option<Self> {
        let IntermediateBlock {
            display_name,
            collision_aabbs,
            is_translucent,
            textures: texture_names,
        } = intermediate.clone();

        let opt_textures = texture_names.map(|_, s| texture_name_to_index.get(&s));

        if opt_textures.iter().any(|(_, opt)| opt.is_none()) {
            return None;
        }

        let textures = opt_textures.map(|_, opt| *opt.unwrap());

        Some(Self {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        })
    }
}
