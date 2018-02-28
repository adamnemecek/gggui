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

#[derive(Clone)]
pub struct Flow {
    pub style: FlowStyle,
    pub pad: f32,
    pub h_gap: f32,
    pub v_gap: f32,
    pub size: Option<Rect>,
    pub background: Background,
    pub enabled: bool,
    cursor: (f32,f32),
    advance: f32,
    counter: u32,
}

impl Flow {
    pub fn new() -> Flow {
        Flow {
            style: FlowStyle::Absolute,
            pad: 0.0,
            h_gap: 8.0,
            v_gap: 8.0,
            size: None,
            background: Background::None,
            enabled: true,
            cursor: (0.0, 0.0),
            advance: 0.0,
            counter: 0,
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.pad = padding;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = background;
        self
    }

    pub fn background_color(mut self, background: Color) -> Self {
        self.background = Background::Color(background);
        self
    }

    pub fn background_image(mut self, background: Image, alpha: f32) -> Self {
        self.background = Background::Image(background, alpha);
        self
    }

    pub fn background_patch(mut self, background: Patch, alpha: f32) -> Self {
        self.background = Background::Patch(background, alpha);
        self
    }

    pub fn enable(mut self, enable: bool) -> Self {
        self.enabled = enable;
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

fn clamp(x: f32, bound: (f32,f32)) -> f32 {
    x.max(bound.0).min(bound.1)
}

impl Widget for Flow {
    type Result = ();
    type State = ();

    fn enabled(&self, _: &Self::State) -> bool {
        self.enabled
    }

    fn measure(&self, _state: &Self::State, layout: Option<Rect>) -> Option<Rect> {
        let layout = layout.or(self.size).unwrap_or(Rect::from_wh(0.0, 0.0));
        self.size
            .and_then(|s| Some(s))
            .or_else(|| {
                match self.style {
                    FlowStyle::LinearHorizontal(Align::Begin) |
                    FlowStyle::LinearVertical(Align::Begin) => 
                        Some(Rect::from_wh(self.cursor.0, self.cursor.1)),

                    FlowStyle::LinearHorizontal(_) =>
                        Some(Rect::from_wh(self.cursor.0, layout.height())),

                    FlowStyle::LinearVertical(_) => 
                        Some(Rect::from_wh(layout.width(), self.cursor.1)),

                    FlowStyle::GridHorizontal(_) => 
                        Some(Rect::from_wh(layout.width(), self.cursor.1 + self.advance)),

                    FlowStyle::GridVertical(_) => 
                        Some(Rect::from_wh(self.cursor.0 + self.advance, layout.height())),

                    _ => self.size,
                }
            })
    }

    fn estimate(
        &self,
        state: &Self::State,
        layout: Rect,
        child: WidgetMeasure
    ) -> Rect {
        self.clone().layout(state, layout, child)
    }

    fn layout(
        &mut self, 
        _state: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        let layout = match &self.background {
            &Background::Patch(ref patch, _) => patch.content_rect(layout),
            _ => layout
        };

        match self.style {
            FlowStyle::LinearVertical(align) => {
                let available_space = Rect::from_wh(
                    layout.width()-self.pad*2.0, 
                    layout.height()-self.cursor.1-self.pad
                );
                let r = child(Some(available_space)).map_or(layout, |child| {
                    let (l, r) = match align {
                        Align::Begin => (
                            self.pad+layout.left, 
                            self.pad+layout.left + child.width()
                        ),
                        Align::Middle => (
                            (layout.left+layout.right-child.width())*0.5, 
                            (layout.left+layout.right+child.width())*0.5
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

                self.cursor.0 = self.cursor.0.max(r.width());
                self.cursor.1 += r.height() + self.v_gap;
                r.round()
            },

            FlowStyle::LinearHorizontal(align) => {
                let available_space = Rect::from_wh(
                    layout.width()-self.cursor.0-self.pad, 
                    layout.height()-self.pad*2.0
                );
                let r = child(Some(available_space)).map_or(layout, |child| {
                    let (t, b) = match align {
                        Align::Begin => (
                            self.pad+layout.top, 
                            self.pad+layout.top + child.height()
                        ),
                        Align::Middle => (
                            (layout.top+layout.bottom-child.height())*0.5, 
                            (layout.top+layout.bottom+child.height())*0.5
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
                self.cursor.1 = self.cursor.1.max(r.height());
                r.round()
            },

            FlowStyle::GridHorizontal(columns) => {
                let column_width = (layout.width() - self.pad*2.0 - (columns as f32-1.0)*self.h_gap) / columns as f32;
                let available_space = Rect::from_wh(
                    column_width, 
                    layout.height()-self.cursor.1-self.pad
                );
                let height = child(Some(available_space)).map_or(32.0, |r| r.height());

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
                let available_space = Rect::from_wh(
                    layout.width()-self.cursor.0-self.pad, 
                    row_height
                );
                let width = child(Some(available_space)).map_or(32.0, |r| r.width());

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
                if self.counter == 0 {
                    let r = child(Some(layout)).map_or(layout, |child| {
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

                    self.counter += 1;
                    r.round()
                } else {
                    Rect::from_wh(0.0, 0.0)
                }
            },

            FlowStyle::Absolute => {
                let r = child(Some(layout)).map_or(layout, |child| {
                    Rect {
                        left: layout.left + child.left,
                        right: layout.left + child.right,
                        top: layout.top + child.top,
                        bottom: layout.top + child.bottom,
                    }
                });
                r.round()
            }
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, _state: &Self::State, layout: Rect, mut submit: F) { 
        match &self.background {
            &Background::None => (),
            &Background::Color(ref color) => {
                submit(Primitive::DrawRect(layout, *color));
            },
            &Background::Image(ref image, a) => {
                submit(Primitive::DrawImage(image.clone(), layout, Color::white().with_alpha(a)));
            },
            &Background::Patch(ref patch, a) => {
                submit(Primitive::Draw9(patch.clone(), layout, Color::white().with_alpha(a)));
            },
        }
    }

    fn child_area(&self, _: &Self::State, layout: Rect) -> ChildArea {
        match &self.background {
            &Background::Patch(ref patch, _) => {
                ChildArea::ConfineContentAndInput(patch.content_rect(layout))
            },
            &Background::None => {
                ChildArea::OverflowContentAndInput
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