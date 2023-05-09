use crate::graphics_context::GraphicsContext;
use crate::window::{Window, WindowEvents};

mod graphics_context;
mod window;

fn main() {
    let window = Window::new();
    let mut graphics_context = GraphicsContext::new(&window);

    window.run(move |event| match event {
        WindowEvents::Resize { width, height } => graphics_context.resize(width, height),
        WindowEvents::Draw => {
            graphics_context.render().expect("Error while rendering");
        }
    });
}
