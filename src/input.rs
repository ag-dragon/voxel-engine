use winit::event;

pub struct InputState {
    pub keys: [bool; 163],
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys: [false; 163],
        }
    }

    pub fn update(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        self.keys[key as usize] = state == event::ElementState::Pressed;
    }
}
