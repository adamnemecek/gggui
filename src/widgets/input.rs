use std::time::Instant;
use clipboard::{ClipboardContext,ClipboardProvider};
use super::*;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum InputState {
    Selecting(usize, usize, Instant, f32, f32),
    Selected(usize, usize, Instant, f32, f32),
    Hovered(f32, f32),
    Idle(f32, f32),
}

pub struct Input<'a> {
    buffer: &'a mut String,
    password: bool,
    submit: bool,
}

impl<'a> Input<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            buffer: text,
            password: false,
            submit: false,
        }
    }

    pub fn password(text: &'a mut String) -> Self {
        Self {
            buffer: text,
            password: true,
            submit: false,
        }
    }
}

impl<'a> WidgetBase for Input<'a> {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        let text = Text {
            text: self.buffer.clone(),
            size: 16.0,
            wrap: TextWrap::NoWrap,
            font: style.font.clone(),
            color: Color::black(),
            border: None,
            padding: Rect { left: 4.0, right: 4.0, top: 4.0, bottom: 4.0 },
        };

        let layout = Layout::new().with_intrinsic_size_constraints(128.0, 32.0, 250.0);

        world.create_component(id, InputState::Idle(0.0, 0.0));
        world.create_component(id, layout);
        world.create_component(id, text);
        world.create_component(id, Drawing{ primitives: vec![] });
        world.create_component(id, Clipper{ rect: Rect::from_wh(0.0, 0.0), intersect: true });
        world.create_component(id, WidgetBackground {
            normal: Background::Patch(style.input.clone(), 1.0),
            hover: Background::Patch(style.input.clone(), 1.0),
            click: Background::Patch(style.input.clone(), 1.0),
        });
    }
    
    fn event(&mut self, id: dag::Id, world: &mut Ui, style: &Style, context: &mut EventSystemContext) {
        let mut layout = world.component::<Layout>(id).unwrap();
        let mut layout = layout.borrow_mut();

        if layout.current.is_none() {
            return;
        }

        let current = layout.current.unwrap();
        let content = style.input.content_rect(current);

        let mut text = world.component::<Text>(id).unwrap();
        let mut text = text.borrow_mut();

        let mut state = world.component::<InputState>(id).unwrap();
        let mut state = state.borrow_mut();

        let mut clipper = world.component::<Clipper>(id).unwrap();
        clipper.borrow_mut().rect = content;

        let mut drawing = world.component::<Drawing>(id).unwrap();
        let mut drawing = drawing.borrow_mut();

        let relative_cursor = (context.cursor.x - content.left, context.cursor.y - content.top);

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

        if context.cursor.inside(&current) {
            context.style = MouseStyle::Text;
        }

        // event related state update
        *state = match *state {
            InputState::Idle(sx, sy) => {
                if context.focused {
                    let count = self.buffer.chars().count();
                    InputState::Selected(count, count, Instant::now(), sx, sy)
                } else if context.cursor.inside(&current) {
                    InputState::Hovered(sx, sy)
                } else {
                    InputState::Idle(sx, sy)
                }
            },
            InputState::Hovered(sx, sy) => {
                if let Event::Press(Key::LeftMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(MouseStyle::Text);
                    let hit = text.hitdetect(relative_cursor, content);
                    InputState::Selecting(hit, hit, Instant::now(), sx, sy)
                } else if context.cursor.inside(&current) {
                    InputState::Hovered(sx, sy)
                } else {
                    InputState::Idle(sx, sy)
                }
            },
            InputState::Selecting(from, to, since, sx, sy) => {
                context.style = MouseStyle::Text;
                context.capture = Capture::CaptureFocus(MouseStyle::Text);
                if let Event::Release(Key::LeftMouseButton, _) = context.event {
                    InputState::Selected(from, to, since, sx, sy)
                } else {
                    let relative_cursor = (relative_cursor.0 + sx, relative_cursor.1 + sy);
                    let hit = text.hitdetect(relative_cursor, content);
                    if let Event::Idle = context.event {
                        InputState::Selecting(from, hit, since, sx, sy)
                    } else {
                        InputState::Selecting(from, hit, Instant::now(), sx, sy)
                    }
                }
            },
            InputState::Selected(from, to, since, sx, sy) => match context.event {
                Event::Press(Key::LeftMouseButton, _) => {
                    if context.cursor.inside(&current) {
                        context.capture = Capture::CaptureFocus(MouseStyle::Text);
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
        match state.deref_mut() {
            &mut InputState::Selecting(_, pos, _, ref mut sx, ref mut sy) |
            &mut InputState::Selected(_, pos, _, ref mut sx, ref mut sy) => {
                let content = style.input.content_rect(layout.current.unwrap());
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

        // update rendering
        let scroll;
        drawing.primitives.clear();
        match state.deref() {
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
                    drawing.primitives.push(Primitive::DrawRect(
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

                    drawing.primitives.push(Primitive::DrawRect(
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

        text.padding = Rect {
            top: content.top - current.top,
            bottom: current.bottom - content.bottom,
            left: content.left - current.left - scroll.0,
            right: current.right - content.right - scroll.1,
        };
    }
}

impl<'a> Widget for Input<'a> {
    type Result = bool;

    fn result(&mut self, _id: dag::Id) -> Self::Result {
        self.submit
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