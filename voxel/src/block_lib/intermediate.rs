use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice as json_de;
use string_interner::DefaultSymbol;
use walkdir::WalkDir;

use crate::math::signed_axis::SignedAxisMap;

use super::{Identifier, Interner};

#[derive(Deserialize, Serialize)]
struct BlockLibConfig {
    libraries: Vec<String>,
    texture_size: UVec2,
}

#[derive(Asset, TypePath)]
pub struct IntermediateBlockLib {
    pub interner: Interner,
    pub blocks: Vec<(Identifier, Handle<IntermediateBlock>)>,
    pub textures: Vec<(Identifier, Handle<Image>)>,
    pub texture_size: UVec2,
}

pub struct IntermediateBlockLibLoader;

impl AssetLoader for IntermediateBlockLibLoader {
    type Asset = IntermediateBlockLib;
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

            let BlockLibConfig {
                libraries,
                texture_size,
            } = json_de(&bytes)?;

            let mut blocks = Vec::new();
            let mut textures = Vec::new();

            let mut interner = Interner::new();

            for lib_name in libraries {
                let namespace = interner.get_or_intern(&lib_name);

                let blocks_path = format!("block_libs/{lib_name}/blocks");
                let textures_path = format!("block_libs/{lib_name}/textures");

                push_names_and_handles(
                    namespace,
                    &mut blocks,
                    &blocks_path,
                    load_context,
                    &mut interner,
                );
                push_names_and_handles(
                    namespace,
                    &mut textures,
                    &textures_path,
                    load_context,
                    &mut interner,
                );
            }

            Ok(IntermediateBlockLib {
                blocks,
                textures,
                texture_size,
                interner,
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
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>>
    {
        async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            Ok(json_de(&buffer)?)
        }
    }
}

#[inline]
fn push_names_and_handles<A: Asset>(
    namespace: DefaultSymbol,
    vec: &mut Vec<(Identifier, Handle<A>)>,
    path: &str,
    load_context: &mut LoadContext,
    interner: &mut Interner,
) {
    for result in WalkDir::new(path) {
        let dir = match result {
            Ok(dir) => dir,
            Err(e) => {
                warn!("Error {e} walking {path}");
                continue;
            }
        };

        if !dir.file_type().is_file() {
            continue;
        }

        let path = dir.path();
        let Some(os_name) = path.file_stem() else {
            warn!("Nameless at {path:?} skipping");
            continue;
        };

        let Some(name) = os_name.to_str() else {
            warn!("Invalid utf-8 name at {path:?} skipping");
            continue;
        };

        let name = interner.get_or_intern(name);

        let identifier = Identifier { namespace, name };

        let handle = load_context.load(path);

        vec.push((identifier, handle));
    }
}