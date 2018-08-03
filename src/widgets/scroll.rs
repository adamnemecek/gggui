use cassowary::WeightedRelation::EQ;
use cassowary::strength::REQUIRED;
use super::*;

#[derive(Clone,Copy,PartialEq)]
pub struct ScrollState {
    scroll: (f32, f32),
    scroll_vars: (cassowary::Variable, cassowary::Variable),
    content: (cassowary::Variable, cassowary::Variable),
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
    content: Rect,
    horizontal: bool,
    vertical: bool,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            content: Rect::zero(),
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
        let scroll_h = cassowary::Variable::new();
        let scroll_v = cassowary::Variable::new();
        let content_w = cassowary::Variable::new();
        let content_h = cassowary::Variable::new();

        let layout = Layout::new()
            .with_margins(Rect {
                left: 0.0, 
                top: 0.0, 
                right: if self.vertical { style.scroll_vertical.0.image.size.width() } else { 0.0 }, 
                bottom: if self.horizontal { style.scroll_horizontal.0.image.size.width() } else { 0.0 }, 
            })
            .with_detached_margin()
            .with_constraints(|layout| vec![
                layout.margin_left |EQ(REQUIRED)| layout.left - scroll_h,
                layout.margin_top |EQ(REQUIRED)| layout.top - scroll_v,
                layout.margin_left + content_w |EQ(REQUIRED)| layout.margin_right,
                layout.margin_top + content_h |EQ(REQUIRED)| layout.margin_bottom,
            ])
            .with_edit(scroll_h, &mut world.layout_solver)
            .with_edit(scroll_v, &mut world.layout_solver);

        world.create_component(id, ScrollState {
            scroll: (0.0, 0.0),
            scroll_vars: (scroll_h, scroll_v),
            content: (content_w, content_h),
            inner: MouseState::Idle,
        });
        world.create_component(id, layout);
        world.create_component(id, Clipper::new(Rect::zero()).with_updater(|clip, layout| {
            let current = layout.as_ref().unwrap().current.clone();
            if current.is_some() {
                clip.rect = current.unwrap();
            }
        }));
        world.create_component(id, Drawing::new());      
    }

    fn update(&mut self, id: dag::Id, world: &mut Ui, style: &Style, window: Option<Rect>) -> Option<Rect> {
        let mut state = world.component::<ScrollState>(id).unwrap();
        let mut state = state.borrow_mut().clone();

        let (current, padded) = {
            let layout = world.component::<Layout>(id).unwrap();
            let layout = layout.borrow();
            if layout.current.is_none() {
                return None;
            }

            let current = layout.current.unwrap();
            let padded = current.after_padding(layout.margin());
            (current, padded)
        };

        self.content = Rect::from_wh(
            (world.layout_solver.get_value(state.content.0) as f32 - current.width()).max(0.0),
            (world.layout_solver.get_value(state.content.1) as f32 - current.height()).max(0.0)
        );

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

        window.and_then(|ir| ir.intersect(&padded))
    }

    fn event(&mut self, id: dag::Id, world: &mut Ui, _style: &Style, context: &mut EventSystemContext) {
        let mut state: FetchComponent<ScrollState> = world.component(id).unwrap();
        let mut state = state.borrow_mut();

        let layout: FetchComponent<Layout> = world.component(id).unwrap();
        let layout = layout.borrow();

        if layout.current.is_none() {
            return;
        }

        let current = layout.current.unwrap();
        let padded = current.after_padding(layout.margin());

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

                world.layout_solver.suggest_value(state.scroll_vars.0, state.scroll.0 as f64).ok();

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

                world.layout_solver.suggest_value(state.scroll_vars.1, state.scroll.1 as f64).ok();

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
                world.layout_solver.suggest_value(state.scroll_vars.0, state.scroll.0 as f64).ok();

                state.scroll.1 = (state.scroll.1 - dy).max(self.content.top).min(self.content.bottom);
                world.layout_solver.suggest_value(state.scroll_vars.1, state.scroll.1 as f64).ok();
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