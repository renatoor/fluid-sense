use crate::gfx::pipeline::Pipeline;
use crate::Renderer;
use std::num::NonZeroU32;

pub struct PipelineBuilder<'a> {
    renderer: &'a Renderer,
    layouts: Vec<(u32, wgpu::BindGroupLayout, wgpu::BindingResource<'a>)>,
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
            layouts: Vec::new(),
            vertex: None,
            fragment: None,
            depth_stencil: None,
            multisample: None,
            multiview: None,
        }
    }

    pub fn add_bind_group_layout(&mut self, index: u32, desc: &wgpu::BindGroupLayoutDescriptor, entries: wgpu::BindingResource) -> &mut Self {
        let label = desc.label.expect("Bind group layouts must have labels");
        let bind_group_layout = self.renderer.create_bind_group_layout(desc);

        self.layouts.push((index, bind_group_layout, gesource));

        self
    }

    pub fn vertex(&'a mut self, state: wgpu::VertexState<'a>) -> &mut Self {
        self.vertex = Some(state);
        self
    }

    pub fn fragment(&mut self, state: wgpu::FragmentState<'a>) -> &mut Self {
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

    pub fn build(&mut self) -> wgpu::RenderPipeline {
        self.layouts.sort_by(|a, b| a.0.cmp(&b.0));
        let layouts: Vec<&wgpu::BindGroupLayout> = self.layouts.iter().map(|layout| &layout.1).collect();
        let pipeline_layout =
            self.renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

        let vertex = self.vertex.clone().expect("Não tem vertex");
        let multisample = self.multisample.expect("Não tem multisample");

        let pipeline = self.renderer
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

        let bind_groups: Vec<u32, wgpu::BindGroup> = self.layouts.iter().map(|(index, _, resource)| {
            let layout = pipeline.get_bind_group_layout(*index);
            let entries = [wgpu::BindGroupEntry {
                binding: 0,
                resource: resource,
            }];
            let bind_group = self.renderer.create_bind_group(&layout, &entries);

            (index, bind_group)
        }).collect();

        pipeline
    }
}
