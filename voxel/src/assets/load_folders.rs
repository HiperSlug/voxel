use bevy::{
    asset::{LoadState, LoadedFolder},
    prelude::*,
};
use std::{
    ffi::OsString,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use walkdir::WalkDir;

pub trait WalkSettings {
    const ROOT: &str = "";
    const TARGET: &str;
    const MIN: usize;
    const MAX: usize;
}

#[derive(Debug, Resource)]
struct FoldersHandles<S>
where
    S: WalkSettings,
{
    folders: Vec<Handle<LoadedFolder>>,
    _marker: PhantomData<S>,
}

impl<S> From<Vec<Handle<LoadedFolder>>> for FoldersHandles<S>
where
    S: WalkSettings,
{
    fn from(folders: Vec<Handle<LoadedFolder>>) -> Self {
        Self {
            folders,
            _marker: PhantomData,
        }
    }
}

impl<S> Deref for FoldersHandles<S>
where
    S: WalkSettings,
{
    type Target = Vec<Handle<LoadedFolder>>;

    fn deref(&self) -> &Self::Target {
        &self.folders
    }
}

impl<S> DerefMut for FoldersHandles<S>
where
    S: WalkSettings,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.folders
    }
}

pub fn init_load_folders<S>(mut commands: Commands, asset_server: Res<AssetServer>)
where
    S: WalkSettings + 'static + Send + Sync,
{
    let entries = WalkDir::new(S::ROOT)
        .max_depth(S::MAX)
        .min_depth(S::MIN)
        .into_iter()
        .filter_map(|r| {
            r.inspect_err(|e| warn!("Error {} walking {}", e, S::ROOT))
                .ok()
        });

    let matching_paths = entries.filter_map(|e| {
        let path = e.path();
        path.file_name().and_then(|name| {
            if name == &OsString::from(S::TARGET) {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
    });

    let folders = matching_paths
        .map(|p| asset_server.load_folder(p))
        .collect::<Vec<_>>();

    commands.insert_resource::<FoldersHandles<S>>(folders.into());
}

#[derive(Debug, Event)]
pub struct Loaded<S> {
    pub folders: Vec<Handle<LoadedFolder>>,
    _marker: PhantomData<S>,
}

impl<S> From<Vec<Handle<LoadedFolder>>> for Loaded<S> {
    fn from(folders: Vec<Handle<LoadedFolder>>) -> Self {
        Self {
            folders,
            _marker: PhantomData,
        }
    }
}

impl<S> Deref for Loaded<S> {
    type Target = Vec<Handle<LoadedFolder>>;

    fn deref(&self) -> &Self::Target {
        &self.folders
    }
}

impl<S> DerefMut for Loaded<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.folders
    }
}

pub fn poll_folders<S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    opt_folders: Option<ResMut<FoldersHandles<S>>>,
    mut event_writer: EventWriter<Loaded<S>>,
) where
    S: WalkSettings + 'static + Send + Sync,
{
    let Some(mut folders_res) = opt_folders else {
        return;
    };

    let all_loaded = folders_res
        .iter()
        .all(|h| matches!(asset_server.get_load_state(h).unwrap(), LoadState::Loaded));

    if all_loaded {
        let folders = std::mem::take(&mut folders_res.folders);
        commands.remove_resource::<FoldersHandles<S>>();
        event_writer.write(folders.into());
    }
}
