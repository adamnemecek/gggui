use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::cell::RefMut;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::mem::replace;
use smallvec::SmallVec;
use std::any::Any;
use std::any::TypeId;

pub mod systems;
pub mod components;
pub mod widgets;

pub mod dag;
pub mod entry;

use primitive::*;
use render::*;
use cache::*;
use loadable::*;
use events::*;

pub use self::widgets::*;
pub use self::components::*;
use self::systems::*;

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

pub struct Ui {
    iteration: usize,
    focus: Option<dag::Id>,
    tree: Option<dag::Tree>,
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
    viewport: Rect,
    cursor: (f32, f32),
    capture: Capture,
    previous_capture: Capture,
    mouse_style: MouseStyle,
    mouse_mode: MouseMode,
}

pub struct Context<'a> {
    parent: &'a mut Ui,
    source: Option<String>,
    widgets: Vec<(dag::Id, Box<WidgetBase>)>,
    window: Viewport,
}

pub struct WidgetResult<'a, T: 'static + Widget> {
    pub result: T::Result,
    pub context: Context<'a>,
}

impl Ui {
    pub fn new() -> Self {

        let sys_render: Vec<Box<SystemDispatch<Vec<Primitive>>>> = vec![
            Box::new(BackgroundRenderSystem{}),
            Box::new(ContentPushClipSystem{}),
        ];

        let sys_render_post: Vec<Box<SystemDispatch<Vec<Primitive>>>> = vec![
            Box::new(ContentPopClipSystem{}),
        ];

        let sys_event: Vec<Box<SystemDispatch<EventSystemContext>>> = vec![
            Box::new(ClickableEventSystem{}),
        ];

        Self {
            iteration: 1,
            focus: None,
            tree: Some(dag::Tree::new()),
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

    pub fn get_font<L: Loadable<'static>>(&mut self, load: L) -> (Font, FontId) {
        self.cache.get_font(load)
    }

    pub fn begin<'a>(&'a mut self, viewport: Rect, mut events: EventVec) -> Context<'a> {
        let tree = self.tree.take().unwrap();

        for event in events.iter() {
            match event {
                &Event::Cursor(x, y) => self.cursor = (x, y),
                _ => (),
            }
        }

        if events.len() == 0 {
            events.push(Event::Idle);
        }

        self.viewport = viewport;

        self.events = events;

        self.tree_stack.push(tree);

        self.capture = Capture::None;

        Context {
            parent: self,
            source: None,
            widgets: vec![],
            window: Viewport {
                child_rect: viewport,
                input_rect: Some(viewport),
            }
        }
    }

    pub fn end(&mut self) -> (DrawList, MouseStyle, MouseMode) {
        let mut tree = self.tree.take().unwrap();

        tree.cleanup(self.iteration, &mut self.free);
        self.iteration += 1;

        let primitives = self.run_systems(&mut tree);

        self.tree = Some(tree);

        let drawlist = self.render(primitives);

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
}

impl<'a> Context<'a> {
    // Add a widget to the context. 
    // The specified id should be unique within this context, it is used to find it's state in
    //  future iterations.
    // If the widget has children, you must add them through the context in the returned WidgetResult
    pub fn add<'b, T: 'static + Widget>(&'b mut self, id: &str, mut w: T) -> WidgetResult<'b, T> {
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
            w.create(internal_id, self.parent);
        }

        //tree.ord.clear();

        self.parent.tree_stack.push(tree);

        let sub_window = w.update(internal_id, self.parent, self.window.clone());

        let result = w.result(internal_id);

        self.widgets.push((internal_id, Box::new(w)));        

        WidgetResult {
            result,
            context: Context {
                parent: self.parent,
                source: Some(id.to_string()),
                widgets: vec![],
                window: sub_window,
            }
        }
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
                        focused,
                    };

                    widget.event(id, self.parent, &mut ctx);

                    for sys in self.parent.sys_event.iter() {
                        sys.run_for(&mut ctx, id, self.parent).ok();
                    }

                    match ctx.capture {
                        Capture::CaptureFocus(mouse_style) => {
                            self.parent.capture = Capture::CaptureFocus(mouse_style);
                            self.parent.focus = Some(id);
                        },
                        Capture::CaptureMouse(mouse_style) => {
                            self.parent.capture = Capture::CaptureMouse(mouse_style);
                            self.parent.focus = Some(id);
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

        if self.source.is_some() {
            let top = self.parent.tree_stack.len()-1;
            let src = self.source.as_ref().unwrap();
            if tree.ids.len() > 0 {
                self.parent.tree_stack[top].ids.get_mut(src).unwrap().subs = Some(tree);
            }
        } else {
            assert!(self.parent.tree_stack.len() == 0);
            self.parent.tree = Some(tree);
        }
    }
}

impl Ui {
    fn render(&mut self, drawlist: Vec<Primitive>) -> DrawList {
        self.previous_capture = self.capture;
        //self.windows.retain(|w| w.updated);
        //if let Some(&Window{ updated: false, .. }) = self.ui.active_window.as_ref() {
        //    self.ui.active_window = None;
        //}

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

        let draw_lists = vec![drawlist];
        //for window in self.ui.windows.iter_mut() {
        //    draw_lists.push(replace(&mut window.drawlist, vec![]));
        //}
        //for window in self.ui.active_window.iter_mut() {
        //    draw_lists.push(replace(&mut window.drawlist, vec![]));
        //}

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