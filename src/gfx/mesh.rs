use crate::gfx::buffer::{IndexBuffer, VertexBuffer};
use crate::gfx::texture::Texture;
use crate::gfx::vertex::ModelVertex;
use crate::Renderer;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub struct Model {
    pub vertices: Vec<ModelVertex>,
    pub indices: Option<Vec<u16>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Material {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
    pub shininess: f32,
    pub padding: [u32; 2],
}

pub struct Mesh {
    texture: Texture,
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

impl Mesh {
    pub fn new(
        renderer: &Renderer,
        _pipeline: &wgpu::RenderPipeline,
        model: Model,
        texture: Texture,
    ) -> Self {
        let vertex_buffer = VertexBuffer::new(renderer, &model.vertices);
        let index_buffer = match &model.indices {
            Some(indices) => Some(IndexBuffer::new(renderer, indices)),
            None => None,
        };

        Self {
            texture,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw_instanced<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_buffer: &'a VertexBuffer,
    ) {
        self.texture.bind(render_pass);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        match &self.index_buffer {
            Some(index_buffer) => {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..index_buffer.len(), 0, 0..instance_buffer.len());
            }
            None => render_pass.draw(0..self.vertex_buffer.len(), 0..instance_buffer.len()),
        }
    }
}
