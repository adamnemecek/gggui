use ecsui::components::background::Background;
use super::*;

pub struct Button {
    clicked: bool,
}

impl Button {
    pub fn new() -> Self {
        Self {
            clicked: false,
        }
    }
}

impl WidgetBase for Button {
    fn create(&mut self, id: dag::Id, world: &mut Ui) {
        let layout = Layout{
            margin: Rect{ left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 },
            padding: Rect{ left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 },
            current: Some(Rect::from_wh(256.0, 64.0)),
            constrain_width: Constraint::FillToParent,
            constrain_height: Constraint::Fixed,
        };
        let background = Background{
            normal: world.get_patch(load_from_static_memory!("../../../img/button_normal.png")),
            hover: world.get_patch(load_from_static_memory!("../../../img/button_hover.png")),
            click: world.get_patch(load_from_static_memory!("../../../img/button_pressed.png")),
        };
        let clickable = Clickable::Idle;

        world.create_component(id, layout);
        world.create_component(id, background);
        world.create_component(id, clickable);
    }

    fn update(&mut self, id: dag::Id, world: &Ui, _window: Rect) -> Rect {
        let mut clickable = world.component::<Clickable>(id);
        let clickable = clickable.as_mut().unwrap();
        let mut clickable = clickable.borrow_mut();

        *clickable = match *clickable {
            Clickable::Released(x) => {
                self.clicked = x;
                Clickable::Idle
            },
            x => x,
        };

        Rect::from_wh(0.0, 0.0)
    }

    fn event(&mut self, _id: dag::Id, _world: &Ui, _ev: Event, _focus: bool) -> Capture {
        // todo
        Capture::None
    }
}

impl Widget for Button {
    type Result = bool;

    fn result(&self, _id: dag::Id) -> Self::Result {
        self.clicked
    }
}