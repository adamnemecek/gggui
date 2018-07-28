use super::*;

pub struct Toggle<T: Clone+PartialEq> {
    clicked: Option<T>,
    state_checked: Option<T>,
    state_normal: Option<T>,
    checked: bool,
    radio: bool,
}

impl<T: Clone+PartialEq> Toggle<T> {
    pub fn checkbox(value: T, checked: T, normal: T) -> Self {
        let is_checked = value == checked;

        Self {
            state_checked: Some(checked),
            state_normal: Some(normal),
            clicked: None,
            checked: is_checked,
            radio: false,
        }
    }

    pub fn radio(value: T, checked: T) -> Self {
        let is_checked = value == checked;

        Self {
            state_checked: Some(checked.clone()),
            state_normal: Some(checked.clone()),
            clicked: None,
            checked: is_checked,
            radio: true,
        }
    }
}

impl<T: Clone+PartialEq> WidgetBase for Toggle<T> {
    fn create(&mut self, id: dag::Id, world: &mut Ui, style: &Style) {
        let background = WidgetBackground{
            normal: Background::None,
            hover: Background::None,
            click: Background::None,
        };

        if self.radio {
            world.create_component(id, Layout::new()
                .with_intrinsic_size_constraints(
                    style.radio_normal.size.width(), 
                    style.radio_normal.size.height(), 252.0));
        } else {
            world.create_component(id, Layout::new()
                .with_intrinsic_size_constraints(
                    style.checkbox_normal.size.width(), 
                    style.checkbox_normal.size.height(), 252.0));
        }        
        world.create_component(id, background);
        world.create_component(id, Clickable::Idle);
    }

    fn update(&mut self, id: dag::Id, world: &Ui, style: &Style, _input: Option<Rect>) -> Option<Rect> {
        let mut clickable = world.component::<Clickable>(id).unwrap();
        let mut clickable = clickable.borrow_mut();

        let mut background = world.component::<WidgetBackground>(id).unwrap();
        let mut background = background.borrow_mut();

        if self.radio {
            if self.checked {
                background.normal = Background::Image(style.radio_checked_normal.clone(), 1.0);
                background.hover = Background::Image(style.radio_checked_hover.clone(), 1.0);
                background.click = Background::Image( style.radio_checked_pressed.clone(), 1.0);
            } else {
                background.normal = Background::Image(style.radio_normal.clone(), 1.0);
                background.hover = Background::Image(style.radio_hover.clone(), 1.0);
                background.click = Background::Image( style.radio_pressed.clone(), 1.0);
            }
        } else {
            if self.checked {
                background.normal = Background::Image(style.checkbox_checked_normal.clone(), 1.0);
                background.hover = Background::Image(style.checkbox_checked_hover.clone(), 1.0);
                background.click = Background::Image( style.checkbox_checked_pressed.clone(), 1.0);
            } else {
                background.normal = Background::Image(style.checkbox_normal.clone(), 1.0);
                background.hover = Background::Image(style.checkbox_hover.clone(), 1.0);
                background.click = Background::Image( style.checkbox_pressed.clone(), 1.0);
            }
        }

        *clickable = match *clickable {
            Clickable::Released(x) => {
                if x {
                    self.clicked = if self.checked {
                        self.state_normal.take()
                    } else {
                        self.state_checked.take()
                    }
                }
                Clickable::Idle
            },
            x => x,
        };

        None
    }
}

impl<T: Clone+PartialEq> Widget for Toggle<T> {
    type Result = Option<T>;

    fn result(&mut self, _id: dag::Id) -> Self::Result {
        self.clicked.clone()
    }
}