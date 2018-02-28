use super::*;

pub struct Lable {
    pub text: Text,
    pub text_color: Color,
    pub border_color: Option<Color>,
    pub size: Option<Rect>,
}

impl Lable {
    pub fn text_color(mut self, text_color: Color) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn bordered(mut self, border_color: Color) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn size(mut self, size: Rect) -> Self {
        self.size = Some(size);
        self
    }
}

impl Widget for Lable {
    type Result = ();
    type State = ();

    fn measure(&self, _state: &Self::State, layout: Option<Rect>) -> Option<Rect> {
        Some(self.text.measure(self.size.or(layout)))
    }
    
    fn predraw<F: FnMut(Primitive)>(
        &self, 
        _: &Self::State, 
        layout: Rect, 
        mut submit: F
    ) {
        if let &Some(color) = &self.border_color {
            submit(Primitive::DrawText(self.text.clone(), layout.translate(-1.0,  0.0), color));
            submit(Primitive::DrawText(self.text.clone(), layout.translate( 1.0,  0.0), color));
            submit(Primitive::DrawText(self.text.clone(), layout.translate( 0.0, -1.0), color));
            submit(Primitive::DrawText(self.text.clone(), layout.translate( 0.0,  1.0), color));
        }
        submit(Primitive::DrawText(self.text.clone(), layout, self.text_color));
    }

    fn result(self, _: &Self::State) -> Self::Result {
        ()
    }

}