use super::*;

#[derive(Clone)]
pub struct Drawing {
    pub primitives: Vec<Primitive>,
    pub updater: Option<Rc<Fn(&mut Drawing, Option<&Layout>)>>,
}

impl Drawing {
    pub fn new() -> Self {
        Self {
            primitives: vec![],
            updater: None,
        }
    }

    pub fn with_updater<F: 'static + Fn(&mut Drawing,Option<&Layout>)>(mut self, f: F) -> Self {
		self.updater = Some(Rc::new(f));
		self
	}

    pub fn update(&mut self, layout: Option<&Layout>) {
		let u = self.updater.clone();
		u.map(|f| f(self, layout));
	}
}