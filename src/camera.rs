use core::f32;

use nalgebra::{Matrix4, Point3, Vector3};

use crate::as_any::AsAny;

pub trait Camera: Sync + Send + AsAny {
    fn view(&self) -> Matrix4<f32>;
}

pub struct StaticCamera {
    eye: Point3<f32>,
    target: Point3<f32>,
}

impl StaticCamera {
    pub fn new(eye: Point3<f32>, target: Point3<f32>) -> Self {
        StaticCamera { eye, target }
    }
}

impl Camera for StaticCamera {
    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.eye, &self.target, &Vector3::y_axis())
    }
}

pub struct ArcballCamera {
    target: Point3<f32>,
    radius: f32,
    phi: f32,
    theta: f32,
}

impl ArcballCamera {
    pub fn new(target: Point3<f32>, radius: f32) -> Self {
        ArcballCamera {
            target,
            radius,
            phi: 0.0,
            theta: 0.0,
        }
    }

    pub fn update(&mut self, x: f32, y: f32) {
        self.theta += x;
        self.phi += y;

        if self.phi > f32::consts::PI / 2.0 {
            self.phi = f32::consts::PI / 2.0;
        } else if self.phi < -f32::consts::PI / 2.0 {
            self.phi = -f32::consts::PI / 2.0;
        }
    }
}

impl Camera for ArcballCamera {
    fn view(&self) -> Matrix4<f32> {
        let forward = Vector3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos());
        let right = Vector3::new(-self.phi.cos(), 0.0, self.phi.sin());
        let up = forward.cross(&right).normalize();

        let eye = self.target - forward * self.radius;

        Matrix4::look_at_rh(&eye, &self.target, &up)
    }
}
