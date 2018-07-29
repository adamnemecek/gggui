#[cfg(feature="winit-events")] extern crate gggui;
#[cfg(feature="winit-events")] extern crate glutin;
#[cfg(feature="winit-events")] extern crate winit;
#[cfg(feature="gfx-renderer")] extern crate gfx;

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

    type ColorFormat = gfx::format::Srgba8;
    type DepthFormat = gfx::format::DepthStencil;

    pub fn demo_frame(style: &Style, ui: &mut Ui) {
        ui.window(style, "test", WindowLayer::Normal).with(|ui| {
            ui.add("", Window::new(Rect::from_wh(256.0, 256.0), true)).with(|ui| {
                //
            });
        });
    }

    pub fn demo() {
        let window_config = glutin::WindowBuilder::new()
            .with_title("gg gui test".to_string())
            .with_dimensions(1024, 768);

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

        while !finished {
            let mut ui_events = vec![];

            events_loop.poll_events(|event| {
                use glutin::{Event, WindowEvent};

                convert_event(event.clone()).map(|event| ui_events.push(event));

                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        WindowEvent::Resized(width, height) => {
                            window.resize(width, height);
                            gfx_window_glutin::update_views(
                                &window, 
                                &mut main_color,
                                &mut main_depth
                            );
                        },

                        WindowEvent::Closed => {
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
            demo_frame(&style, &mut ui);

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
                 Try running `cargo run --release --no-default-features --features=\"winit-events gfx-renderer\" --example <example_name>`");
   }
}