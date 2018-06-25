pub mod components;
pub mod widgets;

pub type EventVec = SmallVec<[Event; 4]>;

mod graph {
    pub type Id = usize;

    struct Item {
        id: Id,
        used: usize,
        subs: Option<Tree>,
    }

    pub struct Tree {
        ids: HashMap<String, Item>
    }

    pub struct FreeList {
        free: Vec<Id>,
        next: Id,
    }

    impl FreeList {
        pub fn push(&mut self, x: Id) {
            self.free.push(x);
        }

        pub fn pop(&mut self) -> Id {
            self.free.pop().unwrap_or_else(|| {
                self.next += 1;
                self.next
            });
        }
    }

    impl Tree {
        pub fn cleanup(&mut self, before: usize, free_list: &mut FreeList) {
            self.ids.retain(|_, &mut v| {
                v.subs.map(|sub| sub.cleanup(before, free_list));
                if v.used >= before {
                    true
                } else { 
                    free_list.push(v.id);
                    false
                }
            });
        }

        pub fn get(&mut self, name: String) -> Id {
            self.ids.entry(name).or_insert_with(|| Item {
                id: self.free_list.pop(),
                used: 0,
                subs: None,
            }).id
        }
    }
}

pub trait Widget {
    type Result;

    fn tabstop() -> bool { false }
    fn enabled<'a>(&self, graph::Id, &Context<'a>) -> bool { true }
    fn autofocus(&self, graph::Id) -> bool { false }
    fn update<'a>(&mut self, graph::Id, &mut Context<'a>);
    fn event<'a>(&mut self, graph::Id, &mut Context<'a>, Event, bool) -> Capture { Capture::None }

    fn draw_before<'a>(&self, graph::Id, &Context<'a>, &mut FnMut(Primitive)) { }
    fn draw_after<'a>(&self, graph::Id, &Context<'a>, &mut FnMut(Primitive)) { }

    fn result(&self, graph::Id) -> Self::Result;
}

pub struct Container<T> {
    content: Vec<Option<T>>,
}

pub struct Ui {
    focus: Option<graph::Id>,
    free_list: Vec<graph::Id>,
    containers: Vec<Box<Any>>,
    events: EventVec,
    cache: Cache,
}

pub struct Context<'a> {
    core: &'a mut Ui,
    parent: Option<&'a Context>,
    drawlist: Vec<Primitive>,
    drawlist_sub: Vec<Primitive>,
}

pub struct WidgetResult<'a, T: Widget> {
    pub result: T::Result,
    pub context: Context<'a>,
}

impl Ui {
    pub fn begin<'a>(&'a mut self) -> Context<'a> {
        Context {
            core: self,
            parent: None,
        }
    }
}

impl<'a> Context<'a> {
    pub fn add<'b, T: Widget>(&'b mut self, w: T) -> WidgetResult<'b, T> {
        //
    }
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
    pub cursor: MousePosition,
    visibility: Rect,
    pub viewport: Rect,
    enabled: bool,
}
