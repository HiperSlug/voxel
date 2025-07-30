use anyhow::Context;
use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};

use raw::{BlockLibraryRaw, SpriteSheetRaw};
pub use raw::BlockVariant;

mod raw {
	use serde::{Deserialize, Serialize};
	use bevy::math::UVec2;

	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockLibraryRaw {
		pub atlas: SpriteSheetRaw,
		pub variants: Vec<BlockVariant>,
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct SpriteSheetRaw {
		pub path: String,
		pub width: usize,
		pub height: usize,
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockVariant {
		pub name: String,
		pub is_transparent: bool,
		pub textures: [UVec2; 6],
	}
}

#[derive(Debug, Asset, TypePath)]
pub struct BlockLibrary {
	pub atlas: SpriteSheet,
	pub variants: Vec<BlockVariant>,
}

#[derive(Debug)]
pub struct SpriteSheet {
    pub handle: Handle<Image>,
    pub width: usize,
    pub height: usize,
}

#[derive(Default)]
pub struct BlockLibraryLoader;

impl AssetLoader for BlockLibraryLoader {
    type Asset = BlockLibrary;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["blocklib.ron"]
    }

    fn load(
        &self,
        rdr: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut bytes = Vec::new();
            rdr.read_to_end(&mut bytes)
                .await
                .context("Failed to read asset bytes for BlockLibrary")?;

            let BlockLibraryRaw { atlas: SpriteSheetRaw { path, width, height }, variants } = ron::de::from_bytes::<BlockLibraryRaw>(&bytes)
                .context("Failed to deserialize BlockLibraryRaw from RON")?;

			let handle = load_context.load(path);
			
			let lib = BlockLibrary {
				atlas: SpriteSheet {
					handle,
					width,
					height,
				},
				variants,
			};

            Ok(lib)
        }
    }
}

#[derive(Debug, Resource)]
pub struct BlockLibraryHandle(Handle<BlockLibrary>);

pub fn load_block_library(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let handle = asset_server.load(".blocklib.ron");
	commands.insert_resource(BlockLibraryHandle(handle));
}
