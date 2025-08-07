use bevy::asset::{LoadedFolder, RenderAssetUsages};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

const SHADER_PATH: &str = "shaders/chunk.wgsl";

#[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
struct TextureArrayMaterialExt {
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

#[derive(Debug, Resource)]
pub struct SharedTextureArrayMaterial(pub Handle<TextureArrayMaterial>);

pub type TextureMap = HashMap<String, u16>;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SharedTextureMap(pub Arc<TextureMap>);

#[derive(Debug, Resource, Deref, DerefMut)]
struct TexturesFolderPaths(Vec<PathBuf>);

impl VecPath for TexturesFolderPaths {
    fn from_paths(paths: &[PathBuf]) -> Self {
        Self(paths.to_vec())
    }

    fn paths(&self) -> &[PathBuf] {
        &self.0
    }

    fn target() -> &'static str {
        "textures"
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct TextureFolders(Vec<Handle<LoadedFolder>>);

impl VecFolder for TextureFolders {
    fn folders(&self) -> &[Handle<LoadedFolder>] {
        &self.0
    }

    fn from_folders(folders: &[Handle<LoadedFolder>]) -> Self {
        Self(folders.to_vec())
    }
}

#[derive(Debug, Default, States, Hash, PartialEq, Eq, Clone, Copy)]
enum TextureArrayState {
    #[default]
    Loading,
    Building,
    Loaded,
}

impl AssetLoadState for TextureArrayState {
    fn build_state() -> Self {
        Self::Building
    }
}

fn build_texture_array(
    mut commands: Commands,
    folders: Res<TextureFolders>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TextureArrayMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_iter = folders
        .iter()
        .map(|h| loaded_folders.get(h).unwrap())
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
        let image = images.get(handle.id()).unwrap();
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

    let textures = images.add(texture_array);
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: TextureArrayMaterialExt { textures },
    });

    commands.insert_resource(SharedTextureArrayMaterial(handle));
    commands.insert_resource(SharedTextureMap(Arc::new(map)));
}

pub struct TextureArrayPlugin;

impl Plugin for TextureArrayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TextureArrayState>()
            .add_systems(
                OnEnter(TextureArrayState::Loading),
                (
                    collect_paths::<TexturesFolderPaths>,
                    load_folders::<TexturesFolderPaths, TextureFolders>,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (poll_folders::<TextureFolders, TextureArrayState>)
                    .run_if(in_state(TextureArrayState::Loading)),
            )
            .add_systems(OnEnter(TextureArrayState::Building), build_texture_array);
    }
}
