//! Объекты сцены

use super::super::geometry::DEFAULT_GEOMETRY_EPSILON;
use super::super::geometry::Ray3d;
use super::material::{MaterialProps, SurfaceData};
use glam::Vec3;

/// Информация о пересечении луча с объектом
#[derive(Debug, Clone, Copy)]
pub struct HitData {
    /// Точка пересечения в мировых координатах
    pub position: Vec3,
    /// Геометрическая нормаль в точке пересечения
    pub normal: Vec3,
    /// Параметр t (дистанция) вдоль луча
    pub t: f32,
    /// Флаг инверсии нормали (критично для CSG-операций)
    pub invert_normal: bool,
    /// Индекс объекта в векторе сцены (проставляется динамически структурой Scene)
    pub object_index: usize,
    /// Оптические свойства материала в данной точке
    pub material: MaterialProps,
}

impl HitData {
    /// Вспомогательный пустой конструктор для инициализации в сегментах CSG
    pub fn new_blank(t: f32, material: MaterialProps) -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Y,
            t,
            invert_normal: false,
            object_index: 0,
            material,
        }
    }
}

impl Default for HitData {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Y,
            t: 0.0,
            invert_normal: false,
            object_index: 0,
            material: Default::default(),
        }
    }
}

/// Интерфейс геометрического объекта сцены
pub trait Object {
    /// Проверить, пересекает ли луч объект.
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData>;

    /// Получить данные поверхности в точке пересечения
    fn get_surface(&self, hit_data: &HitData) -> SurfaceData;
}

/// Найти интервал пересечения луча с AABB вместе с нормалями его граней.
/// Параллельные оси обрабатываются отдельно, чтобы не получать `0 * infinity = NaN`.
pub(super) fn ray_aabb_interval(
    min: Vec3,
    max: Vec3,
    ray: &Ray3d,
) -> Option<(f32, Vec3, f32, Vec3)> {
    if ray.direction.length_squared() == 0.0
        || !ray.origin.is_finite()
        || !ray.direction.is_finite()
        || !min.is_finite()
        || !max.is_finite()
    {
        return None;
    }

    let mut t_near = f32::NEG_INFINITY;
    let mut t_far = f32::INFINITY;
    let mut normal_near = Vec3::ZERO;
    let mut normal_far = Vec3::ZERO;

    for axis in 0..3 {
        let origin = ray.origin[axis];
        let direction = ray.direction[axis];
        if direction == 0.0 {
            if origin < min[axis] || origin > max[axis] {
                return None;
            }
            continue;
        }

        let mut t0 = (min[axis] - origin) / direction;
        let mut t1 = (max[axis] - origin) / direction;
        let mut n0 = Vec3::ZERO;
        let mut n1 = Vec3::ZERO;
        n0[axis] = -1.0;
        n1[axis] = 1.0;

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
            std::mem::swap(&mut n0, &mut n1);
        }

        if t0 > t_near {
            t_near = t0;
            normal_near = n0;
        }
        if t1 < t_far {
            t_far = t1;
            normal_far = n1;
        }
        if t_near > t_far {
            return None;
        }
    }

    Some((t_near, normal_near, t_far, normal_far))
}

/// Сфера
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub radius_sq: f32,
    pub def_surface: MaterialProps,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, def_surface: MaterialProps) -> Self {
        Self {
            center,
            radius,
            radius_sq: radius * radius,
            def_surface,
        }
    }
}

impl Object for Sphere {
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let direction_length_sq = ray.direction.length_squared();
        if direction_length_sq == 0.0 || !direction_length_sq.is_finite() {
            return None;
        }

        let offset = ray.origin - self.center;
        let half_b = offset.dot(ray.direction);
        let c = offset.length_squared() - self.radius_sq;
        let discriminant = half_b * half_b - direction_length_sq * c;
        if discriminant < 0.0 {
            return None;
        }

        let root = f32::sqrt(discriminant);
        let mut t = (-half_b - root) / direction_length_sq;
        let other_t = (-half_b + root) / direction_length_sq;
        if t <= epsilon {
            t = other_t;
        }
        if t <= epsilon {
            return None;
        }

        let position = ray.point_at(t);
        Some(HitData {
            position,
            t,
            invert_normal: false,
            object_index: 0,
            material: self.def_surface,
            normal: (position - self.center).normalize(),
        })
    }

    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        SurfaceData {
            position: hit_data.position,
            normal: hit_data.normal,
            material: hit_data.material,
        }
    }
}

/// Плоскость
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
    pub def_surface: MaterialProps,
}

impl Plane {
    pub fn new(normal: Vec3, d: f32, def_surface: MaterialProps) -> Self {
        let length = normal.length();
        let (normal, d) = if length > 0.0 && length.is_finite() {
            (normal / length, d / length)
        } else {
            (Vec3::ZERO, d)
        };
        Self {
            normal,
            d,
            def_surface,
        }
    }
}

impl Object for Plane {
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let vd = self.normal.dot(ray.direction);

        if vd > -epsilon && vd < epsilon {
            return None;
        }

        let t = -(self.normal.dot(ray.origin) + self.d) / vd;

        if t > epsilon {
            let position = ray.origin + ray.direction * t;
            Some(HitData {
                position,
                t,
                invert_normal: false,
                object_index: 0,
                normal: self.normal,
                material: self.def_surface,
            })
        } else {
            None
        }
    }

    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        SurfaceData {
            position: hit_data.position,
            normal: hit_data.normal,
            material: hit_data.material,
        }
    }
}

/// Треугольник
#[derive(Debug, Clone)]
pub struct Tri {
    pub p: [Vec3; 3],
    pub normal: Vec3,
    pub def_surface: MaterialProps,
}

impl Tri {
    pub fn new(a: Vec3, b: Vec3, c: Vec3, def_surface: MaterialProps) -> Self {
        let edge1 = b - a;
        let edge2 = c - a;
        let normal = edge1.cross(edge2).normalize();
        Self {
            p: [a, b, c],
            normal,
            def_surface,
        }
    }
}

impl Object for Tri {
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let edge1 = self.p[1] - self.p[0];
        let edge2 = self.p[2] - self.p[0];

        let pvec = ray.direction.cross(edge2);
        let det = edge1.dot(pvec);

        if det.abs() < epsilon {
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin - self.p[0];

        let u = tvec.dot(pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(edge1);
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = edge2.dot(qvec) * inv_det;

        if t > epsilon {
            let position = ray.origin + ray.direction * t;
            Some(HitData {
                position,
                t,
                invert_normal: false,
                object_index: 0,
                normal: self.normal,
                material: self.def_surface,
            })
        } else {
            None
        }
    }

    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        SurfaceData {
            position: hit_data.position,
            normal: hit_data.normal,
            material: hit_data.material,
        }
    }
}

/// Выровненная по осям координат коробка (Axis-Aligned Bounding Box)
#[derive(Debug, Clone)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub def_surface: MaterialProps,
}

impl Aabb {
    pub fn new(p1: Vec3, p2: Vec3, def_surface: MaterialProps) -> Self {
        Self {
            min: Vec3::new(
                f32::min(p1.x, p2.x),
                f32::min(p1.y, p2.y),
                f32::min(p1.z, p2.z),
            ),
            max: Vec3::new(
                f32::max(p1.x, p2.x),
                f32::max(p1.y, p2.y),
                f32::max(p1.z, p2.z),
            ),
            def_surface,
        }
    }
}

impl Object for Aabb {
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let (t_near, normal_near, t_far, normal_far) = ray_aabb_interval(self.min, self.max, ray)?;

        let (t, normal) = if t_near > epsilon {
            (t_near, normal_near)
        } else if t_far > epsilon {
            (t_far, normal_far)
        } else {
            return None;
        };

        Some(HitData {
            position: ray.point_at(t),
            normal,
            t,
            invert_normal: false,
            object_index: 0,
            material: self.def_surface,
        })
    }

    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        SurfaceData {
            position: hit_data.position,
            normal: hit_data.normal,
            material: hit_data.material,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_intersection_supports_non_unit_ray_direction() {
        let sphere = Sphere::new(Vec3::ZERO, 1.0, MaterialProps::default());
        let ray = Ray3d {
            origin: Vec3::new(-3.0, 0.0, 0.0),
            direction: Vec3::new(2.0, 0.0, 0.0),
        };

        let hit = sphere.intersect(&ray, None).unwrap();

        assert!((hit.t - 1.0).abs() < 1e-6);
        assert_eq!(hit.position, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn plane_normalization_preserves_plane_location() {
        let plane = Plane::new(Vec3::new(0.0, 2.0, 0.0), -4.0, MaterialProps::default());

        assert_eq!(plane.normal, Vec3::Y);
        assert!((plane.d + 2.0).abs() < 1e-6);
    }

    #[test]
    fn aabb_handles_parallel_ray_on_box_face() {
        let aabb = Aabb::new(
            Vec3::splat(-1.0),
            Vec3::splat(1.0),
            MaterialProps::default(),
        );
        let ray = Ray3d {
            origin: Vec3::new(-1.0, 0.0, -2.0),
            direction: Vec3::Z,
        };

        let hit = aabb.intersect(&ray, None).unwrap();

        assert!((hit.t - 1.0).abs() < 1e-6);
        assert_eq!(hit.normal, -Vec3::Z);
    }
}
