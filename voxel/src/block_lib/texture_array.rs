use bevy::{
    asset::RenderAssetUsages,
    ecs::intern::Interned,
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn build(
    textures: &[(Interned<str>, Handle<Image>)],
    size: UVec2,
    mut image_assets: ResMut<Assets<Image>>,
) -> (HashMap<Interned<str>, u32>, Handle<Image>) {
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

        data.extend(image.data.unwrap().drain(..));

        name_to_index.insert(*name, index as u32);
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
