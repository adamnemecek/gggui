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

        UiContext { 
            ui: self,
            drawlist: Vec::new(),
            drawlist_sub: Vec::new(),
            parent: Box::new(LayoutRoot{ viewport }),
            events: EventVec::from_vec(events),
            capture: Capture::None,
            cursor: cursor,
            viewport: viewport,
            enabled: true,
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
            cursor: cursor,
            viewport: self.viewport,
            enabled: enabled,
        }
    }

    pub fn ui<W: Widget>(&mut self, id: &str, widget: W) -> W::Result {
        self.ui_with_inner(id, widget, |_ui|{ })
    }

    pub fn ui_stateless<W: Widget>(&mut self, widget: W) -> W::Result {
        self.ui_with_inner("", widget, |_ui|{ })
    }

    pub fn ui_with_inner<
        W: Widget, 
        F: FnOnce(&mut UiContext)
    > (
        &mut self, 
        id: &str, 
        mut widget: W, 
        children: F
    ) -> W::Result {
        let mut state = self.ui.get_state::<W>(id);
        let mut is_focused = self.ui.focus.as_ref().map_or(false, |f| f.0 == id);

        let enabled = (self.enabled && widget.enabled(&state)) || W::window();

        //--------------------------------------------------------------------------------------//
        // handle layouting
        let layout = self.parent.layout(Box::new(|layout| widget.measure(&state, layout)));

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
        // handle events, children and rendering
        let is_hovered = if !enabled {
            false
        } else if is_focused || self.ui.previous_capture == Capture::None {
            widget.hover(&mut state, layout, self.cursor)
        } else {
            false
        };

        widget.predraw(&state, layout, |p| self.drawlist.push(p));

        let child_capture = match widget.childs(&state, layout) {
            ChildType::None => Capture::None,
            child_type => {
                let layouter = LayoutCell::new(&mut widget, &mut state, layout);
                let events = self.events.clone();
                let (cursor, drawlist_sub) = match child_type {
                    ChildType::Intersect(ref clip) => (self.cursor.sub(clip), false),
                    ChildType::Expand(ref clip) => (self.cursor.expand(clip), true),
                    _ => unreachable!(),
                };

                cursor.visibility.map_or(Capture::None, |vis| {
                    let (capture, new_drawlist) = {
                        let mut sub = self.sub(
                            Box::new(layouter), 
                            events, 
                            cursor, 
                            enabled,
                            drawlist_sub
                        );
                            
                        sub.drawlist.push(Primitive::PushClip(vis));
                        children(&mut sub);
                        sub.drawlist.append(&mut sub.drawlist_sub);
                        sub.drawlist.push(Primitive::PopClip);

                        (sub.capture, sub.drawlist)
                    };

                    if drawlist_sub {
                        replace(&mut self.drawlist_sub, new_drawlist);
                    } else {
                        replace(&mut self.drawlist, new_drawlist);
                    }

                    capture
                })
            },
        };

        if child_capture == Capture::None && enabled {
            if is_hovered || is_focused {
                for event in self.events.iter() {
                    match widget.event(&mut state, layout, self.cursor, event.clone(), is_focused) {
                        Capture::CaptureFocus => {
                            if id != "" {
                                self.capture = Capture::CaptureFocus;
                                is_focused = true;
                            }
                        },
                        Capture::CaptureMouse => {
                            if id != "" {
                                self.capture = Capture::CaptureMouse;
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

        widget.postdraw(&state, layout, |p| self.drawlist.push(p));
        if W::tabstop() && is_focused && enabled {
            self.drawlist.push(Primitive::DrawRect(layout, Color::white().with_alpha(0.16)));
        }
        widget.result(&state)
    }

    pub fn draw(self) -> DrawList {
        self.ui.previous_capture = self.capture;

        let mut vtx = Vec::new();
        let mut cmd = Vec::new();

        let mut scissors = Vec::new();
        scissors.push(self.viewport);

        let mut current_command = Command::Nop;

        for primitive in self.drawlist {
            match primitive {
                Primitive::PushClip(scissor) => {
                    scissors.push(scissor);

                    current_command
                        .append(Command::Clip{ scissor })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::PopClip => {
                    scissors.pop();
                    let scissor = scissors[scissors.len()-1];

                    current_command
                        .append(Command::Clip{ scissor })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::DrawRect(r, color) => {
                    let r = r.to_device_coordinates(self.viewport);
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 2;
                    let offset = vtx.len();
                    vtx.push(Vertex{ pos: [r.left, r.top],     uv: [0.0; 2], color, mode });
                    vtx.push(Vertex{ pos: [r.right, r.top],    uv: [0.0; 2], color, mode });
                    vtx.push(Vertex{ pos: [r.right, r.bottom], uv: [0.0; 2], color, mode });
                    vtx.push(Vertex{ pos: [r.left, r.top],     uv: [0.0; 2], color, mode });
                    vtx.push(Vertex{ pos: [r.right, r.bottom], uv: [0.0; 2], color, mode });
                    vtx.push(Vertex{ pos: [r.left, r.bottom],  uv: [0.0; 2], color, mode });

                    current_command
                        .append(Command::Colored{ offset, count: 6 })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::DrawText(text, rect, color) => {
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 0;
                    let offset = vtx.len();
                    let vp = self.viewport;

                    self.ui.cache.draw_text(  
                        &text, 
                        rect,
                        |uv, pos| {
                            let rc = pos.to_device_coordinates(vp);
                            vtx.push(Vertex{ pos: [rc.left, rc.top],     uv: uv.pt(0.0, 0.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.right, rc.top],    uv: uv.pt(1.0, 0.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.right, rc.bottom], uv: uv.pt(1.0, 1.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.left, rc.top],     uv: uv.pt(0.0, 0.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.right, rc.bottom], uv: uv.pt(1.0, 1.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.left, rc.bottom],  uv: uv.pt(0.0, 1.0), color, mode });
                        }
                    );

                    current_command
                        .append(Command::Textured{
                            texture: 0,
                            offset, 
                            count: vtx.len()-offset })
                        .and_then(|c| Some(cmd.push(c)));
                },

                Primitive::Draw9(patch, rect, color) => {
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

                            vtx.push(Vertex{ pos: [rc.left, rc.top],     uv: uv.pt(u.0, v.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.right, rc.top],    uv: uv.pt(u.1, v.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.right, rc.bottom], uv: uv.pt(u.1, v.1), color, mode });
                            vtx.push(Vertex{ pos: [rc.left, rc.top],     uv: uv.pt(u.0, v.0), color, mode });
                            vtx.push(Vertex{ pos: [rc.right, rc.bottom], uv: uv.pt(u.1, v.1), color, mode });
                            vtx.push(Vertex{ pos: [rc.left, rc.bottom],  uv: uv.pt(u.0, v.1), color, mode });
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

                Primitive::DrawImage(image, r, color) => {
                    let r = r.to_device_coordinates(self.viewport);
                    let uv = image.texcoords;
                    let color = [color.r, color.g, color.b, color.a];
                    let mode = 1;
                    let offset = vtx.len();

                    vtx.push(Vertex{ pos: [r.left, r.top],     uv: [uv.left, uv.top],     color, mode });
                    vtx.push(Vertex{ pos: [r.right, r.top],    uv: [uv.right, uv.top],    color, mode });
                    vtx.push(Vertex{ pos: [r.right, r.bottom], uv: [uv.right, uv.bottom], color, mode });
                    vtx.push(Vertex{ pos: [r.left, r.top],     uv: [uv.left, uv.top],     color, mode });
                    vtx.push(Vertex{ pos: [r.right, r.bottom], uv: [uv.right, uv.bottom], color, mode });
                    vtx.push(Vertex{ pos: [r.left, r.bottom],  uv: [uv.left, uv.bottom],  color, mode });

                    current_command
                        .append(Command::Textured{ texture: image.texture, offset, count: 6 })
                        .and_then(|c| Some(cmd.push(c)));
                },
            }
        }

        // Flush any commands that are not finalized
        current_command.flush().and_then(|c| Some(cmd.push(c)));

        DrawList {
            updates: self.ui.cache.take_updates(),
            vertices: vtx,
            commands: cmd
        }
    }
}