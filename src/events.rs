#[derive(Clone,Copy,Debug)]
pub enum Key {
    LeftMouseButton,
    MiddleMouseButton,
    RightMouseButton,

    Key1, Key2, Key3,
    Key4, Key5, Key6,
    Key7, Key8, Key9,
    Key0,

    A, B, C, D, 
    E, F, G, H, 
    I, J, K, L, 
    M, N, O, P, 
    Q, R, S, T, 
    U, V, W, X, 
    Y, Z,

    Tab, Shift, Ctrl, 
    Alt, Space, Enter,
    Backspace, Escape,
    Home, End,

    Left, Right, Up, Down,
}

#[derive(Clone,Copy,Debug)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool,
}

impl Modifiers {
    pub fn none() -> Modifiers { Modifiers{ ctrl: false, alt: false, shift: false, logo: false } }
    pub fn ctrl() -> Modifiers { Modifiers{ ctrl: true, alt: false, shift: false, logo: false } }
    pub fn alt() -> Modifiers { Modifiers{ ctrl: false, alt: true, shift: false, logo: false } }
    pub fn shift() -> Modifiers { Modifiers{ ctrl: false, alt: false, shift: true, logo: false } }
    pub fn logo() -> Modifiers { Modifiers{ ctrl: false, alt: false, shift: false, logo: true } }
}

#[derive(Clone,Copy,Debug)]
pub enum Event {
    /// A button on some input device was pressed.
    Press(Key, Modifiers),
    /// A button on some input device was released.
    Release(Key, Modifiers),
    /// The window was resized to the given dimensions.
    Resize(f32, f32),
    /// Some motion input was received (e.g. moving mouse or joystick axis).
    Motion(f32, f32),
    /// The mouse cursor was moved to a location.
    Cursor(f32, f32),
    /// The mouse wheel or touchpad scroll gesture sent us some scroll event.
    Scroll(f32, f32),
    /// Text input was received, usually via the keyboard.
    Text(char),
    /// The window was focused or lost focus.
    Focus(bool),
    /// The application exited it's main event loop
    Exit,
    /// There are no events but we still want event() to be called.
    Idle,
}

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum MouseMode {
    Normal,
    Confined,
    Locked(f32, f32),
}

#[derive(Clone,Copy,PartialEq)]
pub enum Hover {
    HoverIdle,
    HoverActive(MouseStyle),
    NoHover,
}

#[derive(Clone,Copy,PartialEq)]
pub enum Capture {
    CaptureFocus(MouseStyle),
    CaptureMouse(MouseStyle),
    FocusNext,
    FocusPrev,
    None,
}

#[derive(Clone,Copy,PartialEq)]
pub enum MouseStyle {
    Invisible,
    Arrow,
    ArrowClickable,
    ArrowClicking,
    Text,
    ResizeN,
    ResizeS,
    ResizeW,
    ResizeE,
    ResizeNw,
    ResizeNe,
    ResizeSw,
    ResizeSe,
    ResizeWe,
    ResizeNs,
    ResizeNwse,
    ResizeNesw,
}