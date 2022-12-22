use crate::gfx::buffer::VertexBuffer;
use crate::gfx::mesh::{Material, Mesh};
use crate::gfx::texture::Texture;
use crate::gfx::vertex::InstanceVertex;
use crate::scene::object::cube::Cube;
use crate::{Pipeline, Plane, Renderer};
use glam::{Vec3, Vec4};

pub mod object;
pub mod world_map;

pub struct Scene {
    user_position: (f32, f32),
    floor_mesh: Mesh,
    floor_instances: Vec<InstanceVertex>,
    floor_instance_buffer: VertexBuffer,
    wall_mesh: Mesh,
    wall_instances: Vec<InstanceVertex>,
    wall_instance_buffer: VertexBuffer,
}

impl Scene {
    pub fn new(
        renderer: &Renderer,
        pipeline: &wgpu::RenderPipeline,
        user_position: (f32, f32),
        floor_instances: Vec<InstanceVertex>,
        wall_instances: Vec<InstanceVertex>,
    ) -> Self {
        let texture_layout = pipeline.get_bind_group_layout(2);

        let floor_texture_bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/textures/floor-texture.jpg"
        ));

        let wall_texture_bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/textures/wall-texture.jpg"
        ));

        let floor_texture = Texture::from_bytes(renderer, &texture_layout, 2, floor_texture_bytes);
        let floor_mesh = Plane::as_mesh(renderer, pipeline, floor_texture);
        let floor_instance_buffer = VertexBuffer::new(renderer, &floor_instances);

        let wall_texture = Texture::from_bytes(renderer, &texture_layout, 2, wall_texture_bytes);
        let wall_mesh = Cube::as_mesh(renderer, pipeline, wall_texture);
        let wall_instance_buffer = VertexBuffer::new(renderer, &wall_instances);

        Self {
            user_position,
            floor_mesh,
            floor_instances,
            floor_instance_buffer,
            wall_mesh,
            wall_instances,
            wall_instance_buffer,
        }
    }

    pub fn user_position(&self) -> (f32, f32) {
        self.user_position
    }

    pub fn draw_mesh<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.floor_mesh
            .draw_instanced(render_pass, &self.floor_instance_buffer);
        self.wall_mesh
            .draw_instanced(render_pass, &self.wall_instance_buffer);
    }
}
