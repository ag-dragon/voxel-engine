use crate::gpu_state;
use crate::mesh;
use crate::texture;
use crate::camera;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    view_proj: [[f32; 4]; 4],
}

pub struct Renderer {
    depth_texture: texture::Texture,
    texture_atlas_bind_group: wgpu::BindGroup,
    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(gpu: &gpu_state::GpuState) -> Self {
        let depth_texture = texture::Texture::create_depth_texture(gpu, "depth_texture");

        let atlas_bytes = include_bytes!("block-atlas.png");
        let texture_atlas = texture::Texture::from_bytes(gpu, atlas_bytes, "block-atlas.png").unwrap();

        let texture_atlas_bind_group_layout = gpu.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                label: Some("texture_atlas_bind_group_layout"),
            }
        );

        let texture_atlas_bind_group = gpu.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_atlas_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_atlas.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_atlas.sampler),
                    },
                ],
                label: Some("texture_atlas_bind_group"),
            }
        );

        let global_uniform_size = std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;
        let global_uniform_buffer = gpu.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("global_uniform_buffer"),
                size: global_uniform_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let global_uniform_bind_group_layout = gpu.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                    },
                ],
                label: Some("global_uniform_bind_group_layout"),
            }
        );

        let global_uniform_bind_group = gpu.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &global_uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: global_uniform_buffer.as_entire_binding(),
                    },
                ],
                label: Some("global_uniform_bind_group"),
            }
        );

        let shader = gpu.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = gpu.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[
                    &texture_atlas_bind_group_layout,
                    &global_uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = gpu.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("render_pipeline"),
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
            }
        );

        Self {
            depth_texture,
            texture_atlas_bind_group,
            global_uniform_buffer,
            global_uniform_bind_group,
            render_pipeline,
        }
    }

    pub fn render(&self, gpu: &gpu_state::GpuState, camera: &camera::Camera, meshes: &[mesh::Mesh]) -> Result<(), wgpu::SurfaceError> {
        {
            let global_uniforms = GlobalUniforms {
                view_proj: (camera.proj_matrix() * camera.view_matrix()).into(),
            };
            gpu.queue.write_buffer(&self.global_uniform_buffer, 0, bytemuck::bytes_of(&global_uniforms));
        }

        let output = gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
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
                }
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_atlas_bind_group, &[]);
            render_pass.set_bind_group(1, &self.global_uniform_bind_group, &[]);

            for mesh in meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
