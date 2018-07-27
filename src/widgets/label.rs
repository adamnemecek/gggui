use super::*;

pub struct Label<'a> {
    text: &'a str,
    size: f32,
    wrap: TextWrap,
    gravity: (Gravity, Gravity),
    color: Color,
    border: Option<Color>,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str, size: f32, wrap: TextWrap) -> Self {
        Self {
            text, 
            size, 
            wrap, 
            gravity: (Gravity::Begin, Gravity::Begin),
            color: Color::black(),
            border: None,
        }
    }

    pub fn simple(text: &'a str) -> Self {
        Self {
            text, 
            size: 16.0, 
            wrap: TextWrap::NoWrap, 
            gravity: (Gravity::Middle, Gravity::Begin),
            color: Color::black(),
            border: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_border(mut self, color: Color) -> Self {
        self.border = Some(color);
        self
    }
}

impl<'a> WidgetBase for Label<'a> {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        let text = Text {
            text: self.text.to_string(),
            size: self.size,
            wrap: self.wrap,
            font: style.font.clone(),
            color: self.color,
            border: self.border,
            padding: Rect::zero(),
        };

        let measured = text.measure(None);

        world.create_component(id, Layout::new().with_intrinsic_size_constraints(measured.width(), measured.height(), 251.0));
        world.create_component(id, text);
    }

    fn update(&mut self, id: dag::Id, world: &Ui, _style: &Style, _window: Viewport) -> Viewport {
        let mut text = world.component::<Text>(id).unwrap();
        let mut text = text.borrow_mut();

        if text.text != self.text {
            text.text = self.text.to_string();
            let mut layout = world.component::<Layout>(id).unwrap();
            layout.borrow_mut().current = Some(text.measure(None));
        }

        Viewport {
            child_rect: Rect::zero(),
            input_rect: None,
        }
    }
}

impl<'a> Widget for Label<'a> {
    type Result = ();

    fn result(&mut self, _id: dag::Id) -> Self::Result { }
}