use bevy::asset::{LoadedFolder, RenderAssetUsages};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
};

const SHADER_PATH: &str = "shaders/texture_array.wgsl";

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

#[derive(Debug, Resource, Deref)]
pub struct SharedTextureArrayMaterial(pub Handle<TextureArrayMaterial>);

#[derive(Debug, Resource)]
pub struct TextureFolders(pub Vec<Handle<LoadedFolder>>);

fn build_texture_array(
    mut commands: Commands,
    folders: Res<TextureFolders>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TextureArrayMaterial>>,
) {
    let textures = folders
        .0
        .iter()
        .map(|h| loaded_folders.get(h).unwrap())
        .flat_map(|f| {
            f.handles.iter().cloned().filter_map(|i| {
                i.try_typed::<Image>()
                    .inspect_err(|e| warn!("Error loading texture {e}"))
                    .ok()
            })
        })
        .map(|h| images.get(h.id()).unwrap())
        .collect::<Vec<_>>();

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

    let handle = images.add(texture_array);
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: TextureArrayMaterialExt { textures: handle },
    });

    commands.insert_resource(SharedTextureArrayMaterial(handle));
}
