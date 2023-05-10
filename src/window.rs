use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window;

pub enum WindowEvents {
    Resize { width: u32, height: u32 },
    Draw,
    Keyboard(VirtualKeyCode),
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

    pub fn run(self, mut callback: impl 'static + FnMut(WindowEvents)) {
        self.event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.set_exit(),
                    WindowEvent::Resized(physical_size) => callback(WindowEvents::Resize {
                        width: physical_size.width,
                        height: physical_size.height,
                    }),
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => callback(WindowEvents::Keyboard(*keycode)),
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
