use super::*;
use std::mem::replace;
use std::time::Instant;

pub enum MenuItem<'a, T: 'a> {
    Separator,
    StringItem(T, &'a str, &'a[MenuItem<'a, T>]),
    IconItem(T, Image, &'a str, &'a[MenuItem<'a, T>]),
}

pub struct Menu<'a, T: 'a> {
    menu: &'a[MenuItem<'a, T>],
    result: Option<&'a T>,
}

pub type MenuPath = SmallVec<[i8; 8]>;

#[derive(Clone,PartialEq)]
pub enum MenuState {
    Idle,
    Hover(f32, f32, MenuPath, Instant),
}

impl<'a, T:'a> Menu<'a, T> {
    pub fn new(menu: &'a[MenuItem<'a, T>]) -> Self {
        Self {
            menu, result: None
        }
    }
}

impl<'a, T:'a> WidgetBase for Menu<'a, T> {
    fn create(&mut self, id: dag::Id, world: &mut Ui, _style: &Style) {
        world.create_component(id, Drawing{ primitives: vec![] });
        world.create_component(id, MenuState::Idle);
    }

    fn event(&mut self, id: dag::Id, world: &Ui, _style: &Style, context: &mut EventSystemContext) {
        let mut state = world.component::<MenuState>(id).unwrap();
        let mut state = state.borrow_mut();

        let mut drawing = world.component::<Drawing>(id).unwrap();
        let mut drawing = drawing.borrow_mut();

        // handle hovering
        *state = match replace(state.deref_mut(), MenuState::Idle) {
            MenuState::Idle => {
                MenuState::Idle
            },

            MenuState::Hover(x, y, mut path, mut time) => {
                time = for_each_item(
                    self.menu, 
                    0, (x, y), 
                    BorrowMutMenuPath{ x: &mut path }, 
                    time,
                    |rect, _, _, _| {
                        context.cursor.inside(&rect)
                    }
                );
                MenuState::Hover(x, y, path, time)
            },
        };

        // handle event
        *state = match replace(state.deref_mut(), MenuState::Idle) {
            MenuState::Idle => {
                if let Event::Press(Key::RightMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                    MenuState::Hover(context.cursor.x, context.cursor.y, MenuPath::new(), Instant::now())
                } else {
                    MenuState::Idle
                }
            },
            MenuState::Hover(x, y, path, time)  => {
                context.capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut cursor_outside = true;
                for_each_item(
                    self.menu, 
                    0, (x, y), 
                    BorrowMenuPath{ x: &path }, 
                    time,
                    |rect, item, hovered, _| {
                        if context.cursor.inside(&rect) {
                            cursor_outside = false;
                            match &context.event {
                                &Event::Press(Key::LeftMouseButton, _) => {
                                    self.result = Some(item);
                                },
                                &_ => (),
                            }
                        }
                        hovered
                    }
                );

                if self.result.is_some() {
                    MenuState::Idle
                } else if cursor_outside {
                    match context.event {
                        Event::Press(Key::LeftMouseButton, _) => {
                            MenuState::Idle
                        },
                        Event::Press(Key::RightMouseButton, _) => {
                            MenuState::Hover(context.cursor.x, context.cursor.y, MenuPath::new(), Instant::now())
                        },
                        _ => {
                            MenuState::Hover(x, y, path, time)
                        },
                    }
                } else {
                    MenuState::Hover(x, y, path, time)
                }
            },
        };

        drawing.primitives.clear();
        if context.focused {
            match state.deref() {
                &MenuState::Idle => (),
                &MenuState::Hover(x, y, ref path, time) => {
                    for_each_item(
                        self.menu, 
                        0, (x, y), 
                        BorrowMenuPath{ x: &path }, 
                        time,
                        |rect, _, hovered, _| {
                            if hovered {
                                drawing.primitives.push(Primitive::DrawRect(rect, Color{ r: 0.0, g: 0.0, b: 1.0, a: 1.0 }));
                            } else {
                                drawing.primitives.push(Primitive::DrawRect(rect, Color::black()));
                            }
                            hovered
                        }
                    );
                }
            }
        }
    }
}

impl<'a, T:'a> Widget for Menu<'a, T> {
    type Result = Option<&'a T>;

    fn result(&mut self, _id: dag::Id) -> Self::Result {
        self.result
    }
}

trait MenuPathWrapper {
    fn truncate(&mut self, length: usize);
    fn push(&mut self, item: i8);
    fn at(&self, index: usize) -> Option<i8>;
}

struct BorrowMenuPath<'a> { 
    x: &'a MenuPath
}

impl<'a> MenuPathWrapper for BorrowMenuPath<'a> {
    fn truncate(&mut self, _: usize) { }
    fn push(&mut self, _: i8) { }
    fn at(&self, index: usize) -> Option<i8> { self.x.iter().nth(index).map(|r| *r) }
}

struct BorrowMutMenuPath<'a> { 
    x: &'a mut MenuPath
}

impl<'a> MenuPathWrapper for BorrowMutMenuPath<'a> {
    fn truncate(&mut self, length: usize) { self.x.truncate(length); }
    fn push(&mut self, item: i8) { self.x.push(item); }
    fn at(&self, index: usize) -> Option<i8> { self.x.iter().nth(index).map(|r| *r) }
}

fn for_each_item<
    'a, 
    F: FnMut(Rect, &'a T, bool, bool) -> bool, 
    W: MenuPathWrapper,
    T
> (
    slice: &'a[MenuItem<'a, T>], 
    depth: usize,
    position: (f32, f32),
    mut path: W, 
    mut time: Instant,
    mut f: F
) -> Instant {
    // find widest item
    let width = slice.iter().fold(0.0, |_acc, _item| 32.0);

    // layout items
    let x = position.0;
    let mut y = position.1;
    let mut selected_y = None;
    let mut i = 0;

    for item in slice {
        let (item, height, recursive) = match item {
            &MenuItem::Separator => {
                i += 1;
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

        let hovered = path.at(depth).map_or(false, |j| i == j);

        if f(layout, item, hovered, recursive) {
            selected_y = Some(y);
            path.truncate(depth);
            path.push(i);
            time = Instant::now();
        } else if hovered && recursive {
            selected_y = Some(y);
        }

        i += 1;
        y += height;
    }

    // forget hover state if we're not nested deeper than the 
    //  current menu and nothing was hovered
    path.at(depth).map(|i| {
        if (path.at(depth+1).is_none() &&
            time.elapsed().subsec_nanos() > 200_000_000 && i >= 0) || selected_y.is_none() {
            path.truncate(depth);
            path.push(-1);
        }
    });    

    if selected_y.is_some() {
        // recurse into nested menu
        path.at(depth).map(|index| {
            if index >= 0 {
                match &slice[index as usize] {
                    &MenuItem::StringItem(_, _, sub) |
                    &MenuItem::IconItem(_, _, _, sub) => {
                        if sub.len() > 0 {
                            time = for_each_item(
                                sub, depth+1, (x+width, selected_y.unwrap()), path, time, f
                            );
                        }
                    },
                    &_ => (),
                }
            }
        });
    }

    time
}