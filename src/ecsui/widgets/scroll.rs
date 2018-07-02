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
    HoverH,
    HoverV,
    ScrollH(f32),
    ScrollV(f32),
}

pub struct Scroll {
    width: f32,
    height: f32,
    content: Rect,
}

impl Scroll {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            content: Rect::from_wh(0.0, 0.0),
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

    fn update(&mut self, id: dag::Id, world: &Ui, window: Viewport) -> Viewport {
        let state = world.component::<ScrollState>(id).unwrap();
        let state = state.borrow().clone();
        let mut viewport = Viewport {
            child_rect: Rect::from_wh(0.0, 0.0),
            input_rect: None,
        };

        for child in world.children() {
            world.component(*child).map(|mut layout: FetchComponent<Layout>| {
                let mut layout = layout.borrow_mut();
                layout.current = layout.current.map(|rect| {
                    let rect = Rect {
                        left: -state.scroll.0,
                        top: -state.scroll.1,
                        right: rect.width() - state.scroll.0,
                        bottom: rect.height() - state.scroll.1,
                    };

                    viewport.child_rect = rect;
                    viewport.input_rect = window.input_rect.and_then(|ir| ir.intersect(&rect));
                    rect
                }).or(Some(Rect{
                    left: -state.scroll.0,
                    top: -state.scroll.1,
                    right: -state.scroll.0,
                    bottom: -state.scroll.1,
                }));
            });
        }

        viewport
    }

    fn event(&mut self, id: dag::Id, world: &Ui, context: &mut EventSystemContext) {
        let mut state: FetchComponent<ScrollState> = world.component(id).unwrap();
        let mut state = state.borrow_mut();

        let content_rect = Rect::from_wh(0.0, 0.0);
        let horizontal_rect = Rect::from_wh(0.0, 0.0);
        let vertical_rect = Rect::from_wh(0.0, 0.0);

        state.inner = match state.inner {
            MouseState::Idle | MouseState::HoverContent => {
                if context.cursor.inside(&content_rect) {
                    MouseState::HoverContent
                } else if context.cursor.inside(&horizontal_rect) {
                    MouseState::HoverH
                } else if context.cursor.inside(&vertical_rect) {
                    MouseState::HoverV
                } else {
                    MouseState::Idle
                }
            },
            MouseState::HoverH => {
                if let Event::Press(Key::LeftMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    MouseState::ScrollH(context.cursor.x - horizontal_rect.left)
                } else if context.cursor.inside(&horizontal_rect) {
                    MouseState::HoverH
                } else {
                    MouseState::Idle
                }
            },
            MouseState::HoverV => {
                if let Event::Press(Key::LeftMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    MouseState::ScrollV(context.cursor.y - vertical_rect.top)
                } else if context.cursor.inside(&vertical_rect) {
                    MouseState::HoverV
                } else {
                    MouseState::Idle
                }
            },
            MouseState::ScrollH(x) => {
                context.capture = Capture::CaptureFocus(MouseStyle::Arrow);

                // TODO

                if let Event::Release(Key::LeftMouseButton, _) = context.event {
                    MouseState::Idle
                } else {
                    MouseState::ScrollH(x)
                }
            },
            MouseState::ScrollV(y) => {
                context.capture = Capture::CaptureFocus(MouseStyle::Arrow);

                // TODO

                if let Event::Release(Key::LeftMouseButton, _) = context.event {
                    MouseState::Idle
                } else {
                    MouseState::ScrollV(y)
                }
            },
        };

        if state.inner != MouseState::Idle {
            if let Event::Scroll(dx, dy) = context.event {
                state.scroll.0 = (state.scroll.0 - dx).max(self.content.left).min(self.content.right);
                state.scroll.1 = (state.scroll.1 - dy).max(self.content.top).min(self.content.bottom);
            }
        }
    }
}

impl Widget for Scroll {
    type Result = ();

    fn result(&self, _id: dag::Id) -> Self::Result { }
}