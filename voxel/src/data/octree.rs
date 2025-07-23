/// THIS CAN BE HEAVILY OPTIMIZED




use std::array;

fn position_to_index(pos: (u8, u8, u8), level: u8) -> usize {
    let x = (pos.0 >> level) & 1;
    let y = (pos.1 >> level) & 1;
    let z = (pos.2 >> level) & 1;
    ((z << 2) | (y << 1) | x) as usize
}

pub enum OctreeNode<T> {
    Leaf(T),
    Branch(Box<[OctreeNode<T>; 8]>),
}

impl<T: Copy + PartialEq> OctreeNode<T> {
    fn branch_of(inner: T) -> Self {
        let leafs = array::from_fn(|_| OctreeNode::Leaf(inner));
        Self::Branch(Box::new(leafs))
    }

    fn recursive_set(&mut self, to: T, pos: (u8, u8, u8), level: u8) -> bool {
        if level == 0 {
            if let OctreeNode::Leaf(t) = self {
                *t = to;
            } else {
                panic!("Branch at lowest level");
            }
            return true;
        }

        match self {
            OctreeNode::Leaf(t) => {
                let t = *t;

                if t != to {
                    let mut node = self;

                    for level in (0..level).rev() {
                        *node = OctreeNode::branch_of(t);

                        if let OctreeNode::Branch(children) = node {
                            let index = position_to_index(pos, level);
                            node = &mut children[index];
                        } else {
                            unreachable!();
                        }
                    }

                    if let OctreeNode::Leaf(t) = node {
                        *t = to;
                    } else {
                        unreachable!();
                    }
                }

                false
            }
            OctreeNode::Branch(children) => {
                let index: usize = position_to_index(pos, level);
                let was_simplified = children[index].recursive_set(to, pos, level - 1);

                if was_simplified {
                    let base_type = match &children[0] {
                        OctreeNode::Branch(_) => return false,
                        OctreeNode::Leaf(t) => *t,
                    };

                    for child in children.iter().skip(1) {
                        match child {
                            OctreeNode::Branch(_) => return false,
                            OctreeNode::Leaf(t) => {
                                if *t != base_type {
                                    return false;
                                }
                            }
                        }
                    }

                    *self = OctreeNode::Leaf(base_type);

                    true
                } else {
                    false
                }
            }
        }
    }
}

pub struct Octree<T> {
    root: OctreeNode<T>,
    depth: u8,
}

impl<T: Copy + PartialEq> Octree<T> {
    pub fn new(root: OctreeNode<T>, depth: u8) -> Self {
        debug_assert!(depth < 8, "Octree exceeds max depth");
        Self { root, depth }
    }

    pub fn get(&self, pos: (u8, u8, u8)) -> T {
        let mut node = &self.root;

        for level in (0..self.depth).rev() {
            match node {
                OctreeNode::Leaf(t) => return *t,
                OctreeNode::Branch(children) => {
                    let index = position_to_index(pos, level);
                    node = &children[index];
                }
            }
        }

        if let OctreeNode::Leaf(t) = node {
            *t
        } else {
            panic!("Branch at lowest level");
        }
    }

    pub fn set(&mut self, pos: (u8, u8, u8), to: T) {
        self.root.recursive_set(to, pos, self.depth - 1);
    }
}
