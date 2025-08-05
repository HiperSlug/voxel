use bevy::{
    asset::{AssetLoader, LoadContext, LoadedFolder, io::Reader, prelude::*},
    math::bounding::Aabb3d,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use block_mesh::{MergeVoxelContext, VoxelContext, VoxelVisibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc};

use crate::{
    asset::{AssetLoadState, VecFolder, VecPath, collect_paths, load_folders, poll_folders},
    data::voxel::Voxel,
};

#[derive(Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct Block {
    pub display_name: String,
    pub collision_aabbs: Vec<Aabb3d>,
    pub is_translucent: bool,
    pub textures: Textures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Textures {
    pub pos_x: String,
    pub neg_x: String,
    pub pos_y: String,
    pub neg_y: String,
    pub pos_z: String,
    pub neg_z: String,
}

#[derive(Debug, Default)]
pub struct BlockLoader;

impl AssetLoader for BlockLoader {
    type Asset = Block;
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
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            let out = serde_json::de::from_slice::<Block>(&buffer)?;
            Ok(out)
        }
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct BlocksFolderPaths(Vec<PathBuf>);

impl VecPath for BlocksFolderPaths {
    fn from_paths(paths: &[PathBuf]) -> Self {
        Self(paths.to_vec())
    }

    fn paths(&self) -> &[PathBuf] {
        &self.0
    }

    fn target() -> &'static str {
        "blocks"
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct BlocksFolders(Vec<Handle<LoadedFolder>>);

impl VecFolder for BlocksFolders {
    fn folders(&self) -> &[Handle<LoadedFolder>] {
        &self.0
    }

    fn from_folders(folders: &[Handle<LoadedFolder>]) -> Self {
        Self(folders.to_vec())
    }
}

#[derive(Debug, Resource)]
pub struct BlockLibrary {
    pub blocks: Vec<Handle<Block>>,

    pub name_to_index: HashMap<String, usize>,
    pub index_to_name: Vec<String>,
}

impl BlockLibrary {
    pub fn arc_vec(&self, assets: Res<Assets<Block>>) -> ThreadSafeBlockVec {
        self.blocks
            .iter()
            .map(|b| assets.get_arc(b).unwrap())
            .collect()
    }
}

pub type ThreadSafeBlockVec = Vec<Arc<Block>>;

impl VoxelContext<Voxel> for ThreadSafeBlockVec {
    fn get_visibility(&self, voxel: &Voxel) -> VoxelVisibility {
        if voxel.is_sentinel() {
            VoxelVisibility::Empty
        } else if let Some(block) = self.get(voxel.index()) {
            if block.is_translucent {
                VoxelVisibility::Translucent
            } else {
                VoxelVisibility::Opaque
            }
        } else {
            error!("Could not find voxel {:?} in blocks {:?}", voxel, self,);
            VoxelVisibility::Empty
        }
    }
}

impl MergeVoxelContext<Voxel> for ThreadSafeBlockVec {
    type MergeValue = u16;
    type MergeValueFacingNeighbour = u16;

    fn merge_value(&self, voxel: &Voxel) -> Self::MergeValue {
        voxel.id
    }

    fn merge_value_facing_neighbour(&self, voxel: &Voxel) -> Self::MergeValueFacingNeighbour {
        voxel.id
    }
}

pub fn build_block_library(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<LoadedFolder>>,
    folders: Res<BlocksFolders>,
    mut state: ResMut<NextState<BlockLibraryState>>,
) {
    let mut name_to_index = HashMap::new();
    let mut index_to_name = Vec::new();
    let mut blocks = Vec::new();

    for untyped in folders
        .iter()
        .map(|f| assets.get(f.id()).unwrap())
        .flat_map(|f| f.handles.iter())
        .cloned()
    {
        let Some(name) = asset_server.get_path(untyped.id()).and_then(|p| {
            p.path()
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        }) else {
            warn!("Unable to resolve block `name` for handle {untyped:?}. Skipping...");
            continue;
        };

        let block = match untyped.try_typed::<Block>() {
            Ok(b) => b,
            Err(e) => {
                warn!("Handle {name} cannot be converted to `Block` type: {e}. Skipping...");
                continue;
            }
        };

        index_to_name.push(name.clone());
        name_to_index.insert(name, blocks.len());

        blocks.push(block);
    }

    commands.remove_resource::<BlocksFolders>();
    commands.insert_resource(BlockLibrary {
        blocks,
        name_to_index,
        index_to_name,
    });

    state.set(BlockLibraryState::Loaded);
}

#[derive(Debug, States, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum BlockLibraryState {
    #[default]
    Loading,
    Building,
    Loaded,
}

impl AssetLoadState for BlockLibraryState {
    fn build_state() -> Self {
        Self::Building
    }
}

pub struct BlockLibraryPlugin;

impl Plugin for BlockLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BlockLibraryState>()
            .init_asset::<Block>()
            .init_asset_loader::<BlockLoader>()
            .add_systems(
                OnEnter(BlockLibraryState::Loading),
                (
                    collect_paths::<BlocksFolderPaths>,
                    load_folders::<BlocksFolderPaths, BlocksFolders>,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (poll_folders::<BlocksFolders, BlockLibraryState>)
                    .run_if(in_state(BlockLibraryState::Loading)),
            )
            .add_systems(OnEnter(BlockLibraryState::Building), build_block_library);
    }
}
