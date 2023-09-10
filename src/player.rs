use crate::input::InputState;
use crate::camera::Camera;
use crate::terrain::{Terrain, TerrainChanges};
use crate::chunk;
use crate::block::BlockType;
use winit::event::VirtualKeyCode;
use winit::event;
use nalgebra::{Vector3, Point3, point};
use std::time::Duration;
use std::f32::consts::FRAC_PI_2;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Player {
    pub position: Point3<f32>,
    pub chunk_position: Point3<i32>,
    speed: f32,
    sensitivity: f32,
    mouse_p: bool,
}

impl Player {
    pub fn new(position: Point3<f32>, speed: f32, sensitivity: f32) -> Self {
        let chunk_position = point![
            f32::floor(position[0] / chunk::CHUNK_SIZE as f32) as i32,
            f32::floor(position[1] / chunk::CHUNK_SIZE as f32) as i32,
            f32::floor(position[2] / chunk::CHUNK_SIZE as f32) as i32,
        ];
        Self {
            position,
            chunk_position,
            speed,
            sensitivity,
            mouse_p: false,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: Duration, input: &InputState, terrain: &Terrain) -> TerrainChanges {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        let mut forward_back = 0.0;
        if input.key_pressed(VirtualKeyCode::W) {
            forward_back += 1.0;
        }
        if input.key_pressed(VirtualKeyCode::S) {
            forward_back -= 1.0;
        }
        self.position += forward * forward_back * self.speed * dt;

        let mut right_left = 0.0;
        if input.key_pressed(VirtualKeyCode::D) {
            right_left += 1.0;
        }
        if input.key_pressed(VirtualKeyCode::A) {
            right_left -= 1.0;
        }
        self.position += right * right_left * self.speed * dt;

        let mut up_down = 0.0;
        if input.key_pressed(VirtualKeyCode::Space) {
            up_down += 1.0;
        }
        if input.key_pressed(VirtualKeyCode::LShift)  {
            up_down -= 1.0;
        }
        self.position.y += up_down * self.speed * dt;
        camera.position = self.position;

        // Rotate
        camera.yaw += f32::to_radians(input.mouse_delta.0) * self.sensitivity * dt;
        camera.pitch += f32::to_radians(-input.mouse_delta.1) * self.sensitivity * dt;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }

        self.chunk_position = point![
            f32::floor(self.position[0] / chunk::CHUNK_SIZE as f32) as i32,
            f32::floor(self.position[1] / chunk::CHUNK_SIZE as f32) as i32,
            f32::floor(self.position[2] / chunk::CHUNK_SIZE as f32) as i32,
        ];

        let mut terrain_changes = TerrainChanges::new();
        if input.mouse_pressed(event::MouseButton::Left) {
            if !self.mouse_p {
                self.mouse_p = true;
                if let Some(current_chunk) = terrain.get_chunk(self.chunk_position) {
                    if !current_chunk.is_empty {
                        let mut new_blocks = Vec::new();
                        for x in 0..chunk::CHUNK_SIZE {
                            for y in 0..chunk::CHUNK_SIZE {
                                for z in 0..chunk::CHUNK_SIZE {
                                    new_blocks.push((point![x, y, z], BlockType::Air));
                                }
                            }
                        }
                        terrain_changes.modified_chunks.insert(self.chunk_position, new_blocks);
                    }
                }
                /*
                let dir = Vector3::new(
                    camera.yaw.cos()*camera.pitch.cos(),
                    camera.pitch.sin(),
                    camera.yaw.sin()*camera.pitch.cos(),
                ).normalize();
                match terrain.chunk_map.get_mut(&player_chunk_pos) {
                    Some(pchunk) => {
                        pchunk.set_block(BlockType::Air,
                            (self.position.x % chunk::CHUNK_SIZE as f32) as usize,
                            (self.position.y % chunk::CHUNK_SIZE as f32) as usize,
                            (self.position.z % chunk::CHUNK_SIZE as f32) as usize,
                        );
                        terrain.meshes_todo.push_back(player_chunk_pos);
                    },
                    None => {},
                }
                */
            }
        } else {
            self.mouse_p = false;
        }
        terrain_changes
    }
}
