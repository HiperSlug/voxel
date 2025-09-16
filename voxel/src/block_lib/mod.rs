pub mod block;
mod intermediate;
mod texture_array;

use bevy::{platform::collections::HashMap, prelude::*};
pub use block::Block;
use intermediate::{IntermediateBlock, IntermediateBlockLib};
use std::{ops::Index, sync::Arc};
use string_interner::{DefaultSymbol, StringInterner, backend::BufferBackend};

use crate::voxel::Voxel;

pub type Interner = StringInterner<BufferBackend>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: DefaultSymbol,
    pub name: DefaultSymbol,
}

#[derive(Resource, Clone, Deref)]
pub struct BlockLibrary(pub Arc<InnerBlockLibrary>);

pub struct InnerBlockLibrary {
    pub blocks: Vec<Block>,
    pub identifiers: Vec<Identifier>,
    pub blocks_map: HashMap<Identifier, usize>,
    pub interner: Interner,
}

impl InnerBlockLibrary {
    pub fn build(
        intermediate: &IntermediateBlockLib,
        image_assets: ResMut<Assets<Image>>,
        block_assets: Res<Assets<IntermediateBlock>>,
        // material_assets: ResMut<Assets<TextureArrayMaterial>>,
    ) -> Self {
        let IntermediateBlockLib {
            blocks: intermediate_blocks,
            textures,
            texture_size,
            interner,
        } = intermediate;

        let (identifier_to_index, image) =
            texture_array::build(textures, *texture_size, image_assets);

        let mut blocks = Vec::new();
        let mut identifiers = Vec::new();
        let mut blocks_map = HashMap::new();

        for (identifier, handle) in intermediate_blocks {
            let intermediate = block_assets.get(handle).unwrap();

            let Some(block) =
                Block::from_intermediate(intermediate, &identifier_to_index, interner)
            else {
                error!("IntermediateBlock {name:?} has invalid texture");
                continue;
            };

            let index = blocks.len();

            blocks.push(block);
            identifiers.push(*identifier);
            blocks_map.insert(*identifier, index);
        }

        Self {
            blocks,
            identifiers,
            blocks_map,
            interner: interner.clone(),
        }
    }
}

impl Index<Voxel> for InnerBlockLibrary {
    type Output = Block;

    fn index(&self, index: Voxel) -> &Self::Output {
        &self.blocks[index.0.get() as usize]
    }
}

impl Index<Name> for InnerBlockLibrary {
    type Output = Block;

    fn index(&self, index: Name) -> &Self::Output {
        &self.blocks[*self.blocks_map.get(&index).unwrap()]
    }
}
