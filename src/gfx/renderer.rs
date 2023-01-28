use crate::gfx::camera::projection::Projection;
use crate::gfx::pipeline::Pipeline;
use crate::gfx::texture::Texture;
use crate::{App, Camera, DepthTexture, Light, Scene};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use wgpu::BindingResource;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    size: PhysicalSize<u32>,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    clear_color: wgpu::Color,
    depth_texture: Option<DepthTexture>,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let clear_color = wgpu::Color::WHITE;

        surface.configure(&device, &config);

        Self {
            size,
            surface,
            device,
            queue,
            config,
            clear_color,
            depth_texture: None,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
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
        self.size.width as f32 / self.size.height as f32
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
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

    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor,
    ) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(desc)
    }

    pub fn render<T: Projection>(
        &self,
        pipeline: &wgpu::RenderPipeline,
        camera: &Camera<T>,
        scene: &Scene,
        light: &Light,
    ) -> Result<(), wgpu::SurfaceError>
    where
        T: Projection,
    {
        let output = self.surface.get_current_texture()?;

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
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                None => None,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment,
            });

            render_pass.set_pipeline(&pipeline);
            camera.update(&self, &mut render_pass);
            light.update(&self, &mut render_pass);
            scene.draw_mesh(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn render_fn<T>(&self, app: &mut T) -> Result<(), wgpu::SurfaceError>
    where
        T: App,
    {
        let output = self.surface.get_current_texture()?;

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
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                None => None,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment,
            });

            app.render2(&self, &mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
