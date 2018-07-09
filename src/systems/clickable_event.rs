use super::*;

pub struct ClickableEventSystem { }

impl System<EventSystemContext> for ClickableEventSystem {
    type Components = (
        FetchComponent<Layout>, 
        FetchComponent<Clickable>
    );
    fn run(&self, context: &mut EventSystemContext, (layout, mut state): Self::Components) {
        let mut state = state.borrow_mut();
        let state = state.deref_mut();
        let rect = layout.borrow().current;

        if rect.is_none() {
            return;
        }

        let rect = rect.unwrap();

        *state = match state {
            Clickable::Idle => {
                if context.focused {
                    if let Event::Press(Key::Space, _) = context.event {
                        context.capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                        Clickable::Clicked(true)
                    } else {
                        if context.cursor.inside(&rect) {
                            Clickable::Hovering
                        } else {
                            Clickable::Idle
                        }
                    }
                } else {
                    if context.cursor.inside(&rect) {
                        Clickable::Hovering
                    } else {
                        Clickable::Idle
                    }
                }
            },
            Clickable::Hovering => {
                if let Event::Press(Key::LeftMouseButton, _) = context.event {
                    context.capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                    Clickable::Clicked(true)
                } else {
                    if context.cursor.inside(&rect) {
                        Clickable::Hovering
                    } else {
                        Clickable::Idle
                    }
                }
            },
            Clickable::Clicked(hit) => {
                context.capture = Capture::CaptureFocus(MouseStyle::ArrowClicking);
                match context.event {
                    Event::Release(Key::LeftMouseButton, _) => {
                        if context.cursor.inside(&rect) {
                            Clickable::Released(*hit)
                        } else {
                            Clickable::Released(false)
                        }
                    },
                    Event::Release(Key::Space, _) => {
                        Clickable::Released(true)
                    },
                    _ => {
                        if context.cursor.inside(&rect) {
                            Clickable::Clicked(*hit)
                        } else {
                            Clickable::Clicked(false)
                        }
                        
                    },
                }
            },
            Clickable::Released(hit) => {
                Clickable::Released(*hit)
            },
        }
    }
}