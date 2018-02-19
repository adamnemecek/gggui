use super::*;

pub struct Window {
    flow: Flow,
    modal: bool,
}

impl Window {
    pub fn with_color(size: Rect, color: Color) -> Self {
        Window {
            flow: Flow::new().size(size).background_color(color),
            modal: false,
        }
    }

    pub fn with_image(size: Rect, image: Image) -> Self {
        Window {
            flow: Flow::new().size(size).background_image(image),
            modal: false,
        }
    }

    pub fn with_patch(size: Rect, patch: Patch) -> Self {
        Window {
            flow: Flow::new().size(size).background_patch(patch),
            modal: false,
        }
    }

    pub fn style(mut self, style: FlowStyle) -> Self {
        self.flow.style = style;
        self
    }

    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }
}

impl Widget for Window {
    type Result = ();
    type State = GenericWidgetState;

    fn window() -> bool {
        true
    }

    fn enabled(&self, _: &Self::State) -> bool {
        self.flow.enabled
    }

    fn default() -> Self::State { Flow::default() }

    fn measure(&self, _: &Self::State, _: Option<Rect>) -> Option<Rect> {
        self.flow.size
    }

    fn layout(
        &mut self, 
        state: &Self::State, 
        layout: Rect, 
        child: WidgetMeasure
    ) -> Rect {
        self.flow.layout(state, layout, child)
    }

    fn event(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition,
        event: Event,
        is_focused: bool
    ) -> Capture {
        self.flow.event(state, layout, cursor, event, is_focused)
    }

    fn hover(
        &mut self, 
        state: &mut Self::State, 
        layout: Rect, 
        cursor: MousePosition
    ) -> bool {
        self.flow.hover(state, layout, cursor)
    }

    fn predraw<F: FnMut(Primitive)>(
        &self, 
        state: &Self::State, 
        layout: Rect, 
        submit: F
    ) {
        self.flow.predraw(state, layout, submit)
    }

    fn childs(&self, state: &Self::State, layout: Rect) -> ChildType {
        self.flow.childs(state, layout)
    }

    fn result(self, _: &Self::State) -> Self::Result {
        ()
    }

}