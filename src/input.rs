use winit::event;

pub struct InputState {
    keys: [bool; 163],
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys: [false; 163],
        }
    }

    pub fn update_key(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        self.keys[key as usize] = state == event::ElementState::Pressed;
    }

    pub fn key_pressed(&self, key: event::VirtualKeyCode) -> bool {
        self.keys[key as usize]
    }
}
