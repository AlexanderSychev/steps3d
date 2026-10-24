use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d; // Подключаем ваш AABB
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::ray_3d::Ray3d;
use glam::Vec3;

/// Сфера в трехмерном пространстве
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sphere {
    /// Центр сферы
    pub center: Vec3,
    /// Радиус сферы
    pub radius: f32,
}

impl Sphere {
    /// Создать новую сферу по центру и радиусу
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Создать плотную сферу для набора вершин (избавились от ручных мутаций)
    pub fn from_vertices(vertices: &[Vec3]) -> Self {
        if vertices.is_empty() {
            return Self::new(Vec3::ZERO, 0.0);
        }

        // Вычисляем средний центр масс облака точек
        let sum_center: Vec3 = vertices.iter().sum();
        let center = sum_center / vertices.len() as f32;

        // Находим радиальное расстояние до самой дальней точки
        let mut radius: f32 = 0.0;
        for vertex in vertices {
            let r = center.distance(*vertex);
            radius = radius.max(r);
        }

        Self { center, radius }
    }

    /// ВОССТАНОВЛЕННЫЙ МЕТОД: Описать сферу вокруг 3D прямоугольника (AABB)
    /// Радиус равен половине длины главной диагонали параллелепипеда.
    pub fn from_aabb(bbox: &AxisAlignedBoundingBox3d) -> Self {
        let center = bbox.get_center();
        let radius = 0.5 * bbox.get_size().length(); // 0.5f * glm::length ( box.getSize () )
        Self { center, radius }
    }

    /// Получить объем сферы
    pub fn get_volume(&self) -> f32 {
        (4.0 / 3.0) * std::f32::consts::PI * self.radius.powi(3)
    }

    /// Получить площадь полной поверхности сферы
    pub fn get_area(&self) -> f32 {
        4.0 * std::f32::consts::PI * self.radius.powi(2)
    }

    /// Проверить, пересекаются ли две сферы (Высокоскоростной безошибочный тест)
    pub fn is_sphere_intersects(&self, other: &Sphere) -> bool {
        // Использование квадратов длин избавляет от тяжелой операции sqrt() процессора
        let distance_sq = (self.center - other.center).length_squared();
        let radius_sum = self.radius + other.radius;
        distance_sq <= radius_sum * radius_sum
    }

    /// Находит ближайшее пересечение луча со сферой (Полностью исправленный геометрический метод).
    /// Возвращает Some(f32) с расстоянием `t` вдоль луча до точки входа,
    /// или None, если луч пролетает мимо или направлен в другую сторону.
    pub fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let to_center = self.center - ray.origin;
        let length_to_center_sq = to_center.length_squared();
        let t_projection = to_center.dot(ray.direction);

        // ФУНДАМЕНТАЛЬНОЕ ИСПРАВЛЕНИЕ ЗАЩИТЫ "ЗА СПИНОЙ":
        // Если сфера сзади начала луча (t_projection < 0) и мы при этом находимся СНАРУЖИ сферы,
        // то луч летит в противоположную сторону и пересечения нет.
        if t_projection < 0.0 && length_to_center_sq > self.radius * self.radius {
            return None;
        }

        let d_sq = length_to_center_sq - t_projection * t_projection;
        let radius_sq = self.radius * self.radius;
        let t_chord_sq = radius_sq - d_sq;

        if t_chord_sq <= 0.0 {
            return None;
        }

        let t_chord = t_chord_sq.sqrt();

        // По логике автора: вычисляем обе точки
        let t1 = t_projection - t_chord;
        let t2 = t_projection + t_chord;

        // Если t1 меньше нуля, это значит мы находимся внутри сферы,
        // и луч выходит из нее в точке t2. Иначе берем точку входа t1.
        let t_result = if t1 < epsilon { t2 } else { t1 };

        // Финальная проверка: если точка все еще за спиной — коллизии нет
        if t_result < epsilon {
            return None;
        }

        Some(t_result)
    }

    /// Находит абсолютные 3D-координаты точки входа луча в сферу.
    pub fn get_intersection_point(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<Vec3> {
        self.ray_intersection(ray, maybe_epsilon)
            .map(|t| ray.origin + t * ray.direction)
    }

    /// Просто проверяет факт пересечения луча со сферой (да/нет).
    pub fn is_ray_intersects(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> bool {
        self.ray_intersection(ray, maybe_epsilon).is_some()
    }
}
