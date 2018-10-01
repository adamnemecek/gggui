use super::*;

#[macro_use]
pub mod layout;
pub mod background;
pub mod clickable;
pub mod clipper;
pub mod drawing;

pub use self::layout::*;
pub use self::background::*;
pub use self::clickable::*;
pub use self::clipper::*;
pub use self::drawing::*;

pub type Container<T> = Rc<RefCell<Vec<(Option<T>, usize)>>>;

pub trait Fetch: Sized {
    fn fetch(world: &Ui, id: dag::Id) -> Result<Self, ()>;
}

pub struct FetchComponent<T: 'static + Clone> {
    x: Container<T>,
    i: usize,
}

impl<T: 'static + Clone> Fetch for FetchComponent<T> {
    fn fetch(world: &Ui, id: dag::Id) -> Result<Self, ()> {
        world.component(id).ok_or(())
    }
}

impl<T: 'static + Clone> Fetch for Option<FetchComponent<T>> {
    fn fetch(world: &Ui, id: dag::Id) -> Result<Self, ()> {
        Ok(world.component(id))
    }
}

impl<T: 'static + Clone> FetchComponent<T> {
    pub fn new(x: Container<T>, i: usize) -> Self {
        Self {
            x, i
        }
    }
}

pub struct ComponentRef<'a, T: 'a> {
    x: Ref<'a, Vec<(Option<T>, usize)>>,
    i: usize,
}

impl<'a, T: 'a> Deref for ComponentRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.x[self.i].0.as_ref().unwrap()
    }
}

pub struct ComponentRefMut<'a, T: 'a> {
    x: RefMut<'a, Vec<(Option<T>, usize)>>,
    i: usize,
}

impl<'a, T: 'a> Deref for ComponentRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.x[self.i].0.as_ref().unwrap()
    }
}

impl<'a, T: 'a> DerefMut for ComponentRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.x[self.i].0.as_mut().unwrap()
    }
}

impl<T: 'static + Clone> FetchComponent<T> {
    pub fn borrow<'a>(&'a self) -> ComponentRef<'a, T> {
        ComponentRef {
            x: self.x.borrow(),
            i: self.i,
        }
    }

    pub fn borrow_mut<'a>(&'a mut self) -> ComponentRefMut<'a, T> {
        ComponentRefMut {
            x: self.x.borrow_mut(),
            i: self.i,
        }
    }
}
