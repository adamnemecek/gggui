use super::*;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum ButtonState {
    Idle,
    Hovered,
    Pressed,    
    Triggered,
}

pub struct Button {
    pub normal: Patch,
    pub hover: Patch,
    pub pressed: Patch,
    pub size: Option<Rect>,
    pub text: Option<Text>,
    pub text_color: Color,
    pub triggered: bool,
}

impl Button {
    pub fn size(mut self, size: Rect) -> Self {
        self.size = Some(size);
        self
    }

    pub fn text(mut self, text: Text) -> Self {
        self.text = Some(text);
        self
    }

    pub fn text_color(mut self, text_color: Color) -> Self {
        self.text_color = text_color;
        self
    }
}

impl WidgetState for ButtonState { }

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Idle
    }
}

impl Widget for Button {
    type Result = bool;
    type State = ButtonState;

    fn tabstop() -> bool {
        true
    }

    fn measure(&self, _state: &Self::State, _layout: Option<Rect>) -> Option<Rect> {
        let measured = self.size.unwrap_or_else(
            || self.normal.measure_with_content(
                self.text.as_ref().map_or(
                    Rect{ left: 0.0, top: 0.0, right: 0.0, bottom: 0.0},
                    |text| {
                        text.measure(None)
                    }
                )
            )
        );
        Some(measured)
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

        *state = match *state {
            ButtonState::Idle => {
                if is_focused {
                    if let Event::Press(Key::Space, _) = event {
                        capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                        ButtonState::Pressed
                    } else {
                        ButtonState::Idle
                    }
                } else {
                    ButtonState::Idle
                }
            },
            ButtonState::Hovered => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                    ButtonState::Pressed
                } else {
                    ButtonState::Hovered
                }
            },
            ButtonState::Pressed => {
                capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                match event {
                    Event::Release(Key::LeftMouseButton, _) => {
                        if cursor.inside(&layout) {
                            self.triggered = true;
                            ButtonState::Triggered
                        } else {
                            ButtonState::Idle
                        }
                    },
                    Event::Release(Key::Space, _) => {
                        self.triggered = true;
                        ButtonState::Triggered
                    },
                    _ => {
                        ButtonState::Pressed
                    },
                }
            },
            ButtonState::Triggered => {
                ButtonState::Triggered
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
        if *state == ButtonState::Triggered {
            *state = ButtonState::Idle;
        }

        if cursor.inside(&layout) {
            if *state == ButtonState::Idle {
                *state = ButtonState::Hovered;
            }
            Hover::HoverActive(MouseStyle::ArrowClickable)
        } else {
            if *state == ButtonState::Hovered {
                *state = ButtonState::Idle;
            }
            Hover::NoHover
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, state: &Self::State, layout: Rect, mut submit: F) {
        let tx = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };

        let patch = match state {
            &ButtonState::Idle =>
                self.normal.clone(),

            &ButtonState::Hovered =>
                self.hover.clone(),

            &ButtonState::Pressed | &ButtonState::Triggered =>
                self.pressed.clone(),
        };

        submit(Primitive::Draw9(patch.clone(), layout, tx));
        
        self.text.as_ref().map(|text| {
            submit(Primitive::DrawText(
                text.clone(), 
                patch.content_rect(layout), 
                self.text_color
            ))
        });
    }

    fn result(self, _: &Self::State) -> Self::Result {
        self.triggered
    }

}