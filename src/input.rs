use winit::event;

pub struct InputState {
    keys: [bool; 163],
    mouse_buttons: [bool; 32], // kinda arbitrary max size, maybe put more thought into it later
    pub mouse_delta: (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys: [false; 163],
            mouse_buttons: [false; 32],
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

    pub fn update_mouse_button(&mut self, button: event::MouseButton, state: event::ElementState) {
        match button {
            event::MouseButton::Left => {
                self.mouse_buttons[0] = state == event::ElementState::Pressed;
            },
            event::MouseButton::Right => {
                self.mouse_buttons[1] = state == event::ElementState::Pressed;
            },
            event::MouseButton::Middle => {
                self.mouse_buttons[2] = state == event::ElementState::Pressed;
            },
            event::MouseButton::Other(other_button) => {
                self.mouse_buttons[other_button as usize] = state == event::ElementState::Pressed;
            },
        }
    }

    pub fn mouse_pressed(&self, button: event::MouseButton) -> bool {
        match button {
            event::MouseButton::Left => {
                self.mouse_buttons[0]
            },
            event::MouseButton::Right => {
                self.mouse_buttons[1]
            },
            event::MouseButton::Middle => {
                self.mouse_buttons[2]
            },
            event::MouseButton::Other(other_button) => {
                self.mouse_buttons[other_button as usize]
            },
        }
    }
}
