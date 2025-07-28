use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct VoxelViewer {
    pub view_distance: u8,
}

impl VoxelViewer {
    pub fn visible_positions(&self, chunk_pos: IVec3) -> impl Iterator<Item = IVec3> {
        sphere_iter(chunk_pos, self.view_distance)
    }
}

fn sphere_iter(position: IVec3, radius: u8) -> impl Iterator<Item = IVec3> {
    let radius = radius as i32;
    let radius_sq = radius.pow(2);

    (-radius..=radius).flat_map(move |x| {
        (-radius..=radius).flat_map(move |y| {
            (-radius..=radius).filter_map(move |z| {
                let offset = IVec3::new(x, y, z);
                if offset.length_squared() <= radius_sq {
                    Some(offset + position)
                } else {
                    None
                }
            })
        })
    })
}
