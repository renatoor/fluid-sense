use crate::gfx::camera::projection::Projection;
use crate::{Renderer};
use glam::{Mat4, Vec3, Vec4};
use wgpu::RenderPass;

pub mod controller;
pub mod projection;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    position: Vec4,
    view_matrix: Mat4,
    view_projection: Mat4,
}

pub struct Camera<T> {
    eye: Vec3,
    center: Vec3,
    up: Vec3,
    projection: T,
    projection_matrix: Mat4,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl<T: Projection> Camera<T> {
    pub fn new(
        renderer: &Renderer,
        pipeline: &wgpu::RenderPipeline,
        eye: Vec3,
        projection: T,
    ) -> Self
    where
        T: Projection,
    {
        let up = Vec3::Y;
        let center = Vec3::ZERO;

        let view_matrix = Mat4::look_at_rh(eye, center, up);
        let projection_matrix = projection.to_matrix();

        let uniform = CameraUniform {
            position: Vec4::from((eye, 1.0)),
            view_matrix,
            view_projection: projection_matrix * view_matrix,
        };

        let uniform_buffer = renderer.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = pipeline.get_bind_group_layout(0);

        let entries = [wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }];

        let bind_group = renderer.create_bind_group(&layout, &entries);

        Self {
            eye,
            center,
            up,
            projection,
            projection_matrix,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
        self.projection_matrix = self.projection.to_matrix();
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.center, self.up)
    }

    pub fn projection(&self) -> Mat4 {
        self.projection_matrix
    }

    pub fn position(&self) -> Vec3 {
        self.eye
    }

    pub fn update<'a>(&'a self, renderer: &Renderer, render_pass: &mut RenderPass<'a>) {
        let uniform = self.as_uniform();

        renderer.write_buffer(&self.uniform_buffer, uniform);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }

    fn as_uniform(&self) -> CameraUniform {
        CameraUniform {
            position: Vec4::from((self.eye, 1.0)),
            view_matrix: self.view(),
            view_projection: self.projection_matrix * self.view(),
        }
    }
}
