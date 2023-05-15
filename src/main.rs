extern crate core;

use std::time::Duration;

use clap::Parser;
use glam::Vec3;

use winit::event::KeyboardInput;

use crate::app::App;
use crate::cfd::sph::simulation::{SimulationParticle, SPH};
use crate::gfx::buffer::VertexBuffer;
use crate::gfx::camera::controller::FirstPersonController;
use crate::gfx::camera::projection::Perspective;
use crate::gfx::camera::Camera;
use crate::gfx::light::Light;
use crate::gfx::pipeline::Pipeline;
use crate::gfx::renderer::Renderer;
use crate::gfx::texture::DepthTexture;
use crate::scene::object::particle::{Particle, ParticleInstance};
use crate::scene::object::plane::Plane;
use crate::scene::world_map::{Tile, WorldMap};
use crate::scene::Scene;

mod app;
mod cfd;
mod gfx;
mod scene;

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
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config: String,
    #[arg(long, default_value_t = false)]
    headless: bool,
}

impl App for FluidSense {
    fn init(renderer: &mut Renderer) -> Self {
        let args = Args::parse();
        let phong_pipeline = Pipeline::phong(renderer);
        let particle_pipeline = Pipeline::particle(renderer);
        let config = cfd::config::Config::new(&args.config);
        let mut world_map = WorldMap::new(&config);
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
        let sph = SPH::new(&config);
        let particle_instance_buffer = VertexBuffer::new(renderer, sph.get_particle_instances());

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

        self.world_map
            .get_actuators()
            .iter_mut()
            .for_each(|(_, actuator)| match actuator.emit_particle(&dt) {
                None => {}
                Some(particle) => {
                    self.sph.add_particle(particle);
                }
            });

        self.sph.get_particles().iter().for_each(|particle| {
            match self.world_map.get_device_in_position(particle.position) {
                Some(label) => match self.world_map.get_sensor_by_label(&label) {
                    Some(sensor) => sensor.inspect_particle(particle),
                    None => {}
                },
                None => {}
            }
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn render<'a>(&'a mut self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass<'a>) {
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

fn main() {
    let args = Args::parse();

    if args.headless {
        todo!();
    } else {
        pollster::block_on(app::run::<FluidSense>());
    }
}

/*
fn main2() {
    let mut world_map = WorldMap::from_file("./assets/maps/default.txt");
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

    let mut sph = SPH::new(particle_config);

    let mut actuators = world_map.get_actuators();
    let sensors = world_map.get_sensors();

    loop {
        sph.step(0.001);
        sph.check_particles(&world_map);

        let dt = Duration::from_secs_f64(0.016);

        for mut actuator in &mut actuators {
            match actuator.emit_particle(&dt) {
                None => {}
                Some(particle) => {
                    sph.add_particle(particle);
                }
            }
        }

        for particle in sph.get_particles() {
            match world_map.get_device_in_position(particle.position) {
                None => {}
                Some(label) => {
                    let sensors = sensors
                        .iter()
                        .filter(|sensor| sensor.label == label)
                        .collect::<Vec<&Sensor>>();

                    for sensor in sensors {
                        sensor.inspect_particle(particle);
                    }
                }
            }
        }
    }
}
 */
