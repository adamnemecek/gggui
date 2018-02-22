use super::*;

#[derive(Clone,Copy,PartialEq)]
pub enum ScrollState {
    Idle,
    HoverH(f32),
    HoverV(f32),
    ScrollH(f32),
    ScrollV(f32),
}

pub struct Scroll<'a> {
    pub scroll: &'a mut (f32, f32),
    pub background_h: Patch,
    pub background_v: Patch,
    pub bar_h: Patch,
    pub bar_v: Patch,
    pub size: Option<Rect>,
    pub scrollable_h: bool,
    pub scrollable_v: bool,
    content: (f32, f32),
}

impl<'a> Scroll<'a> {
    pub fn new(
        scroll: &'a mut (f32, f32), 
        bg_h: Patch, 
        bg_v: Patch, 
        bar_h: Patch, 
        bar_v: Patch
    ) -> Scroll {
        Scroll {
            scroll: scroll,
            background_h: bg_h,
            background_v: bg_v,
            bar_h: bar_h,
            bar_v: bar_v,
            size: None,
            scrollable_h: true,
            scrollable_v: true,
            content: (0.0, 0.0),
        }
    }

    pub fn size(mut self, size: Rect) -> Self {
        self.size = Some(size);
        self
    }

    pub fn scrollable(mut self, h: bool, v: bool) -> Self {
        self.scrollable_h = h;
        self.scrollable_v = v;
        self
    }

    fn scroll_layout(
        &self, 
        mut scroll: (f32, f32), 
        mut layout: Rect,
        child: WidgetMeasure
    ) -> (Rect, (f32,f32), (f32,f32)) {
        if self.scrollable_h {
            layout.bottom = layout.bottom - self.background_h.image.size.height();
        }
        if self.scrollable_v {
            layout.right = layout.right - self.background_v.image.size.width();
        }

        let measured = child(Some(layout));
        let mut content;

        if measured.is_some() {
            let measured = measured.unwrap();
            let h_bar_size = self.background_h.image.size.width();
            let v_bar_size = self.background_v.image.size.height();

            content = (0.0, 0.0);
            if measured.width() > layout.width() && self.scrollable_h {
                content.0 = measured.width() - (layout.width()-h_bar_size);
            }
            if measured.height() > layout.height() && self.scrollable_v {
                content.1 = measured.height() - (layout.height()-v_bar_size);
            }

            scroll.0 = scroll.0.min(content.0);
            scroll.1 = scroll.1.min(content.1);

            (
                measured.size().translate(layout.left - scroll.0, layout.top - scroll.1), 
                scroll, 
                content
            )
        } else {
            scroll.0 = 0.0;
            scroll.1 = 0.0;
            content = (0.0, 0.0);
            
            (layout, scroll, content)
        }
    }
}

impl WidgetState for ScrollState { }

impl<'a> Widget for Scroll<'a> {
    type Result = ();
    type State = ScrollState;

    fn default() -> Self::State {
        ScrollState::Idle
    }

    fn measure(&self, _: &Self::State, _: Option<Rect>) -> Option<Rect> {
        self.size
    }

    fn estimate(
        &self, 
        _: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        self.scroll_layout(*self.scroll, layout, child).0
    }

    fn layout(
        &mut self, 
        _: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        let (layout, scroll, content) =
            self.scroll_layout(*self.scroll, layout, child);

        *self.scroll = scroll;
        self.content = content;
        layout
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition,
        event: Event,
        _: bool
    ) -> Capture {
        let mut capture = Capture::None;

        *state = match *state {
            ScrollState::Idle => {
                ScrollState::Idle
            },
            ScrollState::HoverH(x) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    ScrollState::ScrollH(x)
                } else {
                    ScrollState::HoverH(x)
                }
            },
            ScrollState::HoverV(y) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    ScrollState::ScrollV(y)
                } else {
                    ScrollState::HoverV(y)
                }
            },
            ScrollState::ScrollH(x) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = layout.clone();
                bar.top = bar.bottom - self.background_h.image.size.height();
                if self.scrollable_v {
                    bar.right = bar.right - self.background_v.image.size.width();
                }
                self.scroll.0 = handle_to_scroll(
                    bar.left, 
                    cursor.x-x, 
                    bar.width(), 
                    self.content.0
                );

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    ScrollState::Idle
                } else {
                    ScrollState::ScrollH(x)
                }
            },
            ScrollState::ScrollV(y) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = layout.clone();
                bar.left = bar.right - self.background_v.image.size.width();
                if self.scrollable_h {
                    bar.bottom = bar.bottom - self.background_h.image.size.height();
                }
                self.scroll.1 = handle_to_scroll(
                    bar.top, 
                    cursor.y-y, 
                    bar.height(), 
                    self.content.1
                );

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    ScrollState::Idle
                } else {
                    ScrollState::ScrollV(y)
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
        match *state {
            ScrollState::ScrollH(_) |
            ScrollState::ScrollV(_) => {
                Hover::HoverIdle
            },
            _ => {
                if self.scrollable_h {
                    let mut bar = layout.clone();
                    bar.top = bar.bottom - self.background_h.image.size.height();
                    if self.scrollable_v {
                        bar.right = bar.right - self.background_v.image.size.width();
                    }

                    let handle_range = handle_range(
                        bar.left, 
                        self.scroll.0, 
                        bar.width(), 
                        self.content.0
                    );
                    bar.left = handle_range.0;
                    bar.right = handle_range.1;

                    if cursor.inside(&bar) {
                        *state = ScrollState::HoverH(cursor.x - bar.left);
                        return Hover::HoverIdle;
                    }
                }

                if self.scrollable_v {
                    let mut bar = layout.clone();
                    bar.left = bar.right - self.background_v.image.size.height();
                    if self.scrollable_h {
                        bar.bottom = bar.bottom - self.background_h.image.size.height();
                    }

                    let handle_range = handle_range(
                        bar.top, 
                        self.scroll.1, 
                        bar.height(), 
                        self.content.1
                    );
                    bar.top = handle_range.0;
                    bar.bottom = handle_range.1;

                    if cursor.inside(&bar) {
                        *state = ScrollState::HoverV(cursor.y - bar.top);
                        return Hover::HoverIdle;
                    }
                }

                Hover::NoHover
            },
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, _: &Self::State, layout: Rect, mut submit: F) {
        submit(Primitive::PushClip(layout));
    }

    fn postdraw<F: FnMut(Primitive)>(&self, _state: &Self::State, layout: Rect, mut submit: F) {
        // draw h scroll
        if self.scrollable_h {
            let mut bar = layout.clone();
            bar.top = bar.bottom - self.background_h.image.size.height();
            if self.scrollable_v {
                bar.right = bar.right - self.background_v.image.size.width();
            }

            submit(Primitive::Draw9(self.background_h.clone(), bar, Color::white()));

            let handle_range = handle_range(bar.left, self.scroll.0, bar.width(), self.content.0);
            bar.left = handle_range.0;
            bar.right = handle_range.1;

            submit(Primitive::Draw9(self.bar_h.clone(), bar, Color::white()));
        }

        // draw v scroll
        if self.scrollable_v {
            let mut bar = layout.clone();
            bar.left = bar.right - self.background_v.image.size.width();
            if self.scrollable_h {
                bar.bottom = bar.bottom - self.background_h.image.size.height();
            }

            submit(Primitive::Draw9(self.background_v.clone(), bar, Color::white()));

            let handle_range = handle_range(bar.top, self.scroll.1, bar.height(), self.content.1);
            bar.top = handle_range.0;
            bar.bottom = handle_range.1;

            submit(Primitive::Draw9(self.bar_v.clone(), bar, Color::white()));
        }

        submit(Primitive::PopClip);
    }

    fn childs(&self, _: &Self::State, mut layout: Rect) -> ChildType {
        if self.scrollable_h {
            layout.bottom = layout.bottom - self.bar_h.image.size.height();
        }
        if self.scrollable_v {
            layout.right = layout.right - self.bar_v.image.size.width();
        }
        ChildType::IntersectInputOnly(layout)
    }

    fn result(self, _: &Self::State) -> Self::Result {
        ()
    }
}

fn handle_to_scroll(offset: f32, x: f32, length: f32, content: f32) -> f32 {
    let range = handle_range(offset, content, length, content);
    let pos = (x-offset)/(range.0-offset);
    (pos * content).max(0.0).min(content).floor()
}

fn handle_range(offset: f32, x: f32, length: f32, content: f32) -> (f32, f32) { 
    let size = length * (length / (length+content));
    let start = length * (x / (length+content));
    ((offset+start).floor(), (offset+start+size).floor())
}