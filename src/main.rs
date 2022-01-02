use winit::{
    event::*,
    
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::Instant;

mod utils;
use utils::state::State;


fn main() {

    env_logger::init();
    let event_loop = EventLoop::new();
    // let event_loop = winit::event_loop::EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("rust NCA")
        .build(&event_loop).unwrap();


    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window, &event_loop, [400, 400]));
    
    let start_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        state.input(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    
                    WindowEvent::Resized(physical_size) => { state.resize(*physical_size); }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render(&window) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}