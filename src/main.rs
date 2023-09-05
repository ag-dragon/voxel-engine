mod gpu_state;
mod texture;
mod mesh;
mod camera;
mod chunk;
use gpu_state::GpuState;
use mesh::{MeshVertex, Mesh};
use chunk::Chunk;

use winit::{
    event::{DeviceEvent, Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
    dpi::PhysicalPosition,
};
use wgpu::util::DeviceExt;
use nalgebra::{Vector3, Point3, point, Matrix4, base::Unit};

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

struct State {
    gpu: GpuState,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    mouse_pressed: bool,
}

impl State {
    async fn new(gpu: GpuState) -> Self {
        let diffuse_bytes = include_bytes!("block-atlas.png");
        let diffuse_texture = texture::Texture::from_bytes(&gpu, diffuse_bytes, "block-atlas.png").unwrap();

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

        let camera = camera::Camera::new(
            Point3::new(0.0, 16.0, 4.0), f32::to_radians(-90.0), f32::to_radians(-20.0),
            gpu.config.width as f32 / gpu.config.height as f32,
            f32::to_radians(45.0), 0.1, 1000.0);
        let camera_controller = camera::CameraController::new(4.0, 60.0);

        let camera_uniform: [[f32; 4]; 4] = (camera.proj_matrix() * camera.view_matrix()).into();

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

        let depth_texture = texture::Texture::create_depth_texture(&gpu, "depth_texture");

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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            gpu,
            render_pipeline,
            depth_texture,
            diffuse_bind_group,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            mouse_pressed: false,
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        let camera_uniform: [[f32; 4]; 4] = (self.camera.proj_matrix() * self.camera.view_matrix()).into();
        self.gpu.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    fn render(&mut self, meshes: &[Mesh]) -> Result<(), wgpu::SurfaceError> {
        // chunk mesh generation (temp putting it here for testing)
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            for mesh in meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

const RENDER_DISTANCE: i32 = 4;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_resizable(false).build(&event_loop).unwrap();

    let gpu = futures::executor::block_on(GpuState::new(window));
    let mut state = futures::executor::block_on(State::new(gpu));

    let mut chunks = Vec::new();
    for x in -RENDER_DISTANCE..RENDER_DISTANCE {
        for z in -RENDER_DISTANCE..RENDER_DISTANCE {
            chunks.push(Chunk::new(point![x, 0, z]));
        }
    }
    let mut chunk_meshes = Vec::new();
    for chunk in &chunks {
        chunk_meshes.push(chunk.gen_mesh(&state.gpu));
    }

    let mut last_render_time = std::time::Instant::now();
    let mut mouse_position = PhysicalPosition::new(-1.0, -1.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
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
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        if mouse_position.x >= 0.0 && mouse_position.y >= 0.0
                            && !((position.x - mouse_position.x).abs() > 20.0
                            || (position.y - mouse_position.y).abs() > 20.0) {
                            state.camera_controller.process_mouse(
                                position.x - mouse_position.x,
                                position.y - mouse_position.y,
                            );
                        }

                        mouse_position = *position;
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.gpu.window.id() => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);

                let c_pos = point![
                    f32::floor(state.camera.position[0] / chunk::CHUNK_SIZE as f32) as i32,
                    f32::floor(state.camera.position[1] / chunk::CHUNK_SIZE as f32) as i32,
                    f32::floor(state.camera.position[2] / chunk::CHUNK_SIZE as f32) as i32,
                ];
                for i in 0..chunks.len() {
                    if (c_pos[0] - chunks[i].position[0]).abs() > RENDER_DISTANCE {
                        let chunk = Chunk::new(point![
                            chunks[i].position[0] + (c_pos[0] - chunks[i].position[0]).signum()*RENDER_DISTANCE*2,
                            chunks[i].position[1],
                            chunks[i].position[2]
                        ]);
                        chunk_meshes.push(chunk.gen_mesh(&state.gpu));
                        chunks.push(chunk);
                        chunks.remove(i);
                        chunk_meshes.remove(i);
                    } else if (c_pos[2] - chunks[i].position[2]).abs() > RENDER_DISTANCE {
                        let chunk = Chunk::new(point![
                            chunks[i].position[0],
                            chunks[i].position[1],
                            chunks[i].position[2] + (c_pos[2] - chunks[i].position[2]).signum()*RENDER_DISTANCE*2
                        ]);
                        chunk_meshes.push(chunk.gen_mesh(&state.gpu));
                        chunks.push(chunk);
                        chunks.remove(i);
                        chunk_meshes.remove(i);
                    }
                    /*
                    if (chunk.position - c_pos).norm().abs()
                        > RENDER_DISTANCE {
                        println!("too far");
                    }
                    */
                }

                match state.render(&chunk_meshes[..]) {
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
