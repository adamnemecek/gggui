use super::*;

pub struct WindowProperties {
    pub default_size: Rect,
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
    HoverFrame(f32, f32),
    Drag(f32, f32),
    Resize(f32, f32),
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

    fn layout(
        &mut self, 
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
                    capture = Capture::CaptureFocus;
                    WindowControllerState::Drag(x, y)
                } else {
                    WindowControllerState::HoverContent(x, y)
                }
            },
            WindowControllerState::HoverFrame(x, y) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus;
                    WindowControllerState::Resize(x, y)
                } else {
                    WindowControllerState::HoverFrame(x, y)
                }
            },
            WindowControllerState::Drag(x, y) => {
                capture = Capture::CaptureFocus;

                if self.properties.draggable {
                    *self.rect = self.rect.size().translate(cursor.x - x, cursor.y - y);
                }

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    WindowControllerState::Idle
                } else {
                    WindowControllerState::Drag(x, y)
                }
            },
            WindowControllerState::Resize(x, y) => {
                capture = Capture::CaptureFocus;
                if let Event::Release(Key::LeftMouseButton, _) = event {
                    WindowControllerState::Idle
                } else {
                    WindowControllerState::Resize(x, y)
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
    ) -> bool {
        let content = self.properties.background
            .content_rect(layout)
            .inset(1.0, 1.0)
            .unwrap_or(layout);

        let busy = match state {
            &mut WindowControllerState::Drag(_, _) |
            &mut WindowControllerState::Resize(_, _) => true,
            &mut _ => false,
        };

        if cursor.inside(&content) {
            if !busy {
                *state = WindowControllerState::HoverContent(
                    cursor.x - layout.left, 
                    cursor.y - layout.top
                );
            }
            true
        } else if cursor.inside(&layout) {
            if !busy {
                *state = WindowControllerState::HoverFrame(
                    cursor.x - layout.left, 
                    cursor.y - layout.top
                );
            }
            true
        } else {
            if !busy {
                *state = WindowControllerState::Idle;
            }
            false
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

    fn childs(&self, _: &Self::State, layout: Rect) -> ChildType {
        match &self.properties.background {
            &Background::Patch(ref patch) => {
                ChildType::Intersect(patch.content_rect(layout))
            },
            &_ => {
                ChildType::Intersect(layout)
            },
        }
    }

    fn result(self, _state: &Self::State) -> Self::Result {
        ()
    }
}