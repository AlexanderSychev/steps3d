use glam::Vec3;

/// Луч в трехмерном пространстве
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3d {
    /// Точка испускания луча
    pub origin: Vec3,
    /// Вектор направления луча
    pub direction: Vec3,
}

impl Ray3d {
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}
