use crate::graphics_context::GraphicsContext;
use crate::window::{Window, WindowEvents};

mod graphics_context;
mod window;

fn main() {
    let window = Window::new();
    let mut context = GraphicsContext::new(&window);

    window.run(move |event| match event {
        WindowEvents::Resize { width, height } => context.resize(width, height),
        WindowEvents::Draw => {
            context.render().expect("Error while rendering");
        }
    });
}
