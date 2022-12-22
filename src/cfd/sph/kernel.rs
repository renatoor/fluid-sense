use glam::Vec3;
use std::f32::consts::PI;
use winit::event::VirtualKeyCode;

struct Poly6 {
    k: f32,
    l: f32,
}

struct Spiky {
    l: f32,
}

struct Viscosity {
    l: f32,
}

struct Kernel {
    radius: f32,
    radius_sqr: f32,
    poly6: Poly6,
    spiky: Spiky,
    viscosity: Viscosity,
}


impl Poly6 {
    pub fn new(radius: f32) -> Self {
        Self {
            k: 315.0 / (64.0 * PI * radius.powf(9.0)),
            l: -945.0 / (32.0 * PI * radius.powf(9.0)),
        }
    }
}

impl Spiky {
    pub fn new(radius: f32) -> Self {
        Self {
            l: -45.0 / (PI * radius.powf(6.0)),
        }
    }
}

impl Viscosity {
    pub fn new(radius: f32) -> Self {
        Self {
            l:
        }
    }
}
