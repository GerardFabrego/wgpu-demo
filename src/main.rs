use std::future::poll_fn;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Error building window");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    )).unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits:
                wgpu::Limits::default()
            ,
            label: None,
        },
        None,
    )).unwrap();

    let size = window.inner_size();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);

    surface.configure(&device, &wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: Default::default(),
        view_formats: vec![],
    });

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                ref event,
                ..
            } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(physical_size) => {
                    surface.configure(&device, &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: surface_format,
                        width: physical_size.width,
                        height: physical_size.height,
                        present_mode: wgpu::PresentMode::Fifo,
                        alpha_mode: Default::default(),
                        view_formats: vec![],
                    });
                },
                _ => {}
            }

            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let output = surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1, // Pick any color you want here
                                    g: 0.9,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                }

                // submit will accept anything that implements IntoIter
                queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            _ => (),
        }
    });
}
