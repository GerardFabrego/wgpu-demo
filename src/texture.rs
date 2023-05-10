use image::{GenericImageView, RgbaImage};
use wgpu::{Device, Extent3d, Queue, Sampler, TextureView};

pub struct Texture {
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8]) -> Texture {
        let diffuse_image = image::load_from_memory(bytes).unwrap();

        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        let size = get_texture_size(dimensions);
        let diffuse_texture = create_texture(size, &device);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture { view, sampler }
    }
}

fn get_texture_size(dimensions: (u32, u32)) -> Extent3d {
    wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    }
}

fn create_texture(texture_size: Extent3d, device: &Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
        view_formats: &[],
    })
}
