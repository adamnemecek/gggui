use super::*;
use clipboard::{ClipboardContext,ClipboardProvider};
use std::time::Instant;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum InputState {
    Selecting(usize, usize, Instant),
    Selected(usize, usize, Instant),
    Hovered,
    Idle,
}

pub struct Input<'a> {
    pub patch: Patch,
    pub size: Option<Rect>,
    buffer: &'a mut String,
    text: Option<Text>,
    focus: bool,
    pub text_color: Color,
}

impl<'a> Input<'a> {
    pub fn new(
        patch: Patch,
        text: &'a mut String, 
        font: Font, 
        text_size: f32, 
        text_color: Color
    ) -> Self {
        let render_text = Text{ text: text.clone(), font, size: text_size };

        Input {
            patch,
            size: None,
            buffer: text,
            text: Some(render_text),
            focus: false,
            text_color
        }
    }

    pub fn size(mut self, size: Rect) -> Self {
        self.size = Some(size);
        self
    }

    pub fn text_color(mut self, text_color: Color) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn take_focus(mut self) -> Self {
        self.focus = true;
        self
    }
}

impl WidgetState for InputState { }

impl<'a> Widget for Input<'a> {
    type Result = &'a str;
    type State = InputState;

    fn default() -> Self::State {
        InputState::Idle
    }

    fn tabstop() -> bool {
        true
    }

    fn autofocus(&self) -> bool {
        self.focus
    }

    fn measure(&self, _state: &Self::State) -> Option<Rect> {
        self.size
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

        let content = self.patch.content_rect(layout);

        let text = self.text.as_mut().unwrap();

        let relative_cursor = (cursor.x - content.left, cursor.y - content.top);

        *state = match *state {
            InputState::Idle => {
                if is_focused {
                    InputState::Selected(self.buffer.len(), self.buffer.len(), Instant::now())
                } else {
                    InputState::Idle
                }
            },
            InputState::Hovered => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus;
                    let hit = text.hitdetect(relative_cursor, None);
                    InputState::Selecting(hit, hit, Instant::now())
                } else {
                    InputState::Hovered
                }
            },
            InputState::Selecting(from, to, since) => {
                capture = Capture::CaptureFocus;
                if let Event::Release(Key::LeftMouseButton, _) = event {
                    InputState::Selected(from, to, since)
                } else {
                    let hit = text.hitdetect(relative_cursor, None);
                    if let Event::Idle = event {
                        InputState::Selecting(from, hit, since)
                    } else {
                        InputState::Selecting(from, hit, Instant::now())
                    }
                }
            },
            InputState::Selected(from, to, since) => {
                match event {
                    Event::Press(Key::LeftMouseButton, _) => {
                        if cursor.inside(&layout) {
                            capture = Capture::CaptureFocus;
                            let hit = text.hitdetect(relative_cursor, None);
                            InputState::Selecting(hit, hit, Instant::now())
                        } else {
                            InputState::Idle
                        }
                    },

                    Event::Text(c) => {
                        match c {
                            '\x08' => {
                                let (from, to) = (from.min(to), from.max(to));

                                if to > from {
                                    let tail = self.buffer.split_off(from);
                                    self.buffer.push_str(tail.split_at(to-from).1);
                                    text.text = self.buffer.clone();

                                    InputState::Selected(from, from, Instant::now())
                                } else if from > 0 {
                                    let tail = self.buffer.split_off(from-1);
                                    self.buffer.push_str(tail.split_at(1).1);
                                    text.text = self.buffer.clone();

                                    InputState::Selected(from-1, from-1, Instant::now())
                                } else {
                                    InputState::Selected(from, to, Instant::now())
                                }                            
                            },
                            '\x7f' => {
                                let (from, to) = (from.min(to), from.max(to));

                                let tail = self.buffer.split_off(from);
                                if to > from {
                                    self.buffer.push_str(tail.split_at(to-from).1);
                                } else if tail.len() > 0 {
                                    self.buffer.push_str(tail.split_at(1).1);
                                }
                                text.text = self.buffer.clone();

                                InputState::Selected(from, from, Instant::now())
                            },
                            c => if c.is_control() {
                                InputState::Selected(from, to, since)
                            } else {
                                let (from, to) = (from.min(to), from.max(to));

                                let mut tail = self.buffer.split_off(from);
                                self.buffer.push(c);
                                if to > from {
                                    self.buffer.push_str(&tail.split_off(to-from));
                                } else {
                                    self.buffer.push_str(&tail);
                                }
                                text.text = self.buffer.clone();
                                InputState::Selected(from+1, from+1, Instant::now())
                            }
                        }
                    },

                    Event::Press(Key::C, Modifiers{ ctrl: true, alt: false, shift: false, logo: false }) => {
                        let (a, b) = (from.min(to), from.max(to));
                        let copy_text = self.buffer[a..b].to_string();
                        ClipboardContext::new().and_then(|mut cc| {
                            cc.set_contents(copy_text)
                        }).ok();

                        InputState::Selected(from, to, since)
                    },

                    Event::Press(Key::X, Modifiers{ ctrl: true, alt: false, shift: false, logo: false }) => {
                        let (from, to) = (from.min(to), from.max(to));
                        let cut_text = self.buffer[from..to].to_string();
                        ClipboardContext::new().and_then(|mut cc| {
                            cc.set_contents(cut_text)
                        }).ok();

                        let tail = self.buffer.split_off(from);
                        if to > from {
                            self.buffer.push_str(tail.split_at(to-from).1);
                        } else if tail.len() > 0 {
                            self.buffer.push_str(tail.split_at(1).1);
                        }
                        text.text = self.buffer.clone();

                        InputState::Selected(from, from, since)
                    },

                    Event::Press(Key::V, Modifiers{ ctrl: true, alt: false, shift: false, logo: false }) => {
                        let (from, to) = (from.min(to), from.max(to));
                        let paste_text = ClipboardContext::new().and_then(|mut cc| {
                            cc.get_contents()
                        }).ok();

                        if let Some(paste_text) = paste_text {
                            let mut tail = self.buffer.split_off(from);
                            self.buffer.push_str(&paste_text);
                            if to > from {
                                self.buffer.push_str(&tail.split_off(to-from));
                            } else {
                                self.buffer.push_str(&tail);
                            }
                            text.text = self.buffer.clone();

                            InputState::Selected(
                                from+paste_text.len(), 
                                from+paste_text.len(), 
                                since
                            )
                        } else {
                            InputState::Selected(from, to, Instant::now())
                        }
                    },

                    Event::Press(Key::Left, Modifiers{ shift: false, .. }) => {
                        let (from, to) = (from.min(to), from.max(to));
                        if from != to || from == 0 {
                            InputState::Selected(from, from, Instant::now())
                        } else {
                            InputState::Selected(from-1, from-1, Instant::now())
                        }
                    },

                    Event::Press(Key::Left, Modifiers{ shift: true, .. }) => {
                        InputState::Selected(from, if to > 0 { to-1 } else { 0 }, Instant::now())
                    },

                    Event::Press(Key::Right, Modifiers{ shift: false, .. }) => {
                        let (from, to) = (from.min(to), from.max(to));
                        if from != to || to >= self.buffer.len() {
                            InputState::Selected(to, to, Instant::now())
                        } else {
                            InputState::Selected(to+1, to+1, Instant::now())
                        }
                    },

                    Event::Press(Key::Right, Modifiers{ shift: true, .. }) => {
                        InputState::Selected(from, (to+1).min(self.buffer.len()), Instant::now())
                    },

                    Event::Press(Key::Home, Modifiers{ shift: false, .. }) => {
                        InputState::Selected(0, 0, Instant::now())
                    },

                    Event::Press(Key::Home, Modifiers{ shift: true, .. }) => {
                        InputState::Selected(from, 0, Instant::now())
                    },

                    Event::Press(Key::End, Modifiers{ shift: false, .. }) => {
                        InputState::Selected(self.buffer.len(), self.buffer.len(), Instant::now())
                    },

                    Event::Press(Key::End, Modifiers{ shift: true, .. }) => {
                        InputState::Selected(from, self.buffer.len(), Instant::now())
                    },

                    _ => {
                        InputState::Selected(from, to, since)
                    },
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
        if cursor.inside(&layout) {
            if *state == InputState::Idle {
                *state = InputState::Hovered;
            }
            true
        } else {
            if *state == InputState::Hovered {
                *state = InputState::Idle;
            }
            false
        }
    }

    fn postdraw<F: FnMut(Primitive)>(&self, state: &Self::State, layout: Rect, mut submit: F) {
        let white = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };

        submit(Primitive::Draw9(self.patch.clone(), layout, white));

        let content = self.patch.content_rect(layout);

        let text = self.text.clone().unwrap();

        submit(Primitive::PushClip(content));

        match state {
            &InputState::Idle => (),
            &InputState::Hovered => (),
            &InputState::Selecting(from, to, since) | &InputState::Selected(from, to, since) => {
                let range = text.measure_range(from.min(to), from.max(to), None);
                if to != from {
                    submit(Primitive::DrawRect(
                        Rect {
                            left: content.left + (range.0).0,
                            right: content.left + (range.1).0,
                            top: content.top,
                            bottom: content.bottom
                        },
                        Color{ r: 0.0, g: 0.0, b: 0.5, a: 0.5 }
                    ));
                } 

                if since.elapsed().subsec_nanos() < 500_000_000 {
                    let caret = if to > from {
                        range.1
                    } else {
                        range.0
                    };

                    submit(Primitive::DrawRect(
                        Rect {
                            left: content.left + caret.0,
                            right: content.left + caret.0 + 1.0,
                            top: content.top,
                            bottom: content.bottom
                        },
                        Color{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
                    ));
                }
            },
        }
        
        submit(Primitive::DrawText(text, content, self.text_color, false));

        submit(Primitive::PopClip);
    }

    fn result(self, _state: &Self::State) -> Self::Result {
        self.buffer.as_str()
    }

}