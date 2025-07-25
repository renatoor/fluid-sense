use glam::Vec3;

use crate::cfd::config::{Config, FluidType, SimulationConfig};
use crate::cfd::sph::kernel::Kernel;
use crate::{ParticleInstance, Tile, WorldMap};

#[derive(Debug)]
pub struct SimulationParticle {
    pub position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    forces: Vec3,
    density: f32,
    density_correction: f32,
    temperature: f32,
    fluid_type: FluidType,
    size: f32,
    color: Vec3,
}

impl SimulationParticle {
    pub fn new(
        position: Vec3,
        velocity: Vec3,
        temperature: f32,
        fluid_type: FluidType,
        size: f32,
        color: Vec3,
    ) -> Self {
        Self {
            position,
            velocity,
            acceleration: Vec3::ZERO,
            forces: Vec3::ZERO,
            density: 0.0,
            density_correction: 0.0,
            temperature,
            fluid_type,
            size,
            color,
        }
    }
}

pub struct SPH {
    kernel: Kernel,
    particles: Vec<SimulationParticle>,
    instances: Vec<ParticleInstance>,
    config: SimulationConfig,
}

impl SPH {
    pub fn new(config: &Config) -> Self {
        let config = config.get_simulation_config().clone();
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
        let size = particle.size;
        let color = particle.color.clone();
        self.particles.push(particle);
        self.instances.push(ParticleInstance {
            position,
            size,
            color,
        });
    }

    pub fn remove_particle(&mut self, index: usize) {
        self.particles.remove(index);
        self.instances.remove(index);
    }

    pub fn get_particles(&self) -> &Vec<SimulationParticle> {
        &self.particles
    }

    pub fn check_particles(&mut self, world_map: &WorldMap) {
        self.particles
            .iter()
            .enumerate()
            .filter(
                |(_, particle)| match world_map.get_tile_in_position(particle.position) {
                    Tile::Floor => particle.position.y > 3.0 || particle.position.y < 0.0,
                    _ => true,
                },
            )
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>()
            .iter()
            .rev()
            .for_each(|idx| self.remove_particle(*idx));
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
                if pi.fluid_type != pj.fluid_type {
                    continue;
                }

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
            if self.particles[i].fluid_type == FluidType::Liquid {
                continue;
            }

            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            let mut density = Vec3::ZERO;

            for pj in before.iter().chain(after.iter()) {
                if pj.fluid_type == FluidType::Liquid {
                    continue;
                }

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
                if pi.fluid_type != pj.fluid_type {
                    continue;
                }

                let pressure_i = self.config.gas_constant * (pi.density - self.config.rest_density);
                let pressure_j = self.config.gas_constant * (pj.density - self.config.rest_density);
                let pressure_k =
                    self.config.gas_constant * (pi.density_correction - self.config.rest_density);

                let diff = pi.position - pj.position;
                let r = diff.dot(diff);

                if r > 0.0 && r <= self.config.radius {
                    match pi.fluid_type {
                        FluidType::Gaseous => {
                            atmospheric_pressure +=
                                (self.config.mass / pressure_j) * self.kernel.spiky_grad_w(diff);

                            pressure -= (self.config.mass / pj.density)
                                * ((pressure_i + pressure_j) / 2.0)
                                * self.kernel.spiky_grad_w(diff)
                                + (self.config.mass / pi.density_correction)
                                    * ((pressure_i + pressure_k) / 2.0)
                                    * self.kernel.spiky_grad_w(self.config.virtual_particle);

                            viscosity += self.config.mass * (pj.velocity - pi.velocity)
                                / pj.density
                                * self.kernel.viscosity_laplacian_w(diff);

                            temperature += (self.config.mass / (pressure_i * pressure_j))
                                * self.config.thermal_conductivity
                                * (pi.temperature - pj.temperature)
                                * (diff.dot(self.kernel.spiky_grad_w(diff))
                                    / (diff.dot(diff) + self.config.small_positive));
                        }
                        FluidType::Liquid => {
                            pressure -= (self.config.mass / pj.density)
                                * ((pressure_i + pressure_j) / 2.0)
                                * self.kernel.spiky_grad_w(diff);

                            viscosity += self.config.mass * (pj.velocity - pi.velocity)
                                / pj.density
                                * self.kernel.viscosity_laplacian_w(diff);
                        }
                    }
                }
            }

            viscosity *= self.config.viscosity;

            match pi.fluid_type {
                FluidType::Gaseous => {
                    if atmospheric_pressure.length() > self.config.damping_threshold {
                        temperature -= pi.temperature / self.config.radiation_half_life;
                        damping = -self.config.damping_coefficient * pi.velocity;
                    }

                    pi.temperature += temperature;

                    let buoyancy = self.config.buoyancy_coefficient
                        * pi.temperature
                        * self.config.buoyancy_direction;

                    pi.forces = (pressure + 1.0 * atmospheric_pressure)
                        + viscosity
                        + pi.density * (self.config.gravity + buoyancy + damping);
                }
                FluidType::Liquid => {
                    pi.forces = pressure + viscosity + pi.density * self.config.gravity;
                }
            }
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
