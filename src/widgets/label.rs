use super::*;

pub struct Label<'a> {
    text: &'a str,
    size: f32,
    wrap: TextWrap,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str, size: f32, wrap: TextWrap) -> Self {
        Self {
            text, size, wrap
        }
    }
}

impl<'a> WidgetBase for Label<'a> {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        let text = Text {
            text: self.text.to_string(),
            size: self.size,
            wrap: self.wrap,
            font: style.font.clone(),
        };

        let layout = Layout {
            current: Some(text.measure(None)),
            margin: Rect { left: 5.0, right: 5.0, top: 5.0, bottom: 5.0 },
            padding: Rect::from_wh(0.0, 0.0),
            constrain_width: Constraint::Fixed,
            constrain_height: Constraint::Fixed,
        };

        world.create_component(id, layout);
        world.create_component(id, text);
    }

    fn update(&mut self, _id: dag::Id, _world: &Ui, _style: &Style, _window: Viewport) -> Viewport {
        Viewport {
            child_rect: Rect::from_wh(0.0, 0.0),
            input_rect: None,
        }
    }
}

impl<'a> Widget for Label<'a> {
    type Result = ();

    fn result(&mut self, _id: dag::Id) -> Self::Result { }
}