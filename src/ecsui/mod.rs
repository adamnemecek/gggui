use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::mem::replace;
use smallvec::SmallVec;
use std::any::Any;
use std::any::TypeId;
use cache::Cache;
use render::DrawList;
use events::*;

pub mod components;
pub mod widgets;

pub type EventVec = SmallVec<[Event; 4]>;

pub mod dag;
//pub mod entry;

pub trait WidgetBase {
    fn tabstop(&self) -> bool { false }
    fn enabled<'a>(&self, dag::Id, &Context<'a>) -> bool { true }
    fn autofocus(&self, dag::Id) -> bool { false }

    fn create<'a>(&mut self, dag::Id, &mut Context<'a>);
    fn update<'a>(&mut self, dag::Id, &mut Context<'a>);
    fn event<'a>(&mut self, dag::Id, &mut Context<'a>, Event, bool) -> Capture { Capture::None }
}

pub trait Widget: WidgetBase {
    type Result;

    fn result(&self, dag::Id) -> Self::Result;
}

pub struct Ui {
    iteration: usize,
    focus: Option<dag::Id>,
    tree: Option<dag::Tree>,
    free: dag::FreeList,
    containers: HashMap<TypeId, Box<Any>>,
    events: EventVec,
    cache: Cache,
    tabstop_last_id: Option<dag::Id>,
    tabstop_focus_id: Option<dag::Id>,
}

enum Parent<'a> {
    Ctx(&'a mut Context<'a>, &'a str),
    Ui(&'a mut Ui),
}

pub struct Context<'a> {
    parent: Parent<'a>,
    widgets: Vec<(dag::Id, Box<WidgetBase>)>,
    capture: Capture,
    tree: dag::Tree,
}

pub struct WidgetResult<'a, T: Widget> {
    pub result: T::Result,
    pub context: Context<'a>,
}

impl Ui {
    pub fn begin<'a>(&'a mut self) -> Context<'a> {
        let tree = self.tree.take().unwrap();

        Context {
            parent: Parent::Ui(self),
            widgets: vec![],
            capture: Capture::None,
            tree
        }
    }

    pub fn end(&mut self) -> (DrawList, MouseStyle, MouseMode) {
        self.iteration += 1;

        unimplemented!();
    }

    pub fn create_component<T: 'static + Clone>(&mut self, (id, gen): dag::Id, value: T) {
        let container: &mut _ = self.containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<(Option<T>, usize)>::new()))
            .downcast_mut::<Vec<(Option<T>, usize)>>()
            .unwrap();

        container.resize(self.free.len(), (None, 0));
        container[id] = (Some(value), gen);
    }

    pub fn component<T: 'static + Clone>(&mut self, (id, gen): dag::Id) -> Option<&mut T> {
        self.containers
            .get_mut(&TypeId::of::<T>())
            .and_then(|x| x.downcast_mut::<Vec<(Option<T>, usize)>>())
            .and_then(|container| {
                if id < container.len() && container[id].1 == gen {
                    container[id].0.as_mut()
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
    pub fn add<'b: 'a, T: 'static + Widget>(&'b mut self, id: &'b str, mut w: T) -> WidgetResult<'b, T> {
        let (internal_id, create, tree) = {
            let iteration = self.parent.iteration;
            let tree = &mut self.tree;
            let free = &mut self.parent.free;
            let internal_id = tree.id(id, free);

            let item = tree.item(id, free);

            (internal_id, 0 == replace(&mut item.used, iteration), item.subs.take().unwrap_or(dag::Tree::new()))
        };

        if create {
            w.create(internal_id, self);
        }

        w.update(internal_id, self);

        let result = w.result(internal_id);
        let capture = self.capture;

        self.widgets.push((internal_id, Box::new(w)));

        WidgetResult {
            result,
            context: Context {
                parent: Parent::Ctx(self, id),
                widgets: vec![],
                capture,
                tree,
            }
        }
    }
}

// When the context is dropped, events and rendering will be evaluated and the results will be 
//  posted to the parent context.
impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        let mut widgets = replace(&mut self.widgets, vec![]);

        for event in self.parent.events.clone() {
            for &mut(id, ref mut widget) in widgets.iter_mut() {
                let focused = self.parent.focus.map(|f| f == id).unwrap_or(false);

                if focused || self.capture == Capture::None {
                    let capture = widget.event(id, self, event, focused);

                    match capture {
                        Capture::CaptureFocus(mouse_style) => {
                            self.capture = Capture::CaptureFocus(mouse_style);
                            self.parent.focus = Some(id);
                        },
                        Capture::CaptureMouse(mouse_style) => {
                            self.capture = Capture::CaptureMouse(mouse_style);
                            self.parent.focus = Some(id);
                        },
                        _ => if focused {
                            match event {
                                Event::Press(Key::Tab, Modifiers{ shift: false, .. }) => {
                                    self.capture = Capture::FocusNext;
                                },
                                Event::Press(Key::Tab, Modifiers{ shift: true, .. }) => {
                                    self.capture = Capture::FocusPrev;
                                    self.parent.tabstop_focus_id = self.parent.tabstop_last_id;
                                },
                                _ => (),
                            }
                        },
                    }
                }
            }
        }

        let tree = replace(&mut self.tree, dag::Tree::new());
        match &mut self.parent {
            &mut Parent::Ctx(ref mut x, id) => {
                if tree.ids.len() > 0 {
                    x.tree.ids.get_mut(id).unwrap().subs = Some(tree);
                }
                x.capture = self.capture;
            },
            &mut Parent::Ui(ref mut x) => {
                x.tree = Some(tree);
            },
        }
    }
}

// Parent can be dereferenced to Ui for global state access
impl<'a> Deref for Parent<'a> {
    type Target = Ui;
    fn deref(&self) -> &Ui {
        match self {
            &Parent::Ctx(ref x, _) => x.parent.deref(),
            &Parent::Ui(ref x) => x,
        }
    }
}

// Parent can be mutably dereferenced to Ui for global state access
impl<'a> DerefMut for Parent<'a> {
    fn deref_mut(&mut self) -> &mut Ui {
        match self {
            &mut Parent::Ctx(ref mut x, _) => x.parent.deref_mut(),
            &mut Parent::Ui(ref mut x) => x,
        }
    }
}
