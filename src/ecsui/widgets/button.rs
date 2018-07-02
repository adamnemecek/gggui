use super::*;
use ecsui::components::background::WidgetBackground;

pub struct Button {
    layout: Layout,
    clicked: bool,
}

impl Button {
    pub fn new() -> Self {
        Self {
            layout: Layout{
                margin: Rect{ left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 },
                padding: Rect{ left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 },
                current: Some(Rect::from_wh(256.0, 64.0)),
                constrain_width: Constraint::Fill,
                constrain_height: Constraint::Fixed,
            },
            clicked: false,
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl WidgetBase for Button {
    fn create(&mut self, id: dag::Id, world: &mut Ui) {
        let background = WidgetBackground{
            normal: Background::Patch(
                world.get_patch(load_from_static_memory!("../../../img/button_normal.png")), 1.0),
            hover: Background::Patch(
                world.get_patch(load_from_static_memory!("../../../img/button_hover.png")), 1.0),
            click: Background::Patch(
                world.get_patch(load_from_static_memory!("../../../img/button_pressed.png")), 1.0),
        };

        world.create_component(id, self.layout.clone());
        world.create_component(id, background);
        world.create_component(id, Clickable::Idle);
    }

    fn update(&mut self, id: dag::Id, world: &Ui, _window: Viewport) -> Viewport {
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

        Viewport {
            child_rect: Rect::from_wh(0.0, 0.0),
            input_rect: Rect::from_wh(0.0, 0.0),
        }
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