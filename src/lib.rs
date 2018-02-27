#[macro_use]
extern crate downcast;
extern crate smallvec;
extern crate rusttype;
extern crate image;
extern crate clipboard;

#[cfg(feature="vulkano-renderer")] #[macro_use] pub extern crate vulkano;
#[cfg(feature="vulkano-renderer")] #[macro_use] pub extern crate vulkano_shader_derive;
#[cfg(feature="winit-events")] pub extern crate winit;

use std::mem::replace;
use std::collections::HashMap;
use smallvec::SmallVec;

pub mod features;
pub mod widgets;
pub mod events;
pub mod primitive;
pub mod render;
#[macro_use]
pub mod loadable;
pub mod style;

mod cache;
#[allow(dead_code)]
mod qtree;

pub use self::widgets::*;
pub use self::events::*;
pub use self::primitive::*;
pub use self::render::*;
pub use self::style::*;
use self::cache::Cache;
use self::loadable::*;

pub type EventVec = SmallVec<[Event; 4]>;

pub type Font = (self::cache::Font, self::cache::FontId);

struct Window {
    drawlist: Vec<Primitive>,
    rect: Rect,
    id: String,
    updated: bool,
    modal: bool,
}

pub struct Ui {
    focus: Option<(String, Box<WidgetState>)>,
    last_tabstop_id: Option<String>,
    focus_to_tabstop: Option<String>,
    state: HashMap<String, Box<WidgetState>>,
    cursor: (f32,f32),
    previous_capture: Capture,
    cache: Cache,
    windows: Vec<Window>,
    active_window: Option<Window>,
}

pub struct UiContext<'a> {
    ui: &'a mut Ui,
    drawlist: Vec<Primitive>,
    drawlist_sub: Vec<Primitive>,
    parent: Box<Layout+'a>,
    events: EventVec,
    capture: Capture,
    mouse_style: Option<MouseStyle>,
    cursor: MousePosition,
    viewport: Rect,
    enabled: bool,
}

impl Ui {
    pub fn new() -> Ui {
        Ui {
            focus: None,
            last_tabstop_id: None,
            focus_to_tabstop: None,
            state: HashMap::new(),
            cursor: (0.0, 0.0),
            previous_capture: Capture::None,
            cache: Cache::new(2048),
            windows: Vec::new(),
            active_window: None,
        }
    }

    pub fn get_patch<L: Loadable>(&mut self, load: L) -> Patch {
        self.cache.get_patch(load)
    }

    pub fn get_image<L: Loadable>(&mut self, load: L) -> Image {
        self.cache.get_image(load)
    }

    pub fn get_font<L: Loadable>(&mut self, load: L) -> Font {
        self.cache.get_font(load)
    }

    pub fn reset_state(&mut self) {
        self.focus = None;
        self.state.clear();
        self.previous_capture = Capture::None;
    }

    pub fn start<'a>(&'a mut self, mut events: Vec<Event>, viewport: Rect) -> UiContext<'a> {
        for event in events.iter() {
            match event {
                &Event::Cursor(x, y) => self.cursor = (x, y),
                _ => (),
            }
        }

        if events.len() == 0 {
            events.push(Event::Idle);
        }

        let cursor = MousePosition{ 
            x: self.cursor.0, 
            y: self.cursor.1, 
            visibility: Some(viewport) 
        };

        let mut enabled = self.active_window.is_none();

        // set all windows to not-updated so they can be garbage collected
        // after the next loop (if not updated again)
        for window in self.windows.iter_mut() {
            window.updated = false;
            enabled &= !cursor.inside(&window.rect);
        }
        for window in self.active_window.iter_mut() { 
            window.updated = false;
            enabled &= !cursor.inside(&window.rect);
        }

        UiContext { 
            ui: self,
            drawlist: Vec::new(),
            drawlist_sub: Vec::new(),
            parent: Box::new(LayoutRoot{ viewport }),
            events: EventVec::from_vec(events),
            capture: Capture::None,
            mouse_style: None,
            cursor: cursor,
            viewport: viewport,
            enabled: enabled,
        }
    }

    fn get_state<'a, W: Widget>(&'a mut self, w_id: &str) -> W::State {
        if w_id != "" {
            match W::state_type() {
                StateType::Focus => 
                    match &self.focus {
                        &Some((ref id, ref state)) => {
                            if w_id == id {
                                state.downcast_ref::<W::State>().unwrap().clone()
                            } else {
                                W::default()
                            }
                        },
                        &None => W::default(),
                    },
                StateType::Persistent => self.state
                    .entry(w_id.to_string())
                    .or_insert(Box::new(W::default()))
                    .downcast_ref::<W::State>()
                    .unwrap()
                    .clone(),
            }
        } else {
            W::default()
        }
    }

    fn update_state<W: Widget>(&mut self, w_id: &str, new_state: W::State, is_focused: bool) {
        if w_id != "" {
            if W::tabstop() {
                self.last_tabstop_id = Some(w_id.to_string());
            }

            match W::state_type() {
                StateType::Focus => {
                    if is_focused {
                        self.focus = Some((w_id.to_string(), Box::new(new_state)));
                    } else {
                        match &mut self.focus {
                            &mut Some((ref id, ref mut state)) => {
                                if w_id == id {
                                    *state.downcast_mut::<W::State>().unwrap() = new_state;
                                }
                            },
                            _ => (),
                        }
                    }
                },
                StateType::Persistent => {
                    self.state.insert(w_id.to_string(), Box::new(new_state));
                },
            }
        }
    }
}

impl<'a> UiContext<'a> {
    fn sub<'b>(
        &'b mut self, 
        parent: Box<Layout+'b>, 
        events: EventVec,
        cursor: MousePosition,
        enabled: bool,
        drawlist_sub: bool
    ) -> UiContext<'b> {
        let drawlist = if drawlist_sub {
            replace(&mut self.drawlist_sub, Vec::new())
        } else {
            replace(&mut self.drawlist, Vec::new())
        };

        UiContext {
            ui: self.ui,
            drawlist: drawlist,
            drawlist_sub: Vec::new(),
            parent: parent,
            events: events,
            capture: self.capture,
            mouse_style: None,
            cursor: cursor,
            viewport: self.viewport,
            enabled: enabled,
        }
    }

    pub fn window<
        F: FnOnce(&mut UiContext)
    > (
        &mut self,
        id: &str,
        properties: WindowProperties,
        children: F
    ) {
        self.custom_window(
            id, 
            properties.default_size, 
            properties.modal, 
            properties.centered,
            |ctx, win| {
                ctx.nested(id, WindowController::new(properties, win), children);
            }
        );
    }

    pub fn custom_window<
        F: FnOnce(&mut UiContext, &mut Rect)
    > (
        &mut self, 
        id: &str, 
        default_size: Rect, 
        modal: bool, 
        centered: bool,
        children: F
    ) {
        let window_position = self.ui.windows.iter().position(|w| w.id == id);

        let occluded = window_position.as_ref().map_or(true, |i| {
            for j in (i+1)..self.ui.windows.len() {
                if self.cursor.inside(&self.ui.windows[j].rect) {
                    return true;
                }
            }
            for w in self.ui.active_window.as_ref() {
                if self.cursor.inside(&w.rect) {
                    return true;
                }
            }
            false
        });

        let enabled = self.ui.active_window.as_ref().map_or(!occluded, |w| w.id == id);

        let (win, capture, mouse_style) = {
            let mut size = self.ui.active_window.as_ref()
                .and_then(|w| if w.id == id { Some(w.rect) } else { None })
                .or_else(|| {
                    let default_size = if centered {
                        let xy = (
                            (self.viewport.left+self.viewport.right-default_size.width()) * 0.5,
                            (self.viewport.top+self.viewport.bottom-default_size.height()) * 0.5
                        );
                        default_size.size().translate(xy.0, xy.1)
                    } else {
                        default_size
                    };
                    window_position.as_ref()
                        .and_then(|i| Some(self.ui.windows[*i].rect))
                        .or(Some(default_size))
                }).unwrap();

            let mut sub = UiContext {
                ui: self.ui,
                drawlist: Vec::new(),
                drawlist_sub: Vec::new(),
                parent: Box::new(LayoutRoot{ viewport: size }),
                events: self.events.clone(),
                capture: Capture::None,
                mouse_style: None,
                cursor: self.cursor.expand(&size),
                viewport: self.viewport,
                enabled: enabled
            };

            sub.drawlist.push(Primitive::PushClip(self.viewport));
            children(&mut sub, &mut size);
            sub.drawlist.append(&mut sub.drawlist_sub);
            sub.drawlist.push(Primitive::PopClip);

            (Window {
                drawlist: sub.drawlist,
                rect: size,
                id: id.to_string(),
                updated: true,
                modal: modal,
            }, sub.capture, sub.mouse_style)
        };

        if modal {
            if !enabled {
                self.ui.active_window.take().map(|w| {
                    assert!(!w.modal);
                    self.ui.windows.push(w)
                });
                self.ui.focus.take();
            }
            self.ui.active_window = Some(win);
        } else if let Capture::CaptureFocus(_) = capture {
            self.ui.active_window = Some(win);
        } else {
            if let Some(i) = window_position {
                self.ui.windows[i] = win;
            } else {
                self.ui.windows.push(win);
            }
        }

        if capture != Capture::None {
            self.capture = capture;
        }

        if mouse_style.is_some() {
            self.mouse_style = mouse_style;
        }
    }

    pub fn simple<W: Widget>(&mut self, id: &str, widget: W) -> W::Result {
        self.nested(id, widget, |_ui|{ })
    }

    pub fn stateless<W: Widget>(&mut self, widget: W) -> W::Result {
        self.nested("", widget, |_ui|{ })
    }

    pub fn nested<W, F>(&mut self, id: &str, mut widget: W, children: F) -> W::Result 
    where
        W: Widget, 
        F: FnOnce(&mut UiContext),
    {
        let mut state = self.ui.get_state::<W>(id);
        let mut is_focused = self.ui.focus.as_ref().map_or(false, |f| f.0 == id);

        let enabled = self.enabled && widget.enabled(&state);

        //--------------------------------------------------------------------------------------//
        // handle layouting
        let layout = self.parent.estimate(Box::new(|layout| widget.measure(&state, layout)));

        //--------------------------------------------------------------------------------------//
        // handle tabstops
        if W::tabstop() && id != "" && enabled {
            if self.ui.focus.is_none() && widget.autofocus() { 
                is_focused = true;
            }
            if self.capture == Capture::FocusNext && 
                self.ui.focus_to_tabstop.is_none() {
                self.ui.focus_to_tabstop = Some(id.to_string());
            }
            if self.ui.previous_capture == Capture::FocusNext && 
                self.ui.focus_to_tabstop.is_none() {
                self.ui.focus_to_tabstop = Some(id.to_string());
            }
            if (self.ui.previous_capture == Capture::FocusPrev ||
                self.ui.previous_capture == Capture::FocusNext) &&
                self.ui.focus_to_tabstop.as_ref().map_or(false, |t| t == id) {
                self.ui.focus_to_tabstop = None;
                self.ui.previous_capture = Capture::None;
                is_focused = true;
            }
        }

        //--------------------------------------------------------------------------------------//
        // hover for predraw + predraw
        if enabled && (is_focused || self.ui.previous_capture == Capture::None) {
            widget.hover(&mut state, layout, self.cursor);
        }
        if let ChildArea::Popup(_) = widget.child_area(&state, layout) {
            widget.predraw(&state, layout, |p| self.drawlist_sub.push(p));
        } else {
            widget.predraw(&state, layout, |p| self.drawlist.push(p));
        }

        //--------------------------------------------------------------------------------------//
        // handle children
        let (child_capture, child_mouse_style) = match widget.child_area(&state, layout) {
            ChildArea::None => (Capture::None, None),
            child_type => {
                let layouter = LayoutCell::new(&mut widget, &mut state, layout);
                let events = self.events.clone();
                let (cursor, drawlist_sub, clip_vis) = match child_type {
                    ChildArea::ConfineContentAndInput(ref clip) => 
                        (self.cursor.sub(clip), false, true),
                    ChildArea::OverflowContentConfineInput(ref clip) => 
                        (self.cursor.sub(clip), false, false),
                    ChildArea::OverflowContentAndInput =>
                        (self.cursor, false, false),
                    ChildArea::Popup(ref clip) => 
                        (self.cursor.expand(clip), true, true),
                    _ => unreachable!(),
                };

                cursor.visibility.map_or((Capture::None, None), |vis| {
                    let (capture, mouse_style, new_drawlist) = {
                        let mut sub = self.sub(
                            Box::new(layouter), 
                            events, 
                            cursor, 
                            enabled,
                            drawlist_sub
                        );
                        
                        if clip_vis {    
                            sub.drawlist.push(Primitive::PushClip(vis));
                            children(&mut sub);
                            sub.drawlist.append(&mut sub.drawlist_sub);
                            sub.drawlist.push(Primitive::PopClip);
                        } else {
                            children(&mut sub);
                            sub.drawlist.append(&mut sub.drawlist_sub);
                        }

                        (sub.capture, sub.mouse_style, sub.drawlist)
                    };

                    if drawlist_sub {
                        replace(&mut self.drawlist_sub, new_drawlist);
                    } else {
                        replace(&mut self.drawlist, new_drawlist);
                    }

                    (capture, mouse_style)
                })
            },
        };

        //--------------------------------------------------------------------------------------//
        // handle events
        if child_capture == Capture::None && enabled {
            let hover = if is_focused || self.ui.previous_capture == Capture::None {
                widget.hover(&mut state, layout, self.cursor)
            } else {
                Hover::NoHover
            };

            let is_hovered = match hover {
                Hover::NoHover => false,
                Hover::HoverActive(mouse_style) => {
                    self.mouse_style = Some(mouse_style);
                    true
                },
                Hover::HoverIdle => true,
            };

            if is_hovered || is_focused {
                for event in self.events.iter() {
                    match widget.event(&mut state, layout, self.cursor, event.clone(), is_focused) {
                        Capture::CaptureFocus(mouse_style) => {
                            if id != "" {
                                self.capture = Capture::CaptureFocus(mouse_style);
                                is_focused = true;
                            }
                        },
                        Capture::CaptureMouse(mouse_style) => {
                            if id != "" {
                                self.capture = Capture::CaptureMouse(mouse_style);
                                is_focused = true;
                            }
                        },
                        _ => if is_focused {
                            match event {
                                &Event::Press(Key::Tab, Modifiers{ shift: false, .. }) => {
                                    self.capture = Capture::FocusNext;
                                },
                                &Event::Press(Key::Tab, Modifiers{ shift: true, .. }) => {
                                    self.capture = Capture::FocusPrev;
                                    self.ui.focus_to_tabstop = self.ui.last_tabstop_id.clone();
                                },
                                &_ => (),
                            }
                        },
                    }
                }
            }

            self.ui.update_state::<W>(id, state.clone(), is_focused);
        } else {
            self.capture = child_capture;
            self.ui.update_state::<W>(id, state.clone(), false);
        }

        if child_mouse_style.is_some() {
            self.mouse_style = child_mouse_style;
        }

        if let ChildArea::Popup(_) = widget.child_area(&state, layout) {
            widget.postdraw(&state, layout, |p| self.drawlist_sub.push(p));
        } else {
            widget.postdraw(&state, layout, |p| self.drawlist.push(p));
        }
        if W::tabstop() && is_focused && enabled {
            self.drawlist.push(Primitive::DrawRect(layout, Color::white().with_alpha(0.16)));
        }

        // apply final layout. Parent will measure this widget again but now
        //  we know how much content there is including children
        self.parent.layout(Box::new(|layout| widget.measure(&state, layout)));

        widget.result(&state)
    }

    pub fn finish(self) -> (DrawList, MouseStyle, MouseMode) {
        self.ui.previous_capture = self.capture;
        self.ui.windows.retain(|w| w.updated);
        if let Some(&Window{ updated: false, .. }) = self.ui.active_window.as_ref() {
            self.ui.active_window = None;
        }

        // drop focus if nothing was clicked
        if let Capture::None = self.capture {
            for event in self.events.iter() {
                if let &Event::Press(Key::LeftMouseButton, _) = event {
                    self.ui.focus = Some(("".to_string(), Box::new(())));
                }
            }
        }

        let mut vtx = Vec::new();
        let mut cmd = Vec::new();

        let mut scissors = Vec::new();
        scissors.push(self.viewport);

        let mut current_command = Command::Nop;

        let mut draw_lists = vec![self.drawlist];
        for window in self.ui.windows.iter_mut() {
            draw_lists.push(replace(&mut window.drawlist, vec![]));
        }
        for window in self.ui.active_window.iter_mut() {
            draw_lists.push(replace(&mut window.drawlist, vec![]));
        }

        let vp = self.viewport.clone();
        let validate_clip = |clip: Rect| {
            let v = Rect { 
                left: clip.left.max(0.0).min(vp.right), 
                top: clip.top.max(0.0).min(vp.bottom), 
                right: clip.right.max(0.0).min(vp.right),
                bottom: clip.bottom.max(0.0).min(vp.bottom)
            };
            if v.width() > 0.0 && v.height() > 0.0 {
                Some(v)
            } else {
                None
            }
        };

        let mut draw_enabled = true;

        for primitive in draw_lists.into_iter().flat_map(|d| d) {
            match primitive {
                Primitive::PushClip(scissor) => {
                    scissors.push(scissor);

                    draw_enabled = validate_clip(scissor).map_or_else(
                        | | false, 
                        |s| {
                            current_command
                                .append(Command::Clip{ scissor: s })
                                .and_then(|c| Some(cmd.push(c)));

                            true
                        }
                    );
                },

                Primitive::PopClip => {
                    scissors.pop();
                    let scissor = scissors[scissors.len()-1];

                    draw_enabled = validate_clip(scissor).map_or_else(
                        | | false, 
                        |s| {
                            current_command
                                .append(Command::Clip{ scissor: s })
                                .and_then(|c| Some(cmd.push(c)));

                            true
                        }
                    );
                },

                Primitive::DrawRect(r, color) => if draw_enabled {
                    let r = r.to_device_coordinates(self.viewport);
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 2;
                    let offset = vtx.len();
                    vtx.push(Vertex{ 
                        pos: [r.left, r.top],     uv: [0.0; 2], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.right, r.top],    uv: [0.0; 2], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.right, r.bottom], uv: [0.0; 2], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.left, r.top],     uv: [0.0; 2], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.right, r.bottom], uv: [0.0; 2], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.left, r.bottom],  uv: [0.0; 2], color, mode 
                    });

                    current_command
                        .append(Command::Colored{ offset, count: 6 })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::DrawText(text, rect, color) => if draw_enabled {
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 0;
                    let offset = vtx.len();
                    let vp = self.viewport;

                    self.ui.cache.draw_text(  
                        &text, 
                        rect,
                        |uv, pos| {
                            let rc = pos.to_device_coordinates(vp);
                            vtx.push(Vertex{ 
                                pos: [rc.left, rc.top],     uv: uv.pt(0.0, 0.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.right, rc.top],    uv: uv.pt(1.0, 0.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.right, rc.bottom], uv: uv.pt(1.0, 1.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.left, rc.top],     uv: uv.pt(0.0, 0.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.right, rc.bottom], uv: uv.pt(1.0, 1.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.left, rc.bottom],  uv: uv.pt(0.0, 1.0), color, mode 
                            });
                        }
                    );

                    current_command
                        .append(Command::Textured{
                            texture: 0,
                            offset, 
                            count: vtx.len()-offset })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::Draw9(patch, rect, color) => if draw_enabled {
                    let uv = patch.image.texcoords;
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 1;
                    let offset = vtx.len();
                    let vp = self.viewport;

                    patch.iterate_sections(false, rect.width(), |x, u| {
                        patch.iterate_sections(true, rect.height(), |y, v| {
                            let rc = Rect {
                                left: x.0 + rect.left,
                                right: x.1 + rect.left,
                                top: y.0 + rect.top,
                                bottom: y.1 + rect.top,
                            }.to_device_coordinates(vp);

                            vtx.push(Vertex{ 
                                pos: [rc.left, rc.top],     uv: uv.pt(u.0, v.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.right, rc.top],    uv: uv.pt(u.1, v.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.right, rc.bottom], uv: uv.pt(u.1, v.1), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.left, rc.top],     uv: uv.pt(u.0, v.0), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.right, rc.bottom], uv: uv.pt(u.1, v.1), color, mode 
                            });
                            vtx.push(Vertex{ 
                                pos: [rc.left, rc.bottom],  uv: uv.pt(u.0, v.1), color, mode 
                            });
                        });
                    });

                    current_command
                        .append(Command::Textured{ 
                            texture: patch.image.texture, 
                            offset, 
                            count: vtx.len()-offset 
                        })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::DrawImage(image, r, color) => if draw_enabled {
                    let r = r.to_device_coordinates(self.viewport);
                    let uv = image.texcoords;
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 1;
                    let offset = vtx.len();

                    vtx.push(Vertex{ 
                        pos: [r.left, r.top],     uv: [uv.left, uv.top],     color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.right, r.top],    uv: [uv.right, uv.top],    color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.right, r.bottom], uv: [uv.right, uv.bottom], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.left, r.top],     uv: [uv.left, uv.top],     color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.right, r.bottom], uv: [uv.right, uv.bottom], color, mode 
                    });
                    vtx.push(Vertex{ 
                        pos: [r.left, r.bottom],  uv: [uv.left, uv.bottom],  color, mode 
                    });

                    current_command
                        .append(Command::Textured{ texture: image.texture, offset, count: 6 })
                        .and_then(|c| Some(cmd.push(c)));
                },
            }
        }

        // Flush any commands that are not finalized
        current_command.flush().and_then(|c| Some(cmd.push(c)));

        // Resolve mouse mode and style for current frame
        let (mouse_style, mouse_mode) = match self.capture {
            Capture::CaptureFocus(style) => (style, MouseMode::Normal),
            Capture::CaptureMouse(style) => (style, MouseMode::Confined),
            _ => (self.mouse_style.unwrap_or(MouseStyle::Arrow), MouseMode::Normal)
        };

        (DrawList {
            updates: self.ui.cache.take_updates(),
            vertices: vtx,
            commands: cmd
        }, mouse_style, mouse_mode)
    }
}