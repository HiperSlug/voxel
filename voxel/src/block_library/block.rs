use bevy::{math::bounding::Aabb3d, prelude::*};
use math::prelude::*;
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
        texture_context: &HashMap<String, usize>,
    ) -> Option<Self> {
        let IntermediateBlock {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        } = intermediate.clone();

        let textures = SignedAxisMap::from_fn(|s| {
            let name = &textures.as_array()[s.into_usize()];
            texture_context.get(name)
        });

        if textures.iter().any(|(_, opt)| opt.is_none()) {
            return None
        }

        let textures = textures.map(|_, opt| *opt.unwrap());

        Some(Self {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        })
    }
}
