use nalgebra::{Vector3, Point3, Matrix4};

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>, yaw: f32, pitch: f32,
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
            &Point3::from(self.position),
            &Point3::from(self.position + look.normalize()),
            &Vector3::new(0.0, 1.0, 0.0),
        )
    }

    pub fn proj_matrix(&self) -> Matrix4<f32> {
        let proj = Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj
    }
}
