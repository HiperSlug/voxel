use bevy::prelude::*;
use core::panic;
use std::array;

fn position_to_index(pos: (u8, u8, u8), level: u8) -> usize {
    let x = (pos.0 >> level) & 1;
    let y = (pos.1 >> level) & 1;
    let z = (pos.2 >> level) & 1;
    ((z << 2) | (y << 1) | x) as usize
}

fn index_to_offset(index: usize) -> Vec3 {
    Vec3 {
        x: (index & 1) as f32,
        y: ((index >> 1) & 1) as f32,
        z: ((index >> 2) & 1) as f32,
    }
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

    fn recursive_get_leafs(
        &self,
        level: u8,
        position: Vec3,
        size: f32,
        dst: &mut Vec<(Vec3, f32, T)>,
    ) {
        if level == 0 {
            if let Self::Leaf(t) = self {
                dst.push((position, size, *t));
            } else {
                panic!("Branch at lowest level");
            }
            return;
        }

        match self {
            Self::Leaf(t) => dst.push((position, size, *t)),
            Self::Branch(children) => {
                let half = size / 2.0;
                for (index, child) in children.iter().enumerate() {
                    let offset = index_to_offset(index);
                    let child_pos = position + offset * half;
                    child.recursive_get_leafs(level - 1, child_pos, half, dst);
                }
            }
        }
    }
}

pub struct Octree<T, const DEPTH: u8> {
    root: OctreeNode<T>,
}

impl<T, const DEPTH: u8> Octree<T, DEPTH>
where
    T: Copy + PartialEq,
{
    pub fn uniform(value: T) -> Self {
        Self::from_root(OctreeNode::Leaf(value))
    }

    pub fn from_root(root: OctreeNode<T>) -> Self {
        debug_assert!(DEPTH < 8);
        Self { root }
    }

    pub fn get(&self, pos: (u8, u8, u8)) -> T {
        let mut node = &self.root;

        for level in (0..DEPTH).rev() {
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
        self.root.recursive_set(to, pos, DEPTH - 1);
    }

    pub fn leaf_vec(&self) -> Vec<(Vec3, f32, T)> {
        let mut vec = Vec::new();
        self.root
            .recursive_get_leafs(DEPTH - 1, Vec3::ZERO, 2.0f32.powf(DEPTH as f32), &mut vec);
        vec
    }
}
