use bevy::{
    asset::RenderAssetUsages,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
};
use std::{collections::HashMap, sync::LazyLock};

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
}

const PLACEHOLDER_DATA: &[u8] = [
    [128, 0, 128, 255],
    [0, 0, 0, 255],
    [128, 0, 128, 255],
    [0, 0, 0, 255],
]
.as_flattened();

static PLACEHOLDER_TEXTURE: LazyLock<Image> = LazyLock::new(|| {
    Image::new(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        PLACEHOLDER_DATA.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
});

pub type TextureArrayMaterial = ExtendedMaterial<StandardMaterial, TextureArrayMaterialExt>;

pub fn build_texture_array(
    textures_map: HashMap<String, Handle<Image>>,
    size: UVec2,
    mut image_assets: ResMut<Assets<Image>>,
    mut material_assets: ResMut<Assets<TextureArrayMaterial>>,
) -> (HashMap<String, usize>, Handle<TextureArrayMaterial>) {
    let mut index_map = HashMap::new();
    let mut data = Vec::new();

    for (index, (name, handle)) in textures_map.into_iter().enumerate() {
        let image = image_assets.get(&handle).unwrap();

        let mut image = image.convert(TextureFormat::Rgba8UnormSrgb).unwrap();
        let _ = image.resize/* TODO: in_place */(Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        });

        if let Some(image_data) = image.data.as_ref() {
            data.extend(image_data);
        } else {
            error!("Image has no data");
            let req_bytes = 4 * size.element_product() as usize;
            data.resize(data.len() + req_bytes, 0);
        }

        index_map.insert(name, index);
    }

    let texture_array = Image::new(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: index_map.len() as u32,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let textures = image_assets.add(texture_array);
    let handle = material_assets.add(ExtendedMaterial {
        base: StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
        extension: TextureArrayMaterialExt { textures },
    });

    (index_map, handle)
}
