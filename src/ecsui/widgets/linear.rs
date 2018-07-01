use ecsui::components::background::Background;
use super::*;

#[derive(Clone,Copy)]
pub enum Flow {
	LeftToRight,
	RightToLeft,
	TopToBottom,
	BottomToTop,
}

pub struct LinearLayout {
    layout: Layout,
    flow: Flow,
}

impl LinearLayout {
    pub fn new(layout: Layout, flow: Flow) -> Self {
        Self {
            layout,
            flow,
        }
    }
}

impl WidgetBase for LinearLayout {
    fn create(&mut self, id: dag::Id, world: &mut Ui) {        
        world.create_component(id, self.layout.clone());
    }

    fn update(&mut self, id: dag::Id, world: &Ui, window: Rect) -> Rect {

    	let mut layout: FetchComponent<Layout> = world.component(id).unwrap();

    	let (mut cursor, mut limit) = {
    		let layout = layout.borrow();
            if layout.current.is_none() {
                return;
            }

            let rect = layout.after_padding();
            let w = if let &Constraint::Fixed = &layout.constrain_width { &window } else { &rect };
            let h = if let &Constraint::Fixed = &layout.constrain_height { &window } else { &rect };
    		match self.flow {
	    		Flow::LeftToRight => ((rect.left, rect.top), (w.right, h.bottom)),
	    		Flow::TopToBottom => ((rect.left, rect.top), (w.right, h.bottom)),
	    		Flow::RightToLeft => ((rect.right, rect.top), (w.left, h.bottom)),
	    		Flow::BottomToTop => ((rect.left, rect.bottom), (w.right, h.top)),
    		}
    	};

    	for child in world.children() {
    		world.component(*child).map(|mut layout: FetchComponent<Layout>| {
    			let mut layout = layout.borrow_mut();

    			let w = layout.current.map(|c| c.width());
                let h = layout.current.map(|c| c.height());

                let (w, window_w) = match &layout.constrain_width {
                    Constraint::Fixed => (w.unwrap(), w.unwrap()),
                    Constraint::Grow => (w.unwrap_or(0.0), window.width()),
                    Constraint::Fill => (window.width(), window.width()),
                }

    			//----------------------------------------------------------------------------//
    			// update child layout
    			layout.current = match self.flow {
    				Flow::LeftToRight => {
    					Rect {
		    				left: cursor.0 + layout.margin.left,
		    				top: cursor.1 + layout.margin.top,
		    				right: cursor.0 + layout.margin.left + w,
		    				bottom: if layout.growable_y { 
		    					limit.1 - layout.margin.bottom 
		    				} else { 
		    					cursor.1 + layout.margin.top + h 
		    				},
		    			}
    				},
    				Flow::TopToBottom => {
    					Rect {
		    				left: cursor.0 + layout.margin.left,
		    				top: cursor.1 + layout.margin.top,
		    				right: if layout.growable_x {
		    					limit.0 - layout.margin.right
		    				} else {
		    					cursor.0 + layout.margin.left + w
		    				},
		    				bottom: cursor.1 + layout.margin.top + h,
		    			}
    				},
    				Flow::RightToLeft => {
    					Rect {
		    				left: cursor.0 - layout.margin.right - w,
		    				top: cursor.1 + layout.margin.top,
		    				right: cursor.0 - layout.margin.right,
		    				bottom: if layout.growable_y {
		    					limit.1 - layout.margin.bottom
		    				} else {
		    					cursor.1 + layout.margin.top + h
		    				},
		    			}
    				},
    				Flow::BottomToTop => {
    					Rect {
		    				left: cursor.0 + layout.margin.left,
		    				top: cursor.1 - layout.margin.bottom - h,
		    				right: if layout.growable_x {
		    					limit.0 - layout.margin.right
		    				} else {
		    					cursor.0 + layout.margin.left + w
		    				},
		    				bottom: cursor.1 - layout.margin.bottom,
		    			}
    				},
    			};

    			//----------------------------------------------------------------------------//
    			// constrain to limits
    			if false /*growable*/ {
    				limit.0 = match self.flow {
    					Flow::RightToLeft => limit.0.min(layout.current.left - layout.margin.left),
    					_ => limit.0.max(layout.current.right + layout.margin.right),
    				};
    				limit.1 = match self.flow {
    					Flow::BottomToTop => limit.1.min(layout.current.top - layout.margin.top),
    					_ => limit.1.max(layout.current.bottom + layout.margin.bottom),
    				};
    			} else {
    				match self.flow {
    					Flow::LeftToRight => {
    						layout.current.right = layout.current.right.min(limit.0 - layout.margin.right);
    						layout.current.bottom = layout.current.bottom.min(limit.1 - layout.margin.bottom);
    					},
    					Flow::TopToBottom => {
    						layout.current.right = layout.current.right.min(limit.0 - layout.margin.right);
    						layout.current.bottom = layout.current.bottom.min(limit.1 - layout.margin.bottom);
    					},
    					Flow::RightToLeft => {
    						layout.current.left = layout.current.left.max(limit.0 + layout.margin.left);
    						layout.current.bottom = layout.current.bottom.min(limit.1 - layout.margin.bottom);
    					},
    					Flow::BottomToTop => {
    						layout.current.right = layout.current.right.min(limit.0 - layout.margin.right);
    						layout.current.top = layout.current.top.max(limit.1 + layout.margin.top);
    					},
    				}
    			}

    			//----------------------------------------------------------------------------//
    			// validate
    			layout.valid = 
    				layout.current.right > layout.current.left && 
    				layout.current.bottom > layout.current.top;
    			
    			//----------------------------------------------------------------------------//
    			// update cursor
    			cursor = match self.flow {
    				Flow::LeftToRight => {
    					(cursor.0 + layout.margin.left + layout.margin.right + w, cursor.1)
    				},
    				Flow::TopToBottom => {
    					(cursor.0, cursor.1 + layout.margin.top + layout.margin.bottom + h)
    				},
    				Flow::RightToLeft => {
    					(cursor.0 - layout.margin.left + layout.margin.right + w, cursor.1)
    				},
    				Flow::BottomToTop => {
    					(cursor.0, cursor.1 - layout.margin.top + layout.margin.bottom + h)
    				},
    			};
    		});
    	}

    	if self.growable {
    		let mut layout = layout.borrow_mut();
    		match self.flow {
    			Flow::LeftToRight |
    			Flow::TopToBottom => {
    				layout.current.right = limit.0 + layout.padding.right;
    				layout.current.bottom = limit.1 + layout.padding.bottom;
    			},
	    		Flow::RightToLeft => {
    				layout.current.left = limit.0 - layout.padding.left;
    				layout.current.bottom = limit.1 + layout.padding.bottom;
    			},
	    		Flow::BottomToTop => {
    				layout.current.right = limit.0 - layout.padding.left;
    				layout.current.bottom = limit.1 + layout.padding.bottom;
    			},
    		};
    		layout.valid = 
				layout.current.right > layout.current.left && 
				layout.current.bottom > layout.current.top;
    	}
    }

    fn event(&mut self, _id: dag::Id, _world: &Ui, _ev: Event, _focus: bool) -> Capture {
        // todo
        Capture::None
    }
}

impl Widget for LinearLayout {
    type Result = ();

    fn result(&self, _id: dag::Id) -> Self::Result { }
}