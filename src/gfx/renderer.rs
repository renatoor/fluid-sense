use crate::{App, DepthTexture};
use bytemuck::{Pod, Zeroable};

use std::sync::Arc;
use wgpu::util::DeviceExt;

use winit::dpi::PhysicalSize;
use winit::window::Window;

pub enum RenderStatus {
    Rendered,
    Reconfigure,
    Skip,
    Failed(&'static str),
}

pub struct Renderer {
    size: PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    clear_color: wgpu::Color,
    depth_texture: Option<DepthTexture>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let width = size.width.max(1);
        let height = size.height.max(1);
        let mut config = surface.get_default_config(&adapter, width, height).unwrap();
        config.present_mode = wgpu::PresentMode::Fifo;

        let clear_color = wgpu::Color::WHITE;

        surface.configure(&device, &config);

        let mut renderer = Renderer {
            size,
            surface,
            device,
            queue,
            config,
            clear_color,
            depth_texture: None,
        };

        let depth_texture = DepthTexture::new(&renderer);
        renderer.set_depth_texture(depth_texture);

        renderer
    }

    #[allow(dead_code)]
    pub fn get_texture_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    #[allow(dead_code)]
    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.clear_color = clear_color;
    }

    #[allow(dead_code)]
    pub fn set_depth_texture(&mut self, depth_texture: DepthTexture) {
        self.depth_texture = Some(depth_texture);
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height.max(1) as f32
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.configure_surface();
        self.depth_texture = match &self.depth_texture {
            Some(_) => Some(DepthTexture::new(&self)),
            None => None,
        };
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn configure_surface(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn create_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor,
    ) -> wgpu::Buffer {
        self.device.create_buffer_init(descriptor)
    }

    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(desc)
    }

    pub fn create_bind_group(
        &self,
        layout: &wgpu::BindGroupLayout,
        entries: &[wgpu::BindGroupEntry],
    ) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries,
        })
    }

    pub fn write_buffer<T: Pod + Zeroable>(&self, buffer: &wgpu::Buffer, data: T) {
        self.queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn create_shader_module(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.device.create_shader_module(desc)
    }

    pub fn render<T>(&self, app: &mut T) -> RenderStatus
    where
        T: App,
    {
        let output = self.surface.get_current_texture();
        let output = match output {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return RenderStatus::Skip;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return RenderStatus::Reconfigure;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return RenderStatus::Failed("surface validation error");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let depth_stencil_attachment = match &self.depth_texture {
                Some(depth_texture) => Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture.get_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                None => None,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            app.render(&self, &mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        RenderStatus::Rendered
    }
}
