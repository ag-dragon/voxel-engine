use crate::gpu_state::GpuState;
use crate::mesh::{Mesh, MeshVertex};
use nalgebra::{point, Point3};
use std::slice::Iter;

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
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

const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    position: Point3<i32>,
    blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
}

impl Chunk {
    pub fn new(position: Point3<i32>) -> Self {
        let mut blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
            = [BlockType::Dirt; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];
        for i in 0..blocks.len() {
            if (i / CHUNK_SIZE) % CHUNK_SIZE == CHUNK_SIZE-1 {
                blocks[i] = BlockType::Grass;
            }
        }
        Self {
            position,
            blocks,
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[x + CHUNK_SIZE*y + CHUNK_SIZE*CHUNK_SIZE*z]
    }

    pub fn get_neighbor(&self, block: usize, face: &BlockFace) -> BlockType {
        let x = block % CHUNK_SIZE;
        let y = (block / CHUNK_SIZE) % CHUNK_SIZE;
        let z = block / (CHUNK_SIZE*CHUNK_SIZE);

        match face {
            BlockFace::Front => if z < CHUNK_SIZE-1 { self.get_block(x, y, z+1) } else { BlockType::Air },
            BlockFace::Back => if z > 0 { self.get_block(x, y, z-1) } else { BlockType::Air },
            BlockFace::Top => if y < CHUNK_SIZE-1 { self.get_block(x, y+1, z) } else { BlockType::Air },
            BlockFace::Bottom => if y > 0 { self.get_block(x, y-1, z) } else { BlockType::Air },
            BlockFace::Left => if x > 0 { self.get_block(x-1, y, z) } else { BlockType::Air },
            BlockFace::Right => if x < CHUNK_SIZE-1 { self.get_block(x+1, y, z) } else { BlockType::Air },
        }
    }

    pub fn gen_mesh(&self, gpu: &GpuState) -> Mesh {
        let mut chunk_vertices: Vec<MeshVertex> = Vec::new();
        let mut chunk_indices: Vec<u32> = Vec::new();
        let mut o: u32 = 0;

        for (i, block) in self.blocks.into_iter().enumerate() {
            match block {
                BlockType::Air => {},
                _ => {
                    for face in BlockFace::iterator() {
                        match self.get_neighbor(i, face) {
                            BlockType::Air => {
                                chunk_vertices.extend(
                                    face.get_vertices().into_iter().map(|v| MeshVertex {
                                        position: [
                                            (self.position[0] * CHUNK_SIZE as i32) as f32
                                                + v.position[0] + (i % CHUNK_SIZE) as f32,
                                            (self.position[1] * CHUNK_SIZE as i32) as f32
                                                + v.position[1] + ((i / CHUNK_SIZE) % CHUNK_SIZE) as f32,
                                            (self.position[2] * CHUNK_SIZE as i32) as f32
                                                + v.position[2] + (i / (CHUNK_SIZE*CHUNK_SIZE)) as f32,
                                        ],
                                        tex_coords: [
                                            (block.texture(face) % 16) as f32 * 0.0625
                                             + (v.tex_coords[0] * 0.0625),
                                            (block.texture(face) / 16) as f32 * 0.0625
                                             + (v.tex_coords[1] * 0.0625),
                                        ],
                                        normal: v.normal,
                                    })
                                );
                                chunk_indices.extend_from_slice(&[o+0,o+2,o+1, o+2,o+3,o+1]);
                                o += 4;
                            },
                            _ => {}
                        };
                    }
                }
            }
        }
        
        Mesh::new(gpu, &chunk_vertices, &chunk_indices)
    }
}
