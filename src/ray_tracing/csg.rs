//! Конструктивная сплошная геометрия (Constructive Solid Geometry или CSG)

use super::super::geometry::Ray3d;
use super::material::{MaterialProps, SurfaceData};
use super::object::{HitData, Object};
use glam::Vec3;

/// Конечная точка отрезка луча (вход или выход из объекта)
#[derive(Debug, Clone, Copy)]
pub struct Endpoint {
    /// Параметр t вдоль луча
    pub t: f32,
    /// Нужно ли инвертировать нормаль в этой точке
    pub invert: bool,
    /// Информация о геометрии пересечения (позиция, normal, материал)
    pub hit: HitData,
}

/// Отрезок луча, находящийся строго внутри твердого тела: от `start` до `end`
#[derive(Debug, Clone, Copy)]
pub struct RaySegment {
    pub start: Endpoint,
    pub end: Endpoint,
}

impl RaySegment {
    /// Проверить, пустой ли сегмент (точка входа лежит дальше или там же, где точка выхода)
    pub fn is_empty(&self) -> bool {
        self.start.t >= self.end.t
    }

    /// Проверить пересечение двух сегментов (Overlap)
    pub fn intersects(&self, other: &RaySegment) -> bool {
        f32::max(self.start.t, other.start.t) < f32::min(self.end.t, other.end.t)
    }

    /// Пересечь текущий сегмент с другим (Логическое AND над двумя отрезками)
    pub fn intersect(&mut self, other: &RaySegment) {
        if other.start.t > self.start.t {
            self.start = other.start;
        }
        if other.end.t < self.end.t {
            self.end = other.end;
        }
    }

    /// Слить (объединить) текущий сегмент с другим (Логическое OR над двумя отрезками)
    pub fn merge(&mut self, other: &RaySegment) {
        if other.start.t < self.start.t {
            self.start = other.start;
        }
        if other.end.t > self.end.t {
            self.end = other.end;
        }
    }
}

/// Логические операции CSG
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsgOperation {
    Or,
    And,
    Sub,
}

/// Представление CSG-объекта в виде дерева
#[derive(Debug, Clone)]
pub enum CsgObject {
    /// Базовая сфера CSG
    Sphere {
        center: Vec3,
        radius: f32,
        radius_sq: f32,
        material: MaterialProps,
    },
    /// Базовый параллелепипед CSG (AABB)
    Box {
        min: Vec3,
        max: Vec3,
        material: MaterialProps,
    },
    /// Композитный узел (Дерево)
    Composite {
        op1: Box<CsgObject>,
        op2: Box<CsgObject>,
        operation: CsgOperation,
    },
}

impl CsgObject {
    /// Метод сбора всех сегментов пересечений луча с CSG-объектом
    pub fn all_intersections(&self, ray: &Ray3d, epsilon: f32, res: &mut Vec<RaySegment>) {
        match self {
            CsgObject::Sphere {
                center,
                radius,
                radius_sq,
                material,
            } => {
                csg_sphere_all_intersections(
                    *center, *radius, *radius_sq, *material, ray, epsilon, res,
                );
            }
            CsgObject::Box { min, max, material } => {
                // ИСПРАВЛЕНО: Вызываем правильную функцию и передаем корректные аргументы
                csg_box_all_intersections(*min, *max, *material, ray, epsilon, res);
            }
            CsgObject::Composite {
                op1,
                op2,
                operation,
            } => {
                let mut l1 = Vec::new();
                let mut l2 = Vec::new();

                op1.all_intersections(ray, epsilon, &mut l1);
                op2.all_intersections(ray, epsilon, &mut l2);

                match operation {
                    CsgOperation::Or => csg_or(&l1, &l2, res),
                    CsgOperation::And => csg_and(&l1, &l2, res),
                    CsgOperation::Sub => csg_sub(&l1, &l2, res),
                }
            }
        }
    }
}

// Реализуем наш стандартный трейт Object для всей CSG-структуры
impl Object for CsgObject {
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(crate::geometry::DEFAULT_GEOMETRY_EPSILON);
        let mut segments = Vec::new();

        self.all_intersections(ray, epsilon, &mut segments);

        if segments.is_empty() {
            return None;
        }

        let first_seg = &segments[0];

        if first_seg.start.t > epsilon {
            Some(first_seg.start.hit)
        } else if first_seg.end.t > epsilon {
            let mut exit_hit = first_seg.end.hit;
            if first_seg.end.invert {
                exit_hit.normal = -exit_hit.normal;
            }
            Some(exit_hit)
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

/// Вычисляет все интервалы пересечения луча со сплошной CSG-сферой
pub fn csg_sphere_all_intersections(
    center: Vec3,
    radius: f32,
    radius_sq: f32,
    material: MaterialProps,
    ray: &Ray3d,
    epsilon: f32,
    res: &mut Vec<RaySegment>,
) -> bool {
    let l = center - ray.origin;
    let l2oc = l.dot(l);
    let tca = l.dot(ray.direction);
    let t2hc = radius_sq - l2oc + tca * tca;

    if t2hc <= 0.0 {
        return false;
    }

    let t2hc_sqrt = f32::sqrt(t2hc);
    let mut t1 = tca - t2hc_sqrt;
    let mut t2 = tca + t2hc_sqrt;

    if t1 < epsilon && t2 < epsilon {
        return false;
    }

    if t1 > t2 {
        std::mem::swap(&mut t1, &mut t2);
    }

    let pos_start = ray.origin + ray.direction * t1;
    let pos_end = ray.origin + ray.direction * t2;

    let mut hit_start = HitData::new_blank(t1, material);
    hit_start.position = pos_start;
    hit_start.normal = (pos_start - center) / radius;

    let mut hit_end = HitData::new_blank(t2, material);
    hit_end.position = pos_end;
    hit_end.normal = (pos_end - center) / radius;

    res.push(RaySegment {
        start: Endpoint {
            t: t1,
            invert: false,
            hit: hit_start,
        },
        end: Endpoint {
            t: t2,
            invert: false,
            hit: hit_end,
        },
    });
    true
}

/// Вычисляет все интервалы пересечения луча со сплошной CSG-коробкой (AABB)
pub fn csg_box_all_intersections(
    min: Vec3,
    max: Vec3,
    material: MaterialProps,
    ray: &Ray3d,
    epsilon: f32,
    res: &mut Vec<RaySegment>,
) -> bool {
    let inv_dir = Vec3::new(
        1.0 / ray.direction.x,
        1.0 / ray.direction.y,
        1.0 / ray.direction.z,
    );

    let mut t1 = (min.x - ray.origin.x) * inv_dir.x;
    let mut t2 = (max.x - ray.origin.x) * inv_dir.x;
    if inv_dir.x < 0.0 {
        std::mem::swap(&mut t1, &mut t2);
    }

    let mut ty1 = (min.y - ray.origin.y) * inv_dir.y;
    let mut ty2 = (max.y - ray.origin.y) * inv_dir.y;
    if inv_dir.y < 0.0 {
        std::mem::swap(&mut ty1, &mut ty2);
    }

    if (t1 > ty2) || (ty1 > t2) {
        return false;
    }
    if ty1 > t1 {
        t1 = ty1;
    }
    if ty2 < t2 {
        t2 = ty2;
    }

    let mut tz1 = (min.z - ray.origin.z) * inv_dir.z;
    let mut tz2 = (max.z - ray.origin.z) * inv_dir.z;
    if inv_dir.z < 0.0 {
        std::mem::swap(&mut tz1, &mut tz2);
    }

    if (t1 > tz2) || (tz1 > t2) {
        return false;
    }
    if tz1 > t1 {
        t1 = tz1;
    }
    if tz2 < t2 {
        t2 = tz2;
    }

    if t1 < epsilon && t2 < epsilon {
        return false;
    }
    if t1 > t2 {
        std::mem::swap(&mut t1, &mut t2);
    }

    let get_hit_data = |t: f32| {
        let pos = ray.origin + ray.direction * t;
        let mut normal = Vec3::ZERO;
        let bias = 1e-4;
        if f32::abs(pos.x - min.x) < bias {
            normal.x = -1.0;
        } else if f32::abs(pos.x - max.x) < bias {
            normal.x = 1.0;
        } else if f32::abs(pos.y - min.y) < bias {
            normal.y = -1.0;
        } else if f32::abs(pos.y - max.y) < bias {
            normal.y = 1.0;
        } else if f32::abs(pos.z - min.z) < bias {
            normal.z = -1.0;
        } else if f32::abs(pos.z - max.z) < bias {
            normal.z = 1.0;
        } else {
            normal.y = 1.0;
        }

        let mut hit = HitData::new_blank(t, material);
        hit.position = pos;
        hit.normal = normal.normalize();
        hit
    };

    res.push(RaySegment {
        start: Endpoint {
            t: t1,
            invert: false,
            hit: get_hit_data(t1),
        },
        end: Endpoint {
            t: t2,
            invert: false,
            hit: get_hit_data(t2),
        },
    });
    true
}

/* --- Реализация алгоритмов комбинирования булевых сегментов --- */

pub fn csg_and(a: &[RaySegment], b: &[RaySegment], c: &mut Vec<RaySegment>) {
    c.clear();
    for seg_a in a {
        for seg_b in b {
            let mut s = *seg_a;
            s.intersect(seg_b);
            if !s.is_empty() {
                c.push(s);
            }
        }
    }
}

pub fn csg_or(a: &[RaySegment], b: &[RaySegment], c: &mut Vec<RaySegment>) {
    c.clear();
    c.extend_from_slice(a);
    for seg_b in b {
        let mut merged = false;
        for seg_c in c.iter_mut() {
            if seg_c.intersects(seg_b) {
                seg_c.merge(seg_b);
                merged = true;
                break;
            }
        }
        if !merged {
            c.push(*seg_b);
        }
    }
    c.sort_by(|x, y| x.start.t.partial_cmp(&y.start.t).unwrap());
}

pub fn csg_sub(a: &[RaySegment], b: &[RaySegment], c: &mut Vec<RaySegment>) {
    c.clear();
    for seg_a in a {
        let mut current_segments = vec![*seg_a];
        for seg_b in b {
            let mut next_segments = Vec::new();
            for s in current_segments {
                if !s.intersects(seg_b) {
                    next_segments.push(s);
                } else {
                    if s.start.t < seg_b.start.t {
                        let mut left = s;
                        left.end = seg_b.start;
                        left.end.invert = !left.end.invert;
                        if !left.is_empty() {
                            next_segments.push(left);
                        }
                    }
                    if s.end.t > seg_b.end.t {
                        let mut right = s;
                        right.start = seg_b.end;
                        right.start.invert = !right.start.invert;
                        if !right.is_empty() {
                            next_segments.push(right);
                        }
                    }
                }
            }
            current_segments = next_segments;
        }
        c.extend(current_segments);
    }
}
