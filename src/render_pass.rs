use crate::graphics_context::create_render_pipeline;
use wgpu::BindGroupLayout;

pub struct RenderPass {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl RenderPass {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> RenderPass {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };

            create_render_pipeline(device, &render_pipeline_layout, shader, config.format)
        };

        RenderPass { render_pipeline }
    }
}
