use crate::gfx::buffer::{IndexBuffer, VertexBuffer};
use crate::gfx::mesh::{Mesh, Model};
use crate::gfx::texture::Texture;
use crate::gfx::vertex::ModelVertex;

use crate::Renderer;
use glam::{Vec2, Vec3};

pub struct Plane {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
}

impl Plane {
    const VERTICES: [ModelVertex; 4] = [
        ModelVertex {
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            tex_coords: Vec2 { x: 0.0, y: 0.0 },
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        },
        ModelVertex {
            position: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            tex_coords: Vec2 { x: 0.0, y: 1.0 },
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        },
        ModelVertex {
            position: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            tex_coords: Vec2 { x: 1.0, y: 1.0 },
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        },
        ModelVertex {
            position: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            tex_coords: Vec2 { x: 1.0, y: 0.0 },
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        },
    ];

    const INDICES: [u16; 6] = [0, 2, 1, 0, 3, 2];

    pub fn new(renderer: &Renderer) -> Self {
        let vertex_buffer = VertexBuffer::new(renderer, &Self::VERTICES);
        let index_buffer = IndexBuffer::new(renderer, &Self::INDICES);

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_buffer.len(), 0, 0..1);
    }

    pub fn draw_instanced<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_buffer: &'a VertexBuffer,
    ) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_buffer.len(), 0, 0..instance_buffer.len());
    }

    pub fn as_mesh(renderer: &Renderer, pipeline: &wgpu::RenderPipeline, texture: Texture) -> Mesh {
        let model = Model {
            vertices: Vec::from(Self::VERTICES),
            indices: Some(Vec::from(Self::INDICES)),
        };

        Mesh::new(renderer, pipeline, model, texture)
    }
}
