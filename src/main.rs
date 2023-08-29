mod gpu_state;
mod texture;
mod mesh;
use gpu_state::GpuState;
use mesh::{MeshVertex, Mesh};

use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};
use wgpu::util::DeviceExt;
use nalgebra::{Vector3, Point3, Matrix4, base::Unit};

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

const VERTICES: &[MeshVertex] = &[
    MeshVertex { position: [-1.0, 1.0, 1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], }, // front face
    MeshVertex { position: [-1.0, 1.0, 1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], }, // top face
    MeshVertex { position: [-1.0, 1.0, 1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], }, // left face
    MeshVertex { position: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], }, // front face
    MeshVertex { position: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], }, // top face
    MeshVertex { position: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], }, // right face
    MeshVertex { position: [-1.0, -1.0, 1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], }, // front face
    MeshVertex { position: [-1.0, -1.0, 1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], }, // bottom face
    MeshVertex { position: [-1.0, -1.0, 1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], }, // left face
    MeshVertex { position: [1.0, -1.0, 1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], }, // front face
    MeshVertex { position: [1.0, -1.0, 1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], }, // bottom face
    MeshVertex { position: [1.0, -1.0, 1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], }, // right face
    MeshVertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], }, // back face
    MeshVertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], }, // top face
    MeshVertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], }, // left face
    MeshVertex { position: [1.0, 1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], }, // back face
    MeshVertex { position: [1.0, 1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], }, // top face
    MeshVertex { position: [1.0, 1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], }, // right face
    MeshVertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 0.0], }, // back face
    MeshVertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], }, // bottom face
    MeshVertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 0.0], }, // left face
    MeshVertex { position: [1.0, -1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 0.0], }, // back face
    MeshVertex { position: [1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], }, // bottom face
    MeshVertex { position: [1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 0.0], }, // right face
];

const INDICES: &[u16] = &[
    9, 3, 6, // front 1
    6, 3, 0, // front 2
    1, 16, 13, // top 1
    1, 4, 16, // top 2
    12, 15, 18, // back 1
    18, 15, 21, // back 2
    7, 19, 22, // bottom 1
    7, 22, 10, // bottom 2
    17, 5, 11, // right 1
    17, 11, 23, // right 2
    2, 14, 20, // left 1
    2, 20, 8, // left 2
];

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = Unit::new_normalize(forward);
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm.scale(self.speed);
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm.scale(self.speed);
        }

        let right = forward_norm.cross(&camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.eye = camera.target - Unit::new_normalize(forward + right * self.speed).scale(forward_mag);
        }
        if self.is_left_pressed {
            camera.eye = camera.target - Unit::new_normalize(forward - right * self.speed).scale(forward_mag);
        }
    }
}

struct State {
    gpu: GpuState,
    render_pipeline: wgpu::RenderPipeline,
    test_mesh: Mesh,
    diffuse_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl State {
    async fn new(gpu: GpuState) -> Self {
        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&gpu, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = gpu.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let camera = Camera {
            eye: Point3::new(0.0, 1.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: gpu.config.width as f32 / gpu.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = gpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let camera_controller = CameraController::new(0.2);

        let shader = gpu.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    mesh::MeshVertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let test_mesh = Mesh::new(&gpu, &VERTICES, &INDICES);

        Self {
            gpu,
            render_pipeline,
            test_mesh,
            diffuse_bind_group,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.gpu.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.test_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.test_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.test_mesh.num_elements, 0, 0..1);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_resizable(false).build(&event_loop).unwrap();

    let gpu = futures::executor::block_on(GpuState::new(window));
    let mut state = futures::executor::block_on(State::new(gpu));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.gpu.window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.gpu.window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                state.gpu.window.request_redraw();
            }
            _ => (),
        }
    });
}
