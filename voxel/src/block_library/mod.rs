pub mod block;
mod intermediate;
pub mod texture_array;

use bevy::prelude::*;
use std::collections::HashMap;

pub use block::Block;
use intermediate::{IntermediateBlock, IntermediateBlockLibrary};
use texture_array::{TextureArrayMaterial, build_texture_array};

#[derive(Debug)]
pub struct BlockLibrary {
    pub blocks: Vec<Block>,
    pub names: Vec<String>,
    pub blocks_map: HashMap<String, Block>,
    pub texture_map: HashMap<String, usize>,
    pub material: Handle<TextureArrayMaterial>,
}

impl BlockLibrary {
    pub fn build(
        intermediate: IntermediateBlockLibrary,
        image_assets: ResMut<Assets<Image>>,
        material_assets: ResMut<Assets<TextureArrayMaterial>>,
        block_assets: &Res<Assets<IntermediateBlock>>,
    ) -> Self {
        let IntermediateBlockLibrary {
            texture_size,
            textures,
            blocks: intermediate_blocks,
        } = intermediate;

        let (texture_map, material) =
            build_texture_array(textures, texture_size, image_assets, material_assets);

        let mut blocks = Vec::new();
        let mut names = Vec::new();
        let mut blocks_map = HashMap::new();

        for (name, handle) in intermediate_blocks {
            let block = match block_assets.get(handle.id()) {
                Some(thing) => thing,
                None => {
                    error!("IntermediateBlock asset not yet loaded");
                    continue;
                }
            };

            let block = match Block::from_intermediate(block, &texture_map) {
                Some(thing) => thing,
                None => {
                    error!(
                        "IntermediateBlock {} has invalid texture",
                        block.display_name
                    );
                    continue;
                }
            };
            blocks.push(block.clone());
            names.push(name.clone());
            blocks_map.insert(name, block);
        }

        Self {
            blocks,
            blocks_map,
            names,
            material,
            texture_map,
        }
    }
}
