/// Deserializable structs. The only difference is that they store sub-assets as paths instead of handles
mod raw;
/// Workaround until bevy allows resources to be threaded.
pub mod shared;

// the raw::BlockModelCube has no sub-assets and so can be used as is
pub use raw::BlockModelCube;

use anyhow::Context;
use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    math::bounding::Aabb3d,
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use bevy_materialize::{MaterializePlugin, prelude::JsonMaterialDeserializer};
use block_mesh::{VoxelContext, MergeVoxelContext, VoxelVisibility};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::data::voxel::Voxel;

#[derive(Debug, Asset, TypePath, Clone, Default)]
pub struct BlockLibrary {
    pub materials: Vec<Material>,
    pub variants: Vec<BlockVariant>,
    pub name_to_index: HashMap<String, usize>,
    pub index_to_name: Vec<String>,
}

impl VoxelContext<Voxel> for BlockLibrary {
    fn get_visibility(&self, voxel: &Voxel) -> VoxelVisibility {
        let variant = self.variants.get(voxel.0 as usize).unwrap();
        match &variant.block_model {
            BlockModel::Empty => {
                VoxelVisibility::Empty
            }
            BlockModel::Cube(c) => {
                if c.is_transparent {
                    VoxelVisibility::Translucent
                } else {
                    VoxelVisibility::Opaque
                }
            }
        }
    }
}

impl MergeVoxelContext<Voxel> for BlockLibrary {
    type MergeValue = u16;
    type MergeValueFacingNeighbour = u16;

    fn merge_value(&self, voxel: &Voxel) -> Self::MergeValue {
        voxel.0
    }

    fn merge_value_facing_neighbour(&self, voxel: &Voxel) -> Self::MergeValueFacingNeighbour {
        voxel.0
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub handle: Handle<StandardMaterial>,
    pub size: UVec2,
}

#[derive(Debug, Clone)]
pub struct BlockVariant {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub block_model: BlockModel,
}

#[derive(Debug, Clone)]
pub enum BlockModel {
    Empty,
    Cube(BlockModelCube),
    Mesh(BlockModelMesh),
}

#[derive(Debug, Clone)]
pub struct BlockModelMesh {
    pub handle: Handle<Mesh>,
    pub material_index: usize,
    pub is_transparent: bool,
}

#[derive(Default)]
pub struct BlockLibraryLoader;

impl AssetLoader for BlockLibraryLoader {
    type Asset = BlockLibrary;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["json", "bllib", "bllib.json", "blocklib", "blocklib.json"]
    }

    fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .context("Failed to read asset bytes for BlockLibrary")?;

            let raw::BlockLibrary {
                materials: raw_materials,
                blocks: raw_blocks,
            } = serde_json::de::from_slice::<raw::BlockLibrary>(&bytes)
                .context("Failed to deserialize raw::BlockLibrary from JSON")?;

            let materials = raw_materials
                .into_iter()
                .map(|m| {
                    let raw::Material { path, size } = m;

                    let handle = load_context.load(resolve_path(load_context, &path));

                    Material { handle, size }
                })
                .collect::<Vec<_>>();

            let capacity = raw_blocks.len();

            let mut name_to_index = HashMap::new();
            let mut index_to_name = Vec::with_capacity(capacity);
            let mut variants = Vec::with_capacity(capacity);

            for (i, (name, variant)) in raw_blocks.into_iter().enumerate() {
                let variant = {
                    let raw::BlockVariant {
                        display_name,
                        collision_aabbs,
                        block_model,
                    } = variant;
                    let block_model = match block_model {
                        raw::BlockModel::Empty => BlockModel::Empty,
                        raw::BlockModel::Cube(c) => BlockModel::Cube(c),
                        raw::BlockModel::Mesh(m) => {
                            let raw::BlockModelMesh {
                                material_index,
                                path,
                                is_transparent,
                            } = m;

                            let handle = load_context.load(resolve_path(load_context, &path));
                            let m = BlockModelMesh {
                                handle,
                                material_index,
                                is_transparent,
                            };
                            BlockModel::Mesh(m)
                        }
                    };
                    BlockVariant {
                        display_name,
                        collision_aabbs,
                        block_model,
                    }
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

fn resolve_path(load_context: &LoadContext, path: &str) -> PathBuf {
    if path.starts_with("/") {
        PathBuf::from(path.trim_start_matches('/'))
    } else {
        let mut path_buf = PathBuf::new();
        path_buf.push(load_context.path().parent().unwrap_or(Path::new("")));
        path_buf.push(path);
        path_buf
    }
}

pub struct BlockLibraryPlugin;

impl Plugin for BlockLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterializePlugin::new(JsonMaterialDeserializer))
            .init_asset::<BlockLibrary>()
            .init_asset_loader::<BlockLibraryLoader>();
    }
}
