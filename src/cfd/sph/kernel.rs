use glam::Vec3;
use std::f32::consts::PI;

struct Poly6 {
    radius: f32,
    radius_sqr: f32,
    k: f32,
    l: f32,
}

struct Spiky {
    radius: f32,
    radius_sqr: f32,
    l: f32,
}

struct Viscosity {
    radius: f32,
    radius_sqr: f32,
    l: f32,
}

pub struct Kernel {
    w0: f32,
    poly6: Poly6,
    spiky: Spiky,
    viscosity: Viscosity,
}

impl Poly6 {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            radius_sqr: radius * radius,
            k: 315.0 / (64.0 * PI * radius.powf(9.0)),
            l: -945.0 / (32.0 * PI * radius.powf(9.0)),
        }
    }

    pub fn w(&self, r: f32) -> f32 {
        let r2 = r * r;

        self.k * (self.radius_sqr - r2).powf(3.0)
    }

    pub fn w_vec(&self, r: Vec3) -> f32 {
        let r2 = r.dot(r);

        self.k * (self.radius_sqr - r2).powf(3.0)
    }

    pub fn grad_w(&self, r: Vec3) -> Vec3 {
        let r2 = r.dot(r);
        let hr = self.radius_sqr - r2;
        let hr2 = hr * hr;

        r * self.l * hr2
    }

    pub fn laplacian_w(&self, r: Vec3) -> f32 {
        let r2 = r.dot(r);

        self.l * (self.radius_sqr - r2) * (3.0 * self.radius_sqr - 7.0 * r2)
    }
}

impl Spiky {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            radius_sqr: radius * radius,
            l: -45.0 / (PI * radius.powf(6.0)),
        }
    }

    pub fn grad_w(&self, r: Vec3) -> Vec3 {
        let rl = r.length();
        let hr = self.radius - rl;
        let hr2 = hr * hr;

        self.l * hr2 * (r / rl)
    }
}

impl Viscosity {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            radius_sqr: radius * radius,
            l: 10.0,
        }
    }

    pub fn laplacian_w(&self, r: Vec3) -> f32 {
        let rl = r.length();

        self.l * (self.radius * rl)
    }
}

impl Kernel {
    pub fn new(radius: f32) -> Self {
        let poly6 = Poly6::new(radius);
        let spiky = Spiky::new(radius);
        let viscosity = Viscosity::new(radius);
        let w0 = poly6.w(0.0);

        Self {
            w0,
            poly6,
            spiky,
            viscosity,
        }
    }

    pub fn w0(&self) -> f32 {
        self.w0
    }

    pub fn w(&self, r: Vec3) -> f32 {
        self.poly6.w_vec(r)
    }

    pub fn poly6_grad_w(&self, r: Vec3) -> Vec3 {
        self.poly6.grad_w(r)
    }

    pub fn poly6_laplacian_w(&self, r: Vec3) -> f32 {
        self.poly6.laplacian_w(r)
    }

    pub fn spiky_grad_w(&self, r: Vec3) -> Vec3 {
        self.spiky.grad_w(r)
    }

    pub fn viscosity_laplacian_w(&self, r: Vec3) -> f32 {
        self.viscosity.laplacian_w(r)
    }
}
