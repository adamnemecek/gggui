use super::*;

pub struct Frame { 
    background: Background,
    margin: Rect,
}

impl Frame {
    pub fn new(background: Background) -> Self {
        Self { background, margin: Rect::zero() }
    }

    pub fn with_margins(mut self, margin: Rect) -> Self {
        self.margin = margin;
        self
    }
}

impl WidgetBase for Frame {
    fn create(&mut self, id: dag::Id, world: &mut Ui, _style: &Style) {
        let background = WidgetBackground{
            normal: self.background.clone(),
            hover: self.background.clone(),
            click: self.background.clone(),
        };
        world.create_component(id, Layout::new().with_margins(self.margin));
        world.create_component(id, background);
    }

    fn update(&mut self, id: dag::Id, world: &mut Ui, _style: &Style, input: Option<Rect>) -> Option<Rect> {
        let layout = world.component::<Layout>(id).unwrap();
        let layout = layout.borrow();
        layout.current()
            .and_then(|current| Some(current.after_padding(self.margin)))
            .and_then(|content| input.and_then(|ir| ir.intersect(&content)))        
    }
}

impl Widget for Frame {
    type Result = ();

    fn result(&mut self, _id: dag::Id) -> Self::Result {
        ()
    }
}