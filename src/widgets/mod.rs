use super::*;
use super::events::*;
use downcast::Any;
use std::default::Default;

mod flow;
mod scroll;
mod button;
mod toggle;
mod input;
mod lable;
mod menu;
mod window;

pub use self::flow::*;
pub use self::scroll::*;
pub use self::button::*;
pub use self::toggle::*;
pub use self::input::*;
pub use self::lable::*;
pub use self::menu::*;
pub use self::window::*;

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

pub enum ChildArea {
    ConfineContentAndInput(Rect),
    OverflowContentConfineInput(Rect),
    OverflowContentAndInput,
    Popup(Rect),
    
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

impl Default for GenericWidgetState {
    fn default() -> Self {
        GenericWidgetState::Idle
    }
}

pub type WidgetMeasure<'a> = Box<Fn(Option<Rect>)->Option<Rect>+'a>;

#[allow(unused_variables)]
pub trait Widget {
    type Result;
    type State: WidgetState + Clone + Default;

    fn state_type() -> StateType { 
        StateType::Focus 
    }

    fn tabstop() -> bool { 
        false 
    }

    fn enabled(&self, _state: &Self::State) -> bool {
        true
    }

    fn measure(
        &self, 
        state: &Self::State,
        layout: Option<Rect>
    ) -> Option<Rect> {
        None
    }

    fn estimate(
        &self, 
        state: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        layout
    }

    fn layout(
        &mut self, 
        state: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        self.estimate(state, layout, child)
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition, 
        event: Event,
        is_focused: bool
    ) -> Capture {
        Capture::None
    }

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition
    ) -> Hover {
        Hover::NoHover
    }

    fn predraw<F: FnMut(Primitive)>(
        &self, 
        state: &Self::State,
        layout: Rect, 
        submit: F) { 
    }

    fn postdraw<F: FnMut(Primitive)>(
        &self, 
        state: &Self::State, 
        layout: Rect, 
        submit: F) { 
    }

    fn child_area(
        &self, 
        state: &Self::State,
        layout: Rect,
    ) -> ChildArea {
        ChildArea::None
    }

    fn autofocus(&self) -> bool {
        false
    }
    
    fn result(self, &Self::State) -> Self::Result;
}

pub trait Layout {
    fn estimate(&self, child: WidgetMeasure) -> Rect;
    fn layout(&mut self, child: WidgetMeasure) -> Rect;
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
    fn estimate(&self, child: WidgetMeasure) -> Rect {
        self.widget.estimate(self.state, self.layout, child)
    }
    fn layout(&mut self, child: WidgetMeasure) -> Rect {
        self.widget.layout(self.state, self.layout, child)
    }
}

impl Layout for LayoutRoot {
    fn estimate(&self, child: WidgetMeasure) -> Rect {
        child(None)
            .unwrap_or(self.viewport).size()
            .translate(self.viewport.left, self.viewport.top)
    }
    fn layout(&mut self, child: WidgetMeasure) -> Rect {
        self.estimate(child)
    }
}