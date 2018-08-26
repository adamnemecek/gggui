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

    use cassowary::strength::*;
    use cassowary::WeightedRelation::*;

    type ColorFormat = gfx::format::Rgba8;
    type DepthFormat = gfx::format::DepthStencil;

    pub struct State {
        pub name: String,
        pub pass: String,
        pub remember: bool,
    }

    pub fn demo_frame(style: &Style, ui: &mut Ui, state: &mut State) {
        ui.layer(style, "test", Layer::Normal).with(|ui| {
            ui.add("", Window::new(Rect::from_wh(256.0, 256.0), true)).with(|ui| {
                ui.add("name_txt", Label::simple("Username: "));
                ui.add("name", Input::new(&mut state.name));
                ui.add("pass_txt", Label::simple("Password: "));
                ui.add("pass", Input::password(&mut state.pass));
                ui.add("b1", Button::new().with_size((128.0, 32.0)));
                ui.add("b2", Button::new().with_size((128.0, 32.0)));
                state.remember = ui.add("cb", Toggle::checkbox(state.remember, true, false)).result.unwrap_or(state.remember);
                ui.add("txt", Label::simple("Remember me"));
                ui.add("s", Scroll::new().with_vertical_bar()).with(|ui| {
                    ui.add("test", Button::new().with_size((128.0, 512.0)));

                    layout_rules!(ui,
                        (test.left = super.margin_left + 20.0),
                        (test.right = super.margin_right - 20.0),
                        (test.top = super.margin_top + 20.0),
                        (test.bottom = super.margin_bottom - 20.0),
                    );
                });

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
                    (s.left = super.margin_left + 20.0),
                    (s.right = super.margin_right - 20.0),
                    (s.top = b2.bottom + 8.0),
                    (s.bottom = super.margin_bottom - 20.0),
                );
            });
        });
    }

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
            ui_render.draw(&mut factory, &mut encoder, &main_color, drawlist);

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