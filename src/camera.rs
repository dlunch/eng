use alloc::boxed::Box;
use core::{f32, ops::DerefMut};

use nalgebra::{Matrix4, Point3, Vector3};

use crate::as_any::AsAny;

pub trait CameraController: Sync + Send + AsAny {
    fn position(&self) -> Point3<f32>;
    fn target(&self) -> Point3<f32>;
    fn up(&self) -> Vector3<f32>;
}

pub struct Camera {
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    controller: Box<dyn CameraController>,
}

impl Camera {
    pub fn new<T: CameraController + 'static>(fov: f32, aspect: f32, near: f32, far: f32, controller: T) -> Self {
        Self {
            fov,
            aspect,
            near,
            far,
            controller: Box::new(controller),
        }
    }

    pub fn view(&self) -> Matrix4<f32> {
        let position = self.controller.position();
        let target = self.controller.target();
        let up = self.controller.up();

        Matrix4::look_at_rh(&position, &target, &up)
    }

    pub fn projection(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.fov, self.aspect, self.near, self.far)
    }

    pub fn controller_mut<T: CameraController + 'static>(&mut self) -> Option<&mut T> {
        // why do we need deref_mut here??
        self.controller.deref_mut().as_any_mut().downcast_mut::<T>()
    }
}

pub struct StaticCameraController {
    position: Point3<f32>,
    target: Point3<f32>,
}

impl StaticCameraController {
    pub fn new(position: Point3<f32>, target: Point3<f32>) -> Self {
        Self { position, target }
    }
}

impl CameraController for StaticCameraController {
    fn position(&self) -> Point3<f32> {
        self.position
    }

    fn target(&self) -> Point3<f32> {
        self.target
    }

    fn up(&self) -> Vector3<f32> {
        Vector3::y()
    }
}

pub struct ArcballCameraController {
    target: Point3<f32>,
    radius: f32,
    phi: f32,
    theta: f32,
}

impl ArcballCameraController {
    pub fn new(target: Point3<f32>, radius: f32) -> Self {
        Self {
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

impl CameraController for ArcballCameraController {
    fn position(&self) -> Point3<f32> {
        let forward = Vector3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();

        self.target - forward * self.radius
    }

    fn target(&self) -> Point3<f32> {
        self.target
    }

    fn up(&self) -> Vector3<f32> {
        let forward = Vector3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();
        let right = Vector3::new(-self.phi.cos(), 0.0, self.phi.sin()).normalize();
        forward.cross(&right).normalize()
    }
}
