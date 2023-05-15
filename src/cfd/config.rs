use glam::Vec3;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct SimulationConfig {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ParticleConfig {
    pub size: f32,
    pub color: Vec3,
}

#[derive(Serialize, Deserialize, Debug, EnumString, PartialEq, Clone, Copy)]
pub enum FluidType {
    Gaseous,
    Liquid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActuatorConfig {
    pub height: f32,
    pub direction: Vec3,
    pub initial_velocity: f32,
    pub temperature: Option<f32>,
    pub range: Vec3,
    pub fluid_type: FluidType,
    pub interval: f32,
    pub particle: ParticleConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorConfig {
    pub height: f32,
    pub range: Vec3,
    pub output: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    environment: String,
    actuators: HashMap<char, ActuatorConfig>,
    sensors: HashMap<char, SensorConfig>,
    simulation: SimulationConfig,
}

impl Config {
    pub fn new(filename: &String) -> Self {
        let file = std::fs::File::open(filename).expect("Could not open file");
        serde_yaml::from_reader(file).expect("Could not read file")
    }

    pub fn get_environment(&self) -> &String {
        &self.environment
    }

    pub fn get_actuator_by_label(&self, label: &char) -> Option<&ActuatorConfig> {
        self.actuators.get(label)
    }

    pub fn get_sensor_by_label(&self, label: &char) -> Option<&SensorConfig> {
        self.sensors.get(label)
    }

    pub fn get_simulation_config(&self) -> &SimulationConfig {
        &self.simulation
    }
}
