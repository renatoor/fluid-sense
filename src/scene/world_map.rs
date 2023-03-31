use crate::gfx::vertex::InstanceVertex;
use crate::scene::object::Transform;
use crate::{ParticleConfig, Pipeline, Plane, Renderer, Scene, SimulationParticle};
use glam::{EulerRot, Quat, Vec3};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::process::parent_id;
use std::str::FromStr;
use std::time::Duration;
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActuatorParticleConfig {
    size: f32,
    color: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActuatorConfig {
    label: char,
    height: f32,
    direction: [f32; 3],
    initial_velocity: f32,
    temperature: Option<f32>,
    range: [f32; 3],
    fluid_type: String,
    interval: f32,
    particle: ActuatorParticleConfig,
}

#[derive(Debug, EnumString, Clone, PartialEq)]
pub enum FluidType {
    Gaseous,
    Liquid,
}

#[derive(Debug)]
pub struct ActuatorParticle {
    size: f32,
    color: Vec3,
}

impl ActuatorParticle {
    pub fn new(config: &ActuatorParticleConfig) -> Self {
        Self {
            size: config.size,
            color: Vec3::from(config.color),
        }
    }
}

#[derive(Debug)]
pub struct Actuator {
    rng: ThreadRng,
    label: char,
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
            label: config.label,
            position: Vec3::new(x, config.height, z),
            direction: Vec3::from(config.direction),
            initial_velocity: config.initial_velocity,
            temperature: config.temperature,
            range: Vec3::from(config.range),
            fluid_type: FluidType::from_str(&config.fluid_type).unwrap(),
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

        let particle = SimulationParticle::new(position, velocity, temperature, self.fluid_type.clone(), self.particle.color);

        Some(particle)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorConfig {
    label: char,
    height: f32,
    range: [f32; 3],
}

#[derive(Debug)]
pub struct Sensor {
    pub(crate) label: char,
    position: Vec3,
    range: Vec3,
}

impl Sensor {
    pub fn new(x: f32, z: f32, config: &SensorConfig) -> Self {
        Self {
            label: config.label,
            position: Vec3::new(x, config.height, z),
            range: Vec3::from(config.range),
        }
    }

    pub fn inspect_particle(&self, particle: &SimulationParticle) {
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
    config: DeviceConfig,
}

impl WorldMap {
    pub fn from_file(filename: &str) -> Self {
        let contents = fs::read_to_string(filename).expect("Unable to read map file");
        let split_contents: Vec<&str> = contents.split("---").collect();
        let map = split_contents.first().unwrap();
        let map_config = split_contents.last().unwrap();
        let config: DeviceConfig =
            serde_json::from_str(map_config).expect("Unable to parse map file json");

        let tiles = map
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| Tile::from(c).expect("Invalid character"))
                    .collect()
            })
            .collect();

        Self { tiles, config }
    }

    pub fn build_scene(&mut self, renderer: &Renderer, pipeline: &wgpu::RenderPipeline) -> Scene {
        let mut user_position = (0.0, 0.0);
        let mut floor_instances = Vec::new();
        let mut wall_instances = Vec::new();

        for (z, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let (x, z) = (x as f32, z as f32);

                match tile {
                    Tile::Empty => {}
                    Tile::Wall => {
                        let instance = Self::create_wall_instance(x, z);
                        wall_instances.push(instance);
                    }
                    Tile::Floor => {
                        let instance = Self::create_floor_instance(x, z);
                        floor_instances.push(instance);
                    }
                    Tile::User => {
                        user_position = (x, z);
                        let instance = Self::create_floor_instance(x, z);
                        floor_instances.push(instance);
                    }
                    Tile::Device(c) => {
                        let instance = Self::create_floor_instance(x, z);
                        floor_instances.push(instance);
                    }
                }
            }
        }

        Scene::new(
            renderer,
            pipeline,
            user_position,
            floor_instances,
            wall_instances,
        )
    }

    pub fn get_actuators(&self) -> Vec<Actuator> {
        let mut actuators = Vec::new();

        for (z, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let (x, z) = (x as f32, z as f32);

                match tile {
                    Tile::Device(c) => {
                        let actuator = self
                            .config
                            .actuators
                            .iter()
                            .filter(|config| config.label == *c)
                            .next()
                            .map(|config| Actuator::new(x + 0.5, z + 0.5, config));

                        match actuator {
                            Some(actuator) => {
                                actuators.push(actuator);
                            }
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        actuators
    }

    pub fn get_sensors(&self) -> Vec<Sensor> {
        let mut sensors = Vec::new();

        for (z, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let (x, z) = (x as f32, z as f32);

                match tile {
                    Tile::Device(c) => {
                        let sensors_by_label = self
                            .config
                            .sensors
                            .iter()
                            .filter(|s| s.label == *c)
                            .map(|config| Sensor::new(x, z, config));

                        sensors.extend(sensors_by_label);
                        println!("sensors from {}: {:?}", c, sensors);
                        // match sensor {
                        //     Some(sensor) => {
                        //         sensors.push(sensor);
                        //     }
                        //     None => {}
                        // }
                    }
                    _ => {}
                }
            }
        }

        sensors
    }

    pub fn get_tile_in_position(&self, position: Vec3) -> &Tile {
        let (x, z) = ((position.x) as usize, (position.z) as usize);

        if z <= self.tiles.len() {
            if x <= self.tiles[z].len() {
                let tile = &self.tiles[z][x];

                return match tile {
                    Tile::User => &Tile::Floor,
                    Tile::Device(_) => &Tile::Floor,
                    _ => tile,
                };
            }
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
