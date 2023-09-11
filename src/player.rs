use crate::input::InputState;
use crate::camera::Camera;
use crate::terrain::{Terrain, TerrainChanges};
use crate::chunk;
use crate::block::BlockType;
use winit::event::VirtualKeyCode;
use winit::event;
use nalgebra::{Vector3, vector};
use std::time::Duration;
use std::f32::consts::FRAC_PI_2;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Player {
    pub position: Vector3<f32>,
    pub chunk_position: Vector3<i32>,
    speed: f32,
    sensitivity: f32,
    mouse_p: bool,
}

impl Player {
    pub fn new(position: Vector3<f32>, speed: f32, sensitivity: f32) -> Self {
        let chunk_position = vector![
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

        self.chunk_position = vector![
            f32::floor((self.position[0]-0.5) / chunk::CHUNK_SIZE as f32) as i32,
            f32::floor((self.position[1]-0.5) / chunk::CHUNK_SIZE as f32) as i32,
            f32::floor((self.position[2]-0.5) / chunk::CHUNK_SIZE as f32) as i32,
        ];

        let mut terrain_changes = TerrainChanges::new();
        if input.mouse_pressed(event::MouseButton::Left) {
            if !self.mouse_p {
                self.mouse_p = true;
                let dir = Vector3::new(
                    camera.yaw.cos()*camera.pitch.cos(),
                    camera.pitch.sin(),
                    camera.yaw.sin()*camera.pitch.cos(),
                ).normalize();
                for t in 0..50 {
                    let block_world_pos = vector![
                        (dir.x * (t as f32 / 10.0) + self.position.x).round() as i32,
                        (dir.y * (t as f32 / 10.0) + self.position.y).round() as i32,
                        (dir.z * (t as f32 / 10.0) + self.position.z).round() as i32,
                    ];
                    let c_pos = vector![
                        f32::floor(block_world_pos.x as f32 / chunk::CHUNK_SIZE as f32) as i32,
                        f32::floor(block_world_pos.y as f32 / chunk::CHUNK_SIZE as f32) as i32,
                        f32::floor(block_world_pos.z as f32 / chunk::CHUNK_SIZE as f32) as i32,
                    ];
                    if let Some((block_pos, block)) = terrain.get_block(block_world_pos) {
                        if block != BlockType::Air {
                            terrain_changes.modified_chunks.entry(c_pos)
                                .and_modify(|blocks| blocks.push((block_pos, BlockType::Air)))
                                .or_insert([(block_pos, BlockType::Air)].to_vec());
                            break;
                        }
                    }
                }
            }
        } else {
            self.mouse_p = false;
        }
        terrain_changes
    }
}
