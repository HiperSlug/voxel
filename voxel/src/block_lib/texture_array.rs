use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::Identifier;

pub fn build(
    textures: &[(Identifier, Handle<Image>)],
    texture_size: UVec2,
    mut image_assets: ResMut<Assets<Image>>,
) -> (HashMap<Identifier, u32>, Handle<Image>) {
    let mut identifier_to_index = HashMap::new();
    let mut data = Vec::new();

    for (index, (identifier, handle)) in textures.into_iter().enumerate() {
        let image = image_assets.get(handle).unwrap();

        if image.texture_descriptor.format != TextureFormat::Rgba8UnormSrgb
            || image.size() != texture_size 
        {
            // internal clone
            let mut image = image.convert(TextureFormat::Rgba8UnormSrgb).unwrap();

            image.resize_in_place(Extent3d {
                width: texture_size.x,
                height: texture_size.y,
                depth_or_array_layers: 1,
            });

            data.extend(image.data.unwrap().drain(..));
        } else {
            data.extend(image.data.as_ref().unwrap())
        };

        

        identifier_to_index.insert(*identifier, index as u32);
    }

    let texture_array = Image::new(
        Extent3d {
            width: texture_size.x,
            height: texture_size.y,
            depth_or_array_layers: identifier_to_index.len() as u32,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let handle = image_assets.add(texture_array);

    (identifier_to_index, handle)
}
