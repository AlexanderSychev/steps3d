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
        let l = self.center - ray.origin;
        let l2oc = l.dot(l);
        let tca = l.dot(ray.direction);
        let t2hc = self.radius_sq - l2oc + tca * tca;

        if t2hc <= 0.0 {
            return None;
        }

        let t2hc_sqrt = f32::sqrt(t2hc);
        let t1: f32;
        let t2: f32;

        if tca < t2hc_sqrt {
            t1 = tca + t2hc_sqrt;
            t2 = tca - t2hc_sqrt;
        } else {
            t1 = tca - t2hc_sqrt;
            t2 = tca + t2hc_sqrt;
        }

        let mut final_t = t1;
        if f32::abs(final_t) < epsilon {
            final_t = t2;
        }

        if final_t > epsilon {
            let position = ray.point_at(final_t);
            Some(HitData {
                position,
                t: final_t,
                invert_normal: false,
                object_index: 0,
                material: self.def_surface,
                normal: (position - self.center).normalize(),
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

/// Плоскость
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
    pub def_surface: MaterialProps,
}

impl Plane {
    pub fn new(normal: Vec3, d: f32, def_surface: MaterialProps) -> Self {
        Self {
            normal: normal.normalize(),
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
        let inv_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        let mut tmin = (self.min.x - ray.origin.x) * inv_dir.x;
        let mut tmax = (self.max.x - ray.origin.x) * inv_dir.x;
        if inv_dir.x < 0.0 {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min.y - ray.origin.y) * inv_dir.y;
        let mut tymax = (self.max.y - ray.origin.y) * inv_dir.y;
        if inv_dir.y < 0.0 {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min.z - ray.origin.z) * inv_dir.z;
        let mut tzmax = (self.max.z - ray.origin.z) * inv_dir.z;
        if inv_dir.z < 0.0 {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return None;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        let mut final_t = tmin;
        if final_t < epsilon {
            final_t = tmax;
            if final_t < epsilon {
                return None;
            }
        }

        let position = ray.origin + ray.direction * final_t;
        let mut calculated_normal = Vec3::ZERO;
        let bias = 1e-4;

        // ИСПРАВЛЕНО: Добавлен дефолтный fallback (Vec3::Y), если точка попала точно на ребро
        if f32::abs(position.x - self.min.x) < bias {
            calculated_normal.x = -1.0;
        } else if f32::abs(position.x - self.max.x) < bias {
            calculated_normal.x = 1.0;
        } else if f32::abs(position.y - self.min.y) < bias {
            calculated_normal.y = -1.0;
        } else if f32::abs(position.y - self.max.y) < bias {
            calculated_normal.y = 1.0;
        } else if f32::abs(position.z - self.min.z) < bias {
            calculated_normal.z = -1.0;
        } else if f32::abs(position.z - self.max.z) < bias {
            calculated_normal.z = 1.0;
        } else {
            calculated_normal.y = 1.0;
        }

        Some(HitData {
            position,
            normal: calculated_normal.normalize(),
            t: final_t,
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
