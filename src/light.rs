#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub(crate) position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
            _padding2: 0
        }
    }
}
