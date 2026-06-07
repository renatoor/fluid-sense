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
    normal: Vec3,
    virtual_volume: f32,
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
            normal: Vec3::ZERO,
            virtual_volume: 0.0,
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

    pub fn time_step(&self) -> f32 {
        self.config.step
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
        self.compute_forces(time_step);
        self.integrate(time_step);
    }

    fn compute_uncorrected_densities(&mut self) {
        for i in 0..self.particles.len() {
            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            pi.density = self.config.mass * self.kernel.w0();
            pi.density_correction = pi.density;
            pi.normal = Vec3::ZERO;
            pi.virtual_volume = 0.0;

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
            if self.particles[i].fluid_type == FluidType::Liquid {
                self.particles[i].density_correction = self.particles[i].density;
                continue;
            }

            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            let mut normal = Vec3::ZERO;

            for pj in before.iter().chain(after.iter()) {
                if pj.fluid_type != FluidType::Gaseous {
                    continue;
                }

                let diff = pi.position - pj.position;
                let r = diff.length();

                if r > 0.0 && r <= self.config.radius {
                    normal += self.config.mass / pj.density * self.kernel.poly6_grad_w(diff);
                }
            }

            let virtual_gradient = self
                .kernel
                .poly6_grad_w(self.config.virtual_particle)
                .length();
            let v0 = if virtual_gradient > 0.0 {
                normal.length() / virtual_gradient
            } else {
                0.0
            };
            pi.normal = normal;
            pi.virtual_volume = v0;
            pi.density_correction =
                pi.density * (1.0 + v0 * self.kernel.w(self.config.virtual_particle));
        }
    }

    fn compute_forces(&mut self, time_step: f32) {
        for i in 0..self.particles.len() {
            let (before, nonbefore) = self.particles.split_at_mut(i);
            let (pi, after) = nonbefore.split_first_mut().unwrap();

            let density_i = pi.density_correction;
            let mut damping = Vec3::ZERO;
            let mut pressure = Vec3::ZERO;
            let mut viscosity = Vec3::ZERO;
            let mut temperature = 0.0f32;
            let mut gas_normal = Vec3::ZERO;

            for pj in before.iter().chain(after.iter()) {
                let diff = pi.position - pj.position;
                let r2 = diff.dot(diff);
                let r = r2.sqrt();

                if r <= 0.0 || r > self.config.radius {
                    continue;
                }

                let density_j = pj.density_correction;
                let pressure_i = self.config.gas_constant * (density_i - self.config.rest_density);
                let pressure_j = self.config.gas_constant * (density_j - self.config.rest_density);
                let gradient = self.kernel.spiky_grad_w(diff);

                pressure -=
                    (self.config.mass / density_j) * ((pressure_i + pressure_j) / 2.0) * gradient;

                viscosity += self.config.mass * (pj.velocity - pi.velocity) / density_j
                    * self.kernel.viscosity_laplacian_w(diff);

                if pi.fluid_type == FluidType::Gaseous && pj.fluid_type == FluidType::Gaseous {
                    gas_normal += self.config.mass / density_j * self.kernel.poly6_grad_w(diff);
                    temperature += (self.config.mass / (density_i * density_j))
                        * self.config.thermal_conductivity
                        * (pi.temperature - pj.temperature)
                        * (diff.dot(gradient) / (r2 + self.config.small_positive));
                }
            }

            viscosity *= self.config.viscosity;

            match pi.fluid_type {
                FluidType::Gaseous => {
                    if gas_normal.length() > 0.0 {
                        pi.normal = gas_normal;
                    }
                    let pressure_i =
                        self.config.gas_constant * (density_i - self.config.rest_density);
                    let pressure_k = pressure_i;
                    let virtual_particle = if pi.normal.length() > 0.0 {
                        pi.normal.normalize() * self.config.virtual_particle.length()
                    } else {
                        self.config.virtual_particle
                    };
                    let virtual_weight = self.kernel.w(virtual_particle);
                    let virtual_volume = if 1.0 + pi.virtual_volume * virtual_weight > 0.0 {
                        pi.virtual_volume / (1.0 + pi.virtual_volume * virtual_weight)
                    } else {
                        0.0
                    };

                    pressure -= virtual_volume
                        * ((pressure_i + pressure_k) / 2.0)
                        * self.kernel.spiky_grad_w(virtual_particle);

                    let atmospheric_pressure = self.config.atmospheric_pressure * pi.normal;

                    let normal_length = pi.normal.length();
                    if normal_length > self.config.damping_threshold {
                        let boundary_factor = if self.config.damping_threshold > 0.0 {
                            ((normal_length - self.config.damping_threshold)
                                / self.config.damping_threshold)
                                .clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        temperature -=
                            boundary_factor * pi.temperature / self.config.radiation_half_life;
                        damping = -boundary_factor * self.config.damping_coefficient * pi.velocity;
                    }

                    pi.temperature += temperature * time_step;

                    let buoyancy = self.config.buoyancy_coefficient
                        * pi.temperature
                        * self.config.buoyancy_direction;

                    pi.forces = (pressure + atmospheric_pressure)
                        + viscosity
                        + density_i * (self.config.gravity + buoyancy + damping);
                }
                FluidType::Liquid => {
                    pi.forces = pressure + viscosity + density_i * self.config.gravity;
                }
            }
        }
    }

    fn integrate(&mut self, time_step: f32) {
        for (i, particle) in self.particles.iter_mut().enumerate() {
            let prev_acceleration = particle.acceleration;
            let prev_velocity = particle.velocity;

            particle.acceleration = particle.forces / particle.density_correction;
            particle.velocity += (prev_acceleration + particle.acceleration) / 2.0 * time_step;
            particle.position +=
                prev_velocity * time_step + prev_acceleration / 2.0 * time_step * time_step;

            self.instances[i].position = particle.position;
        }
    }
}
