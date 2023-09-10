mod gpu_state;
mod renderer;
mod texture;
mod mesh;
mod input;
mod camera;
mod player;
mod chunk;
mod block;
mod terrain;
use gpu_state::GpuState;
use crate::terrain::TerrainChanges;
use crate::block::BlockType;
use crate::chunk::CHUNK_SIZE;

use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
    dpi::{PhysicalPosition, LogicalSize},
};
use nalgebra::{Point3, point};
use std::collections::HashMap;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(LogicalSize {
            width: 1600,
            height: 900,
        })
        .build(&event_loop).unwrap();

    let gpu = futures::executor::block_on(GpuState::new(window));

    let renderer = renderer::Renderer::new(&gpu);

    let mut input = input::InputState::new();
    let mut camera = camera::Camera::new(
        Point3::new(0.0, 16.0, 4.0), f32::to_radians(-90.0), f32::to_radians(-20.0),
        gpu.config.width as f32 / gpu.config.height as f32,
        f32::to_radians(90.0), 0.1, 1000.0);
    let mut player = player::Player::new(Point3::new(0.0, 64.0, 0.0), 10.0, 60.0);

    let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();
    let mut terrain = terrain::Terrain::new();
    let mut terrain_mesh = terrain::TerrainMesh::new();

    let mut last_render_time = std::time::Instant::now();
    let mut mouse_position = PhysicalPosition::new(-1.0, -1.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == gpu.window.id() => {
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
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state,
                                ..
                            },
                        ..
                    } => input.update_key(*key, *state),
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        if mouse_position.x >= 0.0 && mouse_position.y >= 0.0
                            && !((position.x - mouse_position.x).abs() > 20.0
                            || (position.y - mouse_position.y).abs() > 20.0) {
                            input.update_mouse(
                                position.x - mouse_position.x,
                                position.y - mouse_position.y,
                            );
                        }

                        mouse_position = *position;
                    },
                    WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => input.update_mouse_button(*button, *state),
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == gpu.window.id() => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                let terrain_changes = player.update(&mut camera, dt, &input, &terrain);

                let terrain_changes = terrain.update(player.chunk_position, terrain_changes, &gpu.device, &thread_pool);
                terrain_mesh.update(&terrain_changes, &terrain, player.chunk_position, &gpu.device, &thread_pool);

                input.update_mouse(0.0, 0.0); // Mouse needs to get reset at end of frame
                
                match renderer.render(&gpu, &camera, &terrain_mesh.get_meshes()[..]) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                gpu.window.request_redraw();
            }
            _ => (),
        }
    });
}
