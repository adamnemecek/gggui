#[cfg(feature="winit-events")] #[macro_use] extern crate gggui;
#[cfg(feature="winit-events")] extern crate glutin;
#[cfg(feature="winit-events")] extern crate winit;
#[cfg(feature="gfx-renderer")] extern crate gfx;

extern crate cassowary;

pub fn main() {
    feature::demo()
}

#[cfg(all(feature="winit-events",feature="gfx-renderer"))]
mod feature {
    extern crate gfx_window_glutin;
    extern crate gfx_device_gl;

    use gfx;
    use glutin;
    use gfx::Device;
    use glutin::GlContext;

    use gggui::*;
    use gggui::features::gfx::Renderer as UiRenderer;
    use gggui::features::winit::*;

    type ColorFormat = gfx::format::Rgba8;
    type DepthFormat = gfx::format::DepthStencil;

    pub struct State {
        pub name: String,
        pub pass: String,
        pub remember: bool,
    }

    pub fn demo_frame(style: &Style, ui: &mut Ui, state: &mut State) {

        // for the example we're going to make some kind of login window.
        // it consists of a name, password, remember me field and login/cancel buttons.
        // to show an example of some more stuff there is a scroll with a list of labels below it.

        // a layer can be Back, Normal or Modal.
        // 1) `Layer::Back` layers will always remain on the background.
        // 2) `Layer::Normal` layers will be ordered where the last used layer is on top.
        // 3) `Layer::Modal` layers will always be in front of all the other layers.
        // this is a normal layer.
        ui.layer(style, "test", Layer::Normal).with(|ui| {
            // a window is just a widget that can be dragged around in it's parent.
            ui.add("w", Window::new(Rect::from_wh(256.0, 256.0), true)).with(|ui| {
                // make widgets for the login controls
                ui.add("name_txt", Label::simple("Username: "));
                ui.add("name", Input::new(&mut state.name));
                ui.add("pass_txt", Label::simple("Password: "));
                ui.add("pass", Input::password(&mut state.pass));

                // the wrap function "wraps" the given widget inside the widget of the WidgetResult.
                // So in this case the Login label lives inside the button.
                // The wrap function also sets the right layout for the child so it is
                //  aligned with the host margins.
                ui.add("b1", Button::new().with_size((128.0, 32.0))).wrap(Label::simple("Login"));
                ui.add("b2", Button::new().with_size((128.0, 32.0))).wrap(Label::simple("Cancel"));
                state.remember = ui.add("cb", Toggle::checkbox(state.remember, true, false)).result.unwrap_or(state.remember);
                ui.add("txt", Label::simple("Remember me"));

                // The wrap_with functions does the same as wrap, but allow you to specify a closure
                //  with which you can add children to the wrapped widget.
                ui.add("list", Scroll::new().with_vertical_bar())
                    .wrap_with(Collection::new(TopToBottomLayout::new(ContentAlign::Leading)), |ui| {
                        for i in 0..20 {
                            let num = format!("{}", i);
                            ui.add(&num, Label::simple_owned(format!("label {}", i)));
                        }
                    });

                // after you have specified widgets and they are not in a `Container`, you have
                //  to specify layout rules. These are simple constraints that hint gggui where to
                //  place widgets. You need to specify just enough rules for the whole thing to
                //  make sense.
                layout_rules!(ui, 
                    (name.right = super.margin_right - 20.0),
                    (name.left = name_txt.right),
                    (name.top = super.margin_top + 20.0),
                    (name_txt.left = super.margin_left + 20.0),
                    (name_txt.bottom = name.margin_bottom),
                    (pass.right = super.margin_right - 20.0),
                    (pass.left = pass_txt.right),
                    (pass.top = name.bottom + 8.0),
                    (pass_txt.left = super.margin_left + 20.0),
                    (pass_txt.bottom = pass.margin_bottom),
                    (name.left = pass.left),
                    (name.left = txt.left),
                    (txt.top = pass.bottom + 8.0),
                    (txt.left = cb.right + 8.0),
                    (txt.bottom = cb.bottom),
                    (b1.left = super.margin_left + 20.0),
                    (b1.top = cb.bottom + 8.0),
                    (b2.top = b1.top),
                    (b2.left = b1.right + 8.0),
                    (b2.width = b1.width),
                    (b2.right = super.margin_right - 20.0),
                    (list.left = super.margin_left + 20.0),
                    (list.right = super.margin_right - 20.0),
                    (list.top = b2.bottom + 8.0),
                    (list.bottom = super.margin_bottom - 20.0),
                );
            });
        });
    }

    // boilerplate code for winit, glutin and gfx.
    pub fn demo() {
        let window_config = glutin::WindowBuilder::new()
            .with_title("gg gui test".to_string())
            .with_dimensions((1024, 768).into());

        let mut events_loop = glutin::EventsLoop::new();

        let (api, version) = if cfg!(target_os = "emscripten") {
            (glutin::Api::WebGl, (2, 0))
        } else {
            (glutin::Api::OpenGl, (3, 2))
        };

        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(api, version))
            .with_srgb(false)
            .with_depth_buffer(0)
            .with_vsync(true);
        
        let (window, mut device, mut factory, mut main_color, mut main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(
                window_config, 
                context, 
                &events_loop);

        let mut encoder = gfx::Encoder::from(factory.create_command_buffer());
        
        let mut ui_render = UiRenderer::new(&mut factory);
        let mut ui = Ui::new();

        let mut finished = false;

        let style = Style::default(&mut ui);

        let mut state = State {
            name: "".to_string(),
            pass: "".to_string(),
            remember: false,
        };

        while !finished {
            let mut ui_events = vec![];

            events_loop.poll_events(|event| {
                use glutin::{Event, WindowEvent};

                convert_event(event.clone()).map(|event| ui_events.push(event));

                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        WindowEvent::Resized(size) => {
                            window.resize(size.to_physical(1.0));
                            gfx_window_glutin::update_views(
                                &window, 
                                &mut main_color,
                                &mut main_depth
                            );
                        },

                        WindowEvent::CloseRequested => {
                            finished = true;
                        }

                        _ => (),
                    }
                }
            });

            let viewport = {
                let (width,height,_depth,_samples) = main_color.get_dimensions();
                Rect {
                    left: 0.0,
                    top: 0.0,
                    right: width as _,
                    bottom: height as _,
                }
            };

            // prepare ui globals
            ui.update(viewport, ui_events.into());

            // process the ui
            demo_frame(&style, &mut ui, &mut state);

            // render ui
            encoder.clear(&main_color, [0.3, 0.3, 0.3, 1.0]);
            let (drawlist, mouse_style, _mouse_mode) = ui.render();
            ui_render.draw(1.0, &mut factory, &mut encoder, &main_color, &drawlist);

            // flush and swap
            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            window.set_cursor(convert_mouse_style(mouse_style));
            device.cleanup();
        }
    }
}

#[cfg(not(all(feature="winit-events",feature="gfx-renderer")))]
mod feature {
    pub fn demo() {
        println!("This example requires the `winit-events` feature and the `gfx-renderer` feature. \
                 Try running `cargo run --release --features=\"winit-events gfx-renderer\" --example <example_name>`");
   }
}