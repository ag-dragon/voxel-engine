use crate::input::InputState;
use nalgebra::{Vector3, Point3, Matrix4};
use winit::event::*;
use std::time::Duration;
use std::f32::consts::FRAC_PI_2;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: f32,
    pitch: f32,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(position: Point3<f32>, yaw: f32, pitch: f32,
        aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let look = Vector3::new(
            cos_pitch * cos_yaw,
            sin_pitch,
            cos_pitch * sin_yaw,
        );

        Matrix4::look_at_rh(
            &self.position,
            &(self.position + look.normalize()),
            //&Point3::from_homogenous(Vector4::new(look.x, look.y, look.z, look.nor)),
            &Vector3::new(0.0, 1.0, 0.0),
        )
    }

    pub fn proj_matrix(&self) -> Matrix4<f32> {
        let proj = Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj
    }
}

#[derive(Debug)]
pub struct CameraController {
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration, input: &InputState) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        let mut forward_back = 0.0;
        if input.keys[VirtualKeyCode::W as usize] {
            forward_back += 1.0;
        }
        if input.keys[VirtualKeyCode::S as usize] {
            forward_back -= 1.0;
        }
        camera.position += forward * forward_back * self.speed * dt;

        let mut right_left = 0.0;
        if input.keys[VirtualKeyCode::D as usize] {
            right_left += 1.0;
        }
        if input.keys[VirtualKeyCode::A as usize] {
            right_left -= 1.0;
        }
        camera.position += right * right_left * self.speed * dt;

        let mut up_down = 0.0;
        if input.keys[VirtualKeyCode::Space as usize] {
            up_down += 1.0;
        }
        if input.keys[VirtualKeyCode::LShift as usize]  {
            up_down -= 1.0;
        }
        camera.position.y += up_down * self.speed * dt;

        // Rotate
        camera.yaw += f32::to_radians(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += f32::to_radians(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}
