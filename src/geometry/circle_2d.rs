use super::constants::DEFAULT_GEOMETRY_EPSILON;
use glam::Vec2;

/// Окружность
#[derive(Debug, Clone, Copy)]
pub struct Circle2d {
    /// Радиус
    pub radius: f32,
    /// Координаты центра окружности
    pub center: Vec2,
}

/// Математические функции для триангуляции
impl Circle2d {
    /// Вычислить описанную окружность (Circumcircle) вокруг трех точек треугольника.
    /// Возвращает None, если точки лежат на одной прямой (коллинеарны).
    pub fn circumcircle_for_triangle(
        a: &Vec2,
        b: &Vec2,
        c: &Vec2,
        maybe_epsilon: Option<f32>,
    ) -> Option<Self> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Используем перпендикуляры к серединам ребер (серединные перпендикуляры)
        // Находим определитель системы уравнений
        let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));

        if d.abs() < epsilon {
            return None; // Точки вырождены в линию, окружность построить нельзя
        }

        let a_sq = a.length_squared();
        let b_sq = b.length_squared();
        let c_sq = c.length_squared();

        // Координаты центра описанной окружности по формулам Крамера
        let center_x = (a_sq * (b.y - c.y) + b_sq * (c.y - a.y) + c_sq * (a.y - b.y)) / d;
        let center_y = (a.x * (b_sq - c_sq) + b.x * (c_sq - a_sq) + c.x * (a_sq - b_sq)) / d;
        let center = Vec2::new(center_x, center_y);

        // Радиус — это расстояние от центра до любой из трех вершин
        let radius = center.distance(*a);

        Some(Circle2d { radius, center })
    }

    /// Проверить, находится ли точка внутри описанной окружности (включая погрешность границы)
    pub fn contains_point_inclusive(&self, point: &Vec2, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let dist = self.center.distance(*point);
        dist < self.radius + epsilon
    }
}
