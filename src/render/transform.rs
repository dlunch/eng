use glam::{EulerRot, Mat4, Quat, Vec3};

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    pub fn with_values(translation: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_matrix(matrix: &Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        let rotation = rotation.to_euler(EulerRot::YXZ);

        Self {
            translation,
            rotation: Vec3::new(rotation.1, rotation.0, rotation.2),
            scale,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        let rotation = Quat::from_euler(EulerRot::YXZ, self.rotation.y, self.rotation.x, self.rotation.z);
        Mat4::from_scale_rotation_translation(self.scale, rotation, self.translation)
    }

    pub fn rotate(&mut self, rotation: Vec3) {
        self.rotation += rotation;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
