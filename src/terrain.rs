use crate::block::{BlockType, BlockFace};
use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::mesh::{Mesh, CMesh, MeshVertex};
use nalgebra::{Point3, point};
use rayon::ThreadPool;
use noise::{NoiseFn, Perlin, Curve};
use std::{
    collections::{HashMap, VecDeque},
    sync::{mpsc, Mutex, Arc},
};

const RENDER_DISTANCE: i32 = 8;

// function used by worker threads
pub fn gen_chunk(chunk_pos: Point3<i32>) -> Chunk {
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

    chunk.set(blocks);
    chunk
}

// function used by worker threads
pub fn mesh_chunk(chunk_pos: Point3<i32>, chunk: Chunk, neighbors: &[Chunk]) -> CMesh {
    let mut chunk_vertices: Vec<MeshVertex> = Vec::new();
    let mut chunk_indices: Vec<u32> = Vec::new();
    let mut o: u32 = 0;

    for (i, block) in chunk.blocks.into_iter().enumerate() {
        match block {
            BlockType::Air => {},
            _ => {
                for face in BlockFace::iterator() {
                    let x = i % CHUNK_SIZE;
                    let y = (i / CHUNK_SIZE) % CHUNK_SIZE;
                    let z = i / (CHUNK_SIZE*CHUNK_SIZE);
                    let mut nx: i32 = x as i32;
                    let mut ny: i32 = y as i32;
                    let mut nz: i32 = z as i32;
                    match face {
                        BlockFace::Top => ny += 1,
                        BlockFace::Bottom => ny -= 1,
                        BlockFace::Front => nz += 1,
                        BlockFace::Back => nz -= 1,
                        BlockFace::Right => nx += 1,
                        BlockFace::Left => nx -= 1,
                    }
                    match chunk.get_block_border(neighbors, nx, ny, nz) {
                        BlockType::Air => {
                            chunk_vertices.extend(
                                face.get_vertices().into_iter().map(|v| {
                                    let mut ao = 0.0;
                                    let n1 = point![
                                        x as i32 + (v.position[0] * 2.0) as i32,
                                        y as i32 + (v.position[1] * 2.0) as i32,
                                        z as i32,
                                    ];
                                    let n2 = point![
                                        x as i32,
                                        y as i32 + (v.position[1] * 2.0) as i32,
                                        z as i32 + (v.position[2] * 2.0) as i32,
                                    ];
                                    let n3 = point![
                                        x as i32 + (v.position[0] * 2.0) as i32,
                                        y as i32 + (v.position[1] * 2.0) as i32,
                                        z as i32 + (v.position[2] * 2.0) as i32,
                                    ];

                                    let mut cv = false;
                                    if chunk.get_block_border(neighbors, n1.x, n1.y, n1.z).opaque() {
                                        ao += 1.0;
                                        cv = true;
                                    }
                                    if chunk.get_block_border(neighbors, n2.x, n2.y, n2.z).opaque() {
                                        ao += 1.0;
                                        cv = true;
                                    }
                                    if cv && chunk.get_block_border(neighbors, n3.x, n3.y, n3.z).opaque() {
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

pub struct TerrainChanges {
    loaded_chunks: Vec<Point3<i32>>,
    unloaded_chunks: Vec<Point3<i32>>,
    modified_chunks: Vec<Point3<i32>>,
}

pub struct Terrain {
    thread_pool: ThreadPool,
    player_chunk: Point3<i32>,
    chunk_map: HashMap<Point3<i32>, Chunk>,
    loading_tx: mpsc::Sender<(Point3<i32>, Chunk)>, // for cloning and handing to worker threads
    loading_rx: mpsc::Receiver<(Point3<i32>, Chunk)>,
    load_todo: Vec<Point3<i32>>,
    loading: Vec<Point3<i32>>,
    unload_todo: Vec<Point3<i32>>,
}

impl Terrain {
    pub fn new() -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        let player_chunk = point![0, 0, 0];
        let chunk_map: HashMap<Point3<i32>, Chunk> = HashMap::new();
        let (loading_tx, loading_rx) = mpsc::channel();
        let load_todo: Vec<Point3<i32>> = Vec::new();
        let loading: Vec<Point3<i32>> = Vec::new();
        let unload_todo: Vec<Point3<i32>> = Vec::new();

        Self {
            thread_pool,
            player_chunk,
            chunk_map,
            loading_tx,
            loading_rx,
            load_todo,
            loading,
            unload_todo,
        }
    }

    pub fn check_neighbors(&self, chunk_pos: Point3<i32>) -> bool {
        let mut result = true;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if !self.chunk_map.contains_key(&point![
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

    pub fn add_chunk(&mut self, chunk_pos: Point3<i32>, chunk: Chunk) {
        if let Some(_) = self.chunk_map.insert(chunk_pos, chunk) {
            // we just overwrote another chunk, no reason this should be able to happen currently
            eprintln!["uh oh, a chunk was overwritten by another"];
        }

    }

    // unload chunk
    pub fn remove_chunk(&mut self, chunk_pos: Point3<i32>) {
        self.chunk_map.remove(&chunk_pos);
        self.load_todo.retain(|chunk| *chunk != chunk_pos);
    }

    // upon entering new chunk, add list of new chunks to load todo
    pub fn load_chunks(&mut self, chunk_pos: Point3<i32>) {
        for x in -RENDER_DISTANCE-1..=RENDER_DISTANCE+1 {
            for y in -RENDER_DISTANCE-1..=RENDER_DISTANCE+1 {
                for z in -RENDER_DISTANCE-1..=RENDER_DISTANCE+1{
                    let cpos = point![
                        chunk_pos.x + x,
                        chunk_pos.y + y,
                        chunk_pos.z + z,
                    ];
                    if !self.chunk_map.contains_key(&cpos)
                    && !self.load_todo.contains(&cpos) 
                    && !self.loading.contains(&cpos) {
                        self.load_todo.push(cpos);
                    }
                }
            }
        }
    }

    // upon entering new chunk, remove all chunks that are too far from player
    pub fn unload_chunks(&mut self, chunk_pos: Point3<i32>) {
        let mut unload_chunks: Vec<Point3<i32>> = self.chunk_map.keys().cloned().collect();
        unload_chunks.retain(|cpos| {
            (cpos.x - chunk_pos.x).abs() > RENDER_DISTANCE+1
            || (cpos.y - chunk_pos.y).abs() > RENDER_DISTANCE+1
            || (cpos.z - chunk_pos.z).abs() > RENDER_DISTANCE+1
        });

        for chunk in unload_chunks {
            self.unload_todo.push(chunk);
        }
    }

    // checks if player enters new chunk
    // loads new chunk from queue
    // spawns new tasks for worker threads from mesh todo list
    // sends completed meshes to gpu and adds to meshed chunks map
    pub fn update(&mut self, player_pos: Point3<i32>, device: &wgpu::Device) -> TerrainChanges {
        let mut loaded_chunks: Vec<Point3<i32>> = Vec::new();
        let mut unloaded_chunks: Vec<Point3<i32>> = Vec::new();
        let mut modified_chunks: Vec<Point3<i32>> = Vec::new();

        if player_pos != self.player_chunk ||
            (self.chunk_map.is_empty() && self.load_todo.is_empty() && self.loading.is_empty()) {
            self.load_chunks(player_pos);
            self.unload_chunks(player_pos);
            self.player_chunk = player_pos;
        }

        for chunk in self.load_todo.drain(..) {
            let tchunk = chunk.clone();
            let loading_tx = self.loading_tx.clone();
            self.thread_pool.spawn(move || {
                let _ = loading_tx.send((tchunk, gen_chunk(tchunk)));
            });
            self.loading.push(chunk);
        }

        let to_add: Vec<(Point3<i32>, Chunk)> = self.loading_rx.try_iter().collect();
        for (pos, chunk) in to_add {
            self.add_chunk(pos, chunk);
            self.loading.retain(|c| *c != pos);
            loaded_chunks.push(pos);
        }

        for _ in 0..10 {
            if let Some(chunk) = self.unload_todo.pop() {
                self.remove_chunk(chunk);
                unloaded_chunks.push(chunk);
            }
        }

        TerrainChanges {
            loaded_chunks,
            unloaded_chunks,
            modified_chunks,
        }
    }
}

pub struct TerrainMesh {
    thread_pool: ThreadPool,
    player_chunk: Point3<i32>,
    meshed_chunks: HashMap<Point3<i32>, Mesh>,
    meshing_tx: mpsc::Sender<(Point3<i32>, CMesh)>,
    meshing_rx: mpsc::Receiver<(Point3<i32>, CMesh)>,
    meshes_todo: VecDeque<Point3<i32>>,
}

impl TerrainMesh {
    pub fn new() -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        let player_chunk = point![0, 0, 0];
        let meshed_chunks: HashMap<Point3<i32>, Mesh> = HashMap::new();
        let (meshing_tx, meshing_rx) = mpsc::channel();
        let meshes_todo: VecDeque<Point3<i32>> = VecDeque::new();

        Self {
            thread_pool,
            player_chunk,
            meshed_chunks,
            meshing_tx,
            meshing_rx,
            meshes_todo,
        }
    }

    pub fn insert_chunk(&mut self, chunk_pos: Point3<i32>, terrain_data: &Terrain) {
        if terrain_data.check_neighbors(chunk_pos) {
            if (chunk_pos.x - self.player_chunk.x).abs() <= RENDER_DISTANCE
            && (chunk_pos.y - self.player_chunk.y).abs() <= RENDER_DISTANCE
            && (chunk_pos.z - self.player_chunk.z).abs() <= RENDER_DISTANCE {
                self.meshes_todo.push_back(chunk_pos);
            }
        }
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let n_pos = point![
                        chunk_pos.x + x,
                        chunk_pos.y + y,
                        chunk_pos.z + z,
                    ];
                    match terrain_data.chunk_map.get(&n_pos) {
                        Some(_) => {
                            if terrain_data.check_neighbors(n_pos) {
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

    pub fn remove_chunk(&mut self, chunk_pos: Point3<i32>) {
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

    pub fn update(&mut self, terrain_changes: &TerrainChanges, terrain_data: &Terrain, player_pos: Point3<i32>, device: &wgpu::Device) {
        self.player_chunk = player_pos;

        for chunk in &terrain_changes.loaded_chunks {
            self.insert_chunk(*chunk, terrain_data);
        }

        for chunk in &terrain_changes.unloaded_chunks {
            self.remove_chunk(*chunk);
        }

        'workers: for _ in 0..10 {
            if let Some(chunk) = self.meshes_todo.pop_front() {
                let tchunk = chunk.clone();
                let chunk_data = (*terrain_data.chunk_map.get(&chunk).unwrap()).clone();
                let mut neighbor_chunks = Vec::new();
                for z in -1..=1 {
                    for y in -1..=1 {
                        for x in -1..=1 {
                            match terrain_data.chunk_map.get(&point![
                                chunk.x+x, chunk.y+y, chunk.z+z
                            ]) {
                                Some(chunk) => neighbor_chunks.push((*chunk).clone()),
                                None => {
                                    self.meshes_todo.push_back(chunk);
                                    continue 'workers;
                                },
                            }
                        }
                    }
                }
                let meshing_tx = self.meshing_tx.clone();
                
                self.thread_pool.spawn(move || {
                    let _ = meshing_tx.send((tchunk, mesh_chunk(tchunk, chunk_data, &neighbor_chunks[..])));
                });
            }
        }

        let completed_meshes: Vec<(Point3<i32>, CMesh)> = self.meshing_rx.try_iter().collect();
        for (chunk, mesh) in completed_meshes {
            if terrain_data.chunk_map.contains_key(&chunk) {
                self.meshed_chunks.insert(chunk, Mesh::new(device, &mesh));
            }
        }
    }
}
