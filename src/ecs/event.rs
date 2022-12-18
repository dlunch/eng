use winit::event::VirtualKeyCode;

pub enum KeyboardEvent {
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode),
}
