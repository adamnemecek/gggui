use super::*;

pub struct BackgroundRenderSystem { }

impl System<Vec<Primitive>> for BackgroundRenderSystem {
    type Components = (
        FetchComponent<Layout>, 
        FetchComponent<WidgetBackground>, 
        Option<FetchComponent<Clickable>>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, (layout, background, state): Self::Components) {
        if layout.borrow().current.is_some() {
            let bg = state.map(|state| match state.borrow().deref() {
                &Clickable::Hovering =>
                    background.borrow().hover.clone(),

                &Clickable::Clicked(true) | &Clickable::Released(true) =>
                    background.borrow().click.clone(),

                &_ =>
                    background.borrow().normal.clone(),
            }).unwrap_or(background.borrow().normal.clone());

            let rect = layout.borrow().current.unwrap();

            let color = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };

            match bg {
                Background::None => {
                    // no background
                },
                Background::Color(color) => {
                    drawlist.push(Primitive::DrawRect(rect, color));
                },
                Background::Image(image, a) => {
                    drawlist.push(Primitive::DrawImage(image, rect, color.with_alpha(a)));
                },
                Background::Patch(patch, a) => {
                    drawlist.push(Primitive::Draw9(patch, rect, color.with_alpha(a)));
                },
            }
        }
    }
}