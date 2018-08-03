use std::rc::Rc;
use super::*;

#[derive(Clone)]
pub struct Clipper { 
    pub rect: Rect,
    pub intersect: bool,
    pub updater: Option<Rc<Fn(&mut Clipper,Option<&Layout>)>>,
}

impl Clipper {
	pub fn new(rect: Rect) -> Self {
		Clipper {
			rect,
			intersect: true,
			updater: None,
		}
	}

	pub fn with_updater<F: 'static + Fn(&mut Clipper,Option<&Layout>)>(mut self, f: F) -> Self {
		self.updater = Some(Rc::new(f));
		self
	}

	pub fn update(&mut self, layout: Option<&Layout>) {
		let u = self.updater.clone();
		u.map(|f| f(self, layout));
	}
}