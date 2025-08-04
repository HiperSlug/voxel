// use bevy::prelude::*;

// // Clone and default are req for `shared` module
// #[derive(Debug, Asset, TypePath, Clone, Default)]
// pub struct BlockLibrary {
//     pub material: Standard
//     pub variants: Vec<BlockVariant>,

//     pub name_to_index: HashMap<String, usize>,
//     pub index_to_name: Vec<String>,
// }

// impl VoxelContext<Voxel> for BlockLibrary {
//     fn get_visibility(&self, voxel: &Voxel) -> VoxelVisibility {
//         if let Some(variant) = self.variants.get(voxel.0 as usize) {
//             if variant.is_empty.unwrap_or(false) {
//                 return VoxelVisibility::Empty;
//             }
//             match &variant.block_model {
//                 BlockModel::Cube(c) => {
//                     if c.is_translucent {
//                         VoxelVisibility::Translucent
//                     } else {
//                         VoxelVisibility::Opaque
//                     }
//                 }
//             }
//         } else {
//             error!(
//                 "Could not find voxel {:?} in block_library {:?}",
//                 voxel, self
//             );
//             VoxelVisibility::Empty
//         }
//     }
// }

// impl MergeVoxelContext<Voxel> for BlockLibrary {
//     type MergeValue = u16;
//     type MergeValueFacingNeighbour = u16;

//     fn merge_value(&self, voxel: &Voxel) -> Self::MergeValue {
//         voxel.0
//     }

//     fn merge_value_facing_neighbour(&self, voxel: &Voxel) -> Self::MergeValueFacingNeighbour {
//         voxel.0
//     }
// }

// #[derive(Debug, Default)]
// pub struct BlockLibraryLoader;

// impl AssetLoader for BlockLibraryLoader {
//     // TODO: Wait for bevy to allow async assets
//     type Asset = BlockLibrary;
//     type Settings = ();
//     type Error = anyhow::Error;

//     fn extensions(&self) -> &[&str] {
//         &["json", "bllib", "bllib.json", "blocklib", "blocklib.json"]
//     }

//     fn load(
//         &self,
//         reader: &mut dyn Reader,
//         _: &Self::Settings,
//         load_context: &mut LoadContext,
//     ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
//         async move {
//             let mut bytes = Vec::new();
//             reader.read_to_end(&mut bytes).await?;

//             let raw::BlockLibrary {
//                 materials: raw_materials,
//                 blocks: raw_blocks,
//             } = serde_json::de::from_slice::<raw::BlockLibrary>(&bytes)?;

//             let materials = raw_materials
//                 .into_iter()
//                 .map(|m| {
//                     let raw::Material { path, size } = m;

//                     let path = resolve_path(load_context, &path);
//                     println!("{path:?}");
//                     let handle = load_context.loader().with_static_type().load(path);

//                     Material { handle, size }
//                 })
//                 .collect::<Vec<_>>();

//             let capacity = raw_blocks.len();

//             let mut variants = Vec::with_capacity(capacity);

//             let mut name_to_index = HashMap::new();
//             let mut index_to_name = Vec::with_capacity(capacity);

//             for (i, (name, variant)) in raw_blocks.into_iter().enumerate() {
//                 variants.push(variant);

//                 name_to_index.insert(name.clone(), i);
//                 index_to_name.push(name);
//             }

//             let lib = BlockLibrary {
//                 materials,
//                 variants,
//                 name_to_index,
//                 index_to_name,
//             };

//             Ok(lib)
//         }
//     }
// }

// fn resolve_path(load_context: &LoadContext, path: &str) -> PathBuf {
//     if path.starts_with("/") {
//         PathBuf::from(path.trim_start_matches('/'))
//     } else {
//         PathBuf::from(load_context.path().parent().unwrap_or(Path::new(""))).join(path)
//     }
// }

// pub struct BlockLibraryPlugin;

// impl Plugin for BlockLibraryPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugins(MaterializePlugin::new(JsonMaterialDeserializer))
//             .init_asset::<BlockLibrary>()
//             .init_asset_loader::<BlockLibraryLoader>();
//     }
// }
