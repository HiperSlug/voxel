pub mod block;
mod intermediate;
mod texture_array;

use bevy::prelude::*;
pub use block::Block;
use intermediate::{IntermediateBlock, IntermediateBlockLibrary};
use std::{collections::HashMap, ops::Index};

use crate::voxel::Voxel;

// TODO: intern Strings for faster lookup and not cloning it everywhere
#[derive(Debug)]
pub struct BlockLibrary {
    pub blocks: Vec<Block>,
    pub names: Vec<String>,
    pub blocks_map: HashMap<String, Block>,
    // pub material: Handle<TextureArrayMaterial>,
}

impl BlockLibrary {
    pub fn build(
        intermediate: &IntermediateBlockLibrary,
        image_assets: ResMut<Assets<Image>>,
        // material_assets: ResMut<Assets<TextureArrayMaterial>>,
        block_assets: Res<Assets<IntermediateBlock>>,
    ) -> Self {
        let IntermediateBlockLibrary {
            texture_size,
            textures,
            blocks: intermediate_blocks,
        } = intermediate;

        let (texture_name_to_index, image) =
            texture_array::build(textures, *texture_size, image_assets);

        let mut blocks = Vec::new();
        let mut names = Vec::new();
        let mut blocks_map = HashMap::new();

        for (name, handle) in intermediate_blocks {
            let Some(intermediate) = block_assets.get(handle) else {
                error!("IntermediateBlock asset not yet loaded");
                continue;
            };

            let Some(block) = Block::from_intermediate(intermediate, &texture_name_to_index) else {
                error!(
                    "IntermediateBlock {} has invalid texture",
                    intermediate.display_name,
                );
                continue;
            };

            blocks.push(block.clone());
            names.push(name.clone());
            blocks_map.insert(name.clone(), block);
        }

        Self {
            blocks,
            blocks_map,
            names,
            // material,
        }
    }
}

impl Index<Voxel> for BlockLibrary {
    type Output = Block;

    fn index(&self, index: Voxel) -> &Self::Output {
        &self.blocks[index.0.get() as usize]
    }
}
