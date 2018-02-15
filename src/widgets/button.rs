use super::*;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum ButtonState {
    Pressed,
    Unpressed,
    Hovered,
    Triggered,
}

pub struct Button {
    pub normal: Patch,
    pub hover: Patch,
    pub pressed: Patch,
    pub size: Option<(f32,f32)>,
    pub text: Option<Text>,
    pub text_color: Color,
}

impl Button {
    pub fn size(mut self, size: (f32, f32)) -> Self {
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

impl Widget for Button {
    type Result = bool;
    type State = ButtonState;

    fn default() -> Self::State {
        ButtonState::Unpressed
    }

    fn tabstop() -> bool {
        true
    }

    fn measure(&self, _state: &Self::State) -> Option<Rect> {
        let measured = self.size.map_or_else(
            || self.normal.measure_with_content(
                self.text.as_ref().map_or(
                    Rect{ left: 0.0, top: 0.0, right: 0.0, bottom: 0.0},
                    |text| {
                        text.measure(None)
                    }
                )
            ),
            |size| Rect{ left: 0.0, top: 0.0, right: size.0, bottom: size.1 }
        );
        Some(measured)
    }

    fn layout(&mut self, _state: &Self::State, layout: Rect, _child: Option<Rect>) -> Rect {
        layout
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
            ButtonState::Unpressed => {
                if is_focused {
                    if let Event::Press(Key::Space, _) = event {
                        capture = Capture::CaptureFocus;
                        ButtonState::Pressed
                    } else {
                        ButtonState::Unpressed
                    }
                } else {
                    ButtonState::Unpressed
                }
            },
            ButtonState::Hovered => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus;
                    ButtonState::Pressed
                } else {
                    ButtonState::Hovered
                }
            },
            ButtonState::Pressed => {
                capture = Capture::CaptureFocus;
                match event {
                    Event::Release(Key::LeftMouseButton, _) => {
                        if cursor.inside(&layout) {
                           ButtonState::Triggered
                        } else {
                            ButtonState::Unpressed
                        }
                    },
                    Event::Release(Key::Space, _) => {
                        ButtonState::Triggered
                    },
                    _ => {
                        ButtonState::Pressed
                    },
                }
            },
            ButtonState::Triggered => {
                ButtonState::Unpressed
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
        if *state == ButtonState::Triggered {
            *state = ButtonState::Unpressed;
        }

        if cursor.inside(&layout) {
            if *state == ButtonState::Unpressed {
                *state = ButtonState::Hovered;
            }
            true
        } else {
            if *state == ButtonState::Hovered {
                *state = ButtonState::Unpressed;
            }
            false
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, state: &Self::State, layout: Rect, mut submit: F) {
        let tx = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };

        let patch = match state {
            &ButtonState::Unpressed =>
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
                self.text_color, 
                true
            ))
        });
    }

    fn result(self, state: &Self::State) -> Self::Result {
        *state == ButtonState::Triggered
    }

}