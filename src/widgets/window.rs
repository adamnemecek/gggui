use super::*;
use cassowary::strength::*;
use cassowary::WeightedRelation::*;

#[derive(Clone)]
pub enum WindowState {
    Idle,
    HoverContent(f32, f32),
    HoverFrame(MouseStyle),
    Drag(f32, f32),
    Resize(MouseStyle),
}

pub struct Window {
    min_size: Rect,
    open: bool,

    draggable: bool,
    resizable: bool,
}

impl Window {
    pub fn new(min_size: Rect, open: bool) -> Self {
        Self {
            min_size, 
            open,
            draggable: true,
            resizable: true,
        }
    }

    pub fn sized(width: f32, height: f32) -> Self {
        Self {
            min_size: Rect::from_wh(width, height).translate(100.0, 100.0), 
            open: true,
            draggable: true,
            resizable: true,
        }
    }
}

impl WidgetBase for Window {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        let layout = Layout::new()
            .with_margins(style.window.margin())
            .with_constraints(|layout| vec![
                layout.top |GE(REQUIRED)| world.viewport_top,
                layout.left |GE(REQUIRED)| world.viewport_left,
                layout.bottom |LE(REQUIRED)| world.viewport_bottom,
                layout.right |LE(REQUIRED)| world.viewport_right,
                layout.width |GE(REQUIRED)| self.min_size.width() as f64,
                layout.height |GE(REQUIRED)| self.min_size.height() as f64
            ])
            .as_editable(&mut world.layout_solver);

        world.create_component(id, layout);
        world.create_component(id, WidgetBackground{
            normal: Background::Patch(style.window.clone(), 1.0),
            hover: Background::Patch(style.window.clone(), 1.0),
            click: Background::Patch(style.window.clone(), 1.0),
        });
        world.create_component(id, WindowState::Idle);
    }

    fn update(&mut self, id: dag::Id, world: &mut Ui, style: &Style, input: Option<Rect>) -> Option<Rect> {
        let layout = world.component::<Layout>(id).unwrap();     

        let content = layout.borrow().current
            .map(|rect| style.window.content_rect(rect))
            .unwrap_or(Rect::zero());

        input.and_then(|ir| ir.intersect(&content))
    }

    fn event(&mut self, id: dag::Id, world: &mut Ui, style: &Style, context: &mut EventSystemContext) {
        let mut layout = world.component::<Layout>(id).unwrap();
        let mut layout = layout.borrow_mut();

        let mut state = world.component::<WindowState>(id).unwrap();
        let mut state = state.borrow_mut();

        if layout.current.is_none() {
            return;
        }

        let rect = layout.current.unwrap();

        let content = layout.current.map(|rect| style.window.content_rect(rect)).unwrap_or(Rect::zero());

        let busy = match state.clone() {
            WindowState::Drag(_, _) |
            WindowState::Resize(_) => true,
            _ => false,
        };

        if context.cursor.inside(&content) {
            if !busy {
                *state = WindowState::HoverContent(
                    context.cursor.x - rect.left, 
                    context.cursor.y - rect.top
                );
            }
        } else if context.cursor.inside(&rect) {
            if !busy && self.resizable {
                let hover_left = context.cursor.x < content.left + 4.0;
                let hover_right = context.cursor.x > content.right - 4.0;
                let hover_top = context.cursor.y < content.top + 4.0;
                let hover_bottom = context.cursor.y > content.bottom - 4.0;
                let anchor = match (hover_left, hover_right, hover_top, hover_bottom) {
                    (true,  false, true,  false) => Some(MouseStyle::ResizeNw),
                    (false, false, true,  false) => Some(MouseStyle::ResizeN),
                    (false, true,  true,  false) => Some(MouseStyle::ResizeNe),
                    (true,  false, false, false) => Some(MouseStyle::ResizeW),
                    (false, true,  false, false) => Some(MouseStyle::ResizeE),
                    (true,  false, false, true ) => Some(MouseStyle::ResizeSw),
                    (false, false, false, true ) => Some(MouseStyle::ResizeS),
                    (false, true,  false, true ) => Some(MouseStyle::ResizeSe),
                    _ => None,
                };

                if anchor.is_some() {
                    *state = WindowState::HoverFrame(anchor.unwrap());
                    context.style = anchor.unwrap();
                } else {
                    *state = WindowState::Idle;
                }
            }
        } else {
            if !busy {
                *state = WindowState::Idle;
            }
        }

        *state = match state.clone() {
            WindowState::Idle => {
                WindowState::Idle
            },
            WindowState::HoverContent(x, y) => {
                if let Event::Press(Key::LeftMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    WindowState::Drag(x, y)
                } else {
                    WindowState::HoverContent(x, y)
                }
            },
            WindowState::HoverFrame(anchor) => {
                if let Event::Press(Key::LeftMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(anchor);
                    WindowState::Resize(anchor)
                } else {
                    WindowState::HoverFrame(anchor)
                }
            },
            WindowState::Drag(x, y) => {
                context.capture = Capture::CaptureFocus(MouseStyle::Arrow);

                if self.draggable {
                    let s = &mut world.layout_solver;
                    s.suggest_value(layout.left, (context.cursor.x - x) as f64).ok();
                    s.suggest_value(layout.top, (context.cursor.y - y) as f64).ok();
                    s.suggest_value(layout.width, rect.width() as f64).ok();
                    s.suggest_value(layout.height, rect.height() as f64).ok();
                }

                if let Event::Release(Key::LeftMouseButton, _) = context.event {
                    WindowState::Idle
                } else {
                    WindowState::Drag(x, y)
                }
            },
            WindowState::Resize(anchor) => {
                context.capture = Capture::CaptureFocus(anchor);

                let min_w = self.min_size.width();
                let min_h = self.min_size.height();

                let s = &mut world.layout_solver;

                let mut new_rect = rect;

                match anchor {
                    MouseStyle::ResizeN => {
                        new_rect.top = context.cursor.y.min(rect.bottom - min_h);
                    },
                    MouseStyle::ResizeS => {
                        new_rect.bottom = context.cursor.y.max(rect.top + min_h);
                    },
                    MouseStyle::ResizeW => {
                        new_rect.left = context.cursor.x.min(rect.right - min_w);
                    },
                    MouseStyle::ResizeE => {
                        new_rect.right = context.cursor.x.max(rect.left + min_w);
                    },
                    MouseStyle::ResizeNw => {
                        new_rect.top =  context.cursor.y.min(rect.bottom - min_h);
                        new_rect.left = context.cursor.x.min(rect.right - min_w);
                    },
                    MouseStyle::ResizeNe => {
                        new_rect.top = context.cursor.y.min(rect.bottom - min_h);
                        new_rect.right = context.cursor.x.max(rect.left + min_w);
                    },
                    MouseStyle::ResizeSw => {
                        new_rect.bottom = context.cursor.y.max(rect.top + min_h);
                        new_rect.left = context.cursor.x.min(rect.right - min_w);
                    },
                    MouseStyle::ResizeSe => {
                        new_rect.bottom = context.cursor.y.max(rect.top + min_h);
                        new_rect.right = context.cursor.x.max(rect.left + min_w);
                    },
                    _ => {
                        unreachable!();
                    },
                }

                s.suggest_value(layout.left, new_rect.left as f64).ok();
                s.suggest_value(layout.top, new_rect.top as f64).ok();
                s.suggest_value(layout.width, new_rect.width() as f64).ok();
                s.suggest_value(layout.height, new_rect.height() as f64).ok();

                if let Event::Release(Key::LeftMouseButton, _) = context.event {
                    WindowState::Idle
                } else {
                    WindowState::Resize(anchor)
                }
            },
        };

        layout.current = Some(rect);
    }
}

impl Widget for Window {
    type Result = bool;

    fn result(&mut self, _id: dag::Id) -> Self::Result { 
        self.open
    }
}