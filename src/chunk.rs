use crate::block::BlockType;
use nalgebra::{Vector3, vector};

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
    pub fn get_block(&self, position: Vector3<usize>) -> BlockType {
        self.blocks[position.x + CHUNK_SIZE*position.y + CHUNK_SIZE*CHUNK_SIZE*position.z]
    }

    #[inline]
    pub fn set_block(&mut self, new_block: BlockType, position: Vector3<usize>) {
        self.blocks[position.x + CHUNK_SIZE*position.y + CHUNK_SIZE*CHUNK_SIZE*position.z] = new_block;
    }

    #[inline]
    pub fn get_block_border(&self, neighbors: &[Chunk], position: Vector3<i32>) -> BlockType {
        let mut n = vector![1, 1, 1];
        let mut b = position;
        let max_b = (CHUNK_SIZE-1) as i32;

        if position.y > max_b {
            n.y = 2;
            b.y = 0;
        } else if position.y < 0 {
            n.y = 0;
            b.y = max_b;
        } 
        if position.z > max_b {
            n.z = 2;
            b.z = 0;
        } else if position.z < 0 {
            n.z = 0;
            b.z = max_b;
        } 
        if position.x > max_b {
            n.x = 2;
            b.x = 0;
        } else if position.x < 0 {
            n.x = 0;
            b.x = max_b;
        }

        if n.x == 1 && n.y == 1 && n.z == 1 {
            self.get_block(b.try_cast::<usize>().unwrap())
        } else {
            neighbors[n.x + n.y*3 + 3*3*n.z].get_block(b.try_cast::<usize>().unwrap())
        }
    }

    pub fn set(&mut self, blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]) {
        self.blocks = blocks;
    }
}
