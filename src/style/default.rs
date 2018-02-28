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
    checkbox_normal: Image,
    checkbox_hover: Image,
    checkbox_pressed: Image,
    checkbox_checked_normal: Image,
    checkbox_checked_hover: Image,
    checkbox_checked_pressed: Image,
    radio_normal: Image,
    radio_hover: Image,
    radio_pressed: Image,
    radio_checked_normal: Image,
    radio_checked_hover: Image,
    radio_checked_pressed: Image,
    input: Patch,
    default_font: Font,
    window: Patch,
    scroll_bg_h: Patch,
    scroll_bg_v: Patch,
    scroll_bar_h: Patch,
    scroll_bar_v: Patch,
}

impl Style for Default {
    fn new(ui: &mut Ui) -> Default {
        Default {
            button_normal: 
                ui.get_patch(load_from_static_memory!("../../img/button_normal.png")),
            button_hover: 
                ui.get_patch(load_from_static_memory!("../../img/button_hover.png")),
            button_pressed: 
                ui.get_patch(load_from_static_memory!("../../img/button_pressed.png")),
            checkbox_normal: 
                ui.get_image(load_from_static_memory!("../../img/checkbox_normal.png")),
            checkbox_hover: 
                ui.get_image(load_from_static_memory!("../../img/checkbox_hover.png")),
            checkbox_pressed: 
                ui.get_image(load_from_static_memory!("../../img/checkbox_pressed.png")),
            checkbox_checked_normal: 
                ui.get_image(load_from_static_memory!("../../img/checkbox_checked_normal.png")),
            checkbox_checked_hover: 
                ui.get_image(load_from_static_memory!("../../img/checkbox_checked_hover.png")),
            checkbox_checked_pressed: 
                ui.get_image(load_from_static_memory!("../../img/checkbox_checked_pressed.png")),
            radio_normal: 
                ui.get_image(load_from_static_memory!("../../img/radio_normal.png")),
            radio_hover: 
                ui.get_image(load_from_static_memory!("../../img/radio_hover.png")),
            radio_pressed: 
                ui.get_image(load_from_static_memory!("../../img/radio_pressed.png")),
            radio_checked_normal: 
                ui.get_image(load_from_static_memory!("../../img/radio_checked_normal.png")),
            radio_checked_hover: 
                ui.get_image(load_from_static_memory!("../../img/radio_checked_hover.png")),
            radio_checked_pressed: 
                ui.get_image(load_from_static_memory!("../../img/radio_checked_pressed.png")),
            input: 
                ui.get_patch(load_from_static_memory!("../../img/input.png")),
            default_font: 
                ui.get_font(load_from_static_memory!("../../img/default_font.ttf")),
            window: 
                ui.get_patch(load_from_static_memory!("../../img/window.png")),
            scroll_bg_h: 
                ui.get_patch(load_from_static_memory!("../../img/scroll_bg.png")),
            scroll_bg_v: 
                ui.get_patch(load_from_static_memory!("../../img/scroll_bg.png")),
            scroll_bar_h: 
                ui.get_patch(load_from_static_memory!("../../img/scroll_bar.png")),
            scroll_bar_v: 
                ui.get_patch(load_from_static_memory!("../../img/scroll_bar.png")),
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
            text_color: Color{ r : 0.0, g: 0.0, b: 0.0, a: 1.0 },
            border_color: None,
            size: None,
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
            text_color: Color{ r : 0.0, g: 0.0, b: 0.0, a: 1.0 },
            border_color: None,
            size: None,
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

    fn checkbox<'a>(&self, val: &'a mut bool) -> Toggle<'a, bool> {
        Toggle::new(
            val,
            true,
            false,
            self.checkbox_normal.clone(),
            self.checkbox_hover.clone(),
            self.checkbox_pressed.clone(),
            self.checkbox_checked_normal.clone(),
            self.checkbox_checked_hover.clone(),
            self.checkbox_checked_pressed.clone(),
            Color::black(),
        )
    }

    fn radio<'a, T: Clone+PartialEq+'a>(&self, val: &'a mut T, target: T) -> Toggle<'a, T> {
        Toggle::new(
            val,
            target.clone(),
            target.clone(),
            self.radio_normal.clone(),
            self.radio_hover.clone(),
            self.radio_pressed.clone(),
            self.radio_checked_normal.clone(),
            self.radio_checked_hover.clone(),
            self.radio_checked_pressed.clone(),
            Color::black(),
        )
    }

    fn flow(&self) -> Flow {
        Flow::new()
    }

    fn scroll<'a>(&self, scroll: &'a mut (f32, f32)) -> Scroll<'a> {
        Scroll::new(
            scroll,
            self.scroll_bg_h.clone(),
            self.scroll_bg_v.clone(),
            self.scroll_bar_h.clone(),
            self.scroll_bar_v.clone(),
        )
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
            minimum_size: Rect::from_wh(w, h),
            resizable: false,
            draggable: true,
            centered: true,
            modal: false,
            background: Background::Patch(self.window.clone(), 1.0),
        }
    }

    fn modal(&self, w: f32, h: f32) -> WindowProperties {
        WindowProperties {
            default_size: Rect::from_wh(w, h),
            minimum_size: Rect::from_wh(w, h),
            resizable: false,
            draggable: true,
            centered: true,
            modal: true,
            background: Background::Patch(self.window.clone(), 1.0),
        }
    }
}