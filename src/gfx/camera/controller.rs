use crate::gfx::camera::projection::Projection;
use crate::{Camera};
use glam::Vec3;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct FirstPersonController {
    pitch: f32,
    yaw: f32,
    movement: (f32, f32, f32, f32, f32, f32),
    rotation: (f32, f32),
    speed: f32,
    sensitivity: f32,
}

impl FirstPersonController {
    pub fn new(pitch: f32, yaw: f32, speed: f32, sensitivity: f32) -> Self {
        Self {
            pitch: pitch.to_radians(),
            yaw: yaw.to_radians(),
            movement: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            rotation: (0.0, 0.0),
            speed,
            sensitivity,
        }
    }

    pub fn keyboard_input(&mut self, input: KeyboardInput) {
        match input {
            KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(key),
                ..
            } => match key {
                VirtualKeyCode::W => self.movement.0 = 1.0,
                VirtualKeyCode::A => self.movement.1 = 1.0,
                VirtualKeyCode::S => self.movement.2 = 1.0,
                VirtualKeyCode::D => self.movement.3 = 1.0,
                VirtualKeyCode::LShift => self.movement.4 = 1.0,
                VirtualKeyCode::Space => self.movement.5 = 1.0,
                _ => {}
            },
            KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(key),
                ..
            } => match key {
                VirtualKeyCode::W => self.movement.0 = 0.0,
                VirtualKeyCode::A => self.movement.1 = 0.0,
                VirtualKeyCode::S => self.movement.2 = 0.0,
                VirtualKeyCode::D => self.movement.3 = 0.0,
                VirtualKeyCode::LShift => self.movement.4 = 0.0,
                VirtualKeyCode::Space => self.movement.5 = 0.0,
                _ => {}
            },
            _ => {}
        }
    }

    pub fn mouse_movement(&mut self, dx: f32, dy: f32) {
        self.rotation.0 += dx;
        self.rotation.1 += dy;
    }

    pub fn update<T: Projection>(&mut self, camera: &mut Camera<T>, dt: Duration) {
        let dt = dt.as_secs_f32();

        self.yaw += self.rotation.0 * self.sensitivity * dt;
        self.pitch -= self.rotation.1 * self.sensitivity * dt;
        self.pitch = self.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);

        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();

        let direction = Vec3::new(yaw_sin * pitch_cos, pitch_sin, -yaw_cos * pitch_cos).normalize();
        let forward = Vec3::new(yaw_sin, 0.0, -yaw_cos).normalize();
        let right = direction.cross(Vec3::Y).normalize();

        camera.up = right.cross(direction);
        camera.eye += forward * (self.movement.0 - self.movement.2) * self.speed * dt;
        camera.eye -= right * (self.movement.1 - self.movement.3) * self.speed * dt;
        camera.eye += Vec3::Y * (self.movement.4 - self.movement.5) * self.speed * dt;
        camera.center = camera.eye + direction;

        self.rotation = (0.0, 0.0);
    }
}
