use glam::{Mat4, Quat, Vec3};

pub mod cube;
pub mod particle;
pub mod plane;

pub struct Transform {
    pub scale: Vec3,
    pub rotation: Quat,
    pub translation: Vec3,
}

impl Transform {
    pub fn new(scale: Vec3, rotation: Quat, translation: Vec3) -> Self {
        Self {
            scale,
            rotation,
            translation,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            translation: Vec3::ZERO,
        }
    }
}

pub trait Object {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}
