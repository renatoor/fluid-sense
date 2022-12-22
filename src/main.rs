extern crate core;

use crate::app::App;
use crate::gfx::buffer::VertexBuffer;
use crate::gfx::camera::controller::FirstPersonController;
use crate::gfx::camera::projection::{Perspective, Projection};
use crate::gfx::camera::Camera;
use crate::gfx::light::Light;
use crate::gfx::pipeline::Pipeline;
use crate::gfx::renderer::Renderer;
use crate::gfx::texture::{DepthTexture, Texture};
use crate::scene::object::particle::{Particle, ParticleInstance};
use crate::scene::object::plane::Plane;
use crate::scene::world_map::WorldMap;
use crate::scene::Scene;
use glam::{Mat4, Vec3, Vec4};
use std::time::Duration;
use winit::event::{ElementState, KeyboardInput, WindowEvent};
use crate::gfx::uniform::Uniform;

mod app;
mod cfd;
mod gfx;
mod scene;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    position: Vec4,
    view_matrix: Mat4,
    view_projection: Mat4,
}

impl CameraUniform {
    pub fn new<T>(camera: Camera<T>) -> Self
    where
        T: Projection,
    {
        let position = Vec4::from((camera.position(), 1.0));
        let view_matrix = camera.view();
        let view_projection = camera.projection() * camera.view();

        Self {
            position,
            view_matrix,
            view_projection,
        }
    }

    pub fn update<T>(&mut self, camera: Camera<T>)
    where
        T: Projection,
    {
        self.position = Vec4::from((camera.position(), 1.0));
        self.view_matrix = camera.view();
        self.view_projection = camera.projection() * camera.view();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: Vec4,
    color: Vec4,
}

impl LightUniform {
    pub fn new(position: Vec3, color: Vec3) -> Self {
        Self {
            position: Vec4::from((position, 1.0)),
            color: Vec4::from((color, 1.0)),
        }
    }
}

struct Uniforms {
    camera: Uniform,
    light: Uniform,
}

struct FluidSense {
    phong_pipeline: wgpu::RenderPipeline,
    particle_pipeline: wgpu::RenderPipeline,
    camera: Camera<Perspective>,
    camera_controller: FirstPersonController,
    scene: Scene,
    light: Light,
    particle: Particle,
    particle_instance_buffer: VertexBuffer,
}

impl App for FluidSense {
    fn init(renderer: &mut Renderer) -> Self {
        let depth_texture = DepthTexture::new(renderer);

        renderer.set_depth_texture(depth_texture);

        let phong_pipeline = Pipeline::phong(renderer);
        let particle_pipeline = Pipeline::particle(renderer);

        let world_map = WorldMap::from_file("./assets/maps/default.txt");

        let scene = world_map.build_scene(renderer, &phong_pipeline);
        let (x, z) = scene.user_position();

        let projection = Perspective::new(45.0, renderer.get_aspect_ratio(), 0.1, 1000.0);
        let camera = Camera::new(
            &renderer,
            &phong_pipeline,
            Vec3::new(x, 1.65, z),
            projection,
        );
        let camera_controller = FirstPersonController::new(0.0, 90.0, 4.0, 0.1);

        let light = Light::new(&renderer, &phong_pipeline, camera.position(), Vec3::ONE);

        let particle = Particle::new(renderer);
        let particle_instances = [
            ParticleInstance {
                position: Vec3 {
                    x: 3.0,
                    y: 1.0,
                    z: 3.0,
                },
                color: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
            ParticleInstance {
                position: Vec3 {
                    x: 2.0,
                    y: 1.5,
                    z: 2.0,
                },
                color: Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        ];
        let particle_instance_buffer = VertexBuffer::new(renderer, &particle_instances);

        Self {
            phong_pipeline,
            particle_pipeline,
            camera,
            camera_controller,
            scene,
            light,
            particle,
            particle_instance_buffer,
        }
    }

    fn keyboard_input(&mut self, input: KeyboardInput) {
        self.camera_controller.keyboard_input(input);
    }

    fn mouse_movement(&mut self, dx: f32, dy: f32) {
        self.camera_controller.mouse_movement(dx, dy);
    }

    fn update(&mut self, dt: Duration) {
        self.camera_controller.update(&mut self.camera, dt);
        self.light.set_position(self.camera.position());
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn render(&self, renderer: &Renderer) -> Result<(), wgpu::SurfaceError> {
        renderer.render(&self.phong_pipeline, &self.camera, &self.scene, &self.light)?;
        Ok(())
    }

    fn render2<'a>(&'a self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.phong_pipeline);
        self.camera.update(&renderer, render_pass);
        self.light.update(&renderer, render_pass);
        self.scene.draw_mesh(render_pass);
        render_pass.set_pipeline(&self.particle_pipeline);
        self.particle
            .draw_instanced(render_pass, &self.particle_instance_buffer);
    }
}

#[tokio::main]
async fn main() {
    app::run::<FluidSense>().await;
}
