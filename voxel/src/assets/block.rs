use std::collections::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use math::PerSignedAxis;
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice as json_de;

#[derive(Debug, Serialize, Deserialize, Asset, TypePath, Clone)]
pub struct IntermediateBlock {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: PerSignedAxis<String>,
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

#[derive(Debug, Clone)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: PerSignedAxis<usize>,
}

impl Block {
    pub fn from_intermediate(intermediate: &IntermediateBlock, texture_context: &HashMap<String, usize>) -> Option<Self> {
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
