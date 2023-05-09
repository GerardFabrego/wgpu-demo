use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window;

pub enum WindowEvents {
    Resize { width: u32, height: u32 },
    Draw,
}

pub struct Window {
    event_loop: EventLoop<()>,
    pub window: window::Window,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_title("Wgpu demo")
            .build(&event_loop)
            .unwrap();

        Self { event_loop, window }
    }

    pub fn run(self, callback: impl 'static + Fn(WindowEvents)) {
        self.event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::Resized(physical_size) => callback(WindowEvents::Resize {
                        width: physical_size.width,
                        height: physical_size.height,
                    }),
                    _ => {}
                },
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => callback(WindowEvents::Draw),
                _ => (),
            };
        })
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }
}
