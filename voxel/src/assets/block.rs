use std::collections::HashMap;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use math::SignedAxis;
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice as json_de;

#[derive(Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct IntermediateBlock {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: IntermediateTextures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntermediateTextures {
    pub pos_x: String,
    pub neg_x: String,
    pub pos_y: String,
    pub neg_y: String,
    pub pos_z: String,
    pub neg_z: String,
}

#[derive(Debug, Default)]
pub struct IntermediateBlockLoader;

impl AssetLoader for IntermediateBlockLoader {
    type Asset = IntermediateBlock;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            Ok(json_de(&buffer)?)
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: Textures,
}

impl Block {
    pub fn from_intermediate(intermediate: IntermediateBlock, texture_context: HashMap<String, usize>) -> Option<Self> {
        let IntermediateBlock {
            display_name,
            collision_aabbs,
            is_translucent,
            textures,
        } = intermediate;

        Some(Self {
            display_name,
            collision_aabbs,
            is_translucent,
            textures: Textures::from_intermediate(textures, texture_context)?
        })
    }
}

#[derive(Debug)]
pub struct Textures {
    pub pos_x: usize,
    pub neg_x: usize,
    pub pos_y: usize,
    pub neg_y: usize,
    pub pos_z: usize,
    pub neg_z: usize,
}

impl Textures {
    pub fn from_intermediate(intermediate: IntermediateTextures, texture_context: HashMap<String, usize>) -> Option<Self> {
        let IntermediateTextures {
            pos_x,
            neg_x,
            pos_y,
            neg_y,
            pos_z,
            neg_z,
        } = intermediate;

        Some(Self {
            pos_x: *texture_context.get(&pos_x)?,
            neg_x: *texture_context.get(&neg_x)?,
            pos_y: *texture_context.get(&pos_y)?,
            neg_y: *texture_context.get(&neg_y)?,
            pos_z: *texture_context.get(&pos_z)?,
            neg_z: *texture_context.get(&neg_z)?,
        })
    }

    pub fn get(&self, signed_axis: SignedAxis) -> usize {
        use SignedAxis::*;

        match signed_axis {
            PosX => self.pos_x,
            NegX => self.neg_x,
            PosY => self.pos_y,
            NegY => self.neg_y,
            PosZ => self.pos_z,
            NegZ => self.neg_z,
        }
    }
}
