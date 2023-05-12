use bind_groups::{
    create_bind_group, create_bind_group_layout, create_camera_bind_group,
    create_camera_bind_group_layout,
};
use camera::{Camera, CameraController, CameraEvent, CameraUniform};
use cgmath::{InnerSpace, Rotation3, Zero};
use wgpu::util::DeviceExt;
use winit::event::VirtualKeyCode;
use crate::bind_groups::{create_light_bind_group, create_light_bind_group_layout};

use crate::graphics_context::{create_render_pipeline, GraphicsContext};
use crate::instance::Instance;
use crate::light::LightUniform;
use crate::object::{DrawLight, DrawModel, Vertex};
use crate::render_pass::RenderPass;
use crate::texture::Texture;
use crate::window::{Window, WindowEvents};

mod bind_groups;
mod camera;
mod graphics_context;
mod instance;
mod object;
mod render_pass;
mod resources;
mod texture;
mod window;
mod light;

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

    let mut light_uniform = LightUniform::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);
    let light_buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    let light_bind_group_layout = create_light_bind_group_layout(&context.device);
    let light_bind_group = create_light_bind_group(&context.device, &light_buffer, &light_bind_group_layout);
    let light_render_pipeline = {
        let layout = &context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
        };
        create_render_pipeline(
            &context.device,
            layout,
            context.config.format,
            Some(Texture::DEPTH_FORMAT),
            &[object::ModelVertex::desc()],
            shader,
        )
    };
    let texture_bind_group_layout = create_bind_group_layout(&context.device);

    let mut depth_texture =
        Texture::create_depth_texture(&context.device, &context.config, "depth_texture");

    let pass = RenderPass::new(
        &context.device,
        &context.config,
        &[&texture_bind_group_layout, &camera_bind_group_layout, &light_bind_group_layout],
    );

    let obj_model = pollster::block_on(resources::load_model(
        "cube.obj",
        &context.device,
        &context.queue,
        &texture_bind_group_layout,
    ))
    .unwrap();

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );
    const SPACE_BETWEEN: f32 = 3.0;
    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

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
    let instance_buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

    window.run(move |event| match event {
        WindowEvents::Resize { width, height } => {
            depth_texture =
                Texture::create_depth_texture(&context.device, &context.config, "depth_texture");
            context.resize(width, height)
        }
        WindowEvents::Draw => {
            //update camera
            camera_uniform.update_view_proj(&camera);
            context
                .queue
                .write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

            //update light
            let old_position: cgmath::Vector3<_> = light_uniform.position.into();
            light_uniform.position =
                (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                    * old_position)
                    .into();
            context.queue.write_buffer(&light_buffer, 0, bytemuck::cast_slice(&[light_uniform]));

            //render

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
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

                render_pass.set_pipeline(&light_render_pipeline);
                render_pass.draw_light_model(
                    &obj_model,
                    &camera_bind_group,
                    &light_bind_group,
                );

                render_pass.set_pipeline(&pass.render_pipeline);
                render_pass.draw_model_instanced(
                    &obj_model,
                    0..instances.len() as u32,
                    &camera_bind_group,
                    &light_bind_group,
                );
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
