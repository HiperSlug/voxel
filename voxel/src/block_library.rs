use anyhow::Context;
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use raw::{BlockLibraryRaw, SpriteSheetRaw};
pub use raw::BlockVariant;

mod raw {
	use std::collections::HashMap;

	use serde::{Deserialize, Serialize};
	use bevy::math::UVec2;

	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockLibraryRaw {
		pub atlas: SpriteSheetRaw,
		pub variants: HashMap<String, BlockVariant>,
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
	pub variants: HashMap<String, BlockVariant>,
}

#[derive(Debug)]
pub struct SpriteSheet {
    pub image: Handle<Image>,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Default)]
pub enum BlockLibraryLoaderSettings {
	#[default]
	Json,
	Ron,
}

#[derive(Default)]
pub struct BlockLibraryLoader;

impl AssetLoader for BlockLibraryLoader {
    type Asset = BlockLibrary;
    type Settings = BlockLibraryLoaderSettings;
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["blocklib.json", "blocklib.ron"]
    }

    fn load(
        &self,
        rdr: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
			let mut bytes = Vec::new();
            rdr.read_to_end(&mut bytes)
                .await
                .context("Failed to read asset bytes for BlockLibrary")?;
			
			use BlockLibraryLoaderSettings::*;
			let raw = match settings {
				Json => {
					serde_json::de::from_slice::<BlockLibraryRaw>(&bytes)
                		.context("Failed to deserialize BlockLibraryRaw from JSON")?
				},
				Ron => {
					ron::de::from_bytes::<BlockLibraryRaw>(&bytes)
                		.context("Failed to deserialize BlockLibraryRaw from RON")?
				},
			};

            let BlockLibraryRaw { atlas: SpriteSheetRaw { path, width, height }, variants } = raw;

			let image = load_context.load(path);
			
			let lib = BlockLibrary {
				atlas: SpriteSheet {
					image,
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
	// TODO make the path configurable
	let handle = asset_server.load(".blocklib.json");
	commands.insert_resource(BlockLibraryHandle(handle));
}

pub struct BlockLibraryPlugin;

impl Plugin for BlockLibraryPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<BlockLibrary>()
            .init_asset_loader::<BlockLibraryLoader>()
            .add_systems(Startup, load_block_library);
	}
}
