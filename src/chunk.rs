use crate::gpu_state::GpuState;
use crate::mesh::{CMesh, Mesh, MeshVertex};
use nalgebra::{Point3, point};
use noise::{NoiseFn, Perlin};
use std::{
    collections::HashMap,
    collections::hash_map::Entry,
    slice::Iter,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
}

impl BlockType {
    pub fn texture(&self, face: &BlockFace) -> u32 {
        match *self {
            BlockType::Grass => match face {
                BlockFace::Top => 0,
                BlockFace::Bottom => 2,
                _ => 1,
            },
            BlockType::Dirt => 2,
            BlockType::Stone => 3,
            BlockType::Sand => 4,
            _ => 255, // Missing Texture
        }
    }
}

#[derive(Debug)]
pub enum BlockFace {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

const FRONT_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];
const BACK_FACE: &[MeshVertex] = &[
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];
const TOP_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];
const BOTTOM_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];
const LEFT_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];
const RIGHT_FACE: &[MeshVertex] = &[
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];

impl BlockFace {
    pub fn iterator() -> Iter<'static, BlockFace> {
        static BLOCK_FACES: [BlockFace; 6] = [
            BlockFace::Front,
            BlockFace::Back,
            BlockFace::Top,
            BlockFace::Bottom,
            BlockFace::Left,
            BlockFace::Right,
        ];
        BLOCK_FACES.iter()
    }

    pub fn get_vertices(&self) -> &[MeshVertex] {
        match *self {
            BlockFace::Front => FRONT_FACE,
            BlockFace::Back => BACK_FACE,
            BlockFace::Top => TOP_FACE,
            BlockFace::Bottom => BOTTOM_FACE,
            BlockFace::Left => LEFT_FACE,
            BlockFace::Right => RIGHT_FACE,
        }
    }
}

pub const CHUNK_SIZE: usize = 32;

#[derive(Clone)]
pub struct Chunk {
    pub blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        let mut blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
            = [BlockType::Air; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];

        Self {
            blocks,
        }
    }

    #[inline]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[x + CHUNK_SIZE*y + CHUNK_SIZE*CHUNK_SIZE*z]
    }

    pub fn set(&mut self, blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]) {
        self.blocks = blocks;
    }
}
