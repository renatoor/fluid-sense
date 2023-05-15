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
    model: Model,
    texture: Texture,
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
    //uniform_buffer: wgpu::Buffer,
    //bind_group: wgpu::BindGroup,
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

        /*
        let uniform_buffer = renderer.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[material]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = renderer.create_bind_group(
            &renderer.pipeline.material_layout,
            uniform_buffer.as_entire_binding(),
        );
         */

        Self {
            model,
            texture,
            vertex_buffer,
            index_buffer,
            //uniform_buffer,
            //bind_group,
        }
    }

    pub fn draw(&self) {
        todo!()
    }

    pub fn draw_instanced<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_buffer: &'a VertexBuffer,
    ) {
        self.texture.bind(render_pass);
        //render_pass.set_bind_group(self.texture.index, &self.texture.bind_group, &[]);
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
