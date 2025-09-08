use crate::math::signed_axis::SignedAxisMap;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice as json_de;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
struct BlockLibraryConfig {
    libraries: Vec<String>,
    texture_size: UVec2,
}

#[derive(Debug, Asset, TypePath)]
pub struct IntermediateBlockLibrary {
    pub blocks: Vec<(String, Handle<IntermediateBlock>)>,
    pub textures: Vec<(String, Handle<Image>)>,
    pub texture_size: UVec2,
}

pub struct IntermediateBlockLibraryLoader;

impl AssetLoader for IntermediateBlockLibraryLoader {
    type Asset = IntermediateBlockLibrary;
    type Error = anyhow::Error;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let BlockLibraryConfig {
                libraries,
                texture_size,
            } = json_de(&bytes)?;

            let mut blocks = Vec::new();
            let mut textures = Vec::new();

            for library in libraries {
                fn push_names_and_handles<T: Asset>(
                    vec: &mut Vec<(String, Handle<T>)>,
                    path: &str,
                    load_context: &mut LoadContext,
                ) {
                    for result in WalkDir::new(path)
                        .into_iter()
                        .filter_entry(|e| e.file_type().is_file())
                    {
                        let dir = match result {
                            Ok(dir) => dir,
                            Err(e) => {
                                warn!("Error {e} walking {path}");
                                continue;
                            }
                        };

                        let path = dir.path();
                        let Some(os_name) = path.file_stem() else {
                            warn!("Nameless at {path:?} skipping");
                            continue;
                        };

                        let Some(name) = os_name.to_str() else {
                            warn!("Invalid utf-8 name at {path:?} skipping");
                            continue;
                        };

                        let handle = load_context.load(path);
                        vec.push((name.to_string(), handle));
                    }
                }

                let blocks_path = format!("block_libs/{library}/blocks");
                let textures_path = format!("block_libs/{library}/textures");

                push_names_and_handles(&mut blocks, &blocks_path, load_context);
                push_names_and_handles(&mut textures, &textures_path, load_context);
            }

            Ok(IntermediateBlockLibrary {
                blocks,
                textures,
                texture_size,
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Asset, TypePath, Clone)]
pub struct IntermediateBlock {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_transparent: bool,
    pub textures: SignedAxisMap<String>,
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
        reader: &mut dyn bevy::asset::io::Reader,
        _: &Self::Settings,
        _: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            Ok(json_de(&buffer)?)
        }
    }
}
