use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: Textures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Textures {
    pub pos_x: String,
    pub neg_x: String,
    pub pos_y: String,
    pub neg_y: String,
    pub pos_z: String,
    pub neg_z: String,
}

#[derive(Debug, Default)]
pub struct BlockLoader;

impl AssetLoader for BlockLoader {
    type Asset = Block;
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

            let out = serde_json::de::from_slice::<Block>(&buffer)?;
            Ok(out)
        }
    }
}
