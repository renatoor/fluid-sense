extern crate core;

use crate::app::App;
use crate::cfd::sph::simulation::{ParticleConfig, SimulationParticle, SPH};
use crate::gfx::buffer::VertexBuffer;
use crate::gfx::camera::controller::FirstPersonController;
use crate::gfx::camera::projection::{Perspective, Projection};
use crate::gfx::camera::Camera;
use crate::gfx::light::Light;
use crate::gfx::pipeline::Pipeline;
use crate::gfx::renderer::Renderer;
use crate::gfx::texture::{DepthTexture, Texture};
use crate::gfx::uniform::Uniform;
use crate::scene::object::particle::{Particle, ParticleInstance};
use crate::scene::object::plane::Plane;
use crate::scene::world_map::{Actuator, Sensor, Tile, WorldMap};
use crate::scene::Scene;
use glam::{Mat4, Vec3, Vec4};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;
use winit::event::{ElementState, KeyboardInput, WindowEvent};

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

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    radius: f32,
    mass: f32,
    gas_constant: f32,
    rest_density: f32,
    thermal_conductivity: f32,
    small_positive: f32,
    viscosity: f32,
    damping_coefficient: f32,
    damping_threshold: f32,
    radiation_half_life: f32,
    buoyancy_coefficient: f32,
    buoyancy_direction: [f32; 3],
    gravity: [f32; 3],
    virtual_particle: [f32; 3],
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
    world_map: WorldMap,
    scene: Scene,
    light: Light,
    particle: Particle,
    particle_instance_buffer: VertexBuffer,
    sph: SPH,
    rng: ThreadRng,
    actuators: Vec<Actuator>,
    sensors: Vec<Sensor>,
}

impl App for FluidSense {
    fn init(renderer: &mut Renderer) -> Self {
        let depth_texture = DepthTexture::new(renderer);

        renderer.set_depth_texture(depth_texture);

        let phong_pipeline = Pipeline::phong(renderer);
        let particle_pipeline = Pipeline::particle(renderer);

        let mut world_map = WorldMap::from_file("./assets/maps/default.txt");

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

        let rng = rand::thread_rng();

        let config_file = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/config.json");

        let config_str = fs::read_to_string(config_file).expect("Config file not found");

        let config: Config = serde_json::from_str(&config_str).expect("Unable to parse json");

        let particle_config = ParticleConfig {
            radius: config.radius,
            mass: config.mass,
            gas_constant: config.gas_constant,
            rest_density: config.rest_density,
            thermal_conductivity: config.thermal_conductivity,
            small_positive: config.small_positive,
            viscosity: config.viscosity,
            damping_coefficient: config.damping_coefficient,
            damping_threshold: config.damping_threshold,
            radiation_half_life: config.radiation_half_life,
            buoyancy_coefficient: config.buoyancy_coefficient,
            buoyancy_direction: Vec3::from(config.buoyancy_direction),
            gravity: Vec3::from(config.gravity),
            virtual_particle: Vec3::from(config.virtual_particle),
        };

        let sph = SPH::new(particle_config);

        let particle_instance_buffer = VertexBuffer::new(renderer, sph.get_particle_instances());

        let actuators = world_map.get_actuators();
        let sensors = world_map.get_sensors();

        Self {
            phong_pipeline,
            particle_pipeline,
            camera,
            camera_controller,
            scene,
            world_map,
            light,
            particle,
            particle_instance_buffer,
            sph,
            rng,
            actuators,
            sensors,
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
        self.sph.step(0.001);
        self.sph.check_particles(&self.world_map);

        for mut actuator in &mut self.actuators {
            match actuator.emit_particle(&dt) {
                None => {}
                Some(particle) => {
                    self.sph.add_particle(particle);
                }
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn render(&self, renderer: &Renderer) -> Result<(), wgpu::SurfaceError> {
        renderer.render(&self.phong_pipeline, &self.camera, &self.scene, &self.light)?;
        Ok(())
    }

    fn render2<'a>(&'a mut self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.phong_pipeline);
        self.camera.update(&renderer, render_pass);
        self.light.update(&renderer, render_pass);
        self.scene.draw_mesh(render_pass);
        render_pass.set_pipeline(&self.particle_pipeline);
        self.particle_instance_buffer
            .update(renderer, self.sph.get_particle_instances());
        self.particle
            .draw_instanced(render_pass, &self.particle_instance_buffer);
    }
}

#[tokio::main]
async fn main() {
    app::run::<FluidSense>().await;
}
