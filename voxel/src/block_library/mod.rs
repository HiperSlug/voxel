mod raw {
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
}

use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use block_mesh::{MergeVoxelContext, VoxelContext, VoxelVisibility};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::data::voxel::Voxel;

#[derive(Debug, Asset, TypePath)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: Textures,
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

struct BlockLoaderSettings {
    index_offset: usize,
}

#[derive(Debug, Default)]
pub struct BlockLoader;

impl AssetLoader for BlockLoader {
    type Asset = Block;
    type Settings = BlockLoaderSettings;
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            let raw::Block {
                display_name,
                collision_aabbs,
                is_translucent,
                textures,
            } = serde_json::de::from_slice::<raw::Block>(&buffer)?;

            let raw::Textures {
                pos_x,
                neg_x,
                pos_y,
                neg_y,
                pos_z,
                neg_z,
            } = textures;

            let textures = Textures {
                pos_x: context.load(pos_x),
            }

            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct BlockLibrary {
    pub blocks: Vec<Block>,

    pub material: Handle<TextureArrayMaterial>,

    pub name_to_index: HashMap<String, usize>,
    pub index_to_name: Vec<String>,
}

impl BlockLibrary {
    #[inline]
    pub fn index_from(&self, name: &str) -> Option<&usize> {
        self.name_to_index.get(name)
    }

    #[inline]
    pub fn name_from(&self, index: usize) -> Option<&String> {
        self.index_to_name.get(index)
    }
}

impl VoxelContext<Voxel> for BlockLibrary {
    fn get_visibility(&self, voxel: &Voxel) -> VoxelVisibility {
        if voxel.is_sentinel() {
            VoxelVisibility::Empty
        } else if let Some(block) = self.blocks.get(voxel.index()) {
            if block.is_translucent {
                VoxelVisibility::Translucent
            } else {
                VoxelVisibility::Opaque
            }
        } else {
            error!(
                "Could not find voxel {:?} in block_library {:?}",
                voxel, self,
            );
            VoxelVisibility::Empty
        }
    }
}

impl MergeVoxelContext<Voxel> for BlockLibrary {
    type MergeValue = u16;
    type MergeValueFacingNeighbour = u16;

    fn merge_value(&self, voxel: &Voxel) -> Self::MergeValue {
        voxel.id
    }

    fn merge_value_facing_neighbour(&self, voxel: &Voxel) -> Self::MergeValueFacingNeighbour {
        voxel.id
    }
}

fn resolve_path(load_context: &LoadContext, path: &str) -> PathBuf {
    if path.starts_with("/") {
        PathBuf::from(path.trim_start_matches('/'))
    } else {
        PathBuf::from(load_context.path().parent().unwrap_or(Path::new(""))).join(path)
    }
}

pub struct BlockLibraryPlugin;

impl Plugin for BlockLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterializePlugin::new(JsonMaterialDeserializer))
            .init_asset::<BlockLibrary>()
            .init_asset_loader::<BlockLibraryLoader>();
    }
}
