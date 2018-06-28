use super::*;

pub trait WidgetBase {
    fn tabstop(&self) -> bool { false }
    fn enabled<'a>(&self, dag::Id, &Context<'a>) -> bool { true }
    fn autofocus(&self, dag::Id) -> bool { false }

    fn create<'a>(&mut self, dag::Id, &mut Context<'a>);
    fn update<'a>(&mut self, dag::Id, &mut Context<'a>);
    fn event<'a>(&mut self, dag::Id, &mut Context<'a>, Event, bool) -> Capture { Capture::None }
}

pub trait Widget: WidgetBase {
    type Result;

    fn result(&self, dag::Id) -> Self::Result;
}