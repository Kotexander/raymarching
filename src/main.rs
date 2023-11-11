use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod raymarcher;
use raymarcher::RayMarcher;

const Q: u32 = 16;
const E: u32 = 18;
const W: u32 = 17;
const A: u32 = 30;
const S: u32 = 31;
const D: u32 = 32;

fn main() {
    env_logger::init();

    let mut dt = 0.0;
    let mut timer = std::time::Instant::now();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Ray Marcher")
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();
    let mut ray_marcher = pollster::block_on(RayMarcher::new(window, size.into()));

    event_loop.run(move |event, _, control_flow| {
        let window = &ray_marcher.wgpu_ctx.window;
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
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
                WindowEvent::KeyboardInput { input, .. } => {
                    let state = ElementState::Pressed == input.state;
                    match input.scancode {
                        W => {
                            ray_marcher.controller.forwards = state;
                        }
                        A => {
                            ray_marcher.controller.left = state;
                        }
                        S => {
                            ray_marcher.controller.backwards = state;
                        }
                        D => {
                            ray_marcher.controller.right = state;
                        }
                        Q => {
                            ray_marcher.controller.up = state;
                        }
                        E => {
                            ray_marcher.controller.down = state;
                        }
                        _ => {}
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    ray_marcher.wgpu_ctx.resize((*physical_size).into());
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    ray_marcher.wgpu_ctx.resize((**new_inner_size).into());
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                ray_marcher.update(dt);
                match ray_marcher.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => ray_marcher.wgpu_ctx.reconfigure_surface(),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => log::error!("{:?}", e),
                }

                dt = timer.elapsed().as_secs_f32();
                timer = std::time::Instant::now();
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        };
    });
}
