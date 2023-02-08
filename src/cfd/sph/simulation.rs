use crate::cfd::sph::kernel::Kernel;
use crate::{ParticleInstance, Tile, WorldMap};
use glam::Vec3;
use std::collections::LinkedList;
use std::time::Duration;

#[derive(Debug)]
pub struct SimulationParticle {
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    forces: Vec3,
    density: f32,
    density_correction: f32,
    temperature: f32,
}

impl SimulationParticle {
    pub fn new(position: Vec3, velocity: Vec3, temperature: f32) -> Self {
        Self {
            position,
            velocity,
            acceleration: Vec3::ZERO,
            forces: Vec3::ZERO,
            density: 0.0,
            density_correction: 0.0,
            temperature,
        }
    }
}

pub struct ParticleConfig {
    pub radius: f32,
    pub mass: f32,
    pub gas_constant: f32,
    pub rest_density: f32,
    pub thermal_conductivity: f32,
    pub small_positive: f32,
    pub viscosity: f32,
    pub damping_coefficient: f32,
    pub damping_threshold: f32,
    pub radiation_half_life: f32,
    pub buoyancy_coefficient: f32,
    pub buoyancy_direction: Vec3,
    pub gravity: Vec3,
    pub virtual_particle: Vec3,
}

pub struct SPH {
    kernel: Kernel,
    particles: Vec<SimulationParticle>,
    instances: Vec<ParticleInstance>,
    config: ParticleConfig,
}

impl SPH {
    pub fn new(config: ParticleConfig) -> Self {
        let kernel = Kernel::new(config.radius);
        let particles = Vec::new();
        let instances = Vec::new();

        Self {
            kernel,
            particles,
            instances,
            config,
        }
    }

    pub fn add_particle(&mut self, particle: SimulationParticle) {
        let position = particle.position.clone();
        self.particles.push(particle);
        self.instances.push(ParticleInstance {
            position,
            color: Vec3::new(0.0, 0.0, 1.0),
        });
    }

    pub fn remove_particle(&mut self, index: usize) {
        self.particles.remove(index);
        self.instances.remove(index);
    }

    pub fn check_particles(&mut self, world_map: &WorldMap) {
        let mut particle_indices = Vec::new();
        for (i, particle) in self.particles.iter().enumerate() {
            if particle.position.y > 3.0 || particle.position.y < 0.0 {
                particle_indices.push(i);
                continue;
            }

            match world_map.get_tile_in_position(particle.position) {
                Tile::Floor => {}
                _ => particle_indices.push(i),
            }
        }

        for i in particle_indices {
            self.remove_particle(i);
        }
    }

    pub fn get_particle_instances(&self) -> &Vec<ParticleInstance> {
        &self.instances
    }

    pub fn step(&mut self, time_step: f32) {
        self.compute_uncorrected_densities();
        self.compute_densities();
        self.compute_forces();
        self.integrate(time_step);
    }

    fn compute_uncorrected_densities(&mut self) {
        for i in 0..self.particles.len() {
            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            pi.density = self.kernel.w0();

            for pj in before.iter().chain(after.iter()) {
                let diff = pi.position - pj.position;
                let r = diff.length();

                if r > 0.0 && r <= self.config.radius {
                    pi.density += self.config.mass * self.kernel.w(diff);
                }
            }
        }
    }

    fn compute_densities(&mut self) {
        for i in 0..self.particles.len() {
            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            let mut density = Vec3::ZERO;

            for pj in before.iter().chain(after.iter()) {
                let diff = pi.position - pj.position;
                let r = diff.length();

                if r > 0.0 && r <= self.config.radius {
                    density -= self.config.mass / pj.density * self.kernel.poly6_grad_w(diff);
                }
            }

            let v0 = density.length()
                / self
                    .kernel
                    .poly6_grad_w(self.config.virtual_particle)
                    .length();
            pi.density_correction =
                pi.density * (1.0 + v0 * self.kernel.w(self.config.virtual_particle));
        }
    }

    fn compute_pressure(&mut self, density: f32) -> f32 {
        self.config.gas_constant * (density - self.config.rest_density)
    }

    fn compute_forces(&mut self) {
        for i in 0..self.particles.len() {
            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            let mut damping = Vec3::ZERO;
            let mut atmospheric_pressure = Vec3::ZERO;
            let mut pressure = Vec3::ZERO;
            let mut viscosity = Vec3::ZERO;
            let mut temperature = 0.0f32;

            for pj in before.iter().chain(after.iter()) {
                let pressure_i = self.config.gas_constant * (pi.density - self.config.rest_density);
                let pressure_j = self.config.gas_constant * (pj.density - self.config.rest_density);
                let pressure_k =
                    self.config.gas_constant * (pi.density_correction - self.config.rest_density);

                let diff = pi.position - pj.position;
                let r = diff.dot(diff);

                if r > 0.0 && r <= self.config.radius {
                    atmospheric_pressure +=
                        (self.config.mass / pressure_j) * self.kernel.spiky_grad_w(diff);

                    pressure -= (self.config.mass / pj.density)
                        * ((pressure_i + pressure_j) / 2.0)
                        * self.kernel.spiky_grad_w(diff)
                        + (self.config.mass / pi.density_correction)
                            * ((pressure_i + pressure_k) / 2.0)
                            * self.kernel.spiky_grad_w(self.config.virtual_particle);

                    viscosity += self.config.mass * (pj.velocity - pi.velocity) / pj.density
                        * self.kernel.viscosity_laplacian_w(diff);

                    temperature += (self.config.mass / (pressure_i * pressure_j))
                        * self.config.thermal_conductivity
                        * (pi.temperature - pj.temperature)
                        * (diff.dot(self.kernel.spiky_grad_w(diff))
                            / (diff.dot(diff) + self.config.small_positive));
                }
            }

            viscosity *= self.config.viscosity;

            if atmospheric_pressure.length() > self.config.damping_threshold {
                temperature -= pi.temperature / self.config.radiation_half_life;
                damping = -self.config.damping_coefficient * pi.velocity;
            }

            pi.temperature += temperature;

            let buoyancy =
                self.config.buoyancy_coefficient * pi.temperature * self.config.buoyancy_direction;

            pi.forces = (pressure + 1.0 * atmospheric_pressure)
                + viscosity
                + pi.density * (self.config.gravity + buoyancy + damping);
        }
    }

    fn integrate(&mut self, time_step: f32) {
        for (i, particle) in self.particles.iter_mut().enumerate() {
            let prev_acceleration = particle.acceleration;
            let prev_velocity = particle.velocity;

            particle.acceleration = particle.forces / particle.density;
            particle.velocity +=
                (prev_acceleration + particle.acceleration) / 2.0 * time_step * time_step;
            particle.position +=
                prev_velocity * time_step + prev_acceleration / 2.0 * time_step * time_step;

            self.instances[i].position = particle.position;
        }
    }
}
