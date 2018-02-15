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
    fn input<'a>(&self, txt: &'a mut String) -> Input<'a>;
    fn text(&self, txt: &str) -> Text;
}

pub use self::default::*;