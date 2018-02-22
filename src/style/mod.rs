mod default;

use ::Ui;
use widgets::*;
use primitive::Text;

pub trait Style {
    fn new(ui: &mut Ui) -> Self;
    fn lable(&self, txt: &str) -> Lable;
    fn title(&self, txt: &str) -> Lable;
    fn button(&self) -> Button;
    fn flow(&self) -> Flow;
    fn scroll<'a>(&self, scroll: &'a mut (f32, f32)) -> Scroll<'a>;
    fn input<'a>(&self, txt: &'a mut String) -> Input<'a>;
    fn text(&self, txt: &str) -> Text;
    fn window(&self, w: f32, h: f32) -> WindowProperties;
    fn modal(&self, w: f32, h: f32) -> WindowProperties;
}

pub use self::default::*;