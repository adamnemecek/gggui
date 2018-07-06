use super::*;

pub mod label;
pub mod button;
pub mod toggle;
pub mod linear;
pub mod scroll;
pub mod input;

pub use self::label::*;
pub use self::button::*;
pub use self::toggle::*;
pub use self::linear::*;
pub use self::scroll::*;
pub use self::input::*;

#[derive(Clone)]
pub struct Viewport {
    pub child_rect: Rect,
    pub input_rect: Option<Rect>,
}

pub trait WidgetBase {
    fn tabstop(&self) -> bool { false }
    fn enabled(&self, dag::Id, &Ui) -> bool { true }
    fn autofocus(&self, dag::Id) -> bool { false }

    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style);
    fn update(&mut self, id: dag::Id, world: &Ui, style: &Style, window: Viewport) -> Viewport;
    fn event(&mut self, _id: dag::Id, _world: &Ui, _style: &Style, _context: &mut EventSystemContext) { }
}

pub trait Widget: WidgetBase {
    type Result;

    fn result(&self, id: dag::Id) -> Self::Result;
}

pub struct Style {
    pub font: (Font, usize),

    pub button_normal: Patch,
    pub button_hover: Patch,
    pub button_pressed: Patch,

    pub input: Patch,

    pub checkbox_checked_normal: Image,
    pub checkbox_checked_hover: Image,
    pub checkbox_checked_pressed: Image,
    pub checkbox_normal: Image,
    pub checkbox_hover: Image,
    pub checkbox_pressed: Image,

    pub radio_checked_normal: Image,
    pub radio_checked_hover: Image,
    pub radio_checked_pressed: Image,
    pub radio_normal: Image,
    pub radio_hover: Image,
    pub radio_pressed: Image,

    pub scroll_horizontal: (Patch, Patch),
    pub scroll_vertical: (Patch, Patch),

    pub window: Patch,
}

impl Style {
    pub fn default(ui: &mut Ui) -> Self {
        Self {
            button_normal: 
                ui.get_patch(load_from_static_memory!("../../../img/button_normal.png")),
            button_hover: 
                ui.get_patch(load_from_static_memory!("../../../img/button_hover.png")),
            button_pressed: 
                ui.get_patch(load_from_static_memory!("../../../img/button_pressed.png")),
            checkbox_normal: 
                ui.get_image(load_from_static_memory!("../../../img/checkbox_normal.png")),
            checkbox_hover: 
                ui.get_image(load_from_static_memory!("../../../img/checkbox_hover.png")),
            checkbox_pressed: 
                ui.get_image(load_from_static_memory!("../../../img/checkbox_pressed.png")),
            checkbox_checked_normal: 
                ui.get_image(load_from_static_memory!("../../../img/checkbox_checked_normal.png")),
            checkbox_checked_hover: 
                ui.get_image(load_from_static_memory!("../../../img/checkbox_checked_hover.png")),
            checkbox_checked_pressed: 
                ui.get_image(load_from_static_memory!("../../../img/checkbox_checked_pressed.png")),
            radio_normal: 
                ui.get_image(load_from_static_memory!("../../../img/radio_normal.png")),
            radio_hover: 
                ui.get_image(load_from_static_memory!("../../../img/radio_hover.png")),
            radio_pressed: 
                ui.get_image(load_from_static_memory!("../../../img/radio_pressed.png")),
            radio_checked_normal: 
                ui.get_image(load_from_static_memory!("../../../img/radio_checked_normal.png")),
            radio_checked_hover: 
                ui.get_image(load_from_static_memory!("../../../img/radio_checked_hover.png")),
            radio_checked_pressed: 
                ui.get_image(load_from_static_memory!("../../../img/radio_checked_pressed.png")),
            input: 
                ui.get_patch(load_from_static_memory!("../../../img/input.png")),
            font: 
                ui.get_font(load_from_static_memory!("../../../img/default_font.ttf")),
            window: 
                ui.get_patch(load_from_static_memory!("../../../img/window.png")),
            scroll_horizontal: 
                (ui.get_patch(load_from_static_memory!("../../../img/scroll_bg.png")),
                 ui.get_patch(load_from_static_memory!("../../../img/scroll_bar.png"))),
            scroll_vertical: 
                (ui.get_patch(load_from_static_memory!("../../../img/scroll_bg.png")),
                 ui.get_patch(load_from_static_memory!("../../../img/scroll_bar.png"))),
        }
    }
}