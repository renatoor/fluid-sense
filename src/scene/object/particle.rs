use crate::gfx::buffer::VertexBuffer;
use crate::gfx::vertex::Vertex;
use crate::Renderer;
use glam::Vec3;

pub struct Particle {
    vertex_buffer: VertexBuffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVertex {
    position: Vec3,
}

impl Vertex for ParticleVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleInstance {
    pub position: Vec3,
    pub color: Vec3,
}

impl Vertex for ParticleInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl Particle {
    const VERTICES: [ParticleVertex; 6] = [
        ParticleVertex {
            position: Vec3 {
                x: -1.0,
                y: -1.0,
                z: 0.0,
            },
        },
        ParticleVertex {
            position: Vec3 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
            },
        },
        ParticleVertex {
            position: Vec3 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
        },
        ParticleVertex {
            position: Vec3 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
        },
        ParticleVertex {
            position: Vec3 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
            },
        },
        ParticleVertex {
            position: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
        },
    ];

    //const VERTICES: [f32; 12] = [
    //    -0.5, -0.5, 0.0,
    //    0.5, -0.5, 0.0,
    //    -0.5, 0.5, 0.0,
    //    0.5, 0.5, 0.0,
    //];

    pub fn new(renderer: &Renderer) -> Self {
        let vertex_buffer = VertexBuffer::new(renderer, &Self::VERTICES);

        Self { vertex_buffer }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_buffer.len(), 0..1);
    }

    pub fn draw_instanced<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_buffer: &'a VertexBuffer,
    ) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        //render_pass.draw_indexed(0..self.index_buffer.len(), 0, 0..instance_buffer.len());
        render_pass.draw(0..self.vertex_buffer.len(), 0..instance_buffer.len());
    }
}
