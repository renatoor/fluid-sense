use crate::gfx::buffer::{VertexBuffer};
use crate::gfx::mesh::{Mesh, Model};
use crate::gfx::texture::Texture;
use crate::gfx::vertex::{ModelVertex};

use crate::{Renderer};
use glam::{Vec2, Vec3};


pub struct Cube {
    vertex_buffer: VertexBuffer,
}

impl Cube {
    /*
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
     */

    pub fn as_mesh(renderer: &Renderer, pipeline: &wgpu::RenderPipeline, texture: Texture) -> Mesh {
        let points = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];

        let tex_coords = [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE];

        let normals = [
            Vec3::Z,
            Vec3::NEG_Z,
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
        ];

        let faces = [
            ([0, 2, 1, 0, 3, 2], [0, 3, 1, 0, 2, 3], 1), // front
            ([2, 3, 4, 2, 4, 5], [1, 0, 2, 1, 2, 3], 4), // top
            ([1, 2, 5, 1, 5, 6], [0, 2, 3, 0, 3, 1], 2), // right
            ([0, 7, 4, 0, 4, 3], [1, 0, 2, 1, 2, 3], 3), // left
            ([5, 4, 7, 5, 7, 6], [2, 3, 1, 2, 1, 0], 0), // back
            ([0, 6, 7, 0, 1, 6], [2, 1, 0, 2, 3, 1], 5), // bottom
        ];

        let mut vertices = Vec::new();

        for face in faces {
            for j in 0..6 {
                vertices.push(ModelVertex::new(
                    points[face.0[j]],
                    tex_coords[face.1[j]],
                    normals[face.2],
                ));
            }
        }

        let model = Model {
            vertices,
            indices: None,
        };

        Mesh::new(renderer, pipeline, model, texture)
    }
}
