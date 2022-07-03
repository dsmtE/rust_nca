use nalgebra_glm as glm;
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent},
};

pub type KeyMap = std::collections::HashMap<VirtualKeyCode, ElementState>;

#[derive(Default)]
pub struct InputsState {
    pub keystates: KeyMap,
    pub mouse: MouseState,
}

pub trait WinitEventHandler {
    fn handle_event<T>(&mut self, event: &Event<T>);
}

impl InputsState {
    pub fn is_key_pressed(&self, keycode: &VirtualKeyCode) -> bool {
        self.keystates.contains_key(keycode) && self.keystates[keycode] == ElementState::Pressed
    }
}

impl WinitEventHandler for InputsState {
    fn handle_event<T>(&mut self, event: &Event<T>) {
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    virtual_keycode: Some(keycode), state, ..
                },
                ..
            } = *event
            {
                *self.keystates.entry(keycode).or_insert(state) = state;
            }
        }

        self.mouse.handle_event(event);
    }
}

#[derive(Default)]
pub struct MouseState {
    pub is_left_clicked: bool,
    pub is_right_clicked: bool,
    pub is_middle_clicked: bool,
    pub position: glm::Vec2,
    pub position_delta: glm::Vec2,
    pub wheel_delta: glm::Vec2,
    pub moved: bool,
    pub scrolled: bool,
}

impl WinitEventHandler for MouseState {
    fn handle_event<T>(&mut self, event: &Event<T>) {
        match event {
            Event::NewEvents { .. } => {
                if !self.scrolled {
                    self.wheel_delta = glm::vec2(0.0, 0.0);
                }
                self.scrolled = false;

                if !self.moved {
                    self.position_delta = glm::vec2(0.0, 0.0);
                }
                self.moved = false;
            },
            Event::WindowEvent { event, .. } => match *event {
                WindowEvent::MouseInput { button, state, .. } => {
                    let clicked = state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => self.is_left_clicked = clicked,
                        MouseButton::Right => self.is_right_clicked = clicked,
                        MouseButton::Middle => self.is_middle_clicked = clicked,
                        _ => {},
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    let last_position = self.position;
                    let current_position = glm::vec2(position.x as _, position.y as _);
                    self.position = current_position;
                    self.position_delta = current_position - last_position;
                    self.moved = true;
                },
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(h_lines, v_lines),
                    ..
                } => {
                    self.wheel_delta = glm::vec2(h_lines, v_lines);
                    self.scrolled = true;
                },
                _ => {},
            },
            _ => {},
        }
    }
}

pub struct SystemState {
    pub window_dimensions: PhysicalSize<u32>,
    pub delta_time: f64,
    last_frame: Instant,
    pub exit_requested: bool,
}

impl SystemState {
    pub fn new(window_dimensions: PhysicalSize<u32>) -> Self {
        Self {
            last_frame: Instant::now(),
            window_dimensions,
            delta_time: 0.00,
            exit_requested: false,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        let width = self.window_dimensions.width;
        let height = std::cmp::max(self.window_dimensions.height as u32, 0);
        width as f32 / height as f32
    }

    pub fn window_center(&self) -> glm::Vec2 { glm::vec2(self.window_dimensions.width as f32 / 2.0, self.window_dimensions.height as f32 / 2.0) }
}

impl WinitEventHandler for SystemState {
    fn handle_event<T>(&mut self, event: &Event<T>) {
        match event {
            Event::NewEvents { .. } => {
                self.delta_time = self.last_frame.elapsed().as_secs_f64();
                self.last_frame = Instant::now();
            },
            Event::WindowEvent { event, .. } => match *event {
                WindowEvent::CloseRequested => self.exit_requested = true,
                WindowEvent::Resized(dimensions) => {
                    self.window_dimensions = dimensions;
                },
                _ => {},
            },
            _ => {},
        }
    }
}
