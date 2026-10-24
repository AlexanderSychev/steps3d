use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::plane::{Plane, PlaneSide};
use super::ray_3d::Ray3d;
use glam::Vec3;

/// Параллелепипед, параллельный осям координат (AABB) в 3D
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisAlignedBoundingBox3d {
    /// Минимальная точка
    pub min_point: Vec3,
    /// Максимальная точка
    pub max_point: Vec3,
}

impl Default for AxisAlignedBoundingBox3d {
    fn default() -> Self {
        // Инициализация "вывернутого наизнанку" AABB для корректной работы слияния
        Self::new(Vec3::MAX, Vec3::MIN)
    }
}

impl AxisAlignedBoundingBox3d {
    /// Создать новый AABB по двум точкам
    pub fn new(min_point: Vec3, max_point: Vec3) -> Self {
        Self {
            min_point,
            max_point,
        }
    }

    /// ИДИОМАТИЧНЫЙ КОНСТРУКТОР: Создать плотный AABB для набора вершин с использованием SIMD
    pub fn from_vertices(vertices: &[Vec3]) -> Self {
        if vertices.is_empty() {
            return Self::default();
        }

        let mut min_p = Vec3::MAX;
        let mut max_p = Vec3::MIN;

        for &vertex in vertices {
            // Встроенные методы glam используют SIMD-оптимизацию для X, Y, Z одновременно
            min_p = min_p.min(vertex);
            max_p = max_p.max(vertex);
        }

        Self::new(min_p, max_p)
    }

    /// Взять вершину по индексу (Исправлена побитовая логика из книги)
    /// Индекс от 0 до 7 определяет один из 8 углов параллелепипеда
    pub fn get_vertex(&self, index: usize) -> Vec3 {
        Vec3::new(
            if index & 1 != 0 {
                self.max_point.x
            } else {
                self.min_point.x
            },
            if index & 2 != 0 {
                self.max_point.y
            } else {
                self.min_point.y
            },
            if index & 4 != 0 {
                self.max_point.z
            } else {
                self.min_point.z
            },
        )
    }

    /// Получить координаты центра AABB
    pub fn get_center(&self) -> Vec3 {
        (self.min_point + self.max_point) * 0.5
    }

    /// Получить размеры (длину, ширину, высоту) AABB
    pub fn get_size(&self) -> Vec3 {
        self.max_point - self.min_point
    }

    /// Проверить, является ли AABB вырожденным или пустым
    pub fn is_empty(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let size = self.get_size();
        size.x < epsilon || size.y < epsilon || size.z < epsilon
    }

    /// Проверить, входит ли точка в AABB
    pub fn contains(&self, point: &Vec3) -> bool {
        point.cmple(self.max_point).all() && point.cmpge(self.min_point).all()
    }

    /// Проверить, пересекается ли этот AABB с другим (Высокоскоростной тест)
    pub fn intersects(&self, other: &AxisAlignedBoundingBox3d) -> bool {
        // Сравниваем проекции на все три оси одновременно
        self.min_point.x <= other.max_point.x
            && self.max_point.x >= other.min_point.x
            && self.min_point.y <= other.max_point.y
            && self.max_point.y >= other.min_point.y
            && self.min_point.z <= other.max_point.z
            && self.max_point.z >= other.min_point.z
    }

    /// Пересечение луча с AABB методом параллельных пластин.
    ///
    /// Возвращает параметры `(t_enter, t_exit)` вдоль луча: точки входа
    /// и выхода. Если источник луча находится внутри AABB, `t_enter`
    /// отрицателен. Возвращает `None`, если луч проходит мимо или AABB
    /// целиком остался позади источника луча.
    pub fn ray_intersection_interval(
        &self,
        ray: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Option<(f32, f32)> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let origin = ray.origin.to_array();
        let direction = ray.direction.to_array();
        let min_point = self.min_point.to_array();
        let max_point = self.max_point.to_array();

        let mut t_enter = f32::NEG_INFINITY;
        let mut t_exit = f32::INFINITY;

        for i in 0..3 {
            if direction[i].abs() > epsilon {
                // Параметры пересечения с ближней и дальней плоскостями.
                let mut t1 = (min_point[i] - origin[i]) / direction[i];
                let mut t2 = (max_point[i] - origin[i]) / direction[i];

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }

                t_enter = t_enter.max(t1);
                t_exit = t_exit.min(t2);

                // Интервалы по осям не перекрываются — луч прошёл мимо.
                if t_enter > t_exit {
                    return None;
                }
            } else if origin[i] < min_point[i] - epsilon || origin[i] > max_point[i] + epsilon {
                // Луч параллелен пластинам и проходит вне их.
                return None;
            }
        }

        // AABB целиком остался позади источника луча.
        if t_exit < 0.0 {
            return None;
        }

        Some((t_enter, t_exit))
    }

    /// Пересекает ли луч этот AABB.
    pub fn is_ray_intersects(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> bool {
        self.ray_intersection_interval(ray, maybe_epsilon).is_some()
    }

    /// Сбросить AABB (вернуть к дефолтному вывернутому состоянию)
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Слияние (поглощение) другого AABB с расширением границ
    pub fn merge(&mut self, other: &AxisAlignedBoundingBox3d) {
        self.min_point = self.min_point.min(other.min_point);
        self.max_point = self.max_point.max(other.max_point);
    }

    /// Расширить границы AABB равномерно во все стороны на дельта-вектор
    pub fn grow_by_vec(&mut self, delta: &Vec3) {
        self.min_point -= *delta;
        self.max_point += *delta;
    }

    /// Расширить границы AABB равномерно во все стороны на число (скаляр)
    pub fn grow_by_number(&mut self, delta: f32) {
        let delta_vec = Vec3::splat(delta);
        self.min_point -= delta_vec;
        self.max_point += delta_vec;
    }

    /// Получить объем параллелепипеда
    pub fn volume(&self) -> f32 {
        let size = self.get_size();
        size.x * size.y * size.z
    }

    /// Получить площадь полной поверхности параллелепипеда
    pub fn area(&self) -> f32 {
        let size = self.get_size();
        2.0 * (size.x * size.y + size.y * size.z + size.z * size.x)
    }

    /// Классифицировать положение этого AABB относительно плоскости
    pub fn classify(&self, plane: &Plane, maybe_epsilon: Option<f32>) -> PlaneSide {
        plane.classify_aabb(self, maybe_epsilon)
    }

    /// Вычислить расстояние от плоскости до AABB (по логике Борескова из книги)
    pub fn distance_to(&self, plane: &Plane) -> f32 {
        let (near_point, far_point) = plane.make_near_and_far_points(self);

        let dist = plane.signed_distance_to_point(near_point);
        if dist > 0.0 {
            return dist;
        }

        let dist_far = plane.signed_distance_to_point(far_point);
        if dist_far < 0.0 {
            return dist_far;
        }

        0.0 // Прямоугольник пересекает плоскость
    }
}
