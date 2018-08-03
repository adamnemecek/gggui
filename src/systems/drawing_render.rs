use super::*;

pub struct DrawingRenderSystem { }

impl System<Vec<Primitive>> for DrawingRenderSystem {
    type Components = (
        FetchComponent<Drawing>,
        Option<FetchComponent<Layout>>,
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, (mut drawing, layout): Self::Components) {
        let mut drawing = drawing.borrow_mut();
        
        layout
            .and_then(|layout| Some(drawing.update(Some(&layout.borrow()))))
            .or_else(|| Some(drawing.update(None)));

        drawlist.append(&mut drawing.primitives.clone());
    }
}