use crate::graphics_context::GraphicsContext;
use crate::render_pass::RenderPass;
use crate::window::{Window, WindowEvents};

mod graphics_context;
mod render_pass;
mod vertex;
mod window;

fn main() {
    let window = Window::new();
    let mut context = GraphicsContext::new(&window);
    let pass = RenderPass::new(&context.device, &context.config);

    window.run(move |event| match event {
        WindowEvents::Resize { width, height } => context.resize(width, height),
        WindowEvents::Draw => {
            let output = context.surface.get_current_texture().unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.9,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&pass.render_pipeline);
                render_pass.draw(0..3, 0..1);
            }

            // submit will accept anything that implements IntoIter
            context.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });
}
