use super::*;
use ecsui::components::background::WidgetBackground;

#[derive(Clone,Copy,PartialEq)]
pub struct ScrollState {
    scroll: (f32, f32),
    inner: MouseState,
}

#[derive(Clone,Copy,PartialEq)]
pub enum MouseState {
    Idle,
    HoverContent,
    HoverH(f32),
    HoverV(f32),
    ScrollH(f32),
    ScrollV(f32),
}

pub struct Scroll {
    width: f32,
    height: f32,
}

impl Scroll {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height
        }
    }
}

impl WidgetBase for Scroll {
    fn create(&mut self, id: dag::Id, world: &mut Ui) {
        world.create_component(id, ScrollState {
            scroll: (0.0, 0.0),
            inner: MouseState::Idle,
        });
        world.create_component(id, Layout {
            current: Some(Rect::from_wh(self.width, self.height)),
            margin: Rect::from_wh(0.0, 0.0),
            padding: Rect { left: 0.0, top: 0.0, right: 16.0, bottom: 16.0 },
            constrain_width: Constraint::Fixed,
            constrain_height: Constraint::Fixed,
        });
        
    }

    fn update(&mut self, id: dag::Id, world: &Ui, _window: Viewport) -> Viewport {
        let state: ScrollState = world.component(id).unwrap().borrow().clone();
        for child in world.children() {
            world.component(*child).map(|mut layout: FetchComponent<Layout>| {
                layout.current = layout.current.map(|rect| Rect {
                    left: -state.scroll.0,
                    top: -state.scroll.1,
                    right: rect.width() - state.scroll.0,
                    bottom: rect.height() - state.scroll.1,
                }).or(Some(Rect{
                    left: -state.scroll.0,
                    top: -state.scroll.1,
                    right: -state.scroll.0,
                    bottom: -state.scroll.1,
                }));
            }
        }

        Viewport {
            child_rect: Rect::from_wh(0.0, 0.0),
            input_rect: Rect::from_wh(0.0, 0.0),
        }
    }

    fn event(&mut self, id: dag::Id, world: &Ui, event: Event, focus: bool) -> Capture {
        let mut capture = Capture::None;
        let mut state: FetchComponent<ScrollState> = world.component(id).unwrap();
        let mut state = state.borrow_mut();

        state.inner = match state.inner {
            MouseState::Idle => {
                MouseState::Idle
            },
            MouseState::HoverContent => {
                MouseState::HoverContent
            },
            MouseState::HoverH(x) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    MouseState::ScrollH(x)
                } else {
                    MouseState::HoverH(x)
                }
            },
            MouseState::HoverV(y) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    MouseState::ScrollV(y)
                } else {
                    MouseState::HoverV(y)
                }
            },
            MouseState::ScrollH(x) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = layout.clone();
                bar.top = bar.bottom - self.background_h.image.size.height();
                if self.scrollable_v {
                    bar.right = bar.right - self.background_v.image.size.width();
                }
                state.scroll.0 = handle_to_scroll(
                    bar.left, 
                    cursor.x-x, 
                    bar.width(), 
                    self.content.0,
                    self.content.1
                );

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    MouseState::Idle
                } else {
                    MouseState::ScrollH(x)
                }
            },
            MouseState::ScrollV(y) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = layout.clone();
                bar.left = bar.right - self.background_v.image.size.width();
                if self.scrollable_h {
                    bar.bottom = bar.bottom - self.background_h.image.size.height();
                }
                state.scroll.1 = handle_to_scroll(
                    bar.top, 
                    cursor.y-y, 
                    bar.height(), 
                    self.content.2,
                    self.content.3
                );

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    MouseState::Idle
                } else {
                    MouseState::ScrollV(y)
                }
            },
        };

        if state.inner != MouseState::Idle {
            if let Event::Scroll(dx, dy) = event {
                state.scroll.0 = (state.scroll.0 - dx).max(self.content.0).min(self.content.1);
                state.scroll.1 = (state.scroll.1 - dy).max(self.content.2).min(self.content.3);
            }
        }

        capture
    }
}

impl Widget for Scroll {
    type Result = ();

    fn result(&self, _id: dag::Id) -> Self::Result { }
}