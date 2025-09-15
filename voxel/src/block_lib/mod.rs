pub mod block;
mod intermediate;
mod texture_array;

use arc_swap::ArcSwap;
use bevy::{platform::collections::HashMap, prelude::*};
pub use block::Block;
use intermediate::{IntermediateBlock, IntermediateBlockLib};
use std::{
    ops::Index,
    sync::{LazyLock, OnceLock},
};
use string_interner::{
    DefaultSymbol, StringInterner,
    backend::{BucketBackend, BufferBackend},
};

use crate::voxel::Voxel;

pub static BLOCK_LIBRARY: OnceLock<ArcSwap<BlockLibrary>> = OnceLock::new();

pub struct BlockLibrary {
    pub blocks: Vec<Block>,
    pub names: Vec<DefaultSymbol>,
    pub blocks_map: HashMap<DefaultSymbol, Block>,
    pub interner: StringInterner<BufferBackend>,
}

impl BlockLibrary {
    pub fn build(
        intermediate: &IntermediateBlockLib,
        image_assets: ResMut<Assets<Image>>,
        block_assets: Res<Assets<IntermediateBlock>>,
        // material_assets: ResMut<Assets<TextureArrayMaterial>>,
    ) -> Self {
        let IntermediateBlockLib {
            texture_size,
            textures,
            blocks: intermediate_blocks,
            block_interner,
            tex_interner,
        } = intermediate;

        let (tex_name_to_index, image) =
            texture_array::build(textures, *texture_size, image_assets);

        let mut blocks = Vec::new();
        let mut names = Vec::new();
        let mut blocks_map = HashMap::new();

        for (name, handle) in intermediate_blocks {
            let Some(intermediate) = block_assets.get(handle) else {
                error!("IntermediateBlock {name:?} not yet loaded");
                continue;
            };

            let Some(block) =
                Block::from_intermediate(intermediate, &tex_name_to_index, tex_interner)
            else {
                error!("IntermediateBlock {name:?} has invalid texture");
                continue;
            };

            blocks.push(block.clone());
            names.push(*name);
            blocks_map.insert(*name, block);
        }

        Self {
            blocks,
            blocks_map,
            names,
            block_interner: *block_interner,
        }
    }
}

impl Index<Voxel> for BlockLibrary {
    type Output = Block;

    fn index(&self, index: Voxel) -> &Self::Output {
        &self.blocks[index.0.get() as usize]
    }
}
