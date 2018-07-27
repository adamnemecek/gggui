use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use super::*;

pub struct Button {
    size: Option<(f32, f32)>,
    clicked: bool,
}

impl Button {
    pub fn new() -> Self {
        Self {
            size: None,
            clicked: false,
        }
    }

    pub fn with_size(mut self, size: (f32, f32)) -> Self {
        self.size = Some(size);
        self
    }
}

impl WidgetBase for Button {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        let background = WidgetBackground{
            normal: Background::Patch(style.button_normal.clone(), 1.0),
            hover: Background::Patch(style.button_hover.clone(), 1.0),
            click: Background::Patch(style.button_pressed.clone(), 1.0),
        };

        let size = style.button_normal.minimum_size();

        world.create_component(id, Layout::new()
            .with_margins(style.button_normal.margin())
            .with_constraints(|layout| vec![
                layout.width |GE(STRONG)| size.0 as f64,
                layout.height |GE(STRONG)| size.1 as f64
            ]));
        world.create_component(id, background);
        world.create_component(id, Clickable::Idle);
    }

    fn update(&mut self, id: dag::Id, world: &Ui, _style: &Style, window: Viewport) -> Viewport {
        let mut clickable = world.component::<Clickable>(id).unwrap();
        let mut clickable = clickable.borrow_mut();

        *clickable = match *clickable {
            Clickable::Released(x) => {
                self.clicked = x;
                Clickable::Idle
            },
            x => x,
        };

        Viewport {
            child_rect: Rect::zero(),
            input_rect: None,
        }
    }
}

impl Widget for Button {
    type Result = bool;

    fn result(&mut self, _id: dag::Id) -> Self::Result {
        self.clicked
    }
}