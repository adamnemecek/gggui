use super::*;

#[derive(Clone, PartialEq)]
pub enum Constraint {
    Fixed,
    Grow,
    Fill,
}

#[derive(Clone)]
pub enum Gravity {
    Begin,
    Middle,
    End,
}

#[derive(Clone)]
pub struct Layout {
    pub current: Option<Rect>,
    pub margin: Rect,
    pub padding: Rect,
    pub constraints: (Constraint, Constraint),
    pub gravity: (Gravity, Gravity),
}

impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            current: Some(Rect::from_wh(width, height)),
            margin: Rect::from_wh(0.0, 0.0),
            padding: Rect::from_wh(0.0, 0.0),
            constraints: (Constraint::Fixed, Constraint::Fixed),
            gravity: (Gravity::Begin, Gravity::Begin),
        }
    }

    pub fn grow() -> Self {
        Self {
            current: None,
            margin: Rect::from_wh(0.0, 0.0),
            padding: Rect::from_wh(0.0, 0.0),
            constraints: (Constraint::Grow, Constraint::Grow),
            gravity: (Gravity::Begin, Gravity::Begin),
        }
    }

    pub fn fill() -> Self {
        Self {
            current: Some(Rect::zero()),
            margin: Rect::from_wh(0.0, 0.0),
            padding: Rect::from_wh(0.0, 0.0),
            constraints: (Constraint::Fill, Constraint::Fill),
            gravity: (Gravity::Begin, Gravity::Begin),
        }
    }

    pub fn with_fill_h(mut self) -> Self {
        self.constraints.0 = Constraint::Fill;
        self
    }

    pub fn with_fill_v(mut self) -> Self {
        self.constraints.1 = Constraint::Fill;
        self
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = Rect {
            left: margin,
            right: margin,
            top: margin,
            bottom: margin,
        };
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Rect {
            left: padding,
            right: padding,
            top: padding,
            bottom: padding,
        };
        self
    }

    pub fn after_margin(&self) -> Rect {
        self.current.clone().unwrap().after_margin(self.margin)
    }

    pub fn after_padding(&self) -> Rect {
        self.current.clone().unwrap().after_padding(self.padding)
    }
}