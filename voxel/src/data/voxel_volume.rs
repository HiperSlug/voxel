use crate::{data::octree::Octree, mesher};
use bevy::{platform::collections::HashMap, prelude::*};

use super::chunk::{CHUNK_LENGTH, Chunk};

#[derive(Component)]
pub struct VoxelVolume {
    chunks: HashMap<IVec3, (Chunk, Entity)>, // Loaded chunks
    load: Vec<IVec3>,                        // Pending load
    visible: Vec<IVec3>,                     // In sphere around `Viewer`
    render: Vec<IVec3>,                      // in `visible` and in view frustum
}

impl VoxelVolume {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            load: Vec::new(),
            visible: Vec::new(),
            render: Vec::new(),
        }
    }

    pub fn update_visible_list(&mut self, visible: Vec<IVec3>) {
        self.load = visible
            .iter()
            .filter_map(|v| {
                if self.chunks.contains_key(v) {
                    None
                } else {
                    Some(*v)
                }
            })
            .collect();

        self.visible = visible;
    }

    pub fn update_render_list(&mut self, render: Vec<IVec3>) {
        self.render = render;
    }

    pub fn load_chunks(&self) -> Vec<(IVec3, Chunk)> {
        self.load
            .iter()
            .map(|position| {
                // TODO REPLACE WITH GENERATOR and LOADING
                (*position, {
                    if position.y < 0 {
                        Chunk {
                            data: Octree::uniform(super::Voxel { id: 1 }),
                        } // solid
                    } else {
                        Chunk {
                            data: Octree::uniform(super::Voxel { id: 0 }),
                        } // non solid
                    }
                })
            })
            .collect()
    }

    pub fn unload_chunks(&mut self, positions: &[IVec3]) {
        for position in positions {
            // TODO SAFE DIRTY
            self.chunks.remove(position);
        }
    }
}

pub fn update_voxel_volumes(
    viewers: Query<(&VoxelViewer, &Transform)>,
    voxel_volumes: Query<(&mut VoxelVolume, &Transform)>,
) {
    for (mut voxel_volume, vol_transform) in voxel_volumes {
        let mut visible = Vec::new();
        let mut render = Vec::new();

        for (viewer, viewer_transform) in viewers {
            let viewer_chunk_pos =
                translation_to_chunk_pos(viewer_transform.translation - vol_transform.translation);
            let radius = viewer.load_distance as i32;
            for x in -radius..=radius {
                for y in -radius..=radius {
                    for z in -radius..=radius {
                        let offset = IVec3::new(x, y, z);
                        if offset.length_squared() <= radius.pow(2) {
                            let chunk_pos = viewer_chunk_pos + offset;
                            visible.push(chunk_pos);
                            // TODO frustum culling.
                            render.push(chunk_pos);
                        }
                    }
                }
            }
        }

        voxel_volume.update_visible_list(visible);
        voxel_volume.update_render_list(render);
    }
}

pub fn load_queue(
    voxel_volumes: Query<&mut VoxelVolume>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for mut voxel_volume in voxel_volumes {
        let mut chunks = voxel_volume.load_chunks();
        chunks.truncate(8);
        for (pos, chunk) in chunks {
            let translation = chunk_pos_to_translation(pos);
            let mesh = mesher::build(&chunk);
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                    Transform::from_xyz(translation.x, translation.y, translation.z),
                ))
                .id();

            voxel_volume.chunks.insert(pos, (chunk, entity));
        }
    }
}

#[derive(Component)]
pub struct VoxelViewer {
    pub load_distance: u8,
}

fn translation_to_chunk_pos(translation: Vec3) -> IVec3 {
    IVec3 {
        x: (translation.x / CHUNK_LENGTH as f32).floor() as i32,
        y: (translation.y / CHUNK_LENGTH as f32).floor() as i32,
        z: (translation.z / CHUNK_LENGTH as f32).floor() as i32,
    }
}

fn chunk_pos_to_translation(chunk_pos: IVec3) -> Vec3 {
    Vec3 {
        x: chunk_pos.x as f32 * CHUNK_LENGTH as f32,
        y: chunk_pos.y as f32 * CHUNK_LENGTH as f32,
        z: chunk_pos.z as f32 * CHUNK_LENGTH as f32,
    }
}
