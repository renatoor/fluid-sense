use crate::gfx::pipeline::builder::PipelineBuilder;
use crate::gfx::pipeline::Pipeline;
use crate::gfx::texture::Texture;
use crate::gfx::vertex::{InstanceVertex, ModelVertex, Vertex};
use crate::Renderer;

pub fn create_pipeline(renderer: &Renderer) -> wgpu::RenderPipeline {
    let shader = renderer.create_shader_module(wgpu::include_wgsl!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/shaders/phong.wgsl"
    )));

    let layouts = [
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        },
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("Light"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        },
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        },
    ];

    let vertex = wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[ModelVertex::desc(), InstanceVertex::desc()],
    };

    let fragment = wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
            format: renderer.get_texture_format(),
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
    };

    let depth_stencil = wgpu::DepthStencilState {
        format: Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    };

    let multisample = wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    };

    PipelineBuilder::new(renderer)
        .add_bind_group_layouts(&layouts)
        .vertex(vertex)
        .fragment(fragment)
        .depth_stencil(depth_stencil)
        .multisample(multisample)
        .build()
}
