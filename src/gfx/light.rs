use crate::Renderer;
use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: Vec4,
    color: Vec4,
}

pub struct Light {
    position: Vec3,
    color: Vec3,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(
        renderer: &Renderer,
        pipeline: &wgpu::RenderPipeline,
        position: Vec3,
        color: Vec3,
    ) -> Self {
        let uniform = LightUniform {
            position: Vec4::from((position, 1.0)),
            color: Vec4::from((color, 1.0)),
        };

        let uniform_buffer = renderer.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = pipeline.get_bind_group_layout(1);

        let entries = [wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }];

        let bind_group = renderer.create_bind_group(&layout, &entries);

        Self {
            position,
            color,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn update<'a>(&'a self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass<'a>) {
        let uniform = self.as_uniform();

        renderer.write_buffer(&self.uniform_buffer, uniform);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
    }

    fn as_uniform(&self) -> LightUniform {
        LightUniform {
            position: Vec4::from((self.position, 1.0)),
            color: Vec4::from((self.color, 1.0)),
        }
    }
}
