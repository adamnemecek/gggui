use super::*;

#[derive(Clone,Copy,Debug)]
pub enum Align {
    Begin, Middle, End
}


#[derive(Clone,Copy,Debug)]
pub enum FlowStyle {
    LinearHorizontal(Align),
    LinearVertical(Align),
    GridHorizontal(u32),
    GridVertical(u32),
    Single(Align, Align),
    Absolute,
}

pub enum Background {
    None,
    Color(Color),
    Image(Image),
    Patch(Patch),
}

pub struct Flow {
    pub style: FlowStyle,
    pub pad: f32,
    pub h_gap: f32,
    pub v_gap: f32,
    pub size: Option<Rect>,
    pub background: Background,
    cursor: (f32,f32),
    advance: f32,
    counter: u32,
}

impl Flow {
    pub fn new() -> Flow {
        Flow {
            style: FlowStyle::LinearVertical(Align::Begin),
            pad: 0.0,
            h_gap: 8.0,
            v_gap: 8.0,
            size: None,
            background: Background::None,
            cursor: (0.0, 0.0),
            advance: 0.0,
            counter: 0,
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.pad = padding;
        self
    }

    pub fn background_color(mut self, background: Color) -> Self {
        self.background = Background::Color(background);
        self
    }

    pub fn background_image(mut self, background: Image) -> Self {
        self.background = Background::Image(background);
        self
    }

    pub fn background_patch(mut self, background: Patch) -> Self {
        self.background = Background::Patch(background);
        self
    }

    pub fn style(mut self, style: FlowStyle) -> Self {
        self.style = style;
        self
    }

    pub fn size(mut self, size: Rect) -> Self {
        self.size = Some(size);
        self
    }
}

impl WidgetState for bool { }

fn clamp(x: f32, bound: (f32,f32)) -> f32 {
    x.max(bound.0).min(bound.1)
}

impl Widget for Flow {
    type Result = ();
    type State = GenericWidgetState;

    fn default() -> Self::State {
        GenericWidgetState::Idle
    }

    fn measure(&self, _state: &Self::State) -> Option<Rect> {
        self.size
    }

    fn layout(&mut self, _state: &Self::State, layout: Rect, child: Option<Rect>) -> Rect {
        let layout = match &self.background {
            &Background::Patch(ref patch) => patch.content_rect(layout),
            _ => layout
        };

        match self.style {
            FlowStyle::LinearVertical(align) => {
                let r = child.map_or(layout, |child| {
                    let (l, r) = match align {
                        Align::Begin => (
                            self.pad+layout.left, 
                            self.pad+layout.left + child.width()
                        ),
                        Align::Middle => (
                            (layout.left+layout.right+child.width())*0.5, 
                            (layout.left+layout.right-child.width())*0.5
                        ),
                        Align::End => (
                            layout.right - self.pad - child.width(),
                            layout.right - self.pad
                        ),
                    };
                    Rect {
                        left: clamp(l, (layout.left+self.pad, layout.right-self.pad)),
                        right: clamp(r, (layout.left+self.pad, layout.right-self.pad)),
                        top: self.pad + layout.top + self.cursor.1,
                        bottom: self.pad + layout.top + self.cursor.1 + child.height()
                    }
                });

                self.cursor.1 += r.height() + self.v_gap;
                r.round()
            },

            FlowStyle::LinearHorizontal(align) => {
                let r = child.map_or(layout, |child| {
                    let (t, b) = match align {
                        Align::Begin => (
                            self.pad+layout.top, 
                            self.pad+layout.top + child.height()
                        ),
                        Align::Middle => (
                            (layout.top+layout.bottom+child.height())*0.5, 
                            (layout.top+layout.bottom-child.height())*0.5
                        ),
                        Align::End => (
                            layout.bottom - self.pad - child.height(),
                            layout.bottom - self.pad
                        ),
                    };

                    Rect {
                        left: self.pad + layout.left + self.cursor.0,
                        right: self.pad + layout.left + self.cursor.0 + child.width(),
                        top: clamp(t, (layout.top+self.pad, layout.bottom-self.pad)),
                        bottom: clamp(b, (layout.top+self.pad, layout.bottom-self.pad)),
                    }
                });

                self.cursor.0 += r.width() + self.h_gap;
                r.round()
            },

            FlowStyle::GridHorizontal(columns) => {
                let column_width = (layout.width() - self.pad*2.0 - (columns as f32-1.0)*self.h_gap) / columns as f32;
                let height = child.map_or(32.0, |r| r.height());

                let r = Rect {
                    left: self.pad + layout.left + self.cursor.0,
                    right: self.pad + layout.left + self.cursor.0 + column_width,
                    top: self.pad + layout.top + self.cursor.1,
                    bottom: self.pad + layout.top + self.cursor.1 + height,
                };

                self.advance = self.advance.max(height + self.v_gap);
                self.cursor.0 += column_width + self.h_gap;
                self.counter += 1;
                if self.counter % columns == 0 {
                    self.cursor.0 = 0.0;
                    self.cursor.1 += self.advance;
                    self.advance = 0.0;
                }

                r.round()
            },

            FlowStyle::GridVertical(rows) => {
                let row_height = (layout.height() - self.pad*2.0 - (rows as f32-1.0)*self.v_gap) / rows as f32;
                let width = child.map_or(32.0, |r| r.width());

                let r = Rect {
                    left: self.pad + layout.left + self.cursor.0,
                    right: self.pad + layout.left + self.cursor.0 + width,
                    top: self.pad + layout.top + self.cursor.1,
                    bottom: self.pad + layout.top + self.cursor.1 + row_height,
                };

                self.advance = self.advance.max(width + self.h_gap);
                self.cursor.1 += row_height + self.v_gap;
                self.counter += 1;
                if self.counter % rows == 0 {
                    self.cursor.1 = 0.0;
                    self.cursor.0 += self.advance;
                    self.advance = 0.0;
                }

                r.round()
            },

            FlowStyle::Single(h, v) => {
                let r = child.map_or(layout, |child| {
                    let (l, r) = match h {
                        Align::Begin => (
                            self.pad+layout.left, 
                            self.pad+layout.left + child.width()
                        ),
                        Align::Middle => (
                            (layout.left+layout.right)*0.5-child.width()*0.5, 
                            (layout.left+layout.right)*0.5+child.width()*0.5
                        ),
                        Align::End => (
                            layout.right - self.pad - child.width(),
                            layout.right - self.pad
                        ),
                    };
                    let (t, b) = match v {
                        Align::Begin => (
                            self.pad+layout.top, 
                            self.pad+layout.top + child.height()
                        ),
                        Align::Middle => (
                            (layout.top+layout.bottom)*0.5-child.height()*0.5, 
                            (layout.top+layout.bottom)*0.5+child.height()*0.5
                        ),
                        Align::End => (
                            layout.bottom - self.pad - child.height(),
                            layout.bottom - self.pad
                        ),
                    };
                    Rect{ 
                        left: clamp(l, (layout.left+self.pad, layout.right-self.pad)), 
                        right: clamp(r, (layout.left+self.pad, layout.right-self.pad)), 
                        top: clamp(t, (layout.top+self.pad, layout.bottom-self.pad)), 
                        bottom: clamp(b, (layout.top+self.pad, layout.bottom-self.pad))
                    }
                });

                r.round()
            },

            FlowStyle::Absolute => {
                let r = child.map_or(layout, |child| {
                    Rect {
                        left: layout.left + child.left,
                        right: layout.left + child.right,
                        top: layout.top + child.top,
                        bottom: layout.bottom + child.bottom,
                    }
                });
                r.round()
            }
        }
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        _: Rect, 
        _: MousePosition,
        event: Event,
        _: bool
    ) -> Capture {
        let mut capture = Capture::None;

        *state = match *state {
            GenericWidgetState::Idle => {
                GenericWidgetState::Idle
            },
            GenericWidgetState::Hovered => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus;
                    GenericWidgetState::Clicked
                } else {
                    GenericWidgetState::Hovered
                }
            },
            GenericWidgetState::Clicked => {
                capture = Capture::CaptureFocus;
                if let Event::Release(Key::LeftMouseButton, _) = event {
                    GenericWidgetState::Idle
                } else {
                    GenericWidgetState::Clicked
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
            if *state == GenericWidgetState::Idle {
                *state = GenericWidgetState::Hovered;
            }
            true
        } else {
            if *state == GenericWidgetState::Hovered {
                *state = GenericWidgetState::Idle;
            }
            false
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, _state: &Self::State, layout: Rect, mut submit: F) { 
        match &self.background {
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
        match &self.background {
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