mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod vectors {
    #[derive(Clone, Copy, Debug)]
    pub struct Vec2f {
        pub x: f32,
        pub y: f32,
    }
}

mod renderer;
pub use renderer::font::TextRenderer;
pub use renderer::{RenderRect, Renderer, Rgb};
pub use vectors::Vec2f;

use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    let size = Vec2f { x: 1600., y: 1200. };
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Renderer")
        .with_resizable(false)
        .with_inner_size(PhysicalSize::new(size.x as u16, size.y as u16));
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

    #[cfg(any(not(feature = "x11"), target_os = "macos", windows))]
    let is_x11 = false;
    #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
    let is_x11 = event_loop.is_x11();

    let estimated_dpr = if cfg!(any(target_os = "macos", windows)) || is_x11 {
        el.available_monitors()
            .next()
            .map(|m| m.scale_factor())
            .unwrap_or(1.)
    } else {
        1.
    };

    let mut renderer = Renderer::new(size).unwrap();
    let mut font = TextRenderer::new("Roboto", 100., size, estimated_dpr).unwrap();

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0., 0., 0., 1.);
                    gl::Clear(gl::COLOR_BUFFER_BIT);

                    gl::Viewport(0, 0, size.x as i32, size.y as i32);

                    // Some basic Text
                    let text = "This is a test!";

                    let (width, height) = (size.x as i16, size.y as i16);
                    let (font_length, font_height) = (font.get_length(text), font.get_height());

                    let x = width / 2 - font_length / 2;
                    let y = height / 2 - font_height / 2;

                    font.draw_string(text, -4, 0, 0xFFFFFF);

                    renderer.draw();
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
