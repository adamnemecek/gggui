use super::*;
use super::events::*;
use downcast::Any;

mod flow;
mod button;
mod input;
mod lable;

pub use self::flow::*;
pub use self::button::*;
pub use self::input::*;
pub use self::lable::*;

pub enum StateType {
    Focus,
    Persistent
}

pub trait WidgetState: Any { }

downcast!(WidgetState);

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum GenericWidgetState { 
    Hovered,
    Clicked,
    Idle,
}

#[derive(Clone,Copy,Debug)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
    pub visibility: Option<Rect>
}

pub enum ChildType {
    Intersect(Rect),
    Expand(Rect),
    None,
}

impl MousePosition {
    pub fn inside(&self, layout: &Rect) -> bool {
        self.visibility.map_or(false, |v| v.intersect(layout).map_or(false, |i| {
            self.x >= i.left && 
            self.x < i.right && 
            self.y >= i.top && 
            self.y < i.bottom
        }))
    }

    pub fn sub(&self, layout: &Rect) -> MousePosition {
        MousePosition {
            x: self.x,
            y: self.y,
            visibility: self.visibility.and_then(|v| v.intersect(layout))
        }
    }

    pub fn expand(&self, layout: &Rect) -> MousePosition {
        MousePosition {
            x: self.x,
            y: self.y,
            visibility: Some(layout.clone())
        }
    }
}

impl WidgetState for GenericWidgetState {  }

impl WidgetState for () { }

pub trait Widget {
    type Result;
    type State: WidgetState + Clone;

    fn state_type() -> StateType { StateType::Focus }
    fn default() -> Self::State;
    fn tabstop() -> bool { false }

    fn measure(
        &self, 
        state: &Self::State
    ) -> Option<Rect>;

    fn layout(
        &mut self, 
        state: &Self::State, 
        layout: Rect, 
        child: Option<Rect>
    ) -> Rect;

    fn event(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition, 
        event: Event,
        is_focused: bool
    ) -> Capture;

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition
    ) -> bool;

    fn predraw<F: FnMut(Primitive)>(
        &self, 
        _state: &Self::State,
        _layout: Rect, 
        _submit: F) { 
    }

    fn postdraw<F: FnMut(Primitive)>(
        &self, 
        _state: &Self::State, 
        _layout: Rect, 
        _submit: F) { 
    }

    fn childs(
        &self, 
        _state: &Self::State,
        _layout: Rect,
    ) -> ChildType {
        ChildType::None
    }

    fn autofocus(&self) -> bool {
        false
    }
    
    fn result(self, &Self::State) -> Self::Result;
}

pub trait Layout {
    fn layout(&mut self, child: Option<Rect>) -> Rect;
}

pub struct LayoutCell<'a, W: Widget+'a> {
    widget: &'a mut W,
    state: &'a mut W::State,
    layout: Rect,
}

pub struct LayoutRoot {
    pub viewport: Rect,
}

impl<'a, W: Widget> LayoutCell<'a, W> {
    pub fn new(widget: &'a mut W, state: &'a mut W::State, layout: Rect) -> LayoutCell<'a, W> {
        LayoutCell {
            widget,
            state,
            layout,
        }
    }
}

impl<'a, W: Widget> Layout for LayoutCell<'a, W> {

    fn layout(&mut self, child: Option<Rect>) -> Rect {
        self.widget.layout(self.state, self.layout, child)
    }

}

impl Layout for LayoutRoot {

    fn layout(&mut self, child: Option<Rect>) -> Rect {
        child.unwrap_or(self.viewport)
    }

}