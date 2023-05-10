use crate::camera::{Camera, CameraEvent};
use cgmath::InnerSpace;

pub struct CameraController {
    speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> CameraController {
        CameraController { speed }
    }

    pub fn update(&self, camera: &mut Camera, event: CameraEvent) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();

        let right = forward_norm.cross(camera.up);

        match event {
            CameraEvent::Up => camera.eye += forward_norm * self.speed,
            CameraEvent::Down => camera.eye -= forward_norm * self.speed,
            CameraEvent::Left => {
                let forward = camera.target - camera.eye;
                let forward_mag = forward.magnitude();

                camera.eye =
                    camera.target - (forward - right * self.speed).normalize() * forward_mag;
            }
            CameraEvent::Right => {
                let forward = camera.target - camera.eye;
                let forward_mag = forward.magnitude();

                camera.eye =
                    camera.target - (forward + right * self.speed).normalize() * forward_mag;
            }
        };
    }
}
