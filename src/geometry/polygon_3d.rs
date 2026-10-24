//! Плоский выпуклый многоугольник в трехмерном пространстве
//!
//! Тип нужен BSP-дереву: рассекатель — это многоугольник, а при построении
//! многоугольники, пересекающие плоскость, разрезаются на две части.
//!
//! Предполагается, что многоугольник выпуклый и его вершины заданы в порядке
//! обхода. Для невыпуклых многоугольников разрезание методом Сазерленда-
//! Ходжмена может дать несколько отдельных кусков, а не один.

use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::plane::Plane;
use glam::Vec3;

/// Положение многоугольника относительно плоскости.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonPlaneSide {
    /// Все вершины в положительном полупространстве (или на плоскости).
    InPositive,
    /// Все вершины в отрицательном полупространстве (или на плоскости).
    InNegative,
    /// Все вершины лежат на плоскости.
    InPlane,
    /// Есть вершины по обе стороны плоскости.
    Intersecting,
}

/// Результат разрезания многоугольника плоскостью.
///
/// Для многоугольника целиком по одну сторону заполнен только один вариант.
/// Для пересекающего плоскость — оба.
#[derive(Debug, Clone, PartialEq)]
pub struct PolygonSplit {
    /// Часть в положительном полупространстве.
    pub positive: Option<Polygon3d>,
    /// Часть в отрицательном полупространстве.
    pub negative: Option<Polygon3d>,
}

/// Плоский выпуклый многоугольник в 3D.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon3d {
    vertices: Vec<Vec3>,
    plane: Plane,
}

impl Polygon3d {
    /// Создать многоугольник по вершинам.
    ///
    /// Возвращает `None`, если вершин меньше трех, многоугольник вырожден
    /// (нулевая площадь) или вершины не лежат в одной плоскости.
    pub fn new(vertices: Vec<Vec3>, maybe_epsilon: Option<f32>) -> Option<Self> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let normal = Self::newell_normal(&vertices);

        // Нулевая нормаль означает вырожденный многоугольник.
        if normal.length_squared() <= epsilon * epsilon {
            return None;
        }

        let plane = Plane::from_point_and_normal(vertices[0], normal);

        Self::from_planar_vertices(vertices, plane, epsilon)
    }

    /// Вершины многоугольника в порядке обхода.
    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    /// Плоскость многоугольника.
    pub fn plane(&self) -> &Plane {
        &self.plane
    }

    /// AABB многоугольника.
    pub fn bounds(&self) -> AxisAlignedBoundingBox3d {
        AxisAlignedBoundingBox3d::from_vertices(&self.vertices)
    }

    /// Среднее арифметическое вершин (удобно для отладочной печати).
    pub fn center(&self) -> Vec3 {
        let mut sum = Vec3::ZERO;

        for &vertex in &self.vertices {
            sum += vertex;
        }

        sum / self.vertices.len() as f32
    }

    /// Площадь многоугольника.
    pub fn area(&self) -> f32 {
        // Для плоского многоугольника длина нормали Ньюэлла равна удвоенной площади.
        Self::newell_normal(&self.vertices).length() * 0.5
    }

    /// Классифицировать многоугольник относительно плоскости.
    pub fn classify(&self, plane: &Plane, maybe_epsilon: Option<f32>) -> PolygonPlaneSide {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let mut has_positive = false;
        let mut has_negative = false;

        for &vertex in &self.vertices {
            let distance = plane.signed_distance_to_point(vertex);

            if distance > epsilon {
                has_positive = true;
            } else if distance < -epsilon {
                has_negative = true;
            }
        }

        match (has_positive, has_negative) {
            (true, true) => PolygonPlaneSide::Intersecting,
            (true, false) => PolygonPlaneSide::InPositive,
            (false, true) => PolygonPlaneSide::InNegative,
            (false, false) => PolygonPlaneSide::InPlane,
        }
    }

    /// Разрезать многоугольник плоскостью.
    ///
    /// Многоугольник, лежащий целиком на плоскости, считается положительным:
    /// так же поступает C++-версия книги (`IN_PLANE` уходит в positive).
    pub fn split(&self, plane: &Plane, maybe_epsilon: Option<f32>) -> PolygonSplit {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        match self.classify(plane, Some(epsilon)) {
            PolygonPlaneSide::InPositive | PolygonPlaneSide::InPlane => PolygonSplit {
                positive: Some(self.clone()),
                negative: None,
            },
            PolygonPlaneSide::InNegative => PolygonSplit {
                positive: None,
                negative: Some(self.clone()),
            },
            PolygonPlaneSide::Intersecting => self.split_intersecting(plane, epsilon),
        }
    }

    fn split_intersecting(&self, plane: &Plane, epsilon: f32) -> PolygonSplit {
        let mut positive_vertices = Vec::new();
        let mut negative_vertices = Vec::new();
        let count = self.vertices.len();

        for i in 0..count {
            let current = self.vertices[i];
            let next = self.vertices[(i + 1) % count];

            let current_distance = plane.signed_distance_to_point(current);
            let next_distance = plane.signed_distance_to_point(next);

            // Вершина на плоскости попадает в обе части.
            if current_distance >= -epsilon {
                Self::push_unique(&mut positive_vertices, current, epsilon);
            }

            if current_distance <= epsilon {
                Self::push_unique(&mut negative_vertices, current, epsilon);
            }

            let crosses = (current_distance > epsilon && next_distance < -epsilon)
                || (current_distance < -epsilon && next_distance > epsilon);

            if crosses {
                let t = current_distance / (current_distance - next_distance);
                let intersection = current + (next - current) * t;

                Self::push_unique(&mut positive_vertices, intersection, epsilon);
                Self::push_unique(&mut negative_vertices, intersection, epsilon);
            }
        }

        PolygonSplit {
            positive: Self::from_planar_vertices(positive_vertices, self.plane, epsilon),
            negative: Self::from_planar_vertices(negative_vertices, self.plane, epsilon),
        }
    }

    /// Собрать многоугольник из вершин, которые уже лежат в заданной плоскости.
    ///
    /// Используется для частей, полученных при разрезании: их плоскость
    /// совпадает с плоскостью исходного многоугольника, поэтому нормаль
    /// заново не вычисляется.
    fn from_planar_vertices(mut vertices: Vec<Vec3>, plane: Plane, epsilon: f32) -> Option<Self> {
        Self::remove_duplicate_vertices(&mut vertices, epsilon);

        if vertices.len() < 3 {
            return None;
        }

        let all_on_plane = vertices
            .iter()
            .all(|vertex| plane.signed_distance_to_point(*vertex).abs() <= epsilon);

        if !all_on_plane {
            return None;
        }

        // После обрезки мог остаться вырожденный (нулевой площади) многоугольник.
        if Self::newell_normal(&vertices).length_squared() <= epsilon * epsilon {
            return None;
        }

        Some(Self { vertices, plane })
    }

    fn remove_duplicate_vertices(vertices: &mut Vec<Vec3>, epsilon: f32) {
        let epsilon_squared = epsilon * epsilon;

        vertices.dedup_by(|a, b| a.distance_squared(*b) <= epsilon_squared);

        if vertices.len() > 1
            && vertices[0].distance_squared(*vertices.last().unwrap()) <= epsilon_squared
        {
            vertices.pop();
        }
    }

    fn push_unique(vertices: &mut Vec<Vec3>, point: Vec3, epsilon: f32) {
        let epsilon_squared = epsilon * epsilon;

        if vertices
            .last()
            .is_none_or(|last| last.distance_squared(point) > epsilon_squared)
        {
            vertices.push(point);
        }
    }

    /// Нормаль Ньюэлла: устойчивее выбора первых трех вершин и работает
    /// для многоугольников с произвольным числом вершин.
    fn newell_normal(vertices: &[Vec3]) -> Vec3 {
        let mut normal = Vec3::ZERO;

        for i in 0..vertices.len() {
            let current = vertices[i];
            let next = vertices[(i + 1) % vertices.len()];

            normal.x += (current.y - next.y) * (current.z + next.z);
            normal.y += (current.z - next.z) * (current.x + next.x);
            normal.z += (current.x - next.x) * (current.y + next.y);
        }

        normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn quad(z: f32) -> Polygon3d {
        Polygon3d::new(
            vec![
                Vec3::new(-2.0, -1.0, z),
                Vec3::new(2.0, -1.0, z),
                Vec3::new(2.0, 1.0, z),
                Vec3::new(-2.0, 1.0, z),
            ],
            None,
        )
        .unwrap()
    }

    #[test]
    fn quad_properties() {
        let polygon = quad(0.0);

        assert_eq!(polygon.vertices().len(), 4);
        assert!((polygon.area() - 8.0).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert_eq!(polygon.center(), Vec3::ZERO);
        assert!((polygon.plane().normal - Vec3::Z).length() < DEFAULT_GEOMETRY_EPSILON);
        assert_eq!(
            polygon.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::new(-2.0, -1.0, 0.0), Vec3::new(2.0, 1.0, 0.0))
        );
    }

    #[test]
    fn rejects_degenerate_and_non_planar_polygons() {
        assert!(Polygon3d::new(vec![Vec3::ZERO, Vec3::X], None).is_none());

        // Три коллинеарные вершины.
        assert!(
            Polygon3d::new(vec![Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0)], None).is_none()
        );

        // Четвертая вершина не лежит в плоскости первых трех.
        assert!(
            Polygon3d::new(
                vec![
                    Vec3::ZERO,
                    Vec3::X,
                    Vec3::new(1.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 1.0),
                ],
                None
            )
            .is_none()
        );
    }

    #[test]
    fn classifies_polygon_relative_to_plane() {
        let plane = Plane::from_point_and_normal(Vec3::ZERO, Vec3::Z);

        assert_eq!(quad(0.0).classify(&plane, None), PolygonPlaneSide::InPlane);
        assert_eq!(
            quad(5.0).classify(&plane, None),
            PolygonPlaneSide::InPositive
        );
        assert_eq!(
            quad(-5.0).classify(&plane, None),
            PolygonPlaneSide::InNegative
        );

        // Квадрат в плоскости z = 0 пересекает плоскость x = 0.
        let x_plane = Plane::from_point_and_normal(Vec3::ZERO, Vec3::X);
        assert_eq!(
            quad(0.0).classify(&x_plane, None),
            PolygonPlaneSide::Intersecting
        );
    }

    #[test]
    fn splits_quad_into_two_halves() {
        let polygon = quad(0.0);
        let plane = Plane::from_point_and_normal(Vec3::ZERO, Vec3::X);

        let split = polygon.split(&plane, None);
        let positive = split.positive.expect("положительная часть");
        let negative = split.negative.expect("отрицательная часть");

        assert!((positive.area() - 4.0).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((negative.area() - 4.0).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!(
            positive
                .vertices()
                .iter()
                .all(|vertex| vertex.x >= -DEFAULT_GEOMETRY_EPSILON)
        );
        assert!(
            negative
                .vertices()
                .iter()
                .all(|vertex| vertex.x <= DEFAULT_GEOMETRY_EPSILON)
        );
    }

    #[test]
    fn coplanar_polygon_is_not_split() {
        let polygon = quad(0.0);
        let plane = Plane::from_point_and_normal(Vec3::ZERO, Vec3::Z);

        let split = polygon.split(&plane, None);

        assert!(split.positive.is_some());
        assert!(split.negative.is_none());
        assert_eq!(split.positive.unwrap().area(), polygon.area());
    }
}
