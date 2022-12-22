use crate::gfx::texture::Texture;
use crate::gfx::vertex::{ColoredVertex, InstanceVertex, ModelVertex, Vertex};
use crate::Renderer;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::ops::Deref;

pub struct Pipeline {
    pub camera_layout: wgpu::BindGroupLayout,
    pub texture_layout: wgpu::BindGroupLayout,
    pub material_layout: wgpu::BindGroupLayout,
    pub light_layout: wgpu::BindGroupLayout,
    //pub light_bind_group_layout: wgpu::BindGroupLayout,
    //pub material_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

pub struct Pipeline2 {
    render_pipeline: wgpu::RenderPipeline,
    layouts: HashMap<String, wgpu::BindGroupLayout>,
}

impl Pipeline2 {
    pub fn new(
        render_pipeline: wgpu::RenderPipeline,
        layouts: HashMap<String, wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            render_pipeline,
            layouts,
        }
    }

    pub fn get_layout(&self, label: &str) -> Option<&wgpu::BindGroupLayout> {
        self.layouts.get(label)
    }
}

pub struct PipelineBuilder<'a> {
    renderer: &'a Renderer,
    bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
    vertex: Option<wgpu::VertexState<'a>>,
    fragment: Option<wgpu::FragmentState<'a>>,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: Option<wgpu::MultisampleState>,
    multiview: Option<NonZeroU32>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        Self {
            renderer,
            bind_group_layouts: HashMap::new(),
            vertex: None,
            fragment: None,
            depth_stencil: None,
            multisample: None,
            multiview: None,
        }
    }

    pub fn add_bind_group_layout(&mut self, desc: &wgpu::BindGroupLayoutDescriptor) -> &mut Self {
        let label = desc.label.expect("Bind group layouts must have labels");
        let bind_group_layout = self.renderer.create_bind_group_layout(desc);

        self.bind_group_layouts
            .insert(label.to_string(), bind_group_layout);
        self
    }

    pub fn add_bind_group_layouts(
        &mut self,
        layouts: &[&wgpu::BindGroupLayoutDescriptor],
    ) -> &mut Self {
        for desc in layouts {
            self.add_bind_group_layout(desc);
        }

        self
    }

    pub fn vertex(&mut self, state: wgpu::VertexState) -> &mut Self {
        self.vertex = Some(state);
        self
    }

    pub fn fragment(&mut self, state: wgpu::FragmentState) -> &mut Self {
        self.fragment = Some(state);
        self
    }

    pub fn depth_stencil(&mut self, state: wgpu::DepthStencilState) -> &mut Self {
        self.depth_stencil = Some(state);
        self
    }

    pub fn multisample(&mut self, state: wgpu::MultisampleState) -> &mut Self {
        self.multisample = Some(state);
        self
    }

    pub fn multiview(&mut self, multiview: NonZeroU32) -> &mut Self {
        self.multiview = Some(multiview);
        self
    }

    pub fn build(&mut self) -> Pipeline2 {
        let pipeline_layout =
            self.renderer
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &self.bind_group_layouts.iter().collect(),
                    push_constant_ranges: &[],
                });

        let vertex = self.vertex.expect("Não tem vertex");
        let multisample = self.multisample.expect("Não tem multisample");

        let render_pipeline =
            self.renderer
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex,
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: self.depth_stencil.clone(),
                    multisample,
                    fragment: self.fragment.clone(),
                    multiview: self.multiview.clone(),
                });

        let bind_group_layouts = self.bind_group_layouts.drain().collect();

        Pipeline2::new(render_pipeline, bind_group_layouts)
    }
}

/*
impl<'a> PipelineBuilder<'a> {
pub fn new(renderer: &'a Renderer) -> Self {
    Self {
        renderer,
        shader_module: None,
        bind_group_layouts: HashMap::new(),
        vertex_entry_point: None,
        vertex_buffer_layouts: Vec::new(),
        fragment_entry_point: None,
    }
}

pub fn shader(&mut self, shader_module: wgpu::ShaderModule) -> &mut Self {
    self.shader_module = Some(shader_module);
    self
}

pub fn add_bind_group_layout(
    &mut self,
    label: String,
    entries: &[wgpu::BindGroupLayoutEntry],
) -> &mut Self {
    let desc = wgpu::BindGroupLayoutDescriptor {
        label: Some(label.as_str()),
        entries,
    };

    let bind_group_layout = self.renderer.create_bind_group_layout(&desc);

    self.bind_group_layouts.insert(label, &bind_group_layout);
    self
}

pub fn vertex_entry_point(&mut self, vertex_entry_point: String) -> &mut Self {
    self.vertex_entry_point = Some(vertex_entry_point);
    self
}

pub fn add_vertex_buffer_layout(
    &mut self,
    vertex_buffer_layout: wgpu::VertexBufferLayout<'a>,
) -> &mut Self {
    self.vertex_buffer_layouts.push(vertex_buffer_layout);
    self
}

pub fn fragment_entry_point(&mut self, fragment_entry_point: String) -> &mut Self {
    self.fragment_entry_point = Some(fragment_entry_point);
    self
}

pub fn build(&mut self) -> Pipeline2 {
    let shader_module = self.shader_module.as_ref().expect("No shader module");

    let pipeline_layout =
        self.renderer
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &self.bind_group_layouts.values().cloned().collect(),
                push_constant_ranges: &[],
            });

    let vertex_entry_point = self
        .vertex_entry_point
        .as_ref()
        .expect("Must provide vertex shader entry point");

    let vertex = wgpu::VertexState {
        module: &shader_module,
        entry_point: vertex_entry_point.as_str(),
        buffers: &self.vertex_buffer_layouts,
    };

    let targets = &[Some(wgpu::ColorTargetState {
        format: self.renderer.get_texture_format(),
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let fragment = match &self.fragment_entry_point {
        Some(entry_point) => Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: entry_point.as_str(),
            targets,
        }),
        None => None,
    };

    let render_pipeline =
        self.renderer
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex,
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment,
                multiview: None,
            });

    Pipeline2 {
        render_pipeline,
        bind_group_layouts: self.bind_group_layouts.clone(),
    }
}

}
 */

impl Pipeline {
    /*
    pub fn builder(renderer: &Renderer) -> PipelineBuilder {
        PipelineBuilder::new(renderer)
    }
     */

    pub fn default(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            //"/assets/shaders/default.wgsl"
            "/assets/shaders/phong.wgsl"
        )));

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
        });

        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
        });

        let light_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            //bind_group_layouts: &[&camera_layout],
            bind_group_layouts: &[&camera_layout, &light_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), InstanceVertex::desc()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            camera_layout,
            texture_layout,
            material_layout,
            light_layout,
            render_pipeline,
        }
    }
}
