use primitive;

#[derive(Clone)]
pub struct WidgetBackground {
    pub normal: primitive::Background,
    pub hover: primitive::Background,
    pub click: primitive::Background,
}