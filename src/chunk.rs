use crate::gpu_state::GpuState;
use crate::mesh::{Mesh, MeshVertex};

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Air,
    Grass,
}

pub enum BlockFace {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

impl BlockType {
    pub fn texture(&self, face: &BlockFace) -> Option<u32> {
        match *self {
            BlockType::Grass => match face {
                BlockFace::Top => Some(0),
                BlockFace::Bottom => Some(2),
                _ => Some(1),
            },
            _ => None,
        }
    }
}

const CHUNK_SIZE: usize = 16;

const FRONT_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
];
const BACK_FACE: &[MeshVertex] = &[
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], },
    MeshVertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], },
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

pub struct Chunk {
    blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        let blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
            = [BlockType::Grass; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];
        Self {
            blocks
        }
    }

    pub fn gen_mesh(&self, gpu: &GpuState) -> Mesh {
        // chunk mesh generation (temp putting it here for testing)
        let mut chunk_vertices: Vec<MeshVertex> = Vec::new();
        let mut chunk_indices: Vec<u16> = Vec::new();

        for (i, block) in self.blocks.into_iter().enumerate() {
            chunk_vertices.extend(
                FRONT_FACE.into_iter().map(|v| MeshVertex {
                    position: [
                        v.position[0] + (i % CHUNK_SIZE) as f32,
                        v.position[1] + ((i / CHUNK_SIZE) % CHUNK_SIZE) as f32,
                        v.position[2] + (i / (CHUNK_SIZE*CHUNK_SIZE)) as f32,
                    ],
                    tex_coords: v.tex_coords,
                    normal: v.normal,
                })
            );
            let o = (i*4) as u16;
            chunk_indices.extend_from_slice(&[o+0,o+2,o+1, o+2,o+3,o+1]);
        }
        
        Mesh::new(gpu, &chunk_vertices, &chunk_indices)
    }
}
