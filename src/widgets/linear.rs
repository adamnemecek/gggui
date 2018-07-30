use super::*;

#[derive(Clone,Copy)]
pub enum Flow {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl Flow {
    pub fn is_horizontal(&self) -> bool {
        match self {
            &Flow::LeftToRight |
            &Flow::RightToLeft => true,
            &Flow::TopToBottom |
            &Flow::BottomToTop => false,
        }
    }

    pub fn is_vertical(&self) -> bool {
        match self {
            &Flow::LeftToRight |
            &Flow::RightToLeft => false,
            &Flow::TopToBottom |
            &Flow::BottomToTop => true,
        }
    }
}

pub struct LinearLayout {
    layout: Layout,
    background: Option<Background>,
    flow: Flow,
    grid: u32,
}

impl LinearLayout {
    pub fn new(layout: Layout, flow: Flow) -> Self {
        Self {
            layout,
            flow,
            background: None,
            grid: 1,
        }
    }

    pub fn with_columns(mut self, columns: u32) -> Self {
        assert!(self.flow.is_vertical());
        self.grid = columns;
        self
    }

    pub fn with_rows(mut self, columns: u32) -> Self {
        assert!(self.flow.is_horizontal());
        self.grid = columns;
        self
    }

    pub fn with_bg(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }
}

impl WidgetBase for LinearLayout {
    fn create(&mut self, id: dag::Id, world: &mut Ui, _style: &Style) { 
        self.background.take().map(|background| {
            let bg = WidgetBackground {
                normal: background.clone(),
                hover: background.clone(),
                click: background.clone(),
            };
            world.create_component(id, bg);
        });   
        world.create_component(id, self.layout.clone());
        world.create_component(id, Clipper{ rect: Rect::from_wh(0.0, 0.0), intersect: true });
    }

    fn update(&mut self, id: dag::Id, world: &mut Ui, _style: &Style, viewport: Viewport) -> Viewport {

        let mut layout: FetchComponent<Layout> = world.component(id).unwrap();

        let mut clipper: FetchComponent<Clipper> = world.component(id).unwrap();

        let window = viewport.child_rect.after_padding(layout.borrow().padding);

        let (r_to_l, b_to_t) = match self.flow {
            Flow::LeftToRight => (false, false),
            Flow::TopToBottom => (false, false),
            Flow::RightToLeft => (true, false),
            Flow::BottomToTop => (false, true),
        };

        let (mut cursor, limit, grow) = {
            let layout = layout.borrow();
            if layout.current.is_none() {
                return viewport;
            }

            let rect = layout.after_padding();
            let w = if let &Constraint::Fixed = &layout.constraints.0 { &rect } else { &window };
            let h = if let &Constraint::Fixed = &layout.constraints.1 { &rect } else { &window };
            let (cursor, limit) = match self.flow {
                Flow::LeftToRight => ((rect.left, rect.top), (w.right, h.bottom)),
                Flow::TopToBottom => ((rect.left, rect.top), (w.right, h.bottom)),
                Flow::RightToLeft => ((rect.right, rect.top), (w.left, h.bottom)),
                Flow::BottomToTop => ((rect.left, rect.bottom), (w.right, h.top)),
            };
            let gw = if let &Constraint::Grow = &layout.constraints.0 { true } else { false };
            let gh = if let &Constraint::Grow = &layout.constraints.1 { true } else { false };
            (cursor, limit, (gw, gh))
        };

        let cursor_start = cursor.clone();

        let mut extents = cursor.clone();

        let cells = if self.grid > 1 {
            let mut cell = 0;
            world.children()
                .filter_map(|id| world.component::<Layout>(*id))
                .fold(Vec::<f32>::new(), |mut cells, layout| {
                    let layout = layout.borrow();

                    if cell >= cells.len() {
                        cells.push(0.0);
                    }
                
                    if self.flow.is_vertical() {
                        cells[cell] = cells[cell].max(layout.current
                            .as_ref()
                            .map(|c| c.width() + layout.margin.left + layout.margin.right)
                            .unwrap_or(0.0));
                    } else {
                        cells[cell] = cells[cell].max(layout.current
                            .as_ref()
                            .map(|c| c.height() + layout.margin.top + layout.margin.bottom)
                            .unwrap_or(0.0));
                    }

                    cell = (cell + 1) % self.grid as usize;
                    cells
                })
        } else {
            vec![]
        };

        let mut fills = world.children()
            .filter_map(|id| world.component::<Layout>(*id))
            .fold((0.0, 0.0), |(w, h), x| match x.borrow().constraints {
                (Constraint::Fill, Constraint::Fill) => (w + 1.0, h + 1.0),
                (Constraint::Fill, _) => (w + 1.0, h),
                (_, Constraint::Fill) => (w, h + 1.0),
                _ => (w, h),
            });

        let mut cell = 0;

        for child in world.children() {
            world.component(*child).map(|mut layout: FetchComponent<Layout>| {
                let mut layout = layout.borrow_mut();
                let w = layout.current.as_ref().map(|c| c.width() + layout.margin.left + layout.margin.right);
                let h = layout.current.as_ref().map(|c| c.height() + layout.margin.top + layout.margin.bottom);
                let w = match &layout.constraints.0 {
                    Constraint::Fixed => w.unwrap_or(0.0),
                    Constraint::Grow => w.unwrap_or(0.0),
                    Constraint::Fill => if self.flow.is_horizontal() {
                        let w = (cursor.0 - limit.0).abs() / fills.0;
                        fills.0 -= 1.0;
                        w
                    } else {
                        (cursor.0 - limit.0).abs()
                    },
                };
                let h = match &layout.constraints.1 {
                    Constraint::Fixed => h.unwrap_or(0.0),
                    Constraint::Grow => h.unwrap_or(0.0),
                    Constraint::Fill => if self.flow.is_vertical() {
                        let h = (cursor.1 - limit.1).abs() / fills.1;
                        fills.1 -= 1.0;
                        h
                    } else {
                        (cursor.1 - limit.1).abs()
                    },
                };
                
                // update layout rect and apply gravity
                layout.current = Some({
                    let mut result = Rect {
                        left:   if r_to_l {cursor.0-w+layout.margin.left} else {cursor.0+layout.margin.left},
                        right:  if r_to_l {cursor.0-layout.margin.right}  else {cursor.0+w-layout.margin.right},
                        top:    if b_to_t {cursor.1-h+layout.margin.top}  else {cursor.1+layout.margin.top},
                        bottom: if b_to_t {cursor.1-layout.margin.bottom} else {cursor.1+h-layout.margin.bottom},
                    };
                    match self.flow {
                        Flow::LeftToRight |
                        Flow::RightToLeft => {
                            let align = limit.1 - cursor.1 - h;
                            let shift = match layout.gravity.1 {
                                Gravity::Begin => 0.0,
                                Gravity::Middle => align * 0.5,
                                Gravity::End => align,
                            };
                            result.top += shift;
                            result.bottom += shift;
                        },
                        Flow::TopToBottom |
                        Flow::BottomToTop => {
                            let align = limit.0 - cursor.0 - w;
                            let shift = match layout.gravity.0 {
                                Gravity::Begin => 0.0,
                                Gravity::Middle => align * 0.5,
                                Gravity::End => align,
                            };
                            result.left += shift;
                            result.right += shift;
                        },
                    };
                    result
                });

                // advance cursor
                if cells.len() > 1 {
                    match self.flow {
                        Flow::LeftToRight => cursor.1 += cells[cell],
                        Flow::TopToBottom => cursor.0 += cells[cell],
                        Flow::RightToLeft => cursor.1 -= cells[cell],
                        Flow::BottomToTop => cursor.0 -= cells[cell],
                    };
                    cell += 1;
                    if cell >= cells.len() {
                        cell = 0;
                        match self.flow {
                            Flow::LeftToRight => cursor = (extents.0, cursor_start.1),
                            Flow::TopToBottom => cursor = (cursor_start.0, extents.1),
                            Flow::RightToLeft => cursor = (extents.0, cursor_start.1),
                            Flow::BottomToTop => cursor = (cursor_start.0, extents.1),
                        };
                    }
                } else {
                    match self.flow {
                        Flow::LeftToRight => cursor.0 += w,
                        Flow::TopToBottom => cursor.1 += h,
                        Flow::RightToLeft => cursor.0 -= w,
                        Flow::BottomToTop => cursor.1 -= h,
                    };
                }

                // apply constraints
                if !grow.0 {
                    if r_to_l {
                        if cursor.0 < limit.0 {
                            let delta = limit.0 - cursor.0;
                            cursor.0 = limit.0;
                            layout.current.as_mut().unwrap().left += delta;
                        }
                    } else {
                        if cursor.0 > limit.0 {
                            let delta = cursor.0 - limit.0;
                            cursor.0 = limit.0;
                            layout.current.as_mut().unwrap().right -= delta;
                        }
                    }
                }
                if !grow.1 {
                    if b_to_t {
                        if cursor.1 < limit.1 {
                            let delta = limit.1 - cursor.1;
                            cursor.1 = limit.1;
                            layout.current.as_mut().unwrap().top += delta;
                        }
                    } else {
                        if cursor.1 > limit.1 {
                            let delta = cursor.1 - limit.1;
                            cursor.1 = limit.1;
                            layout.current.as_mut().unwrap().bottom -= delta;
                        }
                    }
                }

                // validate rect
                if !layout.current.map(|rect| rect.left < rect.right && rect.top < rect.bottom).unwrap_or(false) {
                    //layout.current.take();
                    
                } else {
                    extents.0 = if r_to_l { 
                        extents.0.min(layout.current.as_ref().unwrap().left - layout.margin.left) 
                    } else {
                        extents.0.max(layout.current.as_ref().unwrap().right + layout.margin.right)
                    };
                    extents.1 = if b_to_t {
                        extents.1.min(layout.current.as_ref().unwrap().top - layout.margin.top)
                    } else {
                        extents.1.max(layout.current.as_ref().unwrap().bottom + layout.margin.bottom)
                    };
                }
            });
        }

        // update own layout
        let mut layout = layout.borrow_mut();
        let old = layout.current.unwrap();
        layout.current.as_mut().unwrap().right = match layout.constraints.0.clone() {
            Constraint::Fixed => old.right,
            Constraint::Grow => extents.0 + layout.padding.right,
            Constraint::Fill => window.right + layout.padding.right,
        };
        layout.current.as_mut().unwrap().bottom = match layout.constraints.1.clone() {
            Constraint::Fixed => old.bottom,
            Constraint::Grow => extents.1 + layout.padding.bottom,
            Constraint::Fill => window.bottom + layout.padding.bottom,
        };

        let current = layout.current.as_ref().unwrap();
        let child_rect = Rect {
            left: match layout.constraints.0.clone() { 
                Constraint::Fixed => current.left + layout.padding.left,
                _ => window.left,
            },
            right: match layout.constraints.0.clone() { 
                Constraint::Fixed => current.right - layout.padding.right,
                _ => window.right,
            },
            top: match layout.constraints.1.clone() { 
                Constraint::Fixed => current.top + layout.padding.top,
                _ => window.top,
            },
            bottom: match layout.constraints.1.clone() { 
                Constraint::Fixed => current.bottom - layout.padding.bottom,
                _ => window.bottom,
            },
        };

        let viewport = Viewport {
            child_rect,
            input_rect: viewport.input_rect.and_then(|ir| ir.intersect(&child_rect)),
        };

        clipper.borrow_mut().rect = layout.after_padding();

        viewport
    }
}

impl Widget for LinearLayout {
    type Result = ();

    fn result(&mut self, _id: dag::Id) -> Self::Result { }
}