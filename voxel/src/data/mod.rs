pub mod chunk;

pub mod voxel {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct Voxel(usize);

    impl Voxel {
        pub fn new(val: usize) -> Self {
            Self(val)
        }
    }
}

use voxel::Voxel;
