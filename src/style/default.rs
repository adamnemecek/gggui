use super::Style;
use widgets::*;
use primitive::*;
use loadable::*;
use Ui;
use Font;

pub struct Default {
    button_normal: Patch,
    button_hover: Patch,
    button_pressed: Patch,
    input: Patch,
    default_font: Font,
    window: Patch,
}

impl Style for Default {
    fn new(ui: &mut Ui) -> Default {
        Default {
            button_normal: ui.get_patch(load_from_static_memory!("../../img/button_normal.png")),
            button_hover: ui.get_patch(load_from_static_memory!("../../img/button_hover.png")),
            button_pressed: ui.get_patch(load_from_static_memory!("../../img/button_pressed.png")),
            input: ui.get_patch(load_from_static_memory!("../../img/input.png")),
            default_font: ui.get_font(load_from_static_memory!("../../img/default_font.ttf")),
            window: ui.get_patch(load_from_static_memory!("../../img/window.png")),
        }
    }

    fn lable(&self, txt: &str) -> Lable {
        Lable {
            text: Text {
                text: txt.to_string(),
                font: self.default_font.clone(),
                size: 16.0,
                wrap: TextWrap::WordWrap,
            },
            text_color: Color{ r : 0.0, g: 0.0, b: 0.0, a: 1.0 }
        }
    }

    fn title(&self, txt: &str) -> Lable {
        Lable {
            text: Text {
                text: txt.to_string(),
                font: self.default_font.clone(),
                size: 28.0,
                wrap: TextWrap::NoWrap,
            },
            text_color: Color{ r : 0.0, g: 0.0, b: 0.0, a: 1.0 }
        }
    }

    fn button(&self) -> Button {
        Button { 
            normal: self.button_normal.clone(), 
            hover: self.button_hover.clone(),
            pressed: self.button_pressed.clone(),
            size: None,
            text: None,
            text_color: Color{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        }
    }

    fn flow(&self) -> Flow {
        Flow::new()
    }

    fn input<'a>(&self, txt: &'a mut String) -> Input<'a> {
        Input::new(
            self.input.clone(),
            txt, 
            self.default_font.clone(), 
            16.0, 
            Color{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
        )
    }

    fn text(&self, text: &str) -> Text {
        Text {
            text: text.to_string(),
            font: self.default_font.clone(),
            size: 16.0,
            wrap: TextWrap::NoWrap,
        }
    }

    fn window(&self, w: f32, h: f32) -> WindowProperties {
        WindowProperties {
            default_size: Rect::from_wh(w, h),
            resizable: false,
            draggable: true,
            centered: true,
            modal: false,
            background: Background::Patch(self.window.clone()),
        }
    }

    fn modal(&self, w: f32, h: f32) -> WindowProperties {
        WindowProperties {
            default_size: Rect::from_wh(w, h),
            resizable: false,
            draggable: true,
            centered: true,
            modal: true,
            background: Background::Patch(self.window.clone()),
        }
    }
}