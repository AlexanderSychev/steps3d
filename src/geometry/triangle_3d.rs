use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::ray_3d::Ray3d;
use glam::Vec3;

/// Результат пересечения луча с треугольником в барицентрических координатах
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayTriangleIntersection {
    /// Барицентрический вес для вершины B
    pub u: f32,
    /// Барицентрический вес для вершины C
    /// (Вес для вершины A можно найти как: 1.0 - u - v)
    pub v: f32,
    /// Расстояние от источника луча до точки пересечения (параметр вдоль луча)
    pub t: f32,
}

impl RayTriangleIntersection {
    /// Барицентрический вес для вершины B
    /// (человеко-читаемая обертка над свойством "u")
    pub fn weight_b(&self) -> f32 {
        self.u
    }

    /// Барицентрический вес для вершины C
    /// (человеко-читаемая обертка над свойством "v")
    pub fn weight_c(&self) -> f32 {
        self.v
    }

    /// Барицентрический вес для вершины A
    pub fn weight_a(&self) -> f32 {
        1.0 - self.u - self.v
    }
}

/// Треугольник в трехмерном пространстве
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3d {
    /// Вершина A
    pub a: Vec3,
    /// Вершина B
    pub b: Vec3,
    /// Вершина C
    pub c: Vec3,
}

impl Triangle3d {
    /// Возвращает детали пересечения, если луч пересекает треугольник.
    pub fn get_intersection_barycentric(
        &self,
        ray: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Option<RayTriangleIntersection> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let edge_ab = self.b - self.a;
        let edge_ac = self.c - self.a;

        let p_vec = ray.direction.cross(edge_ac);
        let det = edge_ab.dot(p_vec);

        // Если детерминант близок к нулю, луч параллелен плоскости треугольника
        if det.abs() < epsilon {
            return None;
        }

        let inv_det = 1.0 / det;
        let t_vec = ray.origin - self.a;

        let u = inv_det * t_vec.dot(p_vec);
        if u < -epsilon || u > 1.0 + epsilon {
            return None;
        }

        let q_vec = t_vec.cross(edge_ab);

        let v = inv_det * ray.direction.dot(q_vec);
        if v < -epsilon || u + v > 1.0 + epsilon {
            return None;
        }

        let t = inv_det * edge_ac.dot(q_vec);
        if t < epsilon {
            return None;
        }

        Some(RayTriangleIntersection { u, v, t })
    }

    /// Возвращает абсолютные 3D-координаты точки пересечения в мировом пространстве.
    pub fn get_intersection_ray(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<Vec3> {
        self.get_intersection_barycentric(ray, maybe_epsilon)
            .map(|intersection| ray.origin + intersection.t * ray.direction)
    }

    /// Просто проверяет факт пересечения (да/нет).
    pub fn is_ray_intersects(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> bool {
        self.get_intersection_barycentric(ray, maybe_epsilon)
            .is_some()
    }
}
