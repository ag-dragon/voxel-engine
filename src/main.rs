mod gpu_state;
mod renderer;
mod texture;
mod mesh;
mod input;
mod camera;
mod player;
mod chunk;
pub use renderer::Vertex; // Deleting this breaks MeshVertex trait implementation. No clue why
use gpu_state::GpuState;
use chunk::Chunk;
use mesh::Mesh;

use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
    dpi::{PhysicalPosition, LogicalSize},
};
use nalgebra::{Point3, point};
use noise::{NoiseFn, Perlin, Seedable};
use std::{
    collections::HashMap,
    sync::{Mutex, Arc},
};

const RENDER_DISTANCE: i32 = 8;

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
    let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();

    let mut input = input::InputState::new();
    let mut camera = camera::Camera::new(
        Point3::new(0.0, 16.0, 4.0), f32::to_radians(-90.0), f32::to_radians(-20.0),
        gpu.config.width as f32 / gpu.config.height as f32,
        f32::to_radians(90.0), 0.1, 1000.0);
    let mut player = player::Player::new(Point3::new(0.0, 16.0, 4.0), 20.0, 60.0);

    let mut chunk_map: Arc<Mutex<HashMap<Point3<i32>, (Chunk, Option<Mesh>)>>> = Arc::new(Mutex::new(HashMap::new()));
    let height_map = Perlin::new(2);

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
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == gpu.window.id() => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                player.update(&mut camera, dt, &input);

                let player_chunk_pos = point![
                    f32::floor(player.position[0] / chunk::CHUNK_SIZE as f32) as i32,
                    f32::floor(player.position[1] / chunk::CHUNK_SIZE as f32) as i32,
                    f32::floor(player.position[2] / chunk::CHUNK_SIZE as f32) as i32,
                ];

                Chunk::unload_chunks(&mut chunk_map, player_chunk_pos, RENDER_DISTANCE);
                Chunk::load_chunks(&mut chunk_map, player_chunk_pos, RENDER_DISTANCE, &height_map);
                Chunk::setup_chunks(&mut chunk_map, player_chunk_pos, RENDER_DISTANCE, &gpu);

                input.update_mouse(0.0, 0.0); // Mouse needs to get reset at end of frame

                let mut chunk_meshes = Vec::new();
                let mut cm = chunk_map.lock().unwrap();
                for (_, is_mesh) in cm.values() {
                    match is_mesh {
                        Some(mesh) => chunk_meshes.push(mesh),
                        None => {},
                    };
                }
                match renderer.render(&gpu, &camera, &chunk_meshes[..]) {
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
