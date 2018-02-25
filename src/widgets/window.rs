use super::*;

#[derive(Clone)]
pub struct WindowProperties {
    pub default_size: Rect,
    pub minimum_size: Rect,
    pub resizable: bool,
    pub draggable: bool,
    pub centered: bool,
    pub modal: bool,
    pub background: Background,
}

pub struct WindowController<'a> {
    properties: WindowProperties,
    rect: &'a mut Rect,
}

#[derive(Clone)]
pub enum WindowControllerState {
    Idle,
    HoverContent(f32, f32),
    HoverFrame(MouseStyle),
    Drag(f32, f32),
    Resize(MouseStyle),
}

impl WidgetState for WindowControllerState { }

impl<'a> WindowController<'a> {
    pub fn new(properties: WindowProperties, rect: &'a mut Rect) -> Self { 
        WindowController {
            properties,
            rect,
        }
    }
}

impl<'a> Widget for WindowController<'a> {
    type Result = ();
    type State = WindowControllerState;

    fn default() -> Self::State {
        WindowControllerState::Idle
    }

    fn measure(&self, _: &Self::State, _: Option<Rect>) -> Option<Rect> {
        Some(self.rect.clone())
    }

    fn estimate(
        &self, 
        _: &Self::State, 
        layout: Rect, 
        _: WidgetMeasure
    ) -> Rect {
        self.properties.background.content_rect(layout)
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        _: Rect, 
        cursor: MousePosition,
        event: Event,
        _: bool
    ) -> Capture {
        let mut capture = Capture::None;

        *state = match *state {
            WindowControllerState::Idle => {
                WindowControllerState::Idle
            },
            WindowControllerState::HoverContent(x, y) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    WindowControllerState::Drag(x, y)
                } else {
                    WindowControllerState::HoverContent(x, y)
                }
            },
            WindowControllerState::HoverFrame(anchor) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(anchor);
                    WindowControllerState::Resize(anchor)
                } else {
                    WindowControllerState::HoverFrame(anchor)
                }
            },
            WindowControllerState::Drag(x, y) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                if self.properties.draggable {
                    *self.rect = self.rect.size().translate(cursor.x - x, cursor.y - y);
                }

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    WindowControllerState::Idle
                } else {
                    WindowControllerState::Drag(x, y)
                }
            },
            WindowControllerState::Resize(anchor) => {
                capture = Capture::CaptureFocus(anchor);

                let min_w = self.properties.minimum_size.width();
                let min_h = self.properties.minimum_size.height();

                match anchor {
                    MouseStyle::ResizeN => {
                        self.rect.top = cursor.y.min(self.rect.bottom - min_h);
                    },
                    MouseStyle::ResizeS => {
                        self.rect.bottom = cursor.y.max(self.rect.top + min_h);
                    },
                    MouseStyle::ResizeW => {
                        self.rect.left = cursor.x.min(self.rect.right - min_w);
                    },
                    MouseStyle::ResizeE => {
                        self.rect.right = cursor.x.max(self.rect.left + min_w);
                    },
                    MouseStyle::ResizeNw => {
                        self.rect.top = cursor.y.min(self.rect.bottom - min_h);
                        self.rect.left = cursor.x.min(self.rect.right - min_w);
                    },
                    MouseStyle::ResizeNe => {
                        self.rect.top = cursor.y.min(self.rect.bottom - min_h);
                        self.rect.right = cursor.x.max(self.rect.left + min_w);
                    },
                    MouseStyle::ResizeSw => {
                        self.rect.bottom = cursor.y.max(self.rect.top + min_h);
                        self.rect.left = cursor.x.min(self.rect.right - min_w);
                    },
                    MouseStyle::ResizeSe => {
                        self.rect.bottom = cursor.y.max(self.rect.top + min_h);
                        self.rect.right = cursor.x.max(self.rect.left + min_w);
                    },
                    _ => {
                        unreachable!();
                    },
                }

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    WindowControllerState::Idle
                } else {
                    WindowControllerState::Resize(anchor)
                }
            },
        };

        capture
    }

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition
    ) -> Hover {
        let content = self.properties.background
            .content_rect(layout)
            .inset(1.0, 1.0)
            .unwrap_or(layout);

        let busy = match state {
            &mut WindowControllerState::Drag(_, _) |
            &mut WindowControllerState::Resize(_) => true,
            &mut _ => false,
        };

        if cursor.inside(&content) {
            if !busy {
                *state = WindowControllerState::HoverContent(
                    cursor.x - layout.left, 
                    cursor.y - layout.top
                );
            }
            Hover::HoverIdle
        } else if cursor.inside(&layout) {
            if !busy && self.properties.resizable {
                let hover_left = cursor.x < content.left + 4.0;
                let hover_right = cursor.x > content.right - 4.0;
                let hover_top = cursor.y < content.top + 4.0;
                let hover_bottom = cursor.y > content.bottom - 4.0;
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
                    *state = WindowControllerState::HoverFrame(anchor.unwrap());
                    Hover::HoverActive(anchor.unwrap())
                } else {
                    *state = WindowControllerState::Idle;
                    Hover::NoHover
                }
            } else {
                Hover::HoverIdle
            }
        } else {
            if !busy {
                *state = WindowControllerState::Idle;
            }
            Hover::NoHover
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, _state: &Self::State, layout: Rect, mut submit: F) { 
        match &self.properties.background {
            &Background::None => (),
            &Background::Color(ref color) => {
                submit(Primitive::DrawRect(layout, *color));
            },
            &Background::Image(ref image) => {
                submit(Primitive::DrawImage(image.clone(), layout, Color::white()));
            },
            &Background::Patch(ref patch) => {
                submit(Primitive::Draw9(patch.clone(), layout, Color::white()));
            },
        }
    }

    fn child_area(&self, _: &Self::State, layout: Rect) -> ChildArea {
        match &self.properties.background {
            &Background::Patch(ref patch) => {
                ChildArea::ConfineContentAndInput(patch.content_rect(layout))
            },
            &_ => {
                ChildArea::ConfineContentAndInput(layout)
            },
        }
    }

    fn result(self, _state: &Self::State) -> Self::Result {
        ()
    }
}