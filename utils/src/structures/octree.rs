use std::array;

pub struct Octree<T> {
    root: OctreeNode<T>,
    depth: u8,
}

#[derive(Debug, Clone)]
pub enum OctreeNode<T> {
    Leaf(T),
    Branch(Box<[OctreeNode<T>; 8]>),
}

impl<T: Copy> OctreeNode<T> {
    pub fn filled_branch(value: T) -> Self {
        Self::Branch(Box::new(array::from_fn(|_| OctreeNode::Leaf(value))))
    }
}

impl<T: Copy> Octree<T> {
    pub fn get(&self, pos: (u8, u8, u8)) -> T {
        let mut node = &self.root;
        for level in (0..self.depth).rev() {
            match node {
                OctreeNode::Leaf(t) => return *t,
                OctreeNode::Branch(children) => {
                    let index = Self::positional_index(pos, level as u32);
                    node = &children[index];
                }
            }
        }
        match node {
            OctreeNode::Leaf(t) => *t,
            _ => panic!("Branch at lowest level"),
        }
    }

    fn positional_index(pos: (u8, u8, u8), level: u32) -> usize {
        let x = (pos.0 >> level) & 1;
        let y = (pos.1 >> level) & 1;
        let z = (pos.2 >> level) & 1;
        ((z << 2) | (y << 1) | x) as usize
    }

    pub fn set(&mut self, pos: (u8, u8, u8), to: T) {
        let mut node = &mut self.root;
        for level in (0..self.depth).rev() {
            match node {
                OctreeNode::Leaf(t) => {
                    *node = OctreeNode::filled_branch(*t);
                }
                OctreeNode::Branch(children) => {
                    let index = Self::positional_index(pos, level as u32);
                    node = &mut children[index];
                }
            }
        }
        match node {
            OctreeNode::Leaf(t) => *t = to,
            _ => panic!("Branch at lowest level"),
        }
    }
}
