use bevy::asset::{LoadedFolder, RenderAssetUsages};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
};
use std::{collections::HashMap, sync::Arc};

use super::load_folders::{Loaded, WalkSettings, init_load_folders, poll_folders};

const SHADER_PATH: &str = "shaders/chunk.wgsl";

#[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
pub struct TextureArrayMaterialExt {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    textures: Handle<Image>,
}

impl MaterialExtension for TextureArrayMaterialExt {
    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    // fn specialize(/* .. */) -> /* .. */ {/* .. */}
}

pub type TextureArrayMaterial = ExtendedMaterial<StandardMaterial, TextureArrayMaterialExt>;

#[derive(Debug, Resource, Deref)]
pub struct SharedTextureArrayMaterial(pub Handle<TextureArrayMaterial>);

pub type TextureMap = HashMap<String, u16>;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SharedTextureMap(pub Arc<TextureMap>);

#[derive(Debug, States, Hash, PartialEq, Eq, Clone, Copy)]
enum TextureArrayState {
    Loading,
    Loaded,
}

struct TextureWalkSettings;

impl WalkSettings for TextureWalkSettings {
    const MAX: usize = 2;
    const MIN: usize = 2;
    const ROOT: &str = "block_libs";
    const TARGET: &str = "textures";
}

fn build_texture_array(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<Loaded<TextureWalkSettings>>,
    assets_f: Res<Assets<LoadedFolder>>,
    mut assets_i: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TextureArrayMaterial>>,
    mut state: ResMut<NextState<TextureArrayState>>,
) {
    let Some(folders) = events.read().next() else {
        return;
    };

    let texture_iter = folders
        .iter()
        .map(|h| assets_f.get(h).unwrap())
        .flat_map(|f| {
            f.handles.iter().cloned().filter_map(|i| {
                i.try_typed::<Image>()
                    .inspect_err(|e| warn!("Error loading texture {e}"))
                    .ok()
            })
        });

    let mut textures = Vec::new();

    let mut map = HashMap::new();

    for (idx, handle) in texture_iter.enumerate() {
        let image = assets_i.get(handle.id()).unwrap();
        textures.push(image);

        let name = asset_server
            .get_path(handle.id())
            .unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        map.insert(name, idx as u16);
    }

    let length = textures.len() as u32;
    let size = textures.get(0).unwrap_or(&&Image::default()).size();

    let data = textures
        .into_iter()
        .flat_map(|i| {
            assert_eq!(i.size(), size);
            i.data.clone().unwrap()
        })
        .collect::<Vec<_>>();

    let texture_array = Image::new(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: length,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );

    let textures = assets_i.add(texture_array);
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: TextureArrayMaterialExt { textures },
    });

    state.set(TextureArrayState::Loaded);

    commands.insert_resource(SharedTextureArrayMaterial(handle));
    commands.insert_resource(SharedTextureMap(Arc::new(map)));
}

pub struct TextureArrayPlugin;

impl Plugin for TextureArrayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(TextureArrayState::Loading)
            .add_systems(
                OnEnter(TextureArrayState::Loading),
                init_load_folders::<TextureWalkSettings>,
            )
            .add_systems(
                Update,
                (poll_folders::<TextureWalkSettings>, build_texture_array)
                    .chain()
                    .run_if(in_state(TextureArrayState::Loading)),
            );
    }
}
