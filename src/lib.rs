extern crate downcast;
extern crate smallvec;
extern crate rusttype;
extern crate image;
extern crate clipboard;
extern crate cassowary;

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

#[derive(PartialEq)]
pub enum Layer {
    Back,
    Normal,
    Modal
}

struct UiLayer {
    id: String,
    tree: Option<dag::Tree>,
    used: usize,
    layer: Layer,
    rect: Rect,
}

pub struct Ui {
    iteration: usize,
    focus: Option<dag::Id>,
    layers: Vec<UiLayer>,
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
    active_layer: String,
    capture: Capture,
    previous_capture: Capture,
    mouse_style: MouseStyle,
    mouse_mode: MouseMode,
    layout_solver: cassowary::Solver,
    layout_lookup: HashMap<cassowary::Variable, dag::Id>,

    viewport_left: cassowary::Variable,
    viewport_top: cassowary::Variable,
    viewport_right: cassowary::Variable,
    viewport_bottom: cassowary::Variable,
    viewport_margin_left: cassowary::Variable,
    viewport_margin_top: cassowary::Variable,
    viewport_margin_right: cassowary::Variable,
    viewport_margin_bottom: cassowary::Variable,
    viewport_width: cassowary::Variable,
    viewport_height: cassowary::Variable,
    viewport_center_x: cassowary::Variable,
    viewport_center_y: cassowary::Variable,
}

pub struct Context<'a> {
    parent: &'a mut Ui,
    style: &'a Style,
    id: &'a str,
    source: Option<String>,
    widgets: Vec<(dag::Id, Box<'a + WidgetBase>)>,
    window: Option<Rect>,
    is_new: bool,
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

        let mut layout_solver = cassowary::Solver::new();

        let viewport_left = cassowary::Variable::new();
        let viewport_top = cassowary::Variable::new();
        let viewport_right = cassowary::Variable::new();
        let viewport_bottom = cassowary::Variable::new();
        let viewport_margin_left = cassowary::Variable::new();
        let viewport_margin_top = cassowary::Variable::new();
        let viewport_margin_right = cassowary::Variable::new();
        let viewport_margin_bottom = cassowary::Variable::new();
        let viewport_width = cassowary::Variable::new();
        let viewport_height = cassowary::Variable::new();
        let viewport_center_x = cassowary::Variable::new();
        let viewport_center_y = cassowary::Variable::new();

        let constraints = vec![
            viewport_left + viewport_width |EQ(REQUIRED)| viewport_right,
            viewport_top + viewport_height |EQ(REQUIRED)| viewport_bottom,
            viewport_center_x |EQ(REQUIRED)| (viewport_left+viewport_right)*0.5,
            viewport_center_y |EQ(REQUIRED)| (viewport_top+viewport_bottom)*0.5,
            viewport_margin_left |EQ(STRONG)| viewport_left,
            viewport_margin_right |EQ(STRONG)| viewport_right,
            viewport_margin_top |EQ(STRONG)| viewport_top,
            viewport_margin_bottom |EQ(STRONG)| viewport_bottom,
        ];

        layout_solver.add_constraints(constraints.iter()).ok();
        layout_solver.add_edit_variable(viewport_left, STRONG).ok();
        layout_solver.add_edit_variable(viewport_top, STRONG).ok();
        layout_solver.add_edit_variable(viewport_right, STRONG).ok();
        layout_solver.add_edit_variable(viewport_bottom, STRONG).ok();

        Self {
            iteration: 1,
            focus: None,
            layers: vec![],
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
            active_layer: String::from(""),
            capture: Capture::None,
            previous_capture: Capture::None,
            mouse_style: MouseStyle::Arrow,
            mouse_mode: MouseMode::Normal,
            layout_solver,
            layout_lookup: HashMap::new(),
            viewport_left,
            viewport_top,
            viewport_right,
            viewport_bottom,
            viewport_margin_left,
            viewport_margin_top,
            viewport_margin_right,
            viewport_margin_bottom,
            viewport_width,
            viewport_height,
            viewport_center_x,
            viewport_center_y,
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
        // first, fix the layout
        {
            let free = &mut self.free;
            let layout_lookup = &mut self.layout_lookup;
            let layout_solver = &mut self.layout_solver;

            layout_solver.suggest_value(self.viewport_left, viewport.left as f64).ok();
            layout_solver.suggest_value(self.viewport_right, viewport.right as f64).ok();
            layout_solver.suggest_value(self.viewport_top, viewport.top as f64).ok();
            layout_solver.suggest_value(self.viewport_bottom, viewport.bottom as f64).ok();

            self.containers
            .get(&TypeId::of::<Layout>())
            .and_then(|x| x.downcast_ref::<Container<Layout>>())
            .map(|container| {
                let mut container = container.borrow_mut();

                // remove constraints of removed widgets
                for (id, _) in free.fetch_recently_freed_ids() {
                    container[id].0.take().map(|old_layout| {
                        layout_lookup.remove(&old_layout.left);
                        layout_lookup.remove(&old_layout.top);
                        layout_lookup.remove(&old_layout.right);
                        layout_lookup.remove(&old_layout.bottom);
                        for c in old_layout.constraints() {
                            layout_solver.remove_constraint(c).expect("Layout crash");
                        }
                    });
                }

                for c in free.fetch_recently_freed_constraints() {
                    layout_solver.remove_constraint(&c).expect("Layout crash");
                }

                // apply any changes to the layout
                for (var, val) in layout_solver.fetch_changes() {
                    layout_lookup.get(&var).map(|(id, _)| {
                        let layout = container[*id].0.as_mut().unwrap();
                        if layout.current.is_none() {
                            layout.current = Some(Rect::zero());
                        }
                        if *var == layout.left {
                            layout.current.as_mut().unwrap().left = *val as f32;
                        }
                        else if *var == layout.right {
                            layout.current.as_mut().unwrap().right = *val as f32;
                        }
                        else if *var == layout.top {
                            layout.current.as_mut().unwrap().top = *val as f32;
                        }
                        else if *var == layout.bottom {
                            layout.current.as_mut().unwrap().bottom = *val as f32;
                        }
                    });
                }
            });
        }
        
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
        } else {
            self.events.push(Event::Idle);
        }

        let mut no_modal_found = true;
        self.active_layer = self.layers.iter().rev().find(|x| {
            if x.layer == Layer::Modal {
                no_modal_found =  false;
                true
            } else {
                self.cursor.0 > x.rect.left &&
                self.cursor.0 < x.rect.right &&
                self.cursor.1 > x.rect.top &&
                self.cursor.1 < x.rect.bottom &&
                no_modal_found
            }
        }).map(|x| x.id.clone()).unwrap_or("".to_string());
    }

    pub fn layer<'a>(&'a mut self, style: &'a Style, id: &'a str, layer: Layer) -> Context<'a> {
        let mut tree = None;
        
        // find window
        for ly in self.layers.iter_mut() {
            if ly.id == id {
                assert!(ly.tree.is_some());
                ly.used = self.iteration;
                tree = ly.tree.take();
                break;
            }
        }

        // window not found? make a new one
        let is_new = if tree.is_none() {
            self.layers.push(UiLayer {
                id: id.to_string(),
                tree: None,
                used: self.iteration,
                layer,
                rect: Rect::zero(),
            });

            tree = Some({
                let mut tree = dag::Tree::new();
                tree.vars.insert(String::from("super.left"), self.viewport_left);
                tree.vars.insert(String::from("super.top"), self.viewport_top);
                tree.vars.insert(String::from("super.right"), self.viewport_right);
                tree.vars.insert(String::from("super.bottom"), self.viewport_bottom);
                tree.vars.insert(String::from("super.margin_left"), self.viewport_margin_left);
                tree.vars.insert(String::from("super.margin_top"), self.viewport_margin_top);
                tree.vars.insert(String::from("super.margin_right"), self.viewport_margin_right);
                tree.vars.insert(String::from("super.margin_bottom"), self.viewport_margin_bottom);
                tree.vars.insert(String::from("super.width"), self.viewport_width);
                tree.vars.insert(String::from("super.height"), self.viewport_height);
                tree.vars.insert(String::from("super.center_x"), self.viewport_center_x);
                tree.vars.insert(String::from("super.center_y"), self.viewport_center_y);
                tree
            });            

            true
        } else {
            false
        };

        self.tree_stack.push(tree.unwrap());

        let window = if self.active_layer == id {
            Some(self.viewport)
        } else {
            None
        };

        Context {
            parent: self,
            style,
            id,
            source: None,
            widgets: vec![],
            window,
            is_new,
        }
    }

    pub fn render(&mut self) -> (DrawList, MouseStyle, MouseMode) {
        // Sort layers by type, indepently of last clicked sorting
        self.layers.sort_by(|a,b| {
            let a = match &a.layer {
                Layer::Back => 0,
                Layer::Normal => 1,
                Layer::Modal => 2,
            };
            let b = match &b.layer {
                Layer::Back => 0,
                Layer::Normal => 1,
                Layer::Modal => 2,
            };
            a.cmp(&b)
        });

        // Dispatch render systems
        let mut layers = replace(&mut self.layers, vec![]);

        let mut drawlists = vec![];

        for ly in layers.iter_mut() {
            let tree = ly.tree.as_mut().unwrap();
            tree.cleanup(self.iteration, &mut self.free);
            if ly.used >= self.iteration {
                drawlists.push(self.run_systems(tree));
            } 
        }

        // Remove unused layers
        layers.retain(|ly| ly.used >= self.iteration);

        self.layers = layers;

        // Increase the iteration count. This is used to detect removal of layers and widgets
        //  in the next frame.
        self.iteration += 1;

        // Convert high level drawlist to a lower level format.
        let drawlist = self.render_internal(drawlists);

        (drawlist, self.mouse_style, self.mouse_mode)
    }

    pub fn children(&self) -> impl Iterator<Item=&dag::Id> {
        let top = self.tree_stack.len() - 1;
        self.tree_stack[top].ord.iter()
    }

    pub fn var(&mut self, id: &str) -> cassowary::Variable {
        let top = self.tree_stack.len() - 1;
        let vars = &mut self.tree_stack[top].vars;

        vars.get(id).map(|x| *x).unwrap_or_else(|| {
            println!("unable to retrieve var {:?}", id);
            for (k,v) in vars.iter() {
                println!("available is {:?}/{:?}", k, v);
            }
            panic!();
        })
    }

    pub fn create_component<T: 'static + Clone>(&mut self, (id, gen): dag::Id, value: T) {
        let container = self.containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Container::<T>::new(RefCell::new(Vec::<(Option<T>, usize)>::new()))));

        {
            let container: &mut _ = container
                .downcast_mut::<Container<T>>()
                .unwrap();

            container.borrow_mut().resize(self.free.len(), (None, 0));
            container.borrow_mut()[id] = (Some(value), gen);
        }

        let lookup = &mut self.layout_lookup;
        let solver = &mut self.layout_solver;

        container.downcast_ref::<Container<Layout>>().map(|layouts| {
            let layouts = layouts.borrow();

            let new_layout = layouts[id].0.as_ref().unwrap();
            lookup.insert(new_layout.left, (id, gen));
            lookup.insert(new_layout.top, (id, gen));
            lookup.insert(new_layout.right, (id, gen));
            lookup.insert(new_layout.bottom, (id, gen));
            solver.add_constraints(new_layout.constraints()).expect("Invalid constraints");
        });     
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
        let (internal_id, create, mut tree) = {
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
            self.parent.component(internal_id).map(|layout: FetchComponent<Layout>| {
                let layout = layout.borrow();
                let top = self.parent.tree_stack.len() - 1;
                let parent_tree = &mut self.parent.tree_stack[top];

                tree.vars.insert(String::from("super.left"), layout.left);
                tree.vars.insert(String::from("super.top"), layout.top);
                tree.vars.insert(String::from("super.right"), layout.right);
                tree.vars.insert(String::from("super.bottom"), layout.bottom);
                tree.vars.insert(String::from("super.margin_left"), layout.margin_left);
                tree.vars.insert(String::from("super.margin_top"), layout.margin_top);
                tree.vars.insert(String::from("super.margin_right"), layout.margin_right);
                tree.vars.insert(String::from("super.margin_bottom"), layout.margin_bottom);
                tree.vars.insert(String::from("super.width"), layout.width);
                tree.vars.insert(String::from("super.height"), layout.height);
                tree.vars.insert(String::from("super.center_x"), layout.center_x);
                tree.vars.insert(String::from("super.center_y"), layout.center_y);

                parent_tree.vars.insert(format!("{0}.left", id), layout.left);
                parent_tree.vars.insert(format!("{0}.top", id), layout.top);
                parent_tree.vars.insert(format!("{0}.right", id), layout.right);
                parent_tree.vars.insert(format!("{0}.bottom", id), layout.bottom);
                parent_tree.vars.insert(format!("{0}.margin_left", id), layout.margin_left);
                parent_tree.vars.insert(format!("{0}.margin_top", id), layout.margin_top);
                parent_tree.vars.insert(format!("{0}.margin_right", id), layout.margin_right);
                parent_tree.vars.insert(format!("{0}.margin_bottom", id), layout.margin_bottom);
                parent_tree.vars.insert(format!("{0}.width", id), layout.width);
                parent_tree.vars.insert(format!("{0}.height", id), layout.height);
                parent_tree.vars.insert(format!("{0}.center_x", id), layout.center_x);
                parent_tree.vars.insert(format!("{0}.center_y", id), layout.center_y);
            });
        }

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
                is_new: create,
            }
        }
    }

    pub fn rules<F: FnOnce(&mut FnMut(&str)->cassowary::Variable)->Vec<cassowary::Constraint>>(&mut self, rules: F) {
        if self.is_new {
            let top = self.parent.tree_stack.len() - 1;
            self.parent.tree_stack[top].rules = rules(&mut |id| self.parent.var(id));
            self.parent.layout_solver
                .add_constraints(self.parent.tree_stack[top].rules.iter())
                .expect("Invalid constraints");
        }
    }

    pub fn set_style(&mut self, style: &'a Style) {
        self.style = style;
    }

    pub fn with<'b, F: FnOnce(&'b mut Context<'a>)>(&'b mut self, f: F) {
        f(self);
    }
}

impl<'a, T: 'static + Widget> WidgetResult<'a, T> {
    pub fn with<'b, F: FnOnce(&'b mut Context<'a>)>(&'b mut self, f: F) -> &'b T::Result {
        f(&mut self.context);
        &self.result
    }

    pub fn wrap<W: 'a + Widget>(mut self, widget: W) -> T::Result {
        self.context.add("x", widget);
        self.context.rules(|var| vec![
            var("x.left") |EQ(REQUIRED)| var("super.margin_left"),
            var("x.right") |EQ(REQUIRED)| var("super.margin_right"),
            var("x.top") |EQ(REQUIRED)| var("super.margin_top"),
            var("x.bottom") |EQ(REQUIRED)| var("super.margin_bottom"),
        ]);
        self.result
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
                            visibility: self.window 
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
            let mut i = self.parent.layers.len() - 1;
            loop {
                if self.parent.layers[i].id == self.id {
                    if self.parent.layers[i].layer != Layer::Back {
                        let w = self.parent.layers.remove(i);
                        self.parent.layers.push(w);
                    }
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

            let mut layer_rect = None;
            for (id, _) in widgets {
                self.parent.component(id).map(|layout: FetchComponent<Layout>| {
                    let layout = layout.borrow();
                    layer_rect = layer_rect
                        .and_then(|r| layout.current().map(|layout| layout.union(r)))
                        .or(layout.current().map(|&r| r));
                });
            }

            let id = self.id;
            let ly = self.parent.layers.iter_mut().rev().find(|ly| ly.id == id).unwrap();
            ly.tree = Some(tree);
            ly.rect = layer_rect.unwrap_or(Rect::zero());
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

                Primitive::DrawText(text, rect) => if draw_enabled {
                    let color = [text.color.r, text.color.g, text.color.b, text.color.a];
                    let border = text.border.map(|c| [c.r, c.g, c.b, c.a]).unwrap_or(color);
                    let mode = 0;
                    let offset = vtx.len();
                    let vp = self.viewport;

                    let offsets = [(-1.0,0.0,true),(1.0,0.0,true),(0.0,-1.0,true),(0.0,1.0,true),(0.0,0.0,false)];
                    let offsets = if text.border.is_some() {
                        &offsets[4..5]
                    } else {
                        &offsets[0..1]
                    };

                    self.cache.draw_text(  
                        &text, 
                        rect,
                        |uv, pos| {
                            for (dx, dy, b) in offsets {
                                let rc = Rect{
                                    left: pos.left + dx,
                                    top: pos.top + dy,
                                    right: pos.right + dx,
                                    bottom: pos.bottom + dy,
                                }.to_device_coordinates(vp);

                                let color = if *b { border } else { color };

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
            updates: RefCell::new(self.cache.take_updates()),
            vertices: vtx,
            commands: cmd
        }
    }
}