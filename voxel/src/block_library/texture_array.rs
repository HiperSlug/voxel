use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::collections::HashMap;

// #[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
// pub struct TextureArrayMaterialExt {
//     #[texture(100, dimension = "2d_array")]
//     #[sampler(101)]
//     textures: Handle<Image>,
// }

// impl MaterialExtension for TextureArrayMaterialExt {
//     fn vertex_shader() -> ShaderRef {
//         "shaders/chunk.wgsl".into()
//     }

//     fn fragment_shader() -> ShaderRef {
//         "shaders/chunk.wgsl".into()
//     }

//     fn specialize(
//         _pipeline: &MaterialExtensionPipeline,
//         _descriptor: &mut RenderPipelineDescriptor,
//         _layout: &MeshVertexBufferLayoutRef,
//         _key: MaterialExtensionKey<Self>,
//     ) -> Result<(), SpecializedMeshPipelineError> {
//         todo!()
//     }
// }

// pub type TextureArrayMaterial = ExtendedMaterial<StandardMaterial, TextureArrayMaterialExt>;

pub fn build(
    textures: &[(String, Handle<Image>)],
    size: UVec2,
    mut image_assets: ResMut<Assets<Image>>,
) -> (HashMap<String, u32>, Handle<Image>) {
    let mut name_to_index = HashMap::new();
    let mut data = Vec::new();

    for (index, (name, handle)) in textures.into_iter().enumerate() {
        let image = image_assets.get(handle).unwrap();

        let mut image = image.convert(TextureFormat::Rgba8UnormSrgb).unwrap();

        // TODO: switch to resize_in_place in 0.17
        image.resize(Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        });

        let image_data = &image.data.unwrap();
        data.extend(image_data);

        name_to_index.insert(name.to_string(), index as u32);
    }

    let texture_array = Image::new(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: name_to_index.len() as u32,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let texture_array = image_assets.add(texture_array);

    (name_to_index, texture_array)
}
