use super::*;

#[derive(Clone,Copy,PartialEq)]
pub struct ScrollState {
    scroll: (f32, f32),
    inner: ScrollStateInner,
}

#[derive(Clone,Copy,PartialEq)]
pub enum ScrollStateInner {
    Idle,
    HoverContent,
    HoverH(f32),
    HoverV(f32),
    ScrollH(f32),
    ScrollV(f32),
}

pub struct Scroll {
    pub background_h: Patch,
    pub background_v: Patch,
    pub bar_h: Patch,
    pub bar_v: Patch,
    pub size: Option<Rect>,
    pub scrollable_h: bool,
    pub scrollable_v: bool,
    content: (f32, f32),
}

impl Scroll {
    pub fn new(
        bg_h: Patch, 
        bg_v: Patch, 
        bar_h: Patch, 
        bar_v: Patch
    ) -> Scroll {
        Scroll {
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
        estimate: bool,
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

            if !estimate {
                scroll.0 = scroll.0.min(content.0);
                scroll.1 = scroll.1.min(content.1);
            }

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

impl Default for ScrollState {
    fn default() -> Self {
        ScrollState{
            scroll: (0.0, 0.0),
            inner: ScrollStateInner::Idle
        }
    }
}

impl Widget for Scroll {
    type Result = ();
    type State = ScrollState;

    fn state_type() -> StateType { 
        StateType::Persistent 
    }

    fn measure(&self, _: &Self::State, _: Option<Rect>) -> Option<Rect> {
        self.size
    }

    fn estimate(
        &self, 
        state: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        self.scroll_layout(state.scroll, layout, true, child).0
    }

    fn layout(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        let (layout, scroll, content) =
            self.scroll_layout(state.scroll, layout, false, child);

        state.scroll = scroll;
        self.content = content;
        layout
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition,
        event: Event,
        _is_focused: bool
    ) -> Capture {
        let mut capture = Capture::None;

        state.inner = match state.inner {
            ScrollStateInner::Idle => {
                ScrollStateInner::Idle
            },
            ScrollStateInner::HoverContent => {
                ScrollStateInner::HoverContent
            },
            ScrollStateInner::HoverH(x) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    ScrollStateInner::ScrollH(x)
                } else {
                    ScrollStateInner::HoverH(x)
                }
            },
            ScrollStateInner::HoverV(y) => {
                if let Event::Press(Key::LeftMouseButton, _) = event {
                    capture = Capture::CaptureFocus(MouseStyle::Arrow);
                    ScrollStateInner::ScrollV(y)
                } else {
                    ScrollStateInner::HoverV(y)
                }
            },
            ScrollStateInner::ScrollH(x) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = layout.clone();
                bar.top = bar.bottom - self.background_h.image.size.height();
                if self.scrollable_v {
                    bar.right = bar.right - self.background_v.image.size.width();
                }
                state.scroll.0 = handle_to_scroll(
                    bar.left, 
                    cursor.x-x, 
                    bar.width(), 
                    self.content.0
                );

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    ScrollStateInner::Idle
                } else {
                    ScrollStateInner::ScrollH(x)
                }
            },
            ScrollStateInner::ScrollV(y) => {
                capture = Capture::CaptureFocus(MouseStyle::Arrow);

                let mut bar = layout.clone();
                bar.left = bar.right - self.background_v.image.size.width();
                if self.scrollable_h {
                    bar.bottom = bar.bottom - self.background_h.image.size.height();
                }
                state.scroll.1 = handle_to_scroll(
                    bar.top, 
                    cursor.y-y, 
                    bar.height(), 
                    self.content.1
                );

                if let Event::Release(Key::LeftMouseButton, _) = event {
                    ScrollStateInner::Idle
                } else {
                    ScrollStateInner::ScrollV(y)
                }
            },
        };

        if state.inner != ScrollStateInner::Idle {
            if let Event::Scroll(dx, dy) = event {
                state.scroll.0 = (state.scroll.0 - dx).max(0.0).min(self.content.0);
                state.scroll.1 = (state.scroll.1 - dy).max(0.0).min(self.content.1);
            }
        }

        capture
    }

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition
    ) -> Hover {
        match state.inner {
            ScrollStateInner::ScrollH(_) |
            ScrollStateInner::ScrollV(_) => {
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
                        state.scroll.0, 
                        bar.width(), 
                        self.content.0
                    );
                    bar.left = handle_range.0;
                    bar.right = handle_range.1;

                    if cursor.inside(&bar) {
                        state.inner = ScrollStateInner::HoverH(cursor.x - bar.left);
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
                        state.scroll.1, 
                        bar.height(), 
                        self.content.1
                    );
                    bar.top = handle_range.0;
                    bar.bottom = handle_range.1;

                    if cursor.inside(&bar) {
                        state.inner = ScrollStateInner::HoverV(cursor.y - bar.top);
                        return Hover::HoverIdle;
                    }
                }

                if cursor.inside(&layout) {
                    state.inner = ScrollStateInner::HoverContent;
                    Hover::HoverIdle
                } else {
                    state.inner = ScrollStateInner::Idle;
                    Hover::NoHover
                }
            },
        }
    }

    fn predraw<F: FnMut(Primitive)>(&self, _: &Self::State, layout: Rect, mut submit: F) {
        submit(Primitive::PushClip(layout));
    }

    fn postdraw<F: FnMut(Primitive)>(&self, state: &Self::State, layout: Rect, mut submit: F) {
        // draw h scroll
        if self.scrollable_h {
            let mut bar = layout.clone();
            bar.top = bar.bottom - self.background_h.image.size.height();
            if self.scrollable_v {
                bar.right = bar.right - self.background_v.image.size.width();
            }

            submit(Primitive::Draw9(self.background_h.clone(), bar, Color::white()));

            let handle_range = handle_range(bar.left, state.scroll.0, bar.width(), self.content.0);
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

            let handle_range = handle_range(bar.top, state.scroll.1, bar.height(), self.content.1);
            bar.top = handle_range.0;
            bar.bottom = handle_range.1;

            submit(Primitive::Draw9(self.bar_v.clone(), bar, Color::white()));
        }

        submit(Primitive::PopClip);
    }

    fn child_area(&self, _: &Self::State, mut layout: Rect) -> ChildArea {
        if self.scrollable_h {
            layout.bottom = layout.bottom - self.bar_h.image.size.height();
        }
        if self.scrollable_v {
            layout.right = layout.right - self.bar_v.image.size.width();
        }
        ChildArea::OverflowContentConfineInput(layout)
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