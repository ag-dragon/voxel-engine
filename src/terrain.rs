use crate::block::{BlockType, BlockFace};
use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::mesh::{Mesh, CMesh, MeshVertex};
use nalgebra::{Vector3, vector};
use rayon::ThreadPool;
use noise::{NoiseFn, Perlin, Curve};
use rand::Rng;
use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc,
};

const RENDER_DISTANCE: i32 = 8;

pub struct ChunkGenResponse {
    position: Vector3<i32>,
    chunk: Chunk,
    is_empty: bool,
}

// function used by worker threads
pub fn gen_chunk(chunk_pos: Vector3<i32>) -> ChunkGenResponse {
    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(134);
    let mut continental_noise: Curve<f64, Perlin, 2> = Curve::new(perlin);
    continental_noise = continental_noise.add_control_point(-1.01, 50.0);
    continental_noise = continental_noise.add_control_point(-1.0, 0.0);
    continental_noise = continental_noise.add_control_point(-0.2, 50.0);
    continental_noise = continental_noise.add_control_point(0.2, 60.0);
    continental_noise = continental_noise.add_control_point(0.6, 60.0);
    continental_noise = continental_noise.add_control_point(1.0, 150.0);
    continental_noise = continental_noise.add_control_point(1.01, 100.0);
    let mut chunk = Chunk::new();

    let mut blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
        = [BlockType::Air; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];
    for i in 0..blocks.len() {
        let x = i % CHUNK_SIZE;
        let y = (i / CHUNK_SIZE) % CHUNK_SIZE;
        let z = i / (CHUNK_SIZE*CHUNK_SIZE);
        let px = ((chunk_pos.x * CHUNK_SIZE as i32) + x as i32) as f64;
        let pz = ((chunk_pos.z * CHUNK_SIZE as i32) + z as i32) as f64;
        let continental = &continental_noise.get([
            px / 320.0,
            pz / 320.0,
        ]);
        let nv1 = perlin.get([
            px / 160.0,
            pz / 160.0,
        ]) * 16.0;
        let nv2 = perlin.get([
            px / 80.0,
            pz / 80.0,
        ]) * 16.0;
        let nv3 = perlin.get([
            px / 40.0,
            pz / 40.0,
        ]) * 16.0;
        let terrain_height = continental + nv1 + 0.5*nv2 + 0.25*nv3;
        let block_height = ((chunk_pos.y * CHUNK_SIZE as i32) + y as i32) as f64;
        if terrain_height > block_height {
            blocks[i] = BlockType::Stone;
        }
    }

    for i in 0..blocks.len() {
        match blocks[i] {
            BlockType::Air => {
                let y = (i / CHUNK_SIZE) % CHUNK_SIZE;
                let block_height = ((chunk_pos.y * CHUNK_SIZE as i32) + y as i32) as f64;
                if block_height <= 40.0 {
                    //water goes here
                    blocks[i] = BlockType::Stone;
                }
            },
            _ => {},
        }
    }
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let mut max_y = 0;
            let mut not_air = false;
            for y in 0..CHUNK_SIZE {
                match blocks[x + y*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] {
                    BlockType::Air => {},
                    _ => {
                        max_y = y;
                        not_air = true;
                    },
                }
            }
            if not_air {
                let block_height = ((chunk_pos.y * CHUNK_SIZE as i32) + max_y as i32) as f64;
                if block_height < 42.0 {
                    blocks[x + max_y*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Sand;
                } else {
                    blocks[x + max_y*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Grass;
                    if max_y >= 2 {
                        blocks[x + (max_y-1)*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Dirt;
                        blocks[x + (max_y-2)*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Dirt;
                    } else if max_y >= 1 {
                        blocks[x + (max_y-1)*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Dirt;
                    }
                }
            }
        }
    }

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = blocks[x + y*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE];
                if block == BlockType::Grass {
                    let rval = rng.gen_range(0.0..1.0);
                    if rval > 0.99 {
                        for tree_height in 1..6 {
                            if y+tree_height < CHUNK_SIZE {
                                blocks[x + (y+tree_height)*CHUNK_SIZE + z*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Wood;
                            }
                        }

                        for leaf_height in 4..6 {
                            for lx in -2..=2 {
                                for lz in -2..=2 {
                                    if ((x as i32+lx) as usize) < CHUNK_SIZE && (x as i32+lx) as usize >= 0
                                    && ((z as i32+lz) as usize) < CHUNK_SIZE && (z as i32+lz) as usize >= 0
                                    && y+leaf_height < CHUNK_SIZE {
                                        blocks[
                                            (x as i32+lx) as usize
                                            + (y+leaf_height)*CHUNK_SIZE
                                            + (z as i32+lz) as usize*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Leaves;
                                    }
                                }
                            }
                        }

                        for lx in -1..=1 {
                            for lz in -1..=1 {
                                    if ((x as i32+lx) as usize) < CHUNK_SIZE && (x as i32+lx) as usize >= 0
                                    && ((z as i32+lz) as usize) < CHUNK_SIZE && (z as i32+lz) as usize >= 0
                                    && y+6 < CHUNK_SIZE {
                                        blocks[
                                            (x as i32+lx) as usize
                                            + (y+6)*CHUNK_SIZE
                                            + (z as i32+lz) as usize*CHUNK_SIZE*CHUNK_SIZE] = BlockType::Leaves;
                                    }
                            }
                        }
                    }
                }
            }
        }
    }

    chunk.set(blocks);
    ChunkGenResponse {
        position: chunk_pos,
        chunk,
        is_empty: !blocks.into_iter().any(|b| b != BlockType::Air),
    }
}

pub struct TerrainChanges {
    pub loaded_chunks: Vec<Vector3<i32>>,
    pub unloaded_chunks: Vec<Vector3<i32>>,
    pub modified_chunks: HashMap<Vector3<i32>, Vec<(Vector3<usize>, BlockType)>>,
}

impl TerrainChanges {
    pub fn new() -> Self {
        let loaded_chunks: Vec<Vector3<i32>> = Vec::new();
        let unloaded_chunks: Vec<Vector3<i32>> = Vec::new();
        let modified_chunks: HashMap<Vector3<i32>, Vec<(Vector3<usize>, BlockType)>> = HashMap::new();

        Self {
            loaded_chunks,
            unloaded_chunks,
            modified_chunks,
        }
    }
}

pub struct ChunkData {
    chunk: Chunk,
    pub is_empty: bool,
}

pub struct Terrain {
    player_chunk: Vector3<i32>,
    chunk_map: HashMap<Vector3<i32>, ChunkData>,
    loading_tx: mpsc::Sender<ChunkGenResponse>, // for cloning and handing to worker threads
    loading_rx: mpsc::Receiver<ChunkGenResponse>,
    load_todo: Vec<Vector3<i32>>,
    loading: Vec<Vector3<i32>>,
    unload_todo: Vec<Vector3<i32>>,
}

impl Terrain {
    pub fn new() -> Self {
        let player_chunk = vector![0, 0, 0];
        let chunk_map: HashMap<Vector3<i32>, ChunkData> = HashMap::new();
        let (loading_tx, loading_rx) = mpsc::channel();
        let load_todo: Vec<Vector3<i32>> = Vec::new();
        let loading: Vec<Vector3<i32>> = Vec::new();
        let unload_todo: Vec<Vector3<i32>> = Vec::new();

        Self {
            player_chunk,
            chunk_map,
            loading_tx,
            loading_rx,
            load_todo,
            loading,
            unload_todo,
        }
    }

    pub fn get_chunk(&self, chunk_pos: Vector3<i32>) -> Option<&ChunkData> {
        self.chunk_map.get(&chunk_pos)
    }

    pub fn get_block(&self, block_world_pos: Vector3<i32>) -> Option<(Vector3<usize>, BlockType)> {
        let block_pos = vector![
            ((block_world_pos.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32,
            ((block_world_pos.y % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32,
            ((block_world_pos.z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32,
        ];
        let chunk_pos = (block_world_pos - block_pos) / CHUNK_SIZE as i32;
        if let Some(chunk_data) = self.get_chunk(chunk_pos) {
            Some((
                block_pos.try_cast::<usize>().unwrap(),
                chunk_data.chunk.get_block(block_pos.try_cast::<usize>().unwrap()),
            ))
        } else {
            None
        }
    }

    pub fn check_neighbors(&self, chunk_pos: Vector3<i32>) -> bool {
        let mut result = true;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if !self.chunk_map.contains_key(&vector![
                        chunk_pos.x + x,
                        chunk_pos.y + y,
                        chunk_pos.z + z,
                    ]) {
                        result = false;
                    }
                }
            }
        }
        result
    }

    pub fn add_chunk(&mut self, chunk_pos: Vector3<i32>, chunk: ChunkData) {
        if let Some(_) = self.chunk_map.insert(chunk_pos, chunk) {
            // we just overwrote another chunk, no reason this should be able to happen currently
            eprintln!["uh oh, a chunk was overwritten by another"];
        }

    }

    // unload chunk
    pub fn remove_chunk(&mut self, chunk_pos: Vector3<i32>) {
        self.chunk_map.remove(&chunk_pos);
        self.load_todo.retain(|chunk| *chunk != chunk_pos);
    }

    // upon entering new chunk, add list of new chunks to load todo
    pub fn load_chunks(&mut self, chunk_pos: Vector3<i32>) {
        if !self.chunk_map.contains_key(&chunk_pos)
        && !self.load_todo.contains(&chunk_pos) 
        && !self.loading.contains(&chunk_pos) {
            self.load_todo.push(chunk_pos);
        }
        for radius in 1..=RENDER_DISTANCE+1 {
            for edge in (-radius..=radius).step_by((radius*2) as usize) {
                for x in -radius..=radius {
                    for z in -radius..=radius {
                        let cpos = chunk_pos + vector![x, edge, z];
                        if !self.chunk_map.contains_key(&cpos)
                        && !self.load_todo.contains(&cpos) 
                        && !self.loading.contains(&cpos) {
                            self.load_todo.push(cpos);
                        }
                    }
                }

                for y in -radius+1..=radius-1 {
                    for z in -radius..=radius {
                        let cpos = chunk_pos + vector![edge, y, z];
                        if !self.chunk_map.contains_key(&cpos)
                        && !self.load_todo.contains(&cpos) 
                        && !self.loading.contains(&cpos) {
                            self.load_todo.push(cpos);
                        }
                    }
                }

                for y in -radius+1..=radius-1 {
                    for x in -radius+1..=radius-1 {
                        let cpos = chunk_pos + vector![x, y, edge];
                        if !self.chunk_map.contains_key(&cpos)
                        && !self.load_todo.contains(&cpos) 
                        && !self.loading.contains(&cpos) {
                            self.load_todo.push(cpos);
                        }
                    }
                }
            }
        }
    }

    // upon entering new chunk, remove all chunks that are too far from player
    pub fn unload_chunks(&mut self, chunk_pos: Vector3<i32>) {
        let mut unload_chunks: Vec<Vector3<i32>> = self.chunk_map.keys().cloned().collect();
        unload_chunks.retain(|cpos| {
            (cpos.x - chunk_pos.x).abs() > RENDER_DISTANCE+1
            || (cpos.y - chunk_pos.y).abs() > RENDER_DISTANCE+1
            || (cpos.z - chunk_pos.z).abs() > RENDER_DISTANCE+1
        });

        for chunk in unload_chunks {
            self.unload_todo.push(chunk);
        }
    }

    pub fn update(&mut self, player_pos: Vector3<i32>, terrain_changes_in: TerrainChanges, thread_pool: &ThreadPool) -> TerrainChanges {
        let mut terrain_changes_out = TerrainChanges::new();

        for (chunk_pos, block_changes) in &terrain_changes_in.modified_chunks {
            let mut chunk_data = self.chunk_map.get_mut(&chunk_pos).unwrap();
            for (block_pos, new_block) in block_changes {
                chunk_data.chunk.set_block(*new_block, *block_pos);
            }
            chunk_data.is_empty = !chunk_data.chunk.blocks.into_iter().any(|b| b != BlockType::Air);
            terrain_changes_out.modified_chunks.insert(*chunk_pos, block_changes.to_vec());
        }

        if player_pos != self.player_chunk ||
            (self.chunk_map.is_empty() && self.load_todo.is_empty() && self.loading.is_empty()) {
            self.load_chunks(player_pos);
            self.unload_chunks(player_pos);
            self.player_chunk = player_pos;
        }

        for chunk in self.load_todo.drain(..) {
            let tchunk = chunk.clone();
            let loading_tx = self.loading_tx.clone();
            thread_pool.spawn(move || {
                let _ = loading_tx.send(gen_chunk(tchunk));
            });
            self.loading.push(chunk);
        }

        let to_add: Vec<ChunkGenResponse> = self.loading_rx.try_iter().collect();
        for response in to_add {
            self.add_chunk(response.position, ChunkData {
                chunk: response.chunk,
                is_empty: response.is_empty,
            });
            self.loading.retain(|c| *c != response.position);
            terrain_changes_out.loaded_chunks.push(response.position);
        }

        for _ in 0..10 {
            if let Some(chunk) = self.unload_todo.pop() {
                self.remove_chunk(chunk);
                terrain_changes_out.unloaded_chunks.push(chunk);
            }
        }


        terrain_changes_out
    }
}

// function used by worker threads
pub fn mesh_chunk(chunk_pos: Vector3<i32>, chunk: Chunk, neighbors: &[Chunk]) -> CMesh {
    let mut chunk_vertices: Vec<MeshVertex> = Vec::new();
    let mut chunk_indices: Vec<u32> = Vec::new();
    let mut o: u32 = 0;

    for (i, block) in chunk.blocks.into_iter().enumerate() {
        match block {
            BlockType::Air => {},
            _ => {
                for face in BlockFace::iterator() {
                    let block_pos = vector![
                        i % CHUNK_SIZE,
                        (i / CHUNK_SIZE) % CHUNK_SIZE,
                        i / (CHUNK_SIZE*CHUNK_SIZE),
                    ];
                    let mut n = block_pos.cast::<i32>();
                    match face {
                        BlockFace::Top => n.y += 1,
                        BlockFace::Bottom => n.y -= 1,
                        BlockFace::Front => n.z += 1,
                        BlockFace::Back => n.z -= 1,
                        BlockFace::Right => n.x += 1,
                        BlockFace::Left => n.x -= 1,
                    }
                    match chunk.get_block_border(neighbors, n) {
                        BlockType::Air => {
                            chunk_vertices.extend(
                                face.get_vertices().into_iter().map(|v| {
                                    let mut ao = 0.0;
                                    let n1 = vector![
                                        block_pos.x as i32 + (v.position[0] * 2.0) as i32,
                                        block_pos.y as i32 + (v.position[1] * 2.0) as i32,
                                        block_pos.z as i32,
                                    ];
                                    let n2 = vector![
                                        block_pos.x as i32,
                                        block_pos.y as i32 + (v.position[1] * 2.0) as i32,
                                        block_pos.z as i32 + (v.position[2] * 2.0) as i32,
                                    ];
                                    let n3 = vector![
                                        block_pos.x as i32 + (v.position[0] * 2.0) as i32,
                                        block_pos.y as i32 + (v.position[1] * 2.0) as i32,
                                        block_pos.z as i32 + (v.position[2] * 2.0) as i32,
                                    ];

                                    let mut cv = false;
                                    if chunk.get_block_border(neighbors, n1).opaque() {
                                        ao += 1.0;
                                        cv = true;
                                    }
                                    if chunk.get_block_border(neighbors, n2).opaque() {
                                        ao += 1.0;
                                        cv = true;
                                    }
                                    if cv && chunk.get_block_border(neighbors, n3).opaque() {
                                        ao += 1.0;
                                    }
                                    

                                    MeshVertex {
                                        position: [
                                            (chunk_pos.x * CHUNK_SIZE as i32) as f32
                                                + v.position[0] + (i % CHUNK_SIZE) as f32,
                                            (chunk_pos.y * CHUNK_SIZE as i32) as f32
                                                + v.position[1] + ((i / CHUNK_SIZE) % CHUNK_SIZE) as f32,
                                            (chunk_pos.z * CHUNK_SIZE as i32) as f32
                                                + v.position[2] + (i / (CHUNK_SIZE*CHUNK_SIZE)) as f32,
                                        ],
                                        tex_coords: [
                                            (block.texture(face) % 16) as f32 * 0.0625
                                                + (v.tex_coords[0] * 0.0625),
                                            (block.texture(face) / 16) as f32 * 0.0625
                                                + (v.tex_coords[1] * 0.0625),
                                        ],
                                        normal: v.normal,
                                        ao,
                                    }
                                })
                            );
                            chunk_indices.extend_from_slice(&[o+0,o+2,o+1,o+2,o+3,o+1]);
                            o += 4;
                        },
                        _ => {},
                    };
                }
            },
        }

    }

    CMesh::new(&chunk_vertices, &chunk_indices)
}

pub struct TerrainMesh {
    player_chunk: Vector3<i32>,
    meshed_chunks: HashMap<Vector3<i32>, Mesh>,
    meshing_tx: mpsc::Sender<(Vector3<i32>, CMesh)>,
    meshing_rx: mpsc::Receiver<(Vector3<i32>, CMesh)>,
    meshes_todo: VecDeque<Vector3<i32>>,
}

impl TerrainMesh {
    pub fn new() -> Self {
        let player_chunk = vector![0, 0, 0];
        let meshed_chunks: HashMap<Vector3<i32>, Mesh> = HashMap::new();
        let (meshing_tx, meshing_rx) = mpsc::channel();
        let meshes_todo: VecDeque<Vector3<i32>> = VecDeque::new();

        Self {
            player_chunk,
            meshed_chunks,
            meshing_tx,
            meshing_rx,
            meshes_todo,
        }
    }

    pub fn insert_chunk(&mut self, chunk_pos: Vector3<i32>, mesh: Mesh) {
        if let Some(_) = self.meshed_chunks.insert(chunk_pos, mesh) {
            // old mesh rewritten. If I add metadata for meshes, delete it here
        }
    }

    pub fn remove_chunk(&mut self, chunk_pos: Vector3<i32>) {
        self.meshed_chunks.remove(&chunk_pos);
        self.meshes_todo.retain(|chunk| *chunk != chunk_pos);
    }

    pub fn get_meshes(&self) -> Vec<&Mesh> {
        let mut render_meshes = Vec::new();
        for (_, mesh) in &self.meshed_chunks {
            render_meshes.push(mesh);
        }
        render_meshes
    }

    pub fn update(&mut self, terrain_changes: &TerrainChanges, terrain_data: &Terrain, player_pos: Vector3<i32>, device: &wgpu::Device, thread_pool: &ThreadPool) {
        self.player_chunk = player_pos;

        for chunk in &terrain_changes.unloaded_chunks {
            self.remove_chunk(*chunk);

            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        if x != 0 || y != 0 || z != 0 {
                            let n_pos = chunk + vector![x, y, z];
                            self.meshes_todo.retain(|chunk| *chunk != n_pos);
                        }
                    }
                }
            }
        }

        for (chunk, _) in &terrain_changes.modified_chunks {
            if terrain_data.check_neighbors(*chunk) {
                self.meshes_todo.push_front(*chunk);
            }

            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let n_pos = chunk + vector![x, y, z];
                        if !self.meshes_todo.contains(&n_pos) {
                            match terrain_data.chunk_map.get(&n_pos) {
                                Some(n_data) => {
                                    if !n_data.is_empty && terrain_data.check_neighbors(n_pos) {
                                        if (n_pos.x - self.player_chunk.x).abs() <= RENDER_DISTANCE
                                        && (n_pos.y - self.player_chunk.y).abs() <= RENDER_DISTANCE
                                        && (n_pos.z - self.player_chunk.z).abs() <= RENDER_DISTANCE {
                                            self.meshes_todo.push_front(n_pos);
                                        }
                                    }
                                },
                                None => {},
                            }
                        }
                    }
                }
            }
        }

        for chunk in &terrain_changes.loaded_chunks {
            if !terrain_data.chunk_map.get(chunk).unwrap().is_empty && terrain_data.check_neighbors(*chunk) {
                if (chunk.x - self.player_chunk.x).abs() <= RENDER_DISTANCE
                && (chunk.y - self.player_chunk.y).abs() <= RENDER_DISTANCE
                && (chunk.z - self.player_chunk.z).abs() <= RENDER_DISTANCE {
                    self.meshes_todo.push_back(*chunk);
                }
            }

            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let n_pos = chunk + vector![x, y, z];
                        if !self.meshes_todo.contains(&n_pos) {
                            match terrain_data.chunk_map.get(&n_pos) {
                                Some(n_data) => {
                                    if !n_data.is_empty && terrain_data.check_neighbors(n_pos) {
                                        if (n_pos.x - self.player_chunk.x).abs() <= RENDER_DISTANCE
                                        && (n_pos.y - self.player_chunk.y).abs() <= RENDER_DISTANCE
                                        && (n_pos.z - self.player_chunk.z).abs() <= RENDER_DISTANCE {
                                            self.meshes_todo.push_back(n_pos);
                                        }
                                    }
                                },
                                None => {},
                            }
                        }
                    }
                }
            }
        }

        // TODO: maybe, limit to one per frame? after the first couple its not very important to do
        // it all at once, cause the threads will be busy anyways
        // also this whole section is just very messy
        'workers: for _ in 0..10 {
            if let Some(chunk) = self.meshes_todo.pop_front() {
                let tchunk = chunk.clone();
                let chunk_data = (*terrain_data.chunk_map.get(&chunk).unwrap()).chunk.clone();
                let mut neighbor_chunks = Vec::new();
                for z in -1..=1 {
                    for y in -1..=1 {
                        for x in -1..=1 {
                            match terrain_data.chunk_map.get(&(chunk+vector![x, y, z])) {
                                Some(nchunk) => neighbor_chunks.push((*nchunk).chunk.clone()),
                                None => {
                                    self.meshes_todo.push_back(chunk);
                                    continue 'workers;
                                },
                            }
                        }
                    }
                }
                let meshing_tx = self.meshing_tx.clone();
                
                thread_pool.spawn(move || {
                    let _ = meshing_tx.send((tchunk, mesh_chunk(tchunk, chunk_data, &neighbor_chunks[..])));
                });
            }
        }

        // TODO: limit this to a certain number per second based on delta time, similar to veloren
        let completed_meshes: Vec<(Vector3<i32>, CMesh)> = self.meshing_rx.try_iter().collect();
        for (chunk, mesh) in completed_meshes {
            if terrain_data.chunk_map.contains_key(&chunk) {
                self.insert_chunk(chunk, Mesh::new(device, &mesh));
            }
        }
    }
}
