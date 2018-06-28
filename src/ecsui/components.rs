use super::*;
use primitive::*;

pub type Container<T> = Rc<RefCell<Vec<(Option<T>, usize)>>>;

pub struct FetchComponent<T> {
    x: Container<T>,
    i: usize,
}

impl<T> FetchComponent<T> {
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

impl<T> FetchComponent<T> {
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

#[derive(Clone,Debug)]
pub struct Layout {
    pub current: Rect,
    pub valid: bool,
    pub margin: Rect,
    pub padding: Rect,
}

#[derive(Clone,Debug)]
pub struct Text {
    pub current: String,
    pub layout: Rect,
    pub valid: bool,
}

#[derive(Clone,Debug)]
pub struct Background {
    pub normal: Patch,
    pub hover: Patch,
    pub click: Patch,
}