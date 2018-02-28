use super::*;

pub struct Lable {
    pub text: Text,
    pub text_color: Color,
}

impl Lable {
    pub fn text_color(mut self, text_color: Color) -> Self {
        self.text_color = text_color;
        self
    }
}

impl Widget for Lable {
    type Result = ();
    type State = ();

    fn measure(&self, _state: &Self::State, layout: Option<Rect>) -> Option<Rect> {
        Some(self.text.measure(layout))
    }
    
    fn predraw<F: FnMut(Primitive)>(
        &self, 
        _: &Self::State, 
        layout: Rect, 
        mut submit: F
    ) {
        submit(Primitive::DrawText(self.text.clone(), layout, self.text_color));
    }

    fn result(self, _: &Self::State) -> Self::Result {
        ()
    }

}