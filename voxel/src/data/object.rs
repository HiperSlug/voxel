use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use super::Chunk;

#[derive(Component)]
pub struct VolumetricObject {
    pub data: HashMap<(i32, i32, i32), Chunk>,
}
