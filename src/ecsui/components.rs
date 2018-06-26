use primitive::*;
use super::dag;

#[derive(Clone,Copy,Debug)]
pub enum Clickable {
    Idle,
    Hovering,
    Clicked(bool),
    Released(bool),
}

#[derive(Clone,Copy,Debug)]
pub enum Align {
    Begin, Middle, End
}

#[derive(Clone,Copy,Debug)]
pub enum LayoutStyle {
    Wrap,
    LinearRight(Align),
    LinearLeft(Align),
    LinearDown(Align),
    LinearUp(Align),
    GridHorizontal(u32),
    GridVertical(u32),
    Single(Align, Align),
    Absolute(Rect),
}

impl Default for LayoutStyle {
    fn default() -> Self {
        LayoutStyle::Wrap
    }
}

pub struct Layout {
    pub parent: dag::Id,
    pub style: LayoutStyle,
    pub grow_horizontal: bool,
    pub grow_vertical: bool,
    pub current: Rect,
    pub valid: bool,
    pub margin: Rect,
    pub padding: Rect,
}

pub struct Text {
    pub current: String,
    pub layout: Rect,
    pub valid: bool,
}

pub struct Background {
    pub normal: Patch,
    pub hover: Patch,
    pub click: Patch,
}