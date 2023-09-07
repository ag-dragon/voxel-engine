use crate::chunk::{Chunk, CHUNK_SIZE, BlockType, BlockFace};
use crate::mesh::{Mesh, CMesh, MeshVertex};
use nalgebra::{Point3, point};
use rayon::ThreadPool;
use noise::{NoiseFn, Perlin, Seedable};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, Arc},
};


// TEMP
use std::time::{Duration, SystemTime};

const RENDER_DISTANCE: i32 = 8;

// function used by worker threads
pub fn gen_chunk(chunk_pos: Point3<i32>) -> Chunk {
    let perlin = Perlin::new(1324);
    let mut chunk = Chunk::new();

    let mut blocks: [BlockType; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]
        = [BlockType::Air; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];
    for i in 0..blocks.len() {
        let x = i % CHUNK_SIZE;
        let y = (i / CHUNK_SIZE) % CHUNK_SIZE;
        let z = i / (CHUNK_SIZE*CHUNK_SIZE);
        let terrain_height = ((perlin.get([
            ((chunk_pos.x * CHUNK_SIZE as i32) + x as i32) as f64 / 80.0,
            ((chunk_pos.z * CHUNK_SIZE as i32) + z as i32) as f64 / 80.0,
        ]) + 1.0) / 2.0) * 32.0;
        let block_height = ((chunk_pos.y * CHUNK_SIZE as i32) + y as i32) as f64;
        if terrain_height > block_height {
            if block_height < 8.0 {
                blocks[i] = BlockType::Sand;
            } else if (terrain_height - block_height).abs() <= 1.0 {
                blocks[i] = BlockType::Grass;
            } else {
                blocks[i] = BlockType::Dirt;
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

pub struct Terrain {
    generated: bool,
    thread_pool: ThreadPool,
    player_chunk: Point3<i32>,
    chunk_map: HashMap<Point3<i32>, Chunk>,
    load_todo: Vec<Point3<i32>>,
    loading: Vec<Point3<i32>>,
    loaded_chunks: Arc<Mutex<Vec<(Point3<i32>, Chunk)>>>,
    unload_todo: Vec<Point3<i32>>,
    meshed_chunks: HashMap<Point3<i32>, Mesh>,
    meshes_todo: VecDeque<Point3<i32>>,
    meshes_completed: Arc<Mutex<Vec<(Point3<i32>, CMesh)>>>,
}

impl Terrain {
    pub fn new() -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        let player_chunk = point![0, 0, 0];
        let chunk_map: HashMap<Point3<i32>, Chunk> = HashMap::new();
        let load_todo: Vec<Point3<i32>> = Vec::new();
        let loading: Vec<Point3<i32>> = Vec::new();
        let loaded_chunks: Arc<Mutex<Vec<(Point3<i32>, Chunk)>>>
            = Arc::new(Mutex::new(Vec::new()));
        let unload_todo: Vec<Point3<i32>> = Vec::new();
        let meshed_chunks: HashMap<Point3<i32>, Mesh> = HashMap::new();
        let meshes_todo: VecDeque<Point3<i32>> = VecDeque::new();
        let meshes_completed: Arc<Mutex<Vec<(Point3<i32>, CMesh)>>>
            = Arc::new(Mutex::new(Vec::new()));

        Self {
            generated: false,
            thread_pool,
            player_chunk,
            chunk_map,
            load_todo,
            loading,
            loaded_chunks,
            unload_todo,
            meshed_chunks,
            meshes_todo,
            meshes_completed,
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
        if let Some(old_chunk) = self.chunk_map.insert(chunk_pos, chunk) {
            // we just overwrote another chunk, no reason this should be able to happen currently
            eprintln!["uh oh, a chunk was overwritten by another"];
        }

        if self.check_neighbors(chunk_pos) {
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
                    match self.chunk_map.get(&n_pos) {
                        Some(_) => {
                            if self.check_neighbors(n_pos) {
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

    // unload chunk
    pub fn remove_chunk(&mut self, chunk_pos: Point3<i32>) {
        self.chunk_map.remove(&chunk_pos);
        self.meshed_chunks.remove(&chunk_pos);
        self.load_todo.retain(|chunk| *chunk != chunk_pos);
        self.meshes_todo.retain(|chunk| *chunk != chunk_pos);
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
                    } /*else if !self.meshed_chunks.contains_key(&cpos)
                    && !self.load_todo.contains(&cpos)
                    && !self.loading.contains(&cpos) {
                        if (cpos.x - self.player_chunk.x).abs() <= RENDER_DISTANCE
                        && (cpos.y - self.player_chunk.y).abs() <= RENDER_DISTANCE
                        && (cpos.z - self.player_chunk.z).abs() <= RENDER_DISTANCE {
                            self.meshes_todo.push_back(cpos);
                        }
                    }*/
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
    pub fn update(&mut self, player_pos: Point3<i32>, device: &wgpu::Device) {
        if player_pos != self.player_chunk ||
            (self.chunk_map.is_empty() && self.load_todo.is_empty() && self.loading.is_empty()) {
            self.load_chunks(player_pos);
            self.unload_chunks(player_pos);
            self.player_chunk = player_pos;
        }

        for chunk in self.load_todo.drain(..) {
            let tchunk = chunk.clone();
            let loaded_chunks = Arc::clone(&self.loaded_chunks);
            self.thread_pool.spawn(move || {
                let output_chunk = gen_chunk(tchunk);
                loaded_chunks.lock().unwrap().push((tchunk, output_chunk));
            });
            self.loading.push(chunk);
        }

        let mut lc = self.loaded_chunks.lock().unwrap();
        let mut to_add = Vec::new();
        for (pos, chunk) in lc.drain(..) {
            to_add.push((pos, chunk));
        }
        drop(lc);
        for (pos, chunk) in to_add.drain(..) {
            self.add_chunk(pos, chunk);
            self.loading.retain(|c| *c != pos);
        }

        for _ in 0..10 {
            if let Some(chunk) = self.unload_todo.pop() {
                self.remove_chunk(chunk);
            }
        }

        //let mut assigned_meshes: Vec<Point3<i32>> = Vec::new();
        'workers: for _ in 0..10 {
            if let Some(chunk) = self.meshes_todo.pop_front() {
            //'workers: for chunk in &self.meshes_todo {
                    let tchunk = chunk.clone();
                    let chunk_data = (*self.chunk_map.get(&chunk).unwrap()).clone();
                    let mut neighbor_chunks = Vec::new();
                    for z in -1..=1 {
                        for y in -1..=1 {
                            for x in -1..=1 {
                                match self.chunk_map.get(&point![
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
                    let completed_meshes = Arc::clone(&self.meshes_completed);
                    
                    self.thread_pool.spawn(move || {
                        let output_mesh = mesh_chunk(tchunk, chunk_data, &neighbor_chunks[..]);
                        completed_meshes.lock().unwrap().push((tchunk, output_mesh));
                    });
                //assigned_meshes.push(*chunk);
            }
            //self.meshes_todo.retain(|chunk| !assigned_meshes.contains(chunk));
        }

        let mut cm = self.meshes_completed.lock().unwrap();
        for (chunk, mesh) in cm.drain(..) {
            if self.chunk_map.contains_key(&chunk) {
                self.meshed_chunks.insert(chunk, Mesh::new(device, &mesh));
            }
        }
    }

    pub fn get_meshes(&self) -> Vec<&Mesh> {
        let mut render_meshes = Vec::new();
        for (_, mesh) in &self.meshed_chunks {
            render_meshes.push(mesh);
        }
        render_meshes
    }
}
