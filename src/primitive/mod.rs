use std::iter::*;
use std::str::*;
use smallvec::SmallVec;
use rusttype;
use super::Font;

mod rect;
mod color;
mod text;
mod patch;
mod image;

pub use self::rect::*;
pub use self::color::*;
pub use self::text::*;
pub use self::patch::*;
pub use self::image::*;

pub enum Primitive {
    PushClip(Rect),
    PopClip,
    DrawRect(Rect, Color),
    DrawText(Text, Rect, Color),
    Draw9(Patch, Rect, Color),
    DrawImage(Image, Rect, Color),
}

#[derive(Clone)]
pub enum Background {
    None,
    Color(Color),
    Image(Image, f32),
    Patch(Patch, f32),
}

impl Background {
    pub fn content_rect(&self, span: Rect) -> Rect {
        match self {
            &Background::Patch(ref patch, _) => patch.content_rect(span),
            &_ => span,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            &Background::None => false,
            &_ => true,
        }
    }
}