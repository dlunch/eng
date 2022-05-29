use core::f32;

use glam::{Mat4, Vec3};

pub trait Camera: Sync + Send {
    fn view(&self) -> Mat4;
    fn projection(&self) -> Mat4;
}

pub struct OrthographicCamera {
    width: u32,
    height: u32,
}

impl OrthographicCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Camera for OrthographicCamera {
    fn view(&self) -> Mat4 {
        Mat4::IDENTITY
    }

    fn projection(&self) -> Mat4 {
        Mat4::orthographic_rh(0., self.width as f32, self.height as f32, 0., -1., 1.)
    }
}

pub trait PerspectiveCameraController: Sync + Send {
    fn position(&self) -> Vec3;
    fn target(&self) -> Vec3;
    fn up(&self) -> Vec3;
}

pub struct PerspectiveCamera<T: PerspectiveCameraController> {
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    controller: T,
}

impl<T: PerspectiveCameraController> PerspectiveCamera<T> {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32, controller: T) -> Self {
        Self {
            fov,
            aspect,
            near,
            far,
            controller,
        }
    }

    pub fn controller_mut(&mut self) -> &mut T {
        &mut self.controller
    }
}

impl<T: PerspectiveCameraController> Camera for PerspectiveCamera<T> {
    fn view(&self) -> Mat4 {
        let position = self.controller.position();
        let target = self.controller.target();
        let up = self.controller.up();

        Mat4::look_at_rh(position, target, up)
    }

    fn projection(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
}

pub struct StaticCameraController {
    position: Vec3,
    target: Vec3,
}

impl StaticCameraController {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self { position, target }
    }
}

impl PerspectiveCameraController for StaticCameraController {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn target(&self) -> Vec3 {
        self.target
    }

    fn up(&self) -> Vec3 {
        Vec3::Y
    }
}

pub struct ArcballCameraController {
    target: Vec3,
    radius: f32,
    phi: f32,
    theta: f32,
}

impl ArcballCameraController {
    pub fn new(target: Vec3, radius: f32) -> Self {
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
        let forward_dir = Vec3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();
        let right_dir = Vec3::new(-self.phi.cos(), 0.0, self.phi.sin()).normalize();

        self.target += forward_dir * forward;
        self.target += right_dir * right;
    }
}

impl PerspectiveCameraController for ArcballCameraController {
    fn position(&self) -> Vec3 {
        let forward = Vec3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();

        self.target - forward * self.radius
    }

    fn target(&self) -> Vec3 {
        self.target
    }

    fn up(&self) -> Vec3 {
        let forward = Vec3::new(-self.phi.sin() * self.theta.cos(), -self.theta.sin(), -self.phi.cos() * self.theta.cos()).normalize();
        let right = Vec3::new(-self.phi.cos(), 0.0, self.phi.sin()).normalize();
        forward.cross(right).normalize()
    }
}

#[cfg(test)]
mod test {
    use core::f32::consts::PI;

    use glam::Vec3;

    use super::{Camera, OrthographicCamera, PerspectiveCamera, StaticCameraController};

    #[test]
    fn test_orthographic() {
        let camera = OrthographicCamera::new(100, 100);
        assert_eq!(
            camera.view().to_cols_array(),
            [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,]
        );

        assert_eq!(
            camera.projection().to_cols_array(),
            [0.02, 0.0, 0.0, 0.0, 0.0, -0.02, 0.0, 0.0, 0.0, 0.0, -0.5, 0.0, -1.0, 1.0, 0.5, 1.0]
        )
    }

    #[test]
    fn test_perspective() {
        let controller = StaticCameraController::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));

        let camera = PerspectiveCamera::new(45.0 * PI / 180.0, 200.0f32 / 100.0f32, 0.1, 100.0, controller);
        assert_eq!(
            camera.view().to_cols_array(),
            [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, -0.0, 0.0, -1.0, 0.0, -0.0, 0.0, 0.0, -0.0, -5.0, 1.0]
        );

        assert_eq!(
            camera.projection().to_cols_array(),
            [1.2071067, 0.0, 0.0, 0.0, 0.0, 2.4142134, 0.0, 0.0, 0.0, 0.0, -1.001001, -1.0, 0.0, 0.0, -0.1001001, 0.0]
        )
    }
}
