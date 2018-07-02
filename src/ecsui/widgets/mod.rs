use super::*;

pub mod button;
pub mod linear;
pub mod scroll;

pub use self::button::*;
pub use self::linear::*;
pub use self::scroll::*;

#[derive(Clone)]
pub struct Viewport {
	pub child_rect: Rect,
	pub input_rect: Option<Rect>,
}

pub trait WidgetBase {
    fn tabstop(&self) -> bool { false }
    fn enabled(&self, dag::Id, &Ui) -> bool { true }
    fn autofocus(&self, dag::Id) -> bool { false }

    fn create(&mut self, id: dag::Id, world: &mut Ui);
    fn update(&mut self, id: dag::Id, world: &Ui, window: Viewport) -> Viewport;
    fn event(&mut self, _id: dag::Id, _world: &Ui, _context: &mut EventSystemContext) { }
}

pub trait Widget: WidgetBase {
    type Result;

    fn result(&self, id: dag::Id) -> Self::Result;
}