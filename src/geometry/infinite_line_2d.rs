use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::line_segment_2d::LineSegment2d;
use glam::Vec2;

/// Бесконечная прямая на плоскости
#[derive(Debug, Clone, Copy)]
pub struct InfiniteLine2d {
    /// Вектор направления
    pub direction: Vec2,
    /// Точка, которую пересекает прямая
    pub point: Vec2,
}

impl InfiniteLine2d {
    /// Попробовать создать прямую из отрезка
    pub fn try_from_segment(segment: &LineSegment2d) -> Option<InfiniteLine2d> {
        segment.direction().map(|direction| InfiniteLine2d {
            direction,
            point: segment.a,
        })
    }

    /// Получить косинус угла между двумя прямыми
    pub fn angle_cos(a: &InfiniteLine2d, b: &InfiniteLine2d) -> f32 {
        a.direction.dot(b.direction)
    }

    /// Получить угол между двумя прямыми (в радианах)
    pub fn angle(a: &InfiniteLine2d, b: &InfiniteLine2d) -> f32 {
        Self::angle_cos(a, b).abs().acos()
    }

    /// Получить нормаль прямой
    pub fn normal(&self) -> Vec2 {
        Vec2::new(self.direction.y, -self.direction.x)
    }

    /// Найти точку пересечения двух прямых
    pub fn intersect(&self, other: &InfiniteLine2d, maybe_epsilon: Option<f32>) -> Option<Vec2> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Косое произведение (определитель) векторов направлений
        let det = self.direction.x * other.direction.y - self.direction.y * other.direction.x;

        // Если определитель близок к нулю, прямые параллельны или совпадают
        if det.abs() < epsilon {
            return None;
        }

        let n_self = self.normal();
        let d_self = -self.point.dot(n_self);

        let n_other = other.normal();
        let d_other = -other.point.dot(n_other);

        // Решение системы линейных уравнений по правилу Крамера
        let x = (-d_self * n_other.y - (-d_other * n_self.y)) / det;
        let y = (n_self.x * (-d_other) - (n_other.x * (-d_self))) / det;

        Some(Vec2::new(x, y))
    }
}
