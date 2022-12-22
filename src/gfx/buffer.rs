use crate::gfx::vertex::Vertex;
use crate::Renderer;
use bytemuck::{Pod, Zeroable};
use std::fmt::Debug;
use std::ops::RangeBounds;

pub struct VertexBuffer {
    buffer: wgpu::Buffer,
    len: u32,
}

impl VertexBuffer {
    pub fn new<T>(renderer: &Renderer, vertices: &[T]) -> Self
    where
        T: Vertex + Pod + Zeroable,
    {
        let buffer = renderer.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let len = vertices.len() as u32;

        Self { buffer, len }
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice {
        self.buffer.slice(bounds)
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    len: u32,
}

impl IndexBuffer {
    pub fn new(renderer: &Renderer, indices: &[u16]) -> Self {
        let buffer = renderer.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let len = indices.len() as u32;

        Self { buffer, len }
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice {
        self.buffer.slice(bounds)
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}
