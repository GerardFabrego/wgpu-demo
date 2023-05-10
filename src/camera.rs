use cgmath::InnerSpace;

#[derive(Debug)]
pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    speed: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub enum CameraEvent {
    Up,
    Left,
    Down,
    Right,
}

impl Camera {
    pub fn new(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Camera {
        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            speed: 0.2,
        }
    }
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn update(&mut self, event: CameraEvent) {
        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();

        let right = forward_norm.cross(self.up);

        match event {
            CameraEvent::Up => self.eye += forward_norm * self.speed,
            CameraEvent::Down => self.eye -= forward_norm * self.speed,
            CameraEvent::Left => {
                let forward = self.target - self.eye;
                let forward_mag = forward.magnitude();

                self.eye = self.target - (forward - right * self.speed).normalize() * forward_mag;
            }
            CameraEvent::Right => {
                let forward = self.target - self.eye;
                let forward_mag = forward.magnitude();

                self.eye = self.target - (forward + right * self.speed).normalize() * forward_mag;
            }
        };
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
