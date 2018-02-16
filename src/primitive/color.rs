#[derive(Clone,Copy,Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn white() -> Color { Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
    pub fn black() -> Color { Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } }
    pub fn red() -> Color { Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 } }
    pub fn green() -> Color { Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 } }
    pub fn blue() -> Color { Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 } }
    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}