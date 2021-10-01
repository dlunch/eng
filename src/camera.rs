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

    pub fn r#move(&mut self, x: f32, y: f32, z: f32) {
        self.eye += Vector3::new(x, y, z);
        self.target += Vector3::new(x, y, z);
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
        self.phi += x;
        self.theta += y;

        if self.theta > f32::consts::PI / 2.0 {
            self.theta = f32::consts::PI / 2.0;
        } else if self.theta < -f32::consts::PI / 2.0 {
            self.theta = -f32::consts::PI / 2.0;
        }
    }

    pub fn r#move(&mut self, forward: f32, right: f32) {
        let forward_dir = Vector3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();
        let right_dir = Vector3::new(-self.phi.cos(), 0.0, self.phi.sin()).normalize();

        self.target += forward_dir * forward;
        self.target += right_dir * right;
    }
}

impl Camera for ArcballCamera {
    fn view(&self) -> Matrix4<f32> {
        let forward = Vector3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();
        let right = Vector3::new(-self.phi.cos(), 0.0, self.phi.sin()).normalize();
        let up = forward.cross(&right).normalize();

        let eye = self.target - forward * self.radius;

        Matrix4::look_at_rh(&eye, &self.target, &up)
    }
}
