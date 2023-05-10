use crate::bind_groups::{create_bind_group, create_bind_group_layout};
use wgpu::util::DeviceExt;

use crate::graphics_context::GraphicsContext;
use crate::render_pass::RenderPass;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::window::{Window, WindowEvents};

mod bind_groups;
mod graphics_context;
mod render_pass;
mod texture;
mod vertex;
mod window;

fn main() {
    let window = Window::new();
    let mut context = GraphicsContext::new(&window);

    let diffuse_bytes = include_bytes!("happy-tree.png");
    let texture = Texture::from_bytes(
        &context.device,
        &context.queue,
        diffuse_bytes,
        Some("happy-tree.png"),
    );

    let bind_group_layout = create_bind_group_layout(&context.device);
    let bind_group = create_bind_group(&context.device, &bind_group_layout, &texture);

    let pass = RenderPass::new(&context.device, &context.config, &[&bind_group_layout]);

    const VERTICES: &[Vertex] = &[
        Vertex {
            position: [-0.0868241, 0.49240386, 0.0],
            tex_coords: [0.4131759, 0.00759614],
        },
        Vertex {
            position: [-0.49513406, 0.06958647, 0.0],
            tex_coords: [0.0048659444, 0.43041354],
        },
        Vertex {
            position: [-0.21918549, -0.44939706, 0.0],
            tex_coords: [0.28081453, 0.949397],
        },
        Vertex {
            position: [0.35966998, -0.3473291, 0.0],
            tex_coords: [0.85967, 0.84732914],
        },
        Vertex {
            position: [0.44147372, 0.2347359, 0.0],
            tex_coords: [0.9414737, 0.2652641],
        },
    ];

    const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

    let vertex_buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let index_buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

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
                render_pass.set_bind_group(0, &bind_group, &[]); // NEW!
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
            }

            // submit will accept anything that implements IntoIter
            context.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });
}
