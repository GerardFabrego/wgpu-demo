use camera::{Camera, CameraController, CameraEvent, CameraUniform};
use bind_groups::{create_bind_group, create_bind_group_layout, create_camera_bind_group, create_camera_bind_group_layout};
use cgmath::{InnerSpace, Rotation3, Zero};
use wgpu::util::DeviceExt;
use winit::event::VirtualKeyCode;

use crate::graphics_context::GraphicsContext;
use crate::instance::Instance;
use crate::render_pass::RenderPass;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::window::{Window, WindowEvents};

mod bind_groups;
mod camera;
mod graphics_context;
mod instance;
mod render_pass;
mod texture;
mod vertex;
mod window;

fn main() {
    let window = Window::new();
    let mut context = GraphicsContext::new(&window);

    let mut camera = Camera::new(
        (0.0, 1.0, 2.0).into(),
        (0.0, 0.0, 0.0).into(),
        cgmath::Vector3::unit_y(),
        context.config.width as f32 / context.config.height as f32,
        45.0,
        0.1,
        100.0,
    );

    let camera_controller = CameraController::new(0.2);
    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let camera_bind_group_layout = create_camera_bind_group_layout(&context.device);
    let camera_bind_group =
        create_camera_bind_group(&context.device, &camera_buffer, &camera_bind_group_layout);

    let diffuse_bytes = include_bytes!("happy-tree.png");
    let texture = Texture::from_bytes(
        &context.device,
        &context.queue,
        diffuse_bytes,
        Some("happy-tree.png"),
    );

    let texture_group_layout = create_bind_group_layout(&context.device);
    let texture_bind_group = create_bind_group(&context.device, &texture_group_layout, &texture);

    let mut depth_texture = Texture::create_depth_texture(&context.device, &context.config, "depth_texture");


    let pass = RenderPass::new(
        &context.device,
        &context.config,
        &[&texture_group_layout, &camera_bind_group_layout],
    );

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

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 {
                    x: x as f32,
                    y: 0.0,
                    z: z as f32,
                } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance::new(position, rotation)
            })
        })
        .collect::<Vec<_>>();

    let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer = context.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    window.run(move |event| match event {
        WindowEvents::Resize { width, height } => {
            depth_texture = Texture::create_depth_texture(&context.device, &context.config, "depth_texture");
            context.resize(width, height)
        },
        WindowEvents::Draw => {
            camera_uniform.update_view_proj(&camera);
            context
                .queue
                .write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
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
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

                render_pass.set_pipeline(&pass.render_pipeline);
                render_pass.set_bind_group(0, &texture_bind_group, &[]);
                render_pass.set_bind_group(1, &camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..instances.len() as _);
            }

            context.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
        WindowEvents::Keyboard(keycode) => match keycode {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                camera_controller.update(&mut camera, CameraEvent::Up)
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                camera_controller.update(&mut camera, CameraEvent::Left)
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                camera_controller.update(&mut camera, CameraEvent::Down)
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                camera_controller.update(&mut camera, CameraEvent::Right)
            }
            _ => {}
        },
    });
}
