use bevy::{
    asset::{LoadState, LoadedFolder},
    prelude::*,
};
use std::{ffi::OsString, marker::PhantomData};
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
    handles: Vec<Handle<LoadedFolder>>,
    _marker: PhantomData<S>,
}

impl<S> FoldersHandles<S>
where
    S: WalkSettings,
{
    fn new(handles: Vec<Handle<LoadedFolder>>) -> Self {
        Self {
            handles,
            _marker: PhantomData,
        }
    }
}

pub fn init_load_folders<S>(mut commands: Commands, asset_server: Res<AssetServer>)
where
    S: WalkSettings,
{
    let paths = WalkDir::new(S::ROOT)
        .max_depth(S::MAX)
        .min_depth(S::MIN)
        .into_iter()
        .filter_map(|r| {
            r.inspect_err(|e| warn!("Error {} walking {}", e, S::ROOT))
                .ok()
        })
        .map(|d| d.path());

    let matching_paths = paths.filter_map(|p| {
        p.file_name().and_then(|name| {
            if name == &OsString::from(S::TARGET) {
                Some(p)
            } else {
                None
            }
        })
    });

    let folders = matching_paths
        .map(|p| asset_server.load_folder(p))
        .collect::<Vec<_>>();

    commands.insert_resource(FoldersHandles::<S>::new(folders));
}

pub fn poll_folders<E, S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    opt_folders: Option<Res<FoldersHandles<S>>>,
    mut event_writer: EventWriter<E>,
) where
    E: Event + From<Vec<Handle<LoadedFolder>>>,
    S: 'static + Send + Sync + WalkSettings,
{
    let Some(handles) = opt_folders.map(|f| &f.handles) else {
        return;
    };

    let all_loaded = handles
        .iter()
        .all(|h| matches!(asset_server.get_load_state(h).unwrap(), LoadState::Loaded));

    if all_loaded {
        commands.remove_resource::<FoldersHandles<S>>();
        event_writer.write(E::from(handles));
    }
}
