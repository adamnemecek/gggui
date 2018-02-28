use super::*;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum ToggleState {
    Idle,
    Hovered,
    Pressed,
    Triggered,
}

pub struct Toggle<'a, T: Clone+PartialEq+'a> {
    pub normal: Image,
    pub hover: Image,
    pub pressed: Image,
    pub normal_checked: Image,
    pub hover_checked: Image,
    pub pressed_checked: Image,
    pub size: Option<(f32,f32)>,
    pub text: Option<Text>,
    pub text_color: Color,

    value: &'a mut T,
    reference_a: T,
    reference_b: T,
}

impl<'a, T: Clone+PartialEq> Toggle<'a, T> {
    pub fn new(
        value: &'a mut T,
        reference_a: T,
        reference_b: T,
        normal: Image,
        hover: Image,
        pressed: Image,
        normal_checked: Image,
        hover_checked: Image,
        pressed_checked: Image,
        text_color: Color,
    ) -> Self {
        Toggle {
            normal,
            hover,
            pressed,
            normal_checked,
            hover_checked,
            pressed_checked,
            value,
            reference_a,
            reference_b,
            size: None,
            text: None,
            text_color,
        }
    }

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

impl WidgetState for ToggleState { }

impl Default for ToggleState {
    fn default() -> Self {
        ToggleState::Idle
    }
}

impl<'a, T: Clone+PartialEq> Widget for Toggle<'a, T> {
    type Result = bool;
    type State = ToggleState;
    
    fn tabstop() -> bool {
        true
    }

    fn measure(&self, _state: &Self::State, _layout: Option<Rect>) -> Option<Rect> {
        let measured = self.size.map_or_else(
            || self.text.as_ref().map_or(
                Rect{ left: 0.0, top: 0.0, right: 0.0, bottom: 0.0},
                |text| {
                    let text_size = text.measure(None);
                    Rect::from_wh(
                        self.normal.size.width() + 8.0 + text_size.width(), 
                        text_size.height().max(self.normal.size.height())
                    )
                }
            ),
            |size| Rect{ left: 0.0, top: 0.0, right: size.0, bottom: size.1 }
        );
        Some(measured)
    }

    fn event(
        &mut self, 
        state: &mut ToggleState, 
        layout: Rect, 
        cursor: MousePosition,
        event: Event,
        is_focused: bool
    ) -> Capture {
        let mut capture = Capture::None;

        *state = match *state {
            ToggleState::Idle => {
                if is_focused {
                    if let Event::Press(Key::Space, _) = event {
                        capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                        ToggleState::Pressed
                    } else {
                        ToggleState::Idle
                    }
                } else {
                    ToggleState::Idle
                }
            },
            ToggleState::Hovered => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                    ToggleState::Pressed
                } else {
                    ToggleState::Hovered
                }
            },
            ToggleState::Pressed => {
                capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                match event {
                    Event::Release(Key::LeftMouseButton, _) => {
                        if cursor.inside(&layout) {
                            if *self.value == self.reference_a {
                                *self.value = self.reference_b.clone();
                            } else {
                                *self.value = self.reference_a.clone();
                            }
                            ToggleState::Triggered
                        } else {
                            ToggleState::Idle
                        }
                    },
                    Event::Release(Key::Space, _) => {
                        if *self.value == self.reference_a {
                            *self.value = self.reference_b.clone();
                        } else {
                            *self.value = self.reference_a.clone();
                        }
                        ToggleState::Triggered
                    },
                    _ => {
                        ToggleState::Pressed
                    },
                }
            },
            ToggleState::Triggered => {
                ToggleState::Idle
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
        if *state == ToggleState::Triggered {
            *state = ToggleState::Idle;
        }

        let (w, h) = (self.normal.size.width(), self.normal.size.height());
        let layout = Rect {
            left: layout.left,
            top: layout.top,
            right: layout.left + w,
            bottom: layout.top + h,
        };

        if cursor.inside(&layout) {
            if *state == ToggleState::Idle {
                *state = ToggleState::Hovered;
            }
            Hover::HoverActive(MouseStyle::ArrowClickable)
        } else {
            if *state == ToggleState::Hovered {
                *state = ToggleState::Idle;
            }
            Hover::NoHover
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, state: &Self::State, mut layout: Rect, mut submit: F) {
        let tx = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };

        let is_checked = *self.value == self.reference_a;

        let image = match state {
            &ToggleState::Idle =>
                if is_checked { self.normal_checked.clone() } else { self.normal.clone() },

            &ToggleState::Hovered =>
                if is_checked { self.hover_checked.clone() } else { self.hover.clone() },

            &ToggleState::Pressed | &ToggleState::Triggered =>
                if is_checked { self.pressed_checked.clone() } else { self.pressed.clone() },
        };

        let (w, h) = (image.size.width(), image.size.height());

        submit(Primitive::DrawImage(image, Rect {
            left: layout.left,
            top: layout.top,
            right: layout.left + w,
            bottom: layout.top + h,
        }, tx));

        layout.left += w + 8.0;
        
        self.text.as_ref().map(|text| {
            submit(Primitive::DrawText(
                text.clone(), 
                layout, 
                self.text_color
            ))
        });
    }

    fn result(self, state: &Self::State) -> Self::Result {
        *state == ToggleState::Triggered
    }

}