use bevy::{
    asset::{LoadState, LoadedFolder},
    prelude::*,
    state::state::FreelyMutableState,
};
use std::{ffi::OsString, fmt::Debug, path::PathBuf};
use walkdir::WalkDir;

pub trait VecPath {
    fn target() -> &'static str;

    fn from_paths(paths: &[PathBuf]) -> Self;

    fn paths(&self) -> &[PathBuf];
}

pub fn collect_paths<P>(mut commands: Commands)
where
    P: VecPath + Resource,
{
    let mut paths = Vec::new();

    for entry in WalkDir::new("assets/block_libs")
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .filter_map(|r| {
            r.inspect_err(|e| warn!("Error walking block_libs {e}"))
                .ok()
        })
    {
        let path = entry.path();
        if let Some(name) = path.file_name() {
            if name == &OsString::from(P::target()) {
                paths.push(path.to_path_buf())
            }
        }
    }

    commands.insert_resource(P::from_paths(&paths));
}

pub trait VecFolder {
    fn from_folders(folders: &[Handle<LoadedFolder>]) -> Self;

    fn folders(&self) -> &[Handle<LoadedFolder>];
}

pub fn load_folders<P, F>(mut commands: Commands, asset_server: Res<AssetServer>, paths: Res<P>)
where
    P: VecPath + Resource,
    F: VecFolder + Resource,
{
    let folders = paths
        .paths()
        .iter()
        .cloned()
        .map(|p| asset_server.load_folder(p))
        .collect::<Vec<_>>();

    commands.insert_resource(F::from_folders(&folders));
    commands.remove_resource::<P>();
}

pub trait AssetLoadState {
    fn build_state() -> Self;
}

pub fn poll_folders<F, S>(
    asset_server: Res<AssetServer>,
    folders: Res<F>,
    mut state: ResMut<NextState<S>>,
) where
    F: VecFolder + Resource,
    S: FreelyMutableState + AssetLoadState + Debug,
{
    let all_loaded = folders
        .folders()
        .iter()
        .all(|h| matches!(asset_server.get_load_state(h).unwrap(), LoadState::Loaded));

    if all_loaded {
        let next = S::build_state();
        info!("Loaded to {next:?}");
        state.set(next);
    }
}
