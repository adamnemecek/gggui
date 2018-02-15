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

    fn default() -> Self::State { () }

    fn measure(&self, _state: &Self::State) -> Option<Rect> {
        Some(self.text.measure(None))
    }

    fn layout(&mut self, _state: &Self::State, layout: Rect, _child: Option<Rect>) -> Rect {
        layout
    }

    fn event(
        &mut self, 
        _: &mut Self::State, 
        _: Rect, 
        _: MousePosition,
        _: Event,
        _: bool
    ) -> Capture {
        Capture::None
    }

    fn hover(
        &mut self, 
        _: &mut Self::State, 
        _: Rect, 
        _: MousePosition
    ) -> bool {
        false
    }

    fn predraw<F: FnMut(Primitive)>(
        &self, 
        _: &Self::State, 
        layout: Rect, 
        mut submit: F
    ) {
        submit(Primitive::DrawText(
            self.text.clone(), 
            layout, 
            self.text_color, 
            true
        ));
    }

    fn result(self, _: &Self::State) -> Self::Result {
        ()
    }

}