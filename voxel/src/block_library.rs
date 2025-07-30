use anyhow::Context;
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, LoadContext}, math::bounding::Aabb3d, prelude::*, reflect::TypePath, tasks::ConditionalSendFuture
};
use bevy_materialize::{prelude::TomlMaterialDeserializer, MaterializePlugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod raw {
    use std::collections::HashMap;
    use bevy::math::{bounding::Aabb3d, UVec2};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BlockLibrary {
        pub materials: Vec<Material>,
        pub blocks: HashMap<String, BlockVariant>,
    }

	#[derive(Debug, Serialize, Deserialize)]
	pub struct Material {
		pub path: String,
		pub size: UVec2,
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockVariant {
		pub display_name: String,
		pub collision_aabbs: Vec<Aabb3d>,
		pub block_model: BlockModel,
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub enum BlockModel {
		Empty,
		Cube(BlockModelCube),
		Mesh(BlockModelMesh),
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockModelCube {
		pub material_index: usize,
		pub texture_coords: TextureCoords,
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct TextureCoords {
		pub pos_x: UVec2,
		pub neg_x: UVec2,
		pub pos_y: UVec2,
		pub neg_y: UVec2,
		pub pos_z: UVec2,
		pub neg_z: UVec2,
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct BlockModelMesh {
		pub path: String,
		pub material_index: usize,
	}	
}

#[derive(Debug, Asset, TypePath)]
pub struct BlockLibrary {
    pub materials: Vec<Material>,
    pub variants: Vec<BlockVariant>,
    pub name_to_index: HashMap<String, usize>,
    pub index_to_name: Vec<String>,
}

#[derive(Debug)]
pub struct Material {
	pub handle: Handle<StandardMaterial>,
	pub size: UVec2,
}

#[derive(Debug)]
pub struct BlockVariant {
	pub display_name: String,
	pub collision_aabbs: Vec<Aabb3d>,
	pub block_model: BlockModel,
}

#[derive(Debug)]
pub enum BlockModel {
	Empty,
	Cube(BlockModelCube),
	Mesh(BlockModelMesh),
}

pub use raw::BlockModelCube;

#[derive(Debug)]
pub struct BlockModelMesh {
	pub handle: Handle<Mesh>,
	pub material_index: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum BlockLibraryLoaderSettings {
    #[default]
    Ron,
    Json,
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
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes)
                .await
                .context("Failed to read asset bytes for BlockLibrary")?;

            use BlockLibraryLoaderSettings::*;
            let raw = match settings {
                Json => serde_json::de::from_slice::<raw::BlockLibrary>(&bytes)
                    .context("Failed to deserialize BlockLibraryRaw from JSON")?,
                Ron => ron::de::from_bytes::<raw::BlockLibrary>(&bytes)
                    .context("Failed to deserialize BlockLibraryRaw from RON")?,
            };

            let raw::BlockLibrary {
                materials: raw_materials,
                blocks: raw_blocks,
            } = raw;

            let materials = raw_materials.into_iter().map(|m| {
				let raw::Material {
					path,
					size,
				} = m;

				let handle = load_context.load(path);
				
				Material {
					handle,
					size,
				}
			}).collect::<Vec<_>>();

            let capacity = raw_blocks.len();

            let mut name_to_index = HashMap::new();
            let mut index_to_name = Vec::with_capacity(capacity);
            let mut variants = Vec::with_capacity(capacity);

            for (i, (name, variant)) in raw_blocks.into_iter().enumerate() {
				let variant = {
					let raw::BlockVariant { display_name, collision_aabbs, block_model} = variant;
					let block_model = match block_model {
						raw::BlockModel::Empty => BlockModel::Empty,
						raw::BlockModel::Cube(c) => BlockModel::Cube(c),
						raw::BlockModel::Mesh(m) => {
							let raw::BlockModelMesh { material_index, path} = m;

							let handle = load_context.load(path);
							let m = BlockModelMesh {
								handle,
								material_index,
							};
							BlockModel::Mesh(m)
						},
					};
					BlockVariant { display_name, collision_aabbs, block_model, }
				};

                name_to_index.insert(name.clone(), i);
                index_to_name.push(name);
                variants.push(variant);
            }

            let lib = BlockLibrary {
                materials,
                variants,
                name_to_index,
                index_to_name,
            };

            Ok(lib)
        }
    }
}

#[derive(Debug, Resource)]
pub struct BlockLibraryPath(String);

#[derive(Debug, Resource)]
pub struct BlockLibraryHandle(Handle<BlockLibrary>);

pub fn load_block_library(
	mut commands: Commands, 
	asset_server: Res<AssetServer>,
	path: Res<BlockLibraryPath>,
) {
	let handle = asset_server.load(path.0.clone());
	commands.insert_resource(BlockLibraryHandle(handle));
	commands.remove_resource::<BlockLibraryPath>();
}

pub struct BlockLibraryPlugin {
	path: String,
}

impl BlockLibraryPlugin {
	pub fn new(path: impl Into<String>) -> Self {
		Self { path: path.into() }
	}
}

impl Plugin for BlockLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterializePlugin::new(TomlMaterialDeserializer))
			.init_asset::<BlockLibrary>()
            .init_asset_loader::<BlockLibraryLoader>()
			.insert_resource(BlockLibraryPath(self.path.clone()))
            .add_systems(Startup, load_block_library);
    }
}
