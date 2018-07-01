use super::*;

#[derive(Clone)]
pub enum Constraint {
	Fixed,
	Grow,
	Fill,
}

#[derive(Clone)]
pub struct Layout {
    pub current: Option<Rect>,
    pub margin: Rect,
    pub padding: Rect,
    pub constrain_width: Constraint,
    pub constrain_height: Constraint,
}

impl Layout {
	pub fn after_margin(&self) -> Rect {
		self.current.clone().unwrap().after_margin(self.margin)
	}

	pub fn after_padding(&self) -> Rect {
		self.current.clone().unwrap().after_padding(self.padding)
	}
}