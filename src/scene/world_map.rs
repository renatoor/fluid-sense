use crate::gfx::vertex::InstanceVertex;
use crate::scene::object::Transform;
use crate::{Renderer, Scene, SimulationParticle};
use std::collections::HashMap;
// use crate::cfd::config::Config;
use crate::cfd::config::{ActuatorConfig, Config, FluidType, ParticleConfig, SensorConfig};

use glam::{EulerRot, Quat, Vec3};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};



use std::time::Duration;


#[derive(Debug)]
struct ActuatorParticle {
    size: f32,
    color: Vec3,
}

impl ActuatorParticle {
    pub fn new(config: &ParticleConfig) -> Self {
        Self {
            size: config.size,
            color: config.color,
        }
    }
}

#[derive(Debug)]
pub struct Actuator {
    rng: ThreadRng,
    position: Vec3,
    direction: Vec3,
    initial_velocity: f32,
    temperature: Option<f32>,
    range: Vec3,
    fluid_type: FluidType,
    interval: f32,
    dt: f32,
    particle: ActuatorParticle,
}

impl Actuator {
    pub fn new(x: f32, z: f32, config: &ActuatorConfig) -> Self {
        Self {
            rng: rand::thread_rng(),
            position: Vec3::new(x, config.height, z),
            direction: config.direction,
            initial_velocity: config.initial_velocity,
            temperature: config.temperature,
            range: config.range,
            fluid_type: config.fluid_type,
            interval: config.interval,
            dt: 0.0,
            particle: ActuatorParticle::new(&config.particle),
        }
    }

    pub fn emit_particle(&mut self, dt: &Duration) -> Option<SimulationParticle> {
        self.dt += dt.as_secs_f32();

        if self.dt < self.interval {
            return None;
        }

        self.dt = 0.0;

        let jitter_x: f32 = self.rng.gen::<f32>() * self.range.x;
        let jitter_y: f32 = self.rng.gen::<f32>() * self.range.y;
        let jitter_z: f32 = self.rng.gen::<f32>() * self.range.z;

        let position = Vec3::new(
            self.position.x + jitter_x,
            self.position.y + jitter_y,
            self.position.z + jitter_z,
        );

        let velocity = self.direction * self.initial_velocity;

        let temperature = match self.temperature {
            None => 25.0,
            Some(temperature) => temperature,
        };

        let particle = SimulationParticle::new(
            position,
            velocity,
            temperature,
            self.fluid_type.clone(),
            self.particle.color,
        );

        Some(particle)
    }
}

#[derive(Debug)]
pub struct Sensor {
    position: Vec3,
    range: Vec3,
    output: Option<String>,
}

impl Sensor {
    pub fn new(x: f32, z: f32, config: &SensorConfig) -> Self {
        Self {
            position: Vec3::new(x, config.height, z),
            range: config.range,
            output: config.output.clone(),
        }
    }

    pub fn inspect_particle(&self, _particle: &SimulationParticle) {
        // println!("Sensor {} detected particle: {:?}", self.label, particle);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceConfig {
    actuators: Vec<ActuatorConfig>,
    sensors: Vec<SensorConfig>,
}

#[derive(Debug)]
pub enum Tile {
    Empty,
    Wall,
    Floor,
    User,
    Device(char),
}

impl Tile {
    pub fn from(c: char) -> Result<Tile, char> {
        match c {
            ' ' => Ok(Tile::Empty),
            '#' => Ok(Tile::Wall),
            '.' => Ok(Tile::Floor),
            '@' => Ok(Tile::User),
            _ => Ok(Tile::Device(c)),
        }
    }
}

#[derive(Debug)]
pub struct WorldMap {
    tiles: Vec<Vec<Tile>>,
    actuators: HashMap<char, Actuator>,
    sensors: HashMap<char, Sensor>,
}

impl WorldMap {
    pub fn new(config: &Config) -> Self {
        let tiles: Vec<Vec<Tile>> = config
            .get_environment()
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| Tile::from(c).expect("Invalid character"))
                    .collect()
            })
            .collect();

        let mut actuators = HashMap::new();
        let mut sensors = HashMap::new();

        tiles
            .iter()
            .enumerate()
            .flat_map(|(z, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(x, tile)| (x as f32, z as f32, tile))
            })
            .for_each(|(x, z, tile)| match tile {
                Tile::Device(c) => {
                    match config.get_actuator_by_label(c) {
                        Some(config) => {
                            actuators.insert(*c, Actuator::new(x + 0.5, z + 0.5, config));
                        }
                        None => {}
                    }

                    match config.get_sensor_by_label(c) {
                        Some(config) => {
                            sensors.insert(*c, Sensor::new(x, z, config));
                        }
                        None => {}
                    }
                }
                _ => {}
            });

        Self {
            tiles,
            actuators,
            sensors,
        }
    }

    pub fn build_scene(&mut self, renderer: &Renderer, pipeline: &wgpu::RenderPipeline) -> Scene {
        let mut user_position = (0.0, 0.0);
        let mut floor_instances = Vec::new();
        let mut wall_instances = Vec::new();

        self.iter_tiles().for_each(|(x, z, tile)| match tile {
            Tile::Empty => {}
            Tile::Wall => {
                let instance = Self::create_wall_instance(x, z);
                wall_instances.push(instance);
            }
            Tile::User => {
                user_position = (x, z);
                let instance = Self::create_floor_instance(x, z);
                floor_instances.push(instance);
            }
            Tile::Floor | Tile::Device(_) => {
                let instance = Self::create_floor_instance(x, z);
                floor_instances.push(instance);
            }
        });

        Scene::new(
            renderer,
            pipeline,
            user_position,
            floor_instances,
            wall_instances,
        )
    }

    fn iter_tiles(&self) -> impl Iterator<Item = (f32, f32, &Tile)> {
        self.tiles.iter().enumerate().flat_map(|(z, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, tile)| (x as f32, z as f32, tile))
        })
    }

    pub fn get_actuators(&mut self) -> &mut HashMap<char, Actuator> {
        &mut self.actuators
    }

    pub fn get_sensor_by_label(&self, label: &char) -> Option<&Sensor> {
        self.sensors.get(label)
    }

    pub fn get_tile_in_position(&self, position: Vec3) -> &Tile {
        let (x, z) = ((position.x) as usize, (position.z) as usize);

        if z <= self.tiles.len() && x <= self.tiles[z].len() {
            let tile = &self.tiles[z][x];

            return match tile {
                Tile::User => &Tile::Floor,
                Tile::Device(_) => &Tile::Floor,
                _ => tile,
            };
        }

        return &Tile::Empty;
    }

    pub fn get_device_in_position(&self, position: Vec3) -> Option<char> {
        let (x, z) = ((position.x) as usize, (position.z) as usize);

        if z <= self.tiles.len() {
            if x <= self.tiles[z].len() {
                let tile = &self.tiles[z][x];

                return match tile {
                    Tile::Device(c) => Some(*c),
                    _ => None,
                };
            }
        }

        return None;
    }

    fn create_floor_instance(x: f32, z: f32) -> InstanceVertex {
        let transform = Transform::new(
            Vec3::ONE,
            Quat::from_euler(EulerRot::XYZ, -90.0f32.to_radians(), 0.0, 0.0),
            Vec3::new(x as f32, 0.0, z as f32 + 1.0),
        );

        InstanceVertex::from_transform(transform)
    }

    fn create_wall_instance(x: f32, z: f32) -> InstanceVertex {
        let transform = Transform::new(
            Vec3::new(1.0, 3.0, 1.0),
            Quat::IDENTITY,
            Vec3::new(x as f32, 0.0, z as f32),
        );

        InstanceVertex::from_transform(transform)
    }
}
