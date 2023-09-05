mod gpu_state;
mod renderer;
mod texture;
mod mesh;
mod input;
mod camera;
mod chunk;
pub use renderer::Vertex; // Deleting this breaks MeshVertex trait implementation. No clue why
use gpu_state::GpuState;
use chunk::Chunk;

use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
    dpi::{PhysicalPosition, LogicalSize},
};
use nalgebra::{Point3, point};

const RENDER_DISTANCE: i32 = 4;

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
    let mut camera_controller = camera::CameraController::new(4.0, 60.0);

    let mut chunks = Vec::new();
    for x in -RENDER_DISTANCE..RENDER_DISTANCE {
        for z in -RENDER_DISTANCE..RENDER_DISTANCE {
            chunks.push(Chunk::new(point![x, 0, z]));
        }
    }
    let mut chunk_meshes = Vec::new();
    for chunk in &chunks {
        chunk_meshes.push(chunk.gen_mesh(&gpu));
    }

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
                    } => input.update(*key, *state),
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        if mouse_position.x >= 0.0 && mouse_position.y >= 0.0
                            && !((position.x - mouse_position.x).abs() > 20.0
                            || (position.y - mouse_position.y).abs() > 20.0) {
                            camera_controller.process_mouse(
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
                camera_controller.update_camera(&mut camera, dt, &input);

                let c_pos = point![
                    f32::floor(camera.position[0] / chunk::CHUNK_SIZE as f32) as i32,
                    f32::floor(camera.position[1] / chunk::CHUNK_SIZE as f32) as i32,
                    f32::floor(camera.position[2] / chunk::CHUNK_SIZE as f32) as i32,
                ];
                for i in 0..chunks.len() {
                    if (c_pos[0] - chunks[i].position[0]).abs() > RENDER_DISTANCE {
                        let chunk = Chunk::new(point![
                            chunks[i].position[0] + (c_pos[0] - chunks[i].position[0]).signum()*RENDER_DISTANCE*2,
                            chunks[i].position[1],
                            chunks[i].position[2]
                        ]);
                        chunk_meshes.push(chunk.gen_mesh(&gpu));
                        chunks.push(chunk);
                        chunks.remove(i);
                        chunk_meshes.remove(i);
                    } else if (c_pos[2] - chunks[i].position[2]).abs() > RENDER_DISTANCE {
                        let chunk = Chunk::new(point![
                            chunks[i].position[0],
                            chunks[i].position[1],
                            chunks[i].position[2] + (c_pos[2] - chunks[i].position[2]).signum()*RENDER_DISTANCE*2
                        ]);
                        chunk_meshes.push(chunk.gen_mesh(&gpu));
                        chunks.push(chunk);
                        chunks.remove(i);
                        chunk_meshes.remove(i);
                    }
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
