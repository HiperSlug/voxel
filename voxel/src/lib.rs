mod block_lib;
mod chunk;
mod generator;
mod math;
mod render;
mod terrain;
mod viewer;
mod voxel;
mod voxel_volume;

// use std::{ops::Deref, sync::Arc};
// use bevy::prelude::*;

// #[derive(Resource, Default)]
// pub struct ArcResource<T>(pub Arc<T>);

// impl<T> ArcResource<T> {
// 	pub fn new(value: T) -> Self {
// 		Self(Arc::new(value))
// 	}
// }

// impl<T> Deref for ArcResource<T> {
// 	type Target = T;

// 	fn deref(&self) -> &Self::Target {
// 		&*self.0
// 	}
// }

// impl<T> Clone for ArcResource<T> {
// 	fn clone(&self) -> Self {
// 		ArcResource(self.0.clone())
// 	}
// }
