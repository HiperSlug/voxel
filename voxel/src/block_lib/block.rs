use crate::math::signed_axis::SignedAxisMap;
use bevy::{
    ecs::intern::{Interned, Interner},
    math::bounding::Aabb3d,
    platform::collections::HashMap,
    prelude::*,
};

use super::intermediate::IntermediateBlock;

#[derive(Debug, Clone)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_transparent: bool,
    pub textures: SignedAxisMap<u32>,
}

impl Block {
    pub fn from_intermediate(
        intermediate: &IntermediateBlock,
        tex_name_to_index: &HashMap<Interned<str>, u32>,
        tex_interner: &Interner<str>,
    ) -> Option<Self> {
        let IntermediateBlock {
            display_name,
            collision_aabbs,
            is_transparent,
            textures: texture_names,
        } = intermediate.clone();

        let texture_names = texture_names.map(|_, n| tex_interner.intern(&*n));

        let opt_textures = texture_names.map(|_, s| tex_name_to_index.get(&s));

        if opt_textures.iter().any(|(_, opt)| opt.is_none()) {
            return None;
        }

        let textures = opt_textures.map(|_, opt| *opt.unwrap());

        Some(Self {
            display_name,
            collision_aabbs,
            is_transparent,
            textures,
        })
    }
}
