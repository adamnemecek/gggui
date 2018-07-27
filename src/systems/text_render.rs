use super::*;

pub struct TextRenderSystem { }

impl System<Vec<Primitive>> for TextRenderSystem {
    type Components = (
        FetchComponent<Layout>, 
        FetchComponent<Text>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, (layout, text): Self::Components) {
        let layout = layout.borrow();
        let text = text.borrow();

        if layout.current.is_some() {
            let rect = layout.current().unwrap().after_padding(text.padding);

            drawlist.push(Primitive::DrawText(text.clone(), rect));
        }
    }
}