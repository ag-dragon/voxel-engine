use crate::block::BlockType;

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
    pub fn set_block(&mut self, new_block: BlockType, x: usize, y: usize, z: usize) {
        self.blocks[x + CHUNK_SIZE*y + CHUNK_SIZE*CHUNK_SIZE*z] = new_block;
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
