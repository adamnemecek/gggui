mod default;

use ::Ui;
use widgets::*;
use primitive::Text;

pub trait Style {
    fn new(ui: &mut Ui) -> Self;
    fn lable(&self, txt: &str) -> Lable;
    fn title(&self, txt: &str) -> Lable;
    fn checkbox<'a>(&self, val: &'a mut bool) -> Toggle<'a, bool>;
    fn radio<'a, T: Clone+PartialEq+'a>(&self, val: &'a mut T, target: T) -> Toggle<'a, T>;
    fn button(&self) -> Button;
    fn flow(&self) -> Flow;
    fn scroll(&self) -> Scroll;
    fn input<'a>(&self, txt: &'a mut String) -> Input<'a>;
    fn text(&self, txt: &str) -> Text;
    fn window(&self, w: f32, h: f32) -> WindowProperties;
    fn modal(&self, w: f32, h: f32) -> WindowProperties;
}

pub use self::default::*;