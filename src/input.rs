use winit::event;

pub struct InputState {
    keys: [bool; 163],
    pub mouse_delta: (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys: [false; 163],
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn update_key(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        self.keys[key as usize] = state == event::ElementState::Pressed;
    }

    pub fn key_pressed(&self, key: event::VirtualKeyCode) -> bool {
        self.keys[key as usize]
    }

    pub fn update_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.mouse_delta = (mouse_dx as f32, mouse_dy as f32);
    }
}
