extern crate downcast;
extern crate smallvec;
extern crate rusttype;
extern crate image;
extern crate clipboard;

#[cfg(feature="gfx-renderer")] #[macro_use] pub extern crate gfx;
#[cfg(feature="vulkano-renderer")] #[macro_use] pub extern crate vulkano;
#[cfg(feature="vulkano-renderer")] #[macro_use] pub extern crate vulkano_shader_derive;
#[cfg(feature="winit-events")] pub extern crate winit;

use std::mem::replace;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::cell::RefMut;
use std::ops::Deref;
use std::ops::DerefMut;
use std::any::Any;
use std::any::TypeId;
use smallvec::SmallVec;

pub mod features;
pub mod events;
pub mod primitive;
pub mod render;
#[macro_use]
pub mod loadable;
pub mod systems;
pub mod components;
pub mod widgets;
pub mod dag;
pub mod entry;
mod cache;
#[allow(dead_code)]
mod qtree;

pub use self::widgets::*;
pub use self::events::*;
pub use self::primitive::*;
pub use self::render::*;
pub use self::widgets::*;
pub use self::components::*;
pub use self::loadable::*;
use self::cache::Cache;
use self::systems::*;

pub type Font = (self::cache::Font, self::cache::FontId);

pub type EventVec = SmallVec<[Event; 4]>;

#[derive(Clone,Copy,Debug)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
    pub visibility: Option<Rect>
}

pub struct EventSystemContext {
    pub capture: Capture,
    pub event: Event,
    pub focused: bool,
    pub cursor: MousePosition,
    pub style: MouseStyle,
    pub mode: MouseMode,
}

impl MousePosition {
    pub fn inside(&self, layout: &Rect) -> bool {
        self.visibility.map_or(false, |v| v.intersect(layout).map_or(false, |i| {
            self.x >= i.left && 
            self.x < i.right && 
            self.y >= i.top && 
            self.y < i.bottom
        }))
    }

    pub fn sub(&self, layout: &Rect) -> MousePosition {
        MousePosition {
            x: self.x,
            y: self.y,
            visibility: self.visibility.and_then(|v| v.intersect(layout))
        }
    }

    pub fn expand(&self, layout: &Rect) -> MousePosition {
        MousePosition {
            x: self.x,
            y: self.y,
            visibility: Some(layout.clone())
        }
    }
}

struct UiWindow {
    id: String,
    tree: Option<dag::Tree>,
    used: usize,
    modal: bool,
    rect: Rect,
}

pub struct Ui {
    iteration: usize,
    focus: Option<dag::Id>,
    windows: Vec<UiWindow>,
    tree_stack: Vec<dag::Tree>,
    free: dag::FreeList,
    containers: HashMap<TypeId, Box<Any>>,
    sys_render: Vec<Box<SystemDispatch<Vec<Primitive>>>>,
    sys_render_post: Vec<Box<SystemDispatch<Vec<Primitive>>>>,
    sys_event: Vec<Box<SystemDispatch<EventSystemContext>>>,
    events: EventVec,
    cache: Cache,
    tabstop_last_id: Option<dag::Id>,
    tabstop_focus_id: Option<dag::Id>,
    pub viewport: Rect,
    pub cursor: (f32, f32),
    capture: Capture,
    previous_capture: Capture,
    mouse_style: MouseStyle,
    mouse_mode: MouseMode,
}

pub struct Context<'a> {
    parent: &'a mut Ui,
    style: &'a Style,
    id: &'a str,
    source: Option<String>,
    widgets: Vec<(dag::Id, Box<'a + WidgetBase>)>,
    window: Viewport,
}

pub struct WidgetResult<'a, T: 'a + Widget> {
    pub result: T::Result,
    pub context: Context<'a>,
}

impl Ui {
    pub fn new() -> Self {

        let (clip_push, clip_pop) = new_clip_system();

        let sys_render: Vec<Box<SystemDispatch<Vec<Primitive>>>> = vec![
            Box::new(BackgroundRenderSystem{}),
            Box::new(clip_push),
        ];

        let sys_render_post: Vec<Box<SystemDispatch<Vec<Primitive>>>> = vec![
            Box::new(TextRenderSystem{}),
            Box::new(clip_pop),
            Box::new(DrawingRenderSystem{}),
        ];

        let sys_event: Vec<Box<SystemDispatch<EventSystemContext>>> = vec![
            Box::new(ClickableEventSystem{}),
        ];

        Self {
            iteration: 1,
            focus: None,
            windows: vec![],
            tree_stack: vec![],
            free: dag::FreeList::new(),
            containers: HashMap::new(),
            sys_render,
            sys_render_post,
            sys_event,
            events: EventVec::new(),
            cache: Cache::new(2048),
            tabstop_last_id: None,
            tabstop_focus_id: None,
            viewport: Rect::from_wh(0.0, 0.0),
            cursor: (0.0, 0.0),
            capture: Capture::None,
            previous_capture: Capture::None,
            mouse_style: MouseStyle::Arrow,
            mouse_mode: MouseMode::Normal,
        }
    }

    pub fn get_patch<'a, L: Loadable<'a>>(&mut self, load: L) -> Patch {
        self.cache.get_patch(load)
    }

    pub fn get_image<'a, L: Loadable<'a>>(&mut self, load: L) -> Image {
        self.cache.get_image(load)
    }

    pub fn get_font<L: Loadable<'static>>(&mut self, load: L) -> Font {
        self.cache.get_font(load)
    }

    pub fn update(&mut self, viewport: Rect, events: EventVec) {
        for event in events.iter() {
            match event {
                &Event::Cursor(x, y) => self.cursor = (x, y),
                _ => (),
            }
        }

        self.viewport = viewport;

        self.events = events;

        self.capture = Capture::None;

        if self.events.len() > 0 {
            self.mouse_style = MouseStyle::Arrow;
        }
    }

    pub fn window<'a>(&'a mut self, style: &'a Style, id: &'a str, modal: bool) -> Context<'a> {
        let mut tree = None;
        
        // find window
        for win in self.windows.iter_mut() {
            if win.id == id {
                assert!(win.tree.is_some());
                win.used = self.iteration;
                tree = win.tree.take();
                break;
            }
        }

        // window not found? make a new one
        if tree.is_none() {
            self.windows.push(UiWindow {
                id: id.to_string(),
                tree: None,
                used: self.iteration,
                modal,
                rect: Rect::zero(),
            });
            tree = Some(dag::Tree::new());
        }

        self.tree_stack.push(tree.unwrap());

        let viewport = Viewport {
                child_rect: self.viewport,
                input_rect: Some(self.viewport),
            };

        Context {
            parent: self,
            style,
            id,
            source: None,
            widgets: vec![],
            window: viewport,
        }
    }

    pub fn render(&mut self) -> (DrawList, MouseStyle, MouseMode) {
        let mut drawlists = vec![];

        let mut windows = replace(&mut self.windows, vec![]);

        for win in windows.iter_mut() {
            let tree = win.tree.as_mut().unwrap();
            tree.cleanup(self.iteration, &mut self.free);
            if win.used >= self.iteration {
                drawlists.push(self.run_systems(tree));
            }
        }

        windows.retain(|win| win.used >= self.iteration);

        self.windows = windows;

        self.iteration += 1;

        let drawlist = self.render_internal(drawlists);

        (drawlist, self.mouse_style, self.mouse_mode)
    }

    pub fn children(&self) -> impl Iterator<Item=&dag::Id> {
        let top = self.tree_stack.len() - 1;
        self.tree_stack[top].ord.iter()
    }

    pub fn create_component<T: 'static + Clone>(&mut self, (id, gen): dag::Id, value: T) {
        let container: &mut _ = self.containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Container::<T>::new(RefCell::new(Vec::<(Option<T>, usize)>::new()))))
            .downcast_mut::<Container<T>>()
            .unwrap();

        container.borrow_mut().resize(self.free.len(), (None, 0));
        container.borrow_mut()[id] = (Some(value), gen);
    }

    pub fn component<T: 'static + Clone>(&self, (id, gen): dag::Id) -> Option<FetchComponent<T>> {
        self.containers
            .get(&TypeId::of::<T>())
            .and_then(|x| x.downcast_ref::<Container<T>>())
            .and_then(|container| {
                if id < container.borrow().len() && container.borrow()[id].1 == gen {
                    Some(FetchComponent::new(container.clone(), id))
                } else {
                    None
                }
            })
    }    
}

impl<'a> Context<'a> {
    // Add a widget to the context. 
    // The specified id should be unique within this context, it is used to find it's state in
    //  future iterations.
    // If the widget has children, you must add them through the context in the returned WidgetResult
    pub fn add<'b, T: 'a + Widget>(&'b mut self, id: &str, mut w: T) -> WidgetResult<'b, T> {
        let (internal_id, create, tree) = {
            let top = self.parent.tree_stack.len()-1;
            let iteration = self.parent.iteration;
            let tree = &mut self.parent.tree_stack[top];
            let free = &mut self.parent.free;
            let internal_id = tree.id(id, free);

            tree.ord.push(internal_id);
            let item = tree.item(id, free);

            (internal_id, 0 == replace(&mut item.used, iteration), item.subs.take().unwrap_or(dag::Tree::new()))
        };

        if create {
            w.create(internal_id, self.parent, self.style);
        }

        //tree.ord.clear();

        self.parent.tree_stack.push(tree);

        let sub_window = w.update(internal_id, self.parent, self.style, self.window.clone());

        let result = w.result(internal_id);

        self.widgets.push((internal_id, Box::new(w)));        

        WidgetResult {
            result,
            context: Context {
                parent: self.parent,
                style: self.style,
                id: self.id,
                source: Some(id.to_string()),
                widgets: vec![],
                window: sub_window,
            }
        }
    }

    pub fn set_style(&mut self, style: &'a Style) {
        self.style = style;
    }

    pub fn with<'b, F: FnOnce(&'b mut Context<'a>)>(&'b mut self, f: F) {
        f(self);
    }
}

// When the context is dropped, events and rendering will be evaluated and the results will be 
//  posted to the parent context.
impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        let mut widgets = replace(&mut self.widgets, vec![]);

        let mut tree = self.parent.tree_stack.pop().unwrap();

        let mut activate_window = false;

        tree.ord.clear();

        for &(id, _) in widgets.iter() {
            tree.ord.push(id);
        }

        for event in self.parent.events.clone() {
            for &mut(id, ref mut widget) in widgets.iter_mut() {
                let focused = self.parent.focus.map(|f| f == id).unwrap_or(false);

                if focused || self.parent.capture == Capture::None {
                    let mut ctx = EventSystemContext {
                        capture: Capture::None,
                        event,
                        cursor: MousePosition { 
                            x: self.parent.cursor.0, 
                            y: self.parent.cursor.1, 
                            visibility: self.window.input_rect 
                        },
                        style: self.parent.mouse_style,
                        mode: self.parent.mouse_mode,
                        focused,
                    };

                    widget.event(id, self.parent, self.style, &mut ctx);

                    for sys in self.parent.sys_event.iter() {
                        sys.run_for(&mut ctx, id, self.parent).ok();
                    }

                    self.parent.mouse_style = ctx.style;
                    self.parent.mouse_mode = ctx.mode;

                    match ctx.capture {
                        Capture::CaptureFocus(mouse_style) => {
                            self.parent.capture = Capture::CaptureFocus(mouse_style);
                            self.parent.focus = Some(id);
                            activate_window = true;
                        },
                        Capture::CaptureMouse(mouse_style) => {
                            self.parent.capture = Capture::CaptureMouse(mouse_style);
                            self.parent.focus = Some(id);
                            activate_window = true;
                        },
                        _ => if focused {
                            match event {
                                Event::Press(Key::Tab, Modifiers{ shift: false, .. }) => {
                                    self.parent.capture = Capture::FocusNext;
                                },
                                Event::Press(Key::Tab, Modifiers{ shift: true, .. }) => {
                                    self.parent.capture = Capture::FocusPrev;
                                    self.parent.tabstop_focus_id = self.parent.tabstop_last_id;
                                },
                                _ => (),
                            }
                        },
                    }
                }
            }
        }

        // if the window is activated, move it to the back of the window vec
        if activate_window {
            let mut i = self.parent.windows.len() - 1;
            loop {
                if self.parent.windows[i].id == self.id {
                    let w = self.parent.windows.remove(i);
                    self.parent.windows.push(w);
                    break;
                }

                assert!(i > 0);
                i -= 1;
            }
        }

        if self.source.is_some() {
            let top = self.parent.tree_stack.len()-1;
            let src = self.source.as_ref().unwrap();
            if tree.ids.len() > 0 {
                self.parent.tree_stack[top].ids.get_mut(src).unwrap().subs = Some(tree);
            }
        } else {
            assert!(self.parent.tree_stack.len() == 0);

            let mut win_rect = None;
            for (id, _) in widgets {
                self.parent.component(id).map(|layout: FetchComponent<Layout>| {
                    let layout = layout.borrow();
                    win_rect = win_rect
                        .and_then(|r| layout.current.map(|layout| layout.union(r)))
                        .or(layout.current);
                });
            }

            let id = self.id;
            let win = self.parent.windows.iter_mut().rev().find(|win| win.id == id).unwrap();
            win.tree = Some(tree);
            win.rect = win_rect.unwrap();
        }
    }
}

impl Ui {
    fn run_systems(&mut self, tree: &mut dag::Tree) -> Vec<Primitive> {
        let mut system_context = vec![];

        for id in tree.ord.iter() {
            for sys in self.sys_render.iter() {
                sys.run_for(&mut system_context, *id, self).ok();
            }

            for val in tree.ids.values_mut() {
                if val.id == *id {
                    val.subs.as_mut().map(|x| {
                        system_context.append(&mut self.run_systems(x));
                    });
                }
            }

            for sys in self.sys_render_post.iter() {
                sys.run_for(&mut system_context, *id, self).ok();
            }
        }

        system_context
    }

    fn render_internal(&mut self, draw_lists: Vec<Vec<Primitive>>) -> DrawList {
        self.previous_capture = self.capture;
       
        // drop focus if nothing was clicked
        if let Capture::None = self.capture {
            for event in self.events.iter() {
                if let &Event::Press(Key::LeftMouseButton, _) = event {
                    self.focus = None;
                }
            }
        }

        let mut vtx = Vec::new();
        let mut cmd = Vec::new();

        let mut scissors = Vec::new();
        scissors.push(self.viewport);

        let mut current_command = Command::Nop;

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

                    self.cache.draw_text(  
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
            _ => (self.mouse_style, MouseMode::Normal)
        };

        self.mouse_style = mouse_style;
        self.mouse_mode = mouse_mode;

        DrawList {
            updates: self.cache.take_updates(),
            vertices: vtx,
            commands: cmd
        }
    }
}