use bevy::{asset::AssetLoader, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice as json_de;
use std::collections::HashMap;
use walkdir::WalkDir;

use super::block::Block;

#[derive(Debug, Deserialize, Serialize)]
struct BlockLibraryConfig {
    libraries: Vec<String>,
	texture_size: UVec2, 
}

#[derive(Debug, Asset, TypePath)]
struct IntermediateBlockLibrary {
    blocks: HashMap<String, Handle<Block>>,
    textures: HashMap<String, Handle<Image>>,
	texture_size: UVec2, 
}

const ROOT: &str = "block_libs";
const BLOCKS: &str = "blocks";
const TEXTURES: &str = "textures";

struct IntermediateBlockLibraryLoader;

impl AssetLoader for IntermediateBlockLibraryLoader {
    type Asset = IntermediateBlockLibrary;
    type Error = anyhow::Error;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

			let BlockLibraryConfig {
				libraries,
				texture_size,
			} = json_de::<BlockLibraryConfig>(&bytes)?;

            let mut blocks = HashMap::new();
            let mut textures = HashMap::new();

            for library in libraries {
                let blocks_path = format!("{ROOT}/{library}/{BLOCKS}");
                for result in WalkDir::new(&blocks_path).into_iter().filter_entry(|e| e.file_type().is_file()) {
					let dir = match result {
						Ok(dir) => dir,
						Err(e) => {
							warn!("Error {e} walking {blocks_path}");
							continue;
						},
					};

                    let path = dir.path();
                    let Some(os_name) = path.file_stem() else {
						warn!("Nameless block at {path:?} skipping");
                        continue;
                    };

					let Some(name) = os_name.to_str() else {
						warn!("Invalid utf8 name at {path:?} skipping");
                        continue;
					};

                    let handle = load_context.load(path);
                    blocks.insert(name.to_string(), handle);
                }

                let textures_path = format!("{ROOT}/{library}/{TEXTURES}");
				for result in WalkDir::new(&textures_path).into_iter().filter_entry(|e| e.file_type().is_file()) {
					let dir = match result {
						Ok(dir) => dir,
						Err(e) => {
							warn!("Error {e} walking {textures_path}");
							continue;
						},
					};

                    let path = dir.path();
                    let Some(os_name) = path.file_stem() else {
						warn!("Nameless texture at {path:?} skipping");
                        continue;
                    };

					let Some(name) = os_name.to_str() else {
						warn!("Invalid utf8 name at {path:?} skipping");
                        continue;
					};

                    let handle = load_context.load(path);
                    textures.insert(name.to_string(), handle);
                }
            }

			Ok(IntermediateBlockLibrary {
				blocks,
				textures,
				texture_size,
			})
        }
    }
}

#[derive(Debug)]
pub struct BlockLibrary {
	blocks: Block
}
