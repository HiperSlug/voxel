use bevy::platform::collections::HashMap;
use crate::data::{octree::Octree, voxel::Voxel};

pub struct VoxelObject {
	data: HashMap<(i32, i32, i32), Octree<Voxel>>
}
