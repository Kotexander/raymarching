use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
    let mut ray_marcher = pollster::block_on(RayMarcher::new(window, size.into(), 1.0 / 1.0));

    let mut dm = (0.0, 0.0);

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
                WindowEvent::MouseInput { state, button, .. } => {
                    if let MouseButton::Left = button {
                        let state = ElementState::Pressed == *state;
                        ray_marcher.controller.looking = state;
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    ray_marcher.resize((*physical_size).into());
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    ray_marcher.resize((**new_inner_size).into())
                }
                WindowEvent::Focused(false) => {
                    ray_marcher.controller = raymarcher::Controller::default();
                }
                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                dm.0 += delta.0;
                dm.1 += delta.1;
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if ray_marcher.controller.looking {
                    let sensativity = 0.005;
                    let v = ray_marcher.camera.rot.transform_vector(
                        &(raymarcher::na::Vector3::new(dm.1 as f32, dm.0 as f32, 0.0)
                            * sensativity),
                    );
                    ray_marcher.camera.rot = ray_marcher.camera.rot.append_axisangle_linearized(&v);
                    // log::info!("{}", ray_marcher.camera.rot.angle());
                }
                dm.0 = 0.0;
                dm.1 = 0.0;
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
