use super::*;

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
    horizontal: bool,
    vertical: bool,
}

impl Scroll {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            content: Rect::from_wh(0.0, 0.0),
            horizontal: false,
            vertical: false,
        }
    }

    pub fn with_horizontal_bar(mut self) -> Self {
        self.horizontal = true;
        self
    }

    pub fn with_vertical_bar(mut self) -> Self {
        self.vertical = true;
        self
    }
}

impl WidgetBase for Scroll {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        world.create_component(id, ScrollState {
            scroll: (0.0, 0.0),
            inner: MouseState::Idle,
        });
        world.create_component(id, Layout {
            current: Some(Rect::from_wh(self.width, self.height)),
            margin: Rect::from_wh(0.0, 0.0),
            padding: Rect { 
                left: 0.0, 
                top: 0.0, 
                right: if self.vertical { style.scroll_vertical.0.image.size.width() } else { 0.0 }, 
                bottom: if self.horizontal { style.scroll_horizontal.0.image.size.width() } else { 0.0 }, 
            },
            constrain_width: Constraint::Fixed,
            constrain_height: Constraint::Fixed,
        });
        world.create_component(id, Clipper {
            rect: Rect::from_wh(self.width, self.height),
            intersect: true,
        });
        world.create_component(id, Drawing {
            primitives: vec![],
        });
    }

    fn update(&mut self, id: dag::Id, world: &Ui, style: &Style, window: Viewport) -> Viewport {
        let mut state = world.component::<ScrollState>(id).unwrap();
        let mut state = state.borrow_mut().clone();

        let layout = world.component::<Layout>(id).unwrap();
        let current = layout.borrow().current.unwrap();
        let padded = layout.borrow().after_padding();

        let mut clipper = world.component::<Clipper>(id).unwrap();
        clipper.borrow_mut().rect = layout.borrow().current.unwrap();

        let mut viewport = Viewport {
            child_rect: padded,
            input_rect: window.input_rect.and_then(|ir| ir.intersect(&padded)),
        };

        for child in world.children() {
            world.component(*child).map(|mut layout: FetchComponent<Layout>| {
                let mut layout = layout.borrow_mut();
                layout.current = layout.current.map(|rect| {
                    let rect = Rect {
                        left: viewport.child_rect.left-state.scroll.0,
                        top: viewport.child_rect.top-state.scroll.1,
                        right: viewport.child_rect.left-state.scroll.0 + rect.width(),
                        bottom: viewport.child_rect.top-state.scroll.1 + rect.height(),
                    };

                    viewport.child_rect = rect;
                    rect
                }).or_else(|| {
                    let rect = Rect{
                        left: viewport.child_rect.left-state.scroll.0,
                        top: viewport.child_rect.top-state.scroll.1,
                        right: viewport.child_rect.left-state.scroll.0 + viewport.child_rect.width(),
                        bottom: viewport.child_rect.top-state.scroll.1 + viewport.child_rect.height(),
                    };

                    viewport.child_rect = rect;
                    Some(rect)
                });

                self.content = layout.current.map(|content| Rect {
                    left: 0.0,
                    top: 0.0,
                    right:  (content.width() - current.width()).max(0.0),
                    bottom: (content.height() - current.height()).max(0.0),
                }).unwrap();
            });
        }

        state.scroll.0 = (state.scroll.0).max(self.content.left).min(self.content.right);
        state.scroll.1 = (state.scroll.1).max(self.content.top).min(self.content.bottom);

        let vertical_rect = {
            let mut bar = Rect { 
                left: padded.right, 
                top: current.top, 
                right: current.right, 
                bottom: padded.bottom 
            };
            let handle_range = 
                handle_range(bar.top, state.scroll.1, bar.height(), self.content.top, self.content.bottom);
            bar.top = handle_range.0;
            bar.bottom = handle_range.1;
            bar
        };

        let horizontal_rect = {
            let mut bar = Rect { 
                left: current.left, 
                top: padded.bottom, 
                right: padded.right, 
                bottom: current.bottom 
            };
            let handle_range = 
                handle_range(bar.left, state.scroll.0, bar.width(), self.content.left, self.content.right);
            bar.left = handle_range.0;
            bar.right = handle_range.1;
            bar
        };

        let mut drawing = world.component::<Drawing>(id).unwrap();
        let mut drawing = drawing.borrow_mut();
        drawing.primitives.clear();
        
        if self.horizontal {
            drawing.primitives.push(Primitive::Draw9(style.scroll_horizontal.0.clone(), Rect { 
                left: current.left, 
                top: padded.bottom, 
                right: padded.right, 
                bottom: current.bottom 
            }, Color::white()));
            drawing.primitives.push(Primitive::Draw9(style.scroll_horizontal.1.clone(), horizontal_rect, Color::white()));
        }

        if self.vertical {
            drawing.primitives.push(Primitive::Draw9(style.scroll_vertical.0.clone(), Rect { 
                left: padded.right, 
                top: current.top, 
                right: current.right, 
                bottom: padded.bottom 
            }, Color::white()));
            drawing.primitives.push(Primitive::Draw9(style.scroll_vertical.1.clone(), vertical_rect, Color::white()));
        }

        viewport
    }

    fn event(&mut self, id: dag::Id, world: &Ui, _style: &Style, context: &mut EventSystemContext) {
        let mut state: FetchComponent<ScrollState> = world.component(id).unwrap();
        let mut state = state.borrow_mut();

        let layout: FetchComponent<Layout> = world.component(id).unwrap();
        let layout = layout.borrow();

        let current = layout.current.unwrap();
        let padded = layout.after_padding();

        let content_rect = padded;

        let vertical_rect = {
            let mut bar = Rect { 
                left: padded.right, 
                top: current.top, 
                right: current.right, 
                bottom: padded.bottom 
            };
            let handle_range = 
                handle_range(bar.top, state.scroll.1, bar.height(), self.content.top, self.content.bottom);
            bar.top = handle_range.0;
            bar.bottom = handle_range.1;
            bar
        };

        let horizontal_rect = {
            let mut bar = Rect { 
                left: current.left, 
                top: padded.bottom, 
                right: padded.right, 
                bottom: current.bottom 
            };
            let handle_range = 
                handle_range(bar.left, state.scroll.0, bar.width(), self.content.left, self.content.right);
            bar.left = handle_range.0;
            bar.right = handle_range.1;
            bar
        };

        state.inner = match state.inner {
            MouseState::Idle | MouseState::HoverContent => {
                if context.cursor.inside(&horizontal_rect) {
                    MouseState::HoverH
                } else if context.cursor.inside(&vertical_rect) {
                    MouseState::HoverV
                } else if context.cursor.inside(&content_rect) {
                    MouseState::HoverContent
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

                let mut bar = Rect { 
                    left: current.left, 
                    top: padded.bottom, 
                    right: padded.right, 
                    bottom: current.bottom 
                };
                state.scroll.0 = handle_to_scroll(
                    bar.left, 
                    context.cursor.x-x, 
                    bar.width(), 
                    self.content.left,
                    self.content.right
                );

                if let Event::Release(Key::LeftMouseButton, _) = context.event {
                    MouseState::Idle
                } else {
                    MouseState::ScrollH(x)
                }
            },
            MouseState::ScrollV(y) => {
                context.capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = Rect { 
                    left: padded.right, 
                    top: current.top, 
                    right: current.right, 
                    bottom: padded.bottom 
                };
                state.scroll.1 = handle_to_scroll(
                    bar.top, 
                    context.cursor.y-y, 
                    bar.height(), 
                    self.content.top,
                    self.content.bottom
                );

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

fn handle_to_scroll(offset: f32, x: f32, length: f32, min: f32, max: f32) -> f32 {
    let content = max-min;
    let range = handle_range(offset, max, length, min, max);
    let pos = (x-offset)/(range.0-offset);
    (min+pos*content).max(min).min(max).floor()
}

fn handle_range(offset: f32, x: f32, length: f32, min: f32, max: f32) -> (f32, f32) { 
    let content = max-min;
    let size = length * (length / (length+content));
    let start = length * ((x-min) / (length+content));
    ((offset+start).floor(), (offset+start+size).floor())
}

impl Widget for Scroll {
    type Result = ();

    fn result(&mut self, _id: dag::Id) -> Self::Result { }
}