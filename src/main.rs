use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::{
    time::{Instant, Duration},
    thread,
    sync::mpsc
};

use log::LevelFilter;

mod utils;
use utils::appState::State;
use utils::logger::ConsoleLogger;

fn main() {

    ConsoleLogger::init().unwrap();

    log::set_max_level(LevelFilter::Trace);

    log::error!("error message");
    log::warn!("warn message");
    log::info!("info message");
    log::debug!("debug message");
    log::trace!("trace message");

    let wanted_fps: u64 = 30;

    let (tx, rx): (_, mpsc::Receiver<bool>) = mpsc::channel();
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
    let mut draw_requested = false;

    let mut previous_time = Instant::now();
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
                // window.request_redraw();
                if !draw_requested {
                    let current_time = Instant::now();
                    let elapsed_time = current_time - previous_time;
                    let fps_duration = Duration::from_millis((1.0/(wanted_fps as f32) * 1000.0) as _);
                    let sleeping_time = if elapsed_time < fps_duration {fps_duration - elapsed_time} else {Duration::from_secs(0)};
                    let tx_clone = tx.clone();
                    thread::spawn(move || {
                        thread::sleep(sleeping_time);
                        tx_clone.send(true).unwrap();
                    });
                    previous_time = current_time;
                    draw_requested = true;

                }else {
                    thread::sleep(Duration::from_millis(1)); // avoid using cpu 100%
                    if let Ok(_) = rx.try_recv() {
                        draw_requested = false;
                        window.request_redraw();
                    }
                }
                
                // println!("loop"); 
            }
            _ => {}
        }
    });
}