use crate::gpu_state::GpuState;
use crate::mesh::{CMesh, Mesh, MeshVertex};
use nalgebra::{Point3, point};
use noise::{NoiseFn, Perlin};
use std::{
    collections::HashMap,
    slice::Iter,
    sync::{Arc, Mutex},
};

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

pub const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    pub position: Point3<i32>,
    pub blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
}

impl Chunk {
    pub fn new(position: Point3<i32>, height_map: &Perlin) -> Self {
        let mut blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
            = [BlockType::Air; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];
        for i in 0..blocks.len() {
            let terrain_height = height_map.get([
                (position.x as f64 + ((i % CHUNK_SIZE) as f64) / CHUNK_SIZE as f64) / 10.0,
                (position.z as f64 + ((i / (CHUNK_SIZE*CHUNK_SIZE)) as f64) / CHUNK_SIZE as f64) / 10.0,
            ]) * 64.0;
            if (position.y * CHUNK_SIZE as i32) as f64 + ((i / CHUNK_SIZE) % CHUNK_SIZE) as f64 <= terrain_height {
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
        
        Mesh::new(gpu, &CMesh::new(&chunk_vertices, &chunk_indices))
    }

    pub fn load_chunks(chunk_map: &mut Arc<Mutex<HashMap<Point3<i32>, (Chunk, Option<Mesh>)>>>, position: Point3<i32>, range: i32, height_map: &Perlin) {
        let mut chunk_map = chunk_map.lock().unwrap();
        
        for x in -range..=range {
            for y in -range..=range {
                for z in -range..=range {
                    let chunk_pos = point![
                        position.x + x,
                        position.y + y,
                        position.z + z,
                    ];
                    if !chunk_map.contains_key(&chunk_pos) {
                        chunk_map.insert(chunk_pos, (Chunk::new(chunk_pos, height_map), None));
                    }
                }
            }
        }
    }

    // currently load_chunks allows for things other than the player to load chunks. This doesn't
    pub fn unload_chunks(chunk_map: &mut Arc<Mutex<HashMap<Point3<i32>, (Chunk, Option<Mesh>)>>>, position: Point3<i32>, range: i32) {
        let mut chunk_map = chunk_map.lock().unwrap();
        chunk_map.retain(|&chunk_pos, _| {
            (chunk_pos.x - position.x).abs() <= range &&
            (chunk_pos.y - position.y).abs() <= range &&
            (chunk_pos.z - position.z).abs() <= range
        });
    }

    pub fn setup_chunks(chunk_map: &mut Arc<Mutex<HashMap<Point3<i32>, (Chunk, Option<Mesh>)>>>, position: Point3<i32>, range: i32, gpu: &GpuState) {
        let mut chunk_map = chunk_map.lock().unwrap();
        for x in -range..=range {
            for y in -range..=range {
                for z in -range..=range {
                    let chunk_pos = point![
                        position.x + x,
                        position.y + y,
                        position.z + z,
                    ];
                    chunk_map.entry(chunk_pos).and_modify(|(chunk, is_mesh)| {
                        match is_mesh {
                            Some(_) => {},
                            None => *is_mesh = Some(chunk.gen_mesh(gpu)),
                        };
                    });
                }
            }
        }
    }
}
