use crate::gfx::vertex::InstanceVertex;
use crate::scene::object::Transform;
use crate::{Pipeline, Plane, Renderer, Scene};
use glam::{EulerRot, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::fs;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize)]
struct ActuatorConfig {
    label: char,
    height: f32,
    direction: [f32; 3],
    initial_velocity: f32,
    range: [f32; 3],
    fluid_type: String,
}

#[derive(Debug, EnumString)]
pub enum FluidType {
    Gaseous,
    Liquid,
}

#[derive(Debug)]
struct Actuator {
    label: char,
    position: Vec3,
    direction: Vec3,
    initial_velocity: f32,
    range: Vec3,
    fluid_type: FluidType,
}

impl Actuator {
    pub fn new(x: f32, z: f32, config: &ActuatorConfig) -> Self {
        Self {
            label: config.label,
            position: Vec3::new(x, config.height, z),
            direction: Vec3::from(config.direction),
            initial_velocity: config.initial_velocity,
            range: Vec3::from(config.range),
            fluid_type: FluidType::from_str(&config.fluid_type).unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SensorConfig {
    label: char,
    height: f32,
    range: [f32; 3],
}

#[derive(Debug)]
struct Sensor {
    label: char,
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
                            .map(|config| Actuator::new(x, z, config));

                        match actuator {
                            Some(actuator) => {
                                actuators.push(actuator);
                            }
                            None => {
                                println!("Não encontrada configuração para atuador '{}'", c)
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        actuators
    }

    pub fn get_sensors(self) -> Vec<Sensor> {
        let mut sensors = Vec::new();

        for (z, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let (x, z) = (x as f32, z as f32);

                match tile {
                    Tile::Device(c) => {
                        let sensor = self
                            .config
                            .sensors
                            .iter()
                            .filter(|s| s.label == *c)
                            .next()
                            .map(|config| Sensor::new(x, z, config));

                        match sensor {
                            Some(sensor) => {
                                sensors.push(sensor);
                            }
                            None => {
                                println!("Não encontrada configuração para sensor '{}'", c)
                            }
                        }
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
                    _ => tile,
                };
            }
        }

        return &Tile::Empty;
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
