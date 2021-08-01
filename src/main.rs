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
pub use renderer::{RenderRect, Renderer, Rgb};
pub use vectors::Vec2f;

use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    let size = Vec2f { x: 640., y: 640. };
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(LogicalSize::new(size.x as u16, size.y as u16));
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

    let mut renderer = Renderer::new().unwrap();

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

                    renderer.rectangle(
                        size,
                        &mut RenderRect {
                            x: 0.,
                            y: 0.,
                            width: 200.,
                            height: 200.,
                            color: Rgb {
                                r: 255,
                                g: 255,
                                b: 0,
                            },
                        },
                    );

                    renderer.rectangle(
                        size,
                        &mut RenderRect {
                            x: 150.,
                            y: 150.,
                            width: 200.,
                            height: 200.,
                            color: Rgb { r: 255, g: 0, b: 0 },
                        },
                    );

                    renderer.draw(size);
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
