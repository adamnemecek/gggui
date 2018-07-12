use primitive;

#[derive(Clone)]
pub struct WidgetBackground {
    pub normal: primitive::Background,
    pub hover: primitive::Background,
    pub click: primitive::Background,
}

impl WidgetBackground {
	pub fn new(bg: primitive::Background) -> Self {
		Self {
			normal: bg.clone(),
			hover: bg.clone(),
			click: bg.clone(),
		}
	}
}