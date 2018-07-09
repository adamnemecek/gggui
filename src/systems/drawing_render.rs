use super::*;

pub struct DrawingRenderSystem { }

impl System<Vec<Primitive>> for DrawingRenderSystem {
    type Components = (
        FetchComponent<Drawing>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, drawing: Self::Components) {
        let drawing = drawing.borrow();
        drawlist.append(&mut drawing.primitives.clone());
    }
}