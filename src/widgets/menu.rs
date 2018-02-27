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

fn for_each_item<'a, F, T>(slice: &'a[MenuItem<'a, T>], state: &MenuState, mut f: F) -> MenuState 
where 
    F: FnMut(Rect, &T, bool, bool) -> bool
{
    let mut result = state.clone();
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

                let activated = if let MenuStateInner::Hover(index, _) = state.state {
                    if index == i {
                        selected_y = y;
                        f(layout, item, true, recursive)
                    } else {
                        f(layout, item, false, recursive)
                    }
                } else {
                    f(layout, item, false, recursive)
                };

                if activated {
                    result.state = MenuStateInner::Hover(i, Box::new(MenuStateInner::Open));
                }

                i += 1;
                y += height;
            }

            result.state = match result.state {
                MenuStateInner::Hover(index, inner_state) => {
                    match &slice[index] {
                        &MenuItem::StringItem(_, _, sub) |
                        &MenuItem::IconItem(_, _, _, sub) => {
                            let sub_state = MenuState {
                                x: x + width - 2.0,
                                y: selected_y,
                                state: *inner_state,
                            };

                            if sub.len() > 0 {
                                let inner = Box::new(for_each_item(sub, &sub_state, f).state);
                                MenuStateInner::Hover(index, inner)
                            } else {
                                let inner = Box::new(MenuStateInner::Idle);
                                MenuStateInner::Hover(index, inner)
                            }
                        },
                        &_ => MenuStateInner::Idle,
                    }
                },
                state => state,
            }
        }
    }
    result
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
        _: &Self::State,
        _: Option<Rect>
    ) -> Option<Rect> {
        Some(Rect::from_wh(0.0, 0.0))
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        _: Rect, 
        _: MousePosition, 
        event: Event,
        _: bool
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
                match event {
                    Event::Press(Key::LeftMouseButton, _) |
                    Event::Press(Key::RightMouseButton, _) => {
                        capture = Capture::None;
                        MenuStateInner::Idle
                    },
                    _ => {
                        capture = Capture::CaptureFocus(MouseStyle::Arrow);
                        MenuStateInner::Open
                    }
                }
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
            &MenuStateInner::Open |
            &MenuStateInner::Hover(_, _) => {
                *state = for_each_item(self.menu, state, |rect, item, hovered, recursive| {
                    cursor.inside(&rect)
                });
            },
        };
        Hover::HoverIdle
    }

    fn predraw<F: FnMut(Primitive)>(
        &self, 
        state: &Self::State,
        _: Rect, 
        mut submit: F
    ) { 
        for_each_item(self.menu, state, |rect, item, hovered, recursive| {
            if hovered {
                submit(Primitive::DrawRect(rect, Color{ r: 0.0, g: 0.0, b: 1.0, a: 1.0 }));
            } else {
                submit(Primitive::DrawRect(rect, Color::black()));
            }

            false
        });
    }

    fn child_area(
        &self, 
        _: &Self::State,
        _: Rect,
    ) -> ChildArea {
        ChildArea::Popup(Rect::from_wh(0.0, 0.0))
    }

    fn result(self, _: &Self::State) -> Option<&'a T> {
        self.result
    }
}