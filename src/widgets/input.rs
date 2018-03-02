use super::*;
use clipboard::{ClipboardContext,ClipboardProvider};
use std::time::Instant;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum InputState {
    Selecting(usize, usize, Instant, f32, f32),
    Selected(usize, usize, Instant, f32, f32),
    Hovered(f32, f32),
    Idle(f32, f32),
}

pub struct Input<'a> {
    pub patch: Patch,
    pub size: Option<Rect>,
    pub password: bool,
    pub text_color: Color,
    buffer: &'a mut String,
    text: Option<Text>,
    focus: bool,
    submit: bool,
}

impl<'a> Input<'a> {
    pub fn new(
        patch: Patch,
        text: &'a mut String, 
        font: Font, 
        text_size: f32, 
        text_color: Color
    ) -> Self {
        let render_text = Text{ 
            text: text.clone(), 
            font: font, 
            size: text_size, 
            wrap: TextWrap::NoWrap 
        };

        Input {
            patch,
            size: None,
            password: false,
            text_color,
            buffer: text,
            text: Some(render_text),
            focus: false,
            submit: false,
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

    pub fn password(mut self) -> Self {
        self.password = true;
        self.text.as_mut().unwrap().text = text_display(self.buffer, self.password);
        self
    }
}

fn text_display(buffer: &String, password: bool) -> String {
    if password {
        "\u{25cf}".repeat(buffer.chars().count())
    } else {
        buffer.clone()
    }
}

fn codepoint(s: &String, char_index: usize) -> usize {
    s.char_indices().skip(char_index).next().map_or(s.len(), |(i,_)| i)
}

impl WidgetState for InputState { }

impl Default for InputState {
    fn default() -> Self {
        InputState::Idle(0.0, 0.0)
    }
}

impl<'a> Widget for Input<'a> {
    type Result = bool;
    type State = InputState;

    fn tabstop() -> bool {
        true
    }

    fn autofocus(&self) -> bool {
        self.focus
    }

    fn measure(&self, _state: &Self::State, _layout: Option<Rect>) -> Option<Rect> {
        self.size
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

        // sanity check on the state
        *state = match *state {
            InputState::Selecting(mut from, mut to, since, sx, sy) => {
                if from > self.buffer.len() {
                    from = self.buffer.len();
                }
                if to > self.buffer.len() {
                    to = self.buffer.len();
                }
                InputState::Selecting(from, to, since, sx, sy)
            },
            InputState::Selected(mut from, mut to, since, sx, sy) => {
                if from > self.buffer.len() {
                    from = self.buffer.len();
                }
                if to > self.buffer.len() {
                    to = self.buffer.len();
                }
                InputState::Selected(from, to, since, sx, sy)
            },
            state => state,
        };

        // event related state update
        *state = match *state {
            InputState::Idle(sx, sy) => {
                if is_focused {
                    let count = self.buffer.chars().count();
                    InputState::Selected(count, count, Instant::now(), sx, sy)
                } else {
                    InputState::Idle(sx, sy)
                }
            },
            InputState::Hovered(sx, sy) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Text);
                    let hit = text.hitdetect(relative_cursor, content);
                    InputState::Selecting(hit, hit, Instant::now(), sx, sy)
                } else {
                    InputState::Hovered(sx, sy)
                }
            },
            InputState::Selecting(from, to, since, sx, sy) => {
                capture = Capture::CaptureFocus(MouseStyle::Text);
                if let Event::Release(Key::LeftMouseButton, _) = event {
                    InputState::Selected(from, to, since, sx, sy)
                } else {
                    let relative_cursor = (relative_cursor.0 + sx, relative_cursor.1 + sy);
                    let hit = text.hitdetect(relative_cursor, content);
                    if let Event::Idle = event {
                        InputState::Selecting(from, hit, since, sx, sy)
                    } else {
                        InputState::Selecting(from, hit, Instant::now(), sx, sy)
                    }
                }
            },
            InputState::Selected(from, to, since, sx, sy) => match event {
                Event::Press(Key::LeftMouseButton, _) => {
                    if cursor.inside(&layout) {
                        capture = Capture::CaptureFocus(MouseStyle::Text);
                        let relative_cursor = (relative_cursor.0 + sx, relative_cursor.1 + sy);
                        let hit = text.hitdetect(relative_cursor, content);
                        InputState::Selecting(hit, hit, Instant::now(), sx, sy)
                    } else {
                        InputState::Idle(sx, sy)
                    }
                },

                Event::Text(c) => {
                    match c {
                        '\x08' => {
                            let (from, to) = (from.min(to), from.max(to));

                            if to > from {
                                let pt = codepoint(self.buffer, from);
                                let tail = self.buffer.split_off(pt);
                                self.buffer.push_str(tail.split_at(codepoint(&tail, to-from)).1);
                                text.text = text_display(self.buffer, self.password);

                                InputState::Selected(from, from, Instant::now(), sx, sy)
                            } else if from > 0 {
                                let pt = codepoint(self.buffer, from-1);
                                let tail = self.buffer.split_off(pt);
                                self.buffer.push_str(tail.split_at(codepoint(&tail, 1)).1);
                                text.text = text_display(self.buffer, self.password);

                                InputState::Selected(from-1, from-1, Instant::now(), sx, sy)
                            } else {
                                InputState::Selected(from, to, Instant::now(), sx, sy)
                            }                            
                        },
                        '\x7f' => {
                            let (from, to) = (from.min(to), from.max(to));

                            let pt = codepoint(self.buffer, from);
                            let tail = self.buffer.split_off(pt);
                            if to > from {
                                self.buffer.push_str(tail.split_at(codepoint(&tail, to-from)).1);
                            } else if tail.len() > 0 {
                                self.buffer.push_str(tail.split_at(codepoint(&tail, 1)).1);
                            }
                            text.text = text_display(self.buffer, self.password);

                            InputState::Selected(from, from, Instant::now(), sx, sy)
                        },
                        c => if c.is_control() {
                            InputState::Selected(from, to, since, sx, sy)
                        } else {
                            let (from, to) = (from.min(to), from.max(to));

                            let pt = codepoint(self.buffer, from);
                            let mut tail = self.buffer.split_off(pt);
                            self.buffer.push(c);
                            if to > from {
                                let pt = codepoint(&tail, to-from);
                                self.buffer.push_str(&tail.split_off(pt));
                            } else {
                                self.buffer.push_str(&tail);
                            }
                            text.text = text_display(self.buffer, self.password);

                            InputState::Selected(from+1, from+1, Instant::now(), sx, sy)
                        }
                    }
                },

                Event::Press(Key::Enter, Modifiers{ shift: false, .. }) => {
                    self.submit = true;

                    InputState::Selected(from, to, since, sx, sy)
                },

                Event::Press(Key::C, Modifiers{ ctrl: true, .. }) => {
                    let (a, b) = (from.min(to), from.max(to));
                    let (a, b) = (codepoint(self.buffer, a), codepoint(self.buffer, b));
                    let copy_text = self.buffer[a..b].to_string();
                    ClipboardContext::new().and_then(|mut cc| {
                        cc.set_contents(copy_text)
                    }).ok();

                    InputState::Selected(from, to, since, sx, sy)
                },

                Event::Press(Key::X, Modifiers{ ctrl: true, .. }) => {
                    let (from, to) = (from.min(to), from.max(to));
                    let (a, b) = (codepoint(self.buffer, from), codepoint(self.buffer, to));
                    let cut_text = self.buffer[a..b].to_string();
                    ClipboardContext::new().and_then(|mut cc| {
                        cc.set_contents(cut_text)
                    }).ok();

                    let pt = codepoint(self.buffer, from);
                    let tail = self.buffer.split_off(pt);
                    if to > from {
                        self.buffer.push_str(tail.split_at(codepoint(&tail, to-from)).1);
                    } else if tail.len() > 0 {
                        self.buffer.push_str(tail.split_at(codepoint(&tail, 1)).1);
                    }
                    text.text = text_display(self.buffer, self.password);

                    InputState::Selected(from, from, since, sx, sy)
                },

                Event::Press(Key::V, Modifiers{ ctrl: true, .. }) => {
                    let (from, to) = (from.min(to), from.max(to));
                    let paste_text = ClipboardContext::new().and_then(|mut cc| {
                        cc.get_contents()
                    }).ok();

                    if let Some(paste_text) = paste_text {
                        let pt = codepoint(self.buffer, from);
                        let mut tail = self.buffer.split_off(pt);
                        self.buffer.push_str(&paste_text);
                        if to > from {
                            let pt = codepoint(&tail, to-from);
                            self.buffer.push_str(&tail.split_off(pt));
                        } else {
                            self.buffer.push_str(&tail);
                        }
                        text.text = text_display(self.buffer, self.password);

                        InputState::Selected(
                            from+paste_text.len(), 
                            from+paste_text.len(), 
                            since,
                            sx, sy
                        )
                    } else {
                        InputState::Selected(from, to, Instant::now(), sx, sy)
                    }
                },

                Event::Press(Key::Left, Modifiers{ shift: false, .. }) => {
                    let (from, to) = (from.min(to), from.max(to));
                    if from != to || from == 0 {
                        InputState::Selected(from, from, Instant::now(), sx, sy)
                    } else {
                        InputState::Selected(from-1, from-1, Instant::now(), sx, sy)
                    }
                },

                Event::Press(Key::Left, Modifiers{ shift: true, .. }) => {
                    InputState::Selected(
                        from, 
                        if to > 0 { to-1 } else { 0 }, 
                        Instant::now(),
                        sx, sy
                    )
                },

                Event::Press(Key::Right, Modifiers{ shift: false, .. }) => {
                    let (from, to) = (from.min(to), from.max(to));
                    if from != to || to >= self.buffer.chars().count() {
                        InputState::Selected(to, to, Instant::now(), sx, sy)
                    } else {
                        InputState::Selected(to+1, to+1, Instant::now(), sx, sy)
                    }
                },

                Event::Press(Key::Right, Modifiers{ shift: true, .. }) => {
                    let count = self.buffer.chars().count();
                    InputState::Selected(from, (to+1).min(count), Instant::now(), sx, sy)
                },

                Event::Press(Key::Home, Modifiers{ shift: false, .. }) => {
                    InputState::Selected(0, 0, Instant::now(), sx, sy)
                },

                Event::Press(Key::Home, Modifiers{ shift: true, .. }) => {
                    InputState::Selected(from, 0, Instant::now(), sx, sy)
                },

                Event::Press(Key::End, Modifiers{ shift: false, .. }) => {
                    let count = self.buffer.chars().count();
                    InputState::Selected(count, count, Instant::now(), sx, sy)
                },

                Event::Press(Key::End, Modifiers{ shift: true, .. }) => {
                    let count = self.buffer.chars().count();
                    InputState::Selected(from, count, Instant::now(), sx, sy)
                },

                _ => {
                    InputState::Selected(from, to, since, sx, sy)
                },
            },
        };

        // update scroll state for current text and caret position
        match state {
            &mut InputState::Selecting(_, pos, _, ref mut sx, ref mut sy) |
            &mut InputState::Selected(_, pos, _, ref mut sx, ref mut sy) => {
                let content = self.patch.content_rect(layout);
                let (caret, range) = text.measure_range(pos, text.text.chars().count(), content);

                if *sx + content.width() > range.0 + 2.0 {
                    *sx = (range.0 - content.width() + 2.0).max(0.0);
                }
                if caret.0 - *sx > content.width() - 2.0 {
                    *sx = caret.0 - content.width() + 2.0;
                }
                if caret.0 - *sx < 0.0 {
                    *sx = caret.0;
                }
                if caret.1 - *sy > content.height() - 2.0 {
                    *sy = caret.1 - content.height() + 2.0;
                }
                if caret.1 - *sy < 0.0 {
                    *sy = caret.1;
                }
            },
            &mut _ => (),
        };

        capture
    }

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition
    ) -> Hover {
        if cursor.inside(&layout) {
            if let InputState::Idle(sx, sy) = *state {
                *state = InputState::Hovered(sx, sy);
            }
            Hover::HoverActive(MouseStyle::Text)
        } else {
            if let InputState::Hovered(sx, sy) = *state {
                *state = InputState::Idle(sx, sy);
            }
            Hover::NoHover
        }
    }

    fn postdraw<F: FnMut(Primitive)>(&self, state: &Self::State, layout: Rect, mut submit: F) {
        let white = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };

        submit(Primitive::Draw9(self.patch.clone(), layout, white));

        let mut content = self.patch.content_rect(layout);

        let text = self.text.clone().unwrap();

        submit(Primitive::PushClip(content));

        let scroll;

        match state {
            &InputState::Idle(sx, sy) => {
                scroll = (sx, sy);
            },
            &InputState::Hovered(sx, sy) => {
                scroll = (sx, sy);
            },
            &InputState::Selecting(from, to, since, sx, sy) | 
            &InputState::Selected(from, to, since, sx, sy) => {
                let range = text.measure_range(from.min(to), from.max(to), content);
                scroll = (sx, sy);

                if to != from {
                    submit(Primitive::DrawRect(
                        Rect {
                            left: content.left + (range.0).0,
                            right: content.left + (range.1).0,
                            top: content.top,
                            bottom: content.bottom
                        }.translate(-scroll.0, -scroll.1),
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
                        }.translate(-scroll.0, -scroll.1),
                        Color{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
                    ));
                }
            },
        }

        content.left -= scroll.0;
        content.top -= scroll.1;
        
        submit(Primitive::DrawText(text, content, self.text_color));

        submit(Primitive::PopClip);
    }

    fn result(self, _: &Self::State) -> Self::Result {
        self.submit
    }
}
