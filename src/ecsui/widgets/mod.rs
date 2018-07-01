use super::*;

pub mod button;
pub mod linear;

pub use self::button::Button;
pub use self::linear::LinearLayout;

pub trait WidgetBase {
    fn tabstop(&self) -> bool { false }
    fn enabled(&self, dag::Id, &Ui) -> bool { true }
    fn autofocus(&self, dag::Id) -> bool { false }

    fn create(&mut self, id: dag::Id, world: &mut Ui);
    fn update(&mut self, id: dag::Id, world: &Ui, window: Rect) -> Rect;
    fn event(&mut self, id: dag::Id, world: &Ui, ev: Event, focus: bool) -> Capture;
}

pub trait Widget: WidgetBase {
    type Result;

    fn result(&self, id: dag::Id) -> Self::Result;
}