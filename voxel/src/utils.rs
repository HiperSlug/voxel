use bevy::math::U8Vec3;

pub fn subdivide_index<const BITS: u8>(index: usize) -> U8Vec3 {
	debug_assert!(BITS < 8);
	debug_assert!(index < 1 << (BITS * 3));

	let mask = (1 << BITS) - 1;

	U8Vec3 {
		x: (index & mask) as u8,
		y: ((index >> BITS) & mask) as u8,
		z: ((index >> (BITS * 2)) & mask) as u8,
	}
}
