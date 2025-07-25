use crate::gfx::mesh::{Mesh, Model};
use crate::gfx::texture::Texture;
use crate::gfx::vertex::ModelVertex;

use crate::Renderer;
use glam::{Vec2, Vec3};

pub struct Plane {}

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

    pub fn as_mesh(renderer: &Renderer, pipeline: &wgpu::RenderPipeline, texture: Texture) -> Mesh {
        let model = Model {
            vertices: Vec::from(Self::VERTICES),
            indices: Some(Vec::from(Self::INDICES)),
        };

        Mesh::new(renderer, pipeline, model, texture)
    }
}
