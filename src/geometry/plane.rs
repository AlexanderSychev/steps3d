use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::ray_3d::Ray3d;
use glam::Vec3;

/// Результат классификации положения объекта относительно плоскости
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneSide {
    /// Объект целиком находится в положительном полупространстве (со стороны нормали)
    InPositive,
    /// Объект целиком находится в отрицательном полупространстве
    InNegative,
    /// Объект пересекает плоскость
    Intersecting,
}

/// Плоскость в трехмерном пространстве (заданная уравнением Ax + By + Cz + D = 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    /// Нормализованный вектор перпендикуляра к плоскости (A, B, C)
    pub normal: Vec3,
    /// Смещение плоскости относительно начала координат (D)
    pub d: f32,
}

impl Plane {
    /// Создать плоскость из явного вектора нормали и смещения D.
    /// Нормаль автоматически приводится к единичной длине.
    pub fn new(normal: Vec3, d: f32) -> Self {
        Self {
            normal: normal.normalize(),
            d,
        }
    }

    /// Создать плоскость по точке на ней и вектору нормали.
    /// Формула: D = - (Point ∙ Normal)
    pub fn from_point_and_normal(point: Vec3, normal: Vec3) -> Self {
        let n = normal.normalize();
        let d = -point.dot(n);
        Self { normal: n, d }
    }

    /// Создать плоскость по трем точкам треугольника (задает ориентацию плоскости).
    /// Направление нормали определяется правилом правой руки (против часовой стрелки).
    pub fn from_three_points(
        a: Vec3,
        b: Vec3,
        c: Vec3,
        maybe_epsilon: Option<f32>,
    ) -> Option<Self> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let edge_ab = b - a;
        let edge_ac = c - a;
        let cross = edge_ab.cross(edge_ac);

        // Если точки лежат на одной прямой, нормаль вычислить нельзя
        if cross.length_squared() < epsilon * epsilon {
            return None;
        }

        Some(Self::from_point_and_normal(a, cross))
    }

    /// Вычислить ориентированное расстояние от плоскости до точки.
    /// * Положительное значение — точка со стороны нормали.
    /// * Отрицательное значение — точка с обратной стороны.
    /// * Ноль — точка строго на плоскости.
    pub fn signed_distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.d
    }

    /// Проверить пересечение плоскости с лучом.
    /// Возвращает Some(f32) с расстоянием `t` вдоль луча до точки пересечения,
    /// или None, если луч параллелен плоскости или направлен в другую сторону.
    pub fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Проекция направления луча на нормаль плоскости (скорость сближения)
        let denominator = ray.direction.dot(self.normal);

        // Если знаменатель близок к нулю — луч параллелен плоскости
        if denominator.abs() < epsilon {
            return None;
        }

        // Вычисляем параметр t
        let t = -(ray.origin.dot(self.normal) + self.d) / denominator;

        // Если t < 0, плоскость осталась позади начала луча
        if t < epsilon {
            return None;
        }

        Some(t)
    }

    /// Найти ближайшую точку на плоскости (nearsPoint) и дальнюю точку (farthestPoint)
    /// для заданного AABB. Это ядро алгоритма отсечения (Frustum Culling).
    pub fn make_near_and_far_points(&self, bbox: &AxisAlignedBoundingBox3d) -> (Vec3, Vec3) {
        let mut near_point = Vec3::ZERO;
        let mut far_point = Vec3::ZERO;

        // Выбираем углы AABB в зависимости от знаков компонент нормали плоскости.
        // Если нормаль положительна по оси X, то минимальный X — это ближняя грань, а максимальный — дальняя.
        if self.normal.x >= 0.0 {
            near_point.x = bbox.min_point.x;
            far_point.x = bbox.max_point.x;
        } else {
            near_point.x = bbox.max_point.x;
            far_point.x = bbox.min_point.x;
        }

        if self.normal.y >= 0.0 {
            near_point.y = bbox.min_point.y;
            far_point.y = bbox.max_point.y;
        } else {
            near_point.y = bbox.max_point.y;
            far_point.y = bbox.min_point.y;
        }

        if self.normal.z >= 0.0 {
            near_point.z = bbox.min_point.z;
            far_point.z = bbox.max_point.z;
        } else {
            near_point.z = bbox.max_point.z;
            far_point.z = bbox.min_point.z;
        }

        (near_point, far_point)
    }

    /// Классифицировать положение трехмерного AABB относительно плоскости (метод из книги).
    /// Используется для проверки видимости нод дерева (AABB) в усеченной пирамиде камеры (Frustum).
    pub fn classify_aabb(
        &self,
        bbox: &AxisAlignedBoundingBox3d,
        maybe_epsilon: Option<f32>,
    ) -> PlaneSide {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let (near_point, far_point) = self.make_near_and_far_points(bbox);

        // Считаем ориентированное расстояние до ближней точки AABB
        if self.signed_distance_to_point(near_point) > epsilon {
            return PlaneSide::InPositive;
        }

        // Считаем ориентированное расстояние до дальней точки AABB
        if self.signed_distance_to_point(far_point) < -epsilon {
            return PlaneSide::InNegative;
        }

        PlaneSide::Intersecting
    }
}
