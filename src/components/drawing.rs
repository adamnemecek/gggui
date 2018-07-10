use super::*;

#[derive(Clone)]
pub struct Drawing {
    pub primitives: Vec<Primitive>,
}

impl Drawing {
	pub fn new() -> Self {
		Self {
			primitives: vec![],
		}
	}
}