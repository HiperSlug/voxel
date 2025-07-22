pub mod object;

pub mod chunk;

pub mod voxel {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Voxel(pub usize);
}
