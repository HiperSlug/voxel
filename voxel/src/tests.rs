use bevy::math::U8Vec3;

use crate::data::utils::subdivide_index;

#[test]
fn subdivide_index_generates_y() {
    let coords: Vec<_> = (0..(16usize).pow(3))
        .map(|i| subdivide_index::<4>(i))
        .collect();

    assert_eq!(coords[16usize], U8Vec3::new(0, 1, 0));
}
