use crate::mesh::MeshVertex;
use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
    Leaves,
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
            BlockType::Wood => match face {
                BlockFace::Top => 22,
                BlockFace::Bottom => 22,
                _ => 6,
            },
            BlockType::Leaves => 7,
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
    MeshVertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
    MeshVertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], ao: 0.0 },
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

