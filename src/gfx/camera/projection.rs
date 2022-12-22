use glam::Mat4;

pub trait Projection {
    fn to_matrix(&self) -> Mat4;
    fn resize(&mut self, width: u32, height: u32);
}

pub struct Perspective {
    vertical_fov: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
}

impl Perspective {
    pub fn new(vertical_fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            vertical_fov: vertical_fov.to_radians(),
            aspect_ratio,
            z_far,
            z_near,
        }
    }
}

impl Projection for Perspective {
    fn to_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.vertical_fov,
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }
}
