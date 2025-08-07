use bevy::{
    asset::{LoadedFolder, prelude::*},
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};

use crate::assets::load_folders::Loaded;

use super::{
    block::{Block, BlockLoader},
    load_folders::{WalkSettings, init_load_folders, poll_folders},
};

struct BlocksWalkSettings;

impl WalkSettings for BlocksWalkSettings {
    const MAX: usize = 2;
    const MIN: usize = 2;
    const ROOT: &str = "block_lib";
    const TARGET: &str = "blocks";
}

#[derive(Debug, States, Hash, PartialEq, Eq, Clone, Copy)]
pub enum BlockLibraryState {
    Loading,
    Loaded,
}

#[derive(Debug)]
pub struct BlockLibrary {
    pub blocks: Vec<Arc<Block>>,
    pub name_to_index: HashMap<String, usize>,
    pub index_to_name: Vec<String>,
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SharedBlockLibrary(pub Arc<BlockLibrary>);

pub fn build_block_library(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets_f: Res<Assets<LoadedFolder>>,
    assets_b: Res<Assets<Block>>,
    mut events: EventReader<Loaded<BlocksWalkSettings>>,
    mut state: ResMut<NextState<BlockLibraryState>>,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    let mut name_to_index = HashMap::new();
    let mut index_to_name = Vec::new();
    let mut blocks = Vec::new();

    for untyped in event
        .iter()
        .map(|f| assets_f.get(f.id()).unwrap())
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

        let handle = match untyped.try_typed::<Block>() {
            Ok(b) => b,
            Err(e) => {
                warn!("Handle {name} cannot be converted to `Block` type: {e}. Skipping...");
                continue;
            }
        };

        let block = assets_b.get_arc(handle.id()).unwrap();

        index_to_name.push(name.clone());
        name_to_index.insert(name, blocks.len());

        blocks.push(block);
    }

    commands.insert_resource(SharedBlockLibrary(Arc::new(BlockLibrary {
        blocks,
        name_to_index,
        index_to_name,
    })));

    state.set(BlockLibraryState::Loaded);
}

pub struct BlockLibraryPlugin;

impl Plugin for BlockLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(BlockLibraryState::Loading)
            .init_asset::<Block>()
            .init_asset_loader::<BlockLoader>()
            .add_systems(
                OnEnter(BlockLibraryState::Loading),
                init_load_folders::<BlocksWalkSettings>,
            )
            .add_systems(
                Update,
                (poll_folders::<BlocksWalkSettings>, build_block_library)
                    .chain()
                    .run_if(in_state(BlockLibraryState::Loading)),
            );
    }
}
