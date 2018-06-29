use super::*;

#[derive(Clone,Debug)]
pub struct Layout {
    pub current: Rect,
    pub valid: bool,
    pub margin: Rect,
    pub padding: Rect,
}