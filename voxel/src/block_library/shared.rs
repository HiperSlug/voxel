use super::*;
use arc_swap::ArcSwap;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Debug, Resource)]
struct BlockLibraryHandle(Handle<BlockLibrary>);

#[derive(Debug, Resource, Default)]
pub struct SharedBlockLibrary(pub ArcSwap<BlockLibrary>);

fn update_shared_block_library(
    mut events: EventReader<AssetEvent<BlockLibrary>>,
    shared: Res<SharedBlockLibrary>,
    handle: Res<BlockLibraryHandle>,
    assets: Res<Assets<BlockLibrary>>,
) {
    for (event, _) in events.par_read() {
        match event {
            AssetEvent::Modified { id } | AssetEvent::Added { id } if id == &handle.0.id() => {
                if let Some(block_lib) = assets.get(*id) {
                    shared.0.store(Arc::new(block_lib.clone()));
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Resource)]
struct BlockLibraryPath(&'static str);

fn load_block_library_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    path: Res<BlockLibraryPath>,
) {
    info!("Loading BlockLibrary from path: {}", path.0);
    let handle = asset_server.load(path.0);
    commands.insert_resource(BlockLibraryHandle(handle));
    commands.remove_resource::<BlockLibraryPath>();
}

pub struct SharedBlockLibraryPlugin(pub &'static str);

impl Plugin for SharedBlockLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlockLibraryPlugin)
            .insert_resource(BlockLibraryPath(self.0))
            .init_resource::<SharedBlockLibrary>()
            .add_systems(Startup, load_block_library_handle)
            .add_systems(Update, update_shared_block_library);
    }
}
