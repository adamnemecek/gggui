use super::*;
use ecsui::components::Layout;
use ecsui::components::Background;
use primitive::*;

pub struct BackgroundRenderSystem { }

impl System<Vec<Primitive>> for BackgroundRenderSystem {
    type Components = (
        FetchComponent<Layout>, 
        FetchComponent<Background>, 
        Option<FetchComponent<Clickable>>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, (layout, background, state): Self::Components) {
        let patch = state.map(|state| match state.borrow().deref() {
            &Clickable::Hovering =>
                background.borrow().hover.clone(),

            &Clickable::Clicked(true) | &Clickable::Released(true) =>
                background.borrow().click.clone(),

            &_ =>
                background.borrow().normal.clone(),
        }).unwrap_or(background.borrow().normal.clone());

        let rect = layout.borrow().current;

        let color = Color{ r:1.0, g:1.0, b:1.0, a:1.0 };
        
        drawlist.push(Primitive::Draw9(patch, rect, color));
    }
}