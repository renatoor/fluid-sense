use crate::gfx::vertex::InstanceVertex;
use crate::scene::object::Transform;
use crate::{Pipeline, Plane, Renderer, Scene};
use glam::{EulerRot, Quat, Vec3};
use std::fs;

#[derive(Debug)]
enum Tile {
    Empty,
    Wall,
    Floor,
    User,
    Actuator,
    Sensor,
}

impl Tile {
    pub fn from(c: char) -> Result<Tile, char> {
        match c {
            ' ' => Ok(Tile::Empty),
            '#' => Ok(Tile::Wall),
            '.' => Ok(Tile::Floor),
            '@' => Ok(Tile::User),
            'A' => Ok(Tile::Actuator),
            'S' => Ok(Tile::Sensor),
            _ => Err(c),
        }
    }
}

#[derive(Debug)]
pub struct WorldMap {
    tiles: Vec<Vec<Tile>>,
}

impl WorldMap {
    pub fn from_file(filename: &str) -> Self {
        let contents = fs::read_to_string(filename).expect("Unable to read map file");

        let tiles = contents
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| Tile::from(c).expect("Invalid character"))
                    .collect()
            })
            .collect();

        Self { tiles }
    }

    pub fn build_scene(&self, renderer: &Renderer, pipeline: &wgpu::RenderPipeline) -> Scene {
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
                    Tile::Actuator => {}
                    Tile::Sensor => {}
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

    fn create_floor_instance(x: f32, z: f32) -> InstanceVertex {
        let transform = Transform::new(
            Vec3::ONE,
            Quat::from_euler(EulerRot::XYZ, -90.0f32.to_radians(), 0.0, 0.0),
            Vec3::new(x as f32, 0.0, z as f32),
        );

        InstanceVertex::from_transform(transform)
    }

    fn create_wall_instance(x: f32, z: f32) -> InstanceVertex {
        let transform = Transform::new(
            Vec3::new(1.0, 3.0, 1.0),
            Quat::IDENTITY,
            Vec3::new(x as f32, 0.0, z as f32 - 1.0),
        );

        InstanceVertex::from_transform(transform)
    }
}
