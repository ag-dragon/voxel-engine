use crate::mesh::MeshVertex;
use std::slice::Iter;

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

    pub fn opaque(&self) -> bool {
        match *self {
            BlockType::Air => false,
            _ => true,
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
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
];
const BACK_FACE: &[MeshVertex] = &[
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
];
const TOP_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
];
const BOTTOM_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
];
const LEFT_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
];
const RIGHT_FACE: &[MeshVertex] = &[
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
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
        let blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
            = [BlockType::Air; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];

        Self {
            blocks,
        }
    }

    #[inline]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[x + CHUNK_SIZE*y + CHUNK_SIZE*CHUNK_SIZE*z]
    }

    #[inline]
    pub fn get_block_border(&self, neighbors: &[Chunk], x: i32, y: i32, z: i32) -> BlockType {
        let mut nx = 1;
        let mut ny = 1;
        let mut nz = 1;
        let mut bx = x as i32;
        let mut by = y as i32;
        let mut bz = z as i32;
        let max_b = (CHUNK_SIZE-1) as i32;

        if y > max_b {
            ny = 2;
            by = 0;
        } else if y < 0 {
            ny = 0;
            by = max_b;
        } 
        if z > max_b {
            nz = 2;
            bz = 0;
        } else if z < 0 {
            nz = 0;
            bz = max_b;
        } 
        if x > max_b {
            nx = 2;
            bx = 0;
        } else if x < 0 {
            nx = 0;
            bx = max_b;
        }

        if nx == 1 && ny == 1 && nz == 1 {
            self.get_block(bx as usize, by as usize, bz as usize)
        } else {
            neighbors[nx + ny*3 + 3*3*nz].get_block(bx as usize, by as usize, bz as usize)
        }
    }

    pub fn set(&mut self, blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]) {
        self.blocks = blocks;
    }
}
