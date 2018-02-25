use super::*;
use std::mem::replace;

pub enum MenuItem<'a, T: 'a> {
    Separator,
    StringItem(T, &'a str, &'a[MenuItem<'a, T>]),
    IconItem(T, Image, &'a str, &'a[MenuItem<'a, T>]),
}

pub struct Menu<'a, T: 'a> {
    menu: &'a[MenuItem<'a, T>],
    result: Option<&'a T>,
}

#[derive(Clone)]
pub struct MenuState {
    x: f32,
    y: f32,
    state: MenuStateInner,
}

#[derive(Clone)]
enum MenuStateInner {
    Idle,
    Open,
    Hover(usize, Box<MenuStateInner>),
}

impl WidgetState for MenuState { }

impl<'a, T> Menu<'a, T> {
    pub fn new(menu: &'a[MenuItem<'a, T>]) -> Self {
        Menu {
            menu,
            result: None,
        }
    }
}

fn for_each_item<'a, F, T>(slice: &'a[MenuItem<'a, T>], state: &MenuState, mut f: F) where 
    F: FnMut(Rect, &T, bool)
{
    match &state.state {
        &MenuStateInner::Idle => (),
        &MenuStateInner::Open | 
        &MenuStateInner::Hover(_,_) => {
            // find widest item
            let width = slice.iter().fold(0.0, |_acc, _item| 32.0);

            // layout items
            let x = state.x;
            let mut y = state.y;
            let mut selected_y = 0.0;
            let mut i = 0;
            for item in slice {
                let (item, height, recursive) = match item {
                    &MenuItem::Separator => {
                        y += 8.0;
                        continue;
                    },
                    &MenuItem::StringItem(ref item, _, sub) => { 
                        (item, 24.0, sub.len() > 0)
                    },
                    &MenuItem::IconItem(ref item, ref icon, _, sub) => {
                        (item, icon.size.height().max(24.0), sub.len() > 0)
                    },
                };

                let layout = Rect { 
                    left: x, 
                    top: y, 
                    right: x + width, 
                    bottom: y + height
                };

                f(layout, item, recursive);

                if let MenuStateInner::Hover(index, _) = state.state {
                    if index == i {
                        selected_y = y;
                    }
                }

                i += 1;
                y += height;
            }

            match &state.state {
                &MenuStateInner::Hover(index, ref state) => {
                    match &slice[index] {
                        &MenuItem::StringItem(_, _, sub) |
                        &MenuItem::IconItem(_, _, _, sub) => {
                            let inner: &MenuStateInner = &*state;
                            let sub_state = MenuState {
                                x: x + width - 2.0,
                                y: selected_y,
                                state: inner.clone(),
                            };
                            for_each_item(sub, &sub_state, f);
                        },
                        &_ => (),
                    }
                },
                _ => (),
            }
        }
    }
}

impl<'a, T: 'a> Widget for Menu<'a, T> {
    type Result = Option<&'a T>;
    type State = MenuState;

    fn default() -> MenuState {
        MenuState {
            x: 0.0,
            y: 0.0,
            state: MenuStateInner::Idle,
        }
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
        Some(Rect::from_wh(0.0, 0.0))
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition, 
        event: Event,
        is_focused: bool
    ) -> Capture {
        let mut capture = Capture::None;

        state.state = match replace(&mut state.state, MenuStateInner::Idle) {
            MenuStateInner::Idle => {
                if let Event::Press(Key::RightMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                    MenuStateInner::Open
                } else {
                    MenuStateInner::Idle
                }
            },
            MenuStateInner::Open => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);
                MenuStateInner::Open
            },
            MenuStateInner::Hover(item, sub) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);
                MenuStateInner::Hover(item, sub)
            },
        };

        capture
    }

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        _: Rect, 
        cursor: MousePosition
    ) -> Hover {
        match &state.state {
            &MenuStateInner::Idle => {
                state.x = cursor.x;
                state.y = cursor.y;
            },
            &_ => (), 
        };
        Hover::HoverIdle
    }

    fn predraw<F: FnMut(Primitive)>(
        &self, 
        state: &Self::State,
        layout: Rect, 
        submit: F
    ) { 
        // todo
    }

    fn result(self, _: &Self::State) -> Option<&'a T> {
        self.result
    }
}