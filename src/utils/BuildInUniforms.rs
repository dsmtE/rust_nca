
// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct BuildInUniform {
//     mouse_pos: [f32; 2],
//     pixel_size:
// }


// impl BuildInUniform {
//     fn new() -> Self {
//         use cgmath::SquareMatrix;
//         Self {
//             view_proj: cgmath::Matrix4::identity().into(),
//         }
//     }

//     pub fn input(&mut self, event: &WindowEvent) -> bool {
        
//         match event {
//             WindowEvent::CursorMoved { position, .. } => {
//                 self.clear_color = wgpu::Color {
//                     r: position.x as f64 / self.size.width as f64,
//                     g: position.y as f64 / self.size.height as f64,
//                     b: 1.0,
//                     a: 1.0,
//                 };
//                 true
//             }
//             WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
//                 println!("ScrollDelta {}", y);
//                 true
//             }
//             _ => false,
//         }
//     }

//     pub fn update(&mut self) {}
// }