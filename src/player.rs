use crate::input::InputState;
use crate::camera::Camera;
use winit::event::VirtualKeyCode;
use nalgebra::{Vector3, Point3};
use std::time::Duration;
use std::f32::consts::FRAC_PI_2;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Player {
    pub position: Point3<f32>,
    speed: f32,
    sensitivity: f32,
}

impl Player {
    pub fn new(position: Point3<f32>, speed: f32, sensitivity: f32) -> Self {
        Self {
            position,
            speed,
            sensitivity,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: Duration, input: &InputState) {
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
    }
}
