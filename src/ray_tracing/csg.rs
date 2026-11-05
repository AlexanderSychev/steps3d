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

        let endpoint = segments
            .iter()
            .flat_map(|segment| [&segment.start, &segment.end])
            .filter(|endpoint| endpoint.t > epsilon && endpoint.t.is_finite())
            .min_by(|a, b| a.t.total_cmp(&b.t))?;
        let mut hit = endpoint.hit;
        if endpoint.invert {
            hit.normal = -hit.normal;
        }
        Some(hit)
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
    if !radius.is_finite() || radius <= 0.0 || !radius_sq.is_finite() || radius_sq <= 0.0 {
        return false;
    }
    let direction_length_sq = ray.direction.length_squared();
    if direction_length_sq == 0.0 || !direction_length_sq.is_finite() {
        return false;
    }

    let offset = ray.origin - center;
    let half_b = offset.dot(ray.direction);
    let c = offset.length_squared() - radius_sq;
    let discriminant = half_b * half_b - direction_length_sq * c;
    if discriminant < 0.0 {
        return false;
    }

    let root = f32::sqrt(discriminant);
    let mut t1 = (-half_b - root) / direction_length_sq;
    let mut t2 = (-half_b + root) / direction_length_sq;
    if t1 > t2 {
        std::mem::swap(&mut t1, &mut t2);
    }
    if t1 < epsilon && t2 < epsilon {
        return false;
    }

    let pos_start = ray.point_at(t1);
    let pos_end = ray.point_at(t2);

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
    let (t1, normal1, t2, normal2) = match super::object::ray_aabb_interval(min, max, ray) {
        Some(interval) => interval,
        None => return false,
    };
    if t1 < epsilon && t2 < epsilon {
        return false;
    }

    let get_hit_data = |t: f32, normal: Vec3| {
        let mut hit = HitData::new_blank(t, material);
        hit.position = ray.point_at(t);
        hit.normal = normal;
        hit
    };

    res.push(RaySegment {
        start: Endpoint {
            t: t1,
            invert: false,
            hit: get_hit_data(t1, normal1),
        },
        end: Endpoint {
            t: t2,
            invert: false,
            hit: get_hit_data(t2, normal2),
        },
    });
    true
}

/* --- Реализация алгоритмов комбинирования булевых сегментов --- */

fn sort_segments(segments: &mut Vec<RaySegment>) {
    segments.retain(|segment| {
        segment.start.t.is_finite() && segment.end.t.is_finite() && !segment.is_empty()
    });
    segments.sort_by(|a, b| a.start.t.total_cmp(&b.start.t));
}

fn merge_overlapping_segments(segments: &mut Vec<RaySegment>) {
    sort_segments(segments);
    let mut merged: Vec<RaySegment> = Vec::with_capacity(segments.len());

    for segment in segments.drain(..) {
        if let Some(last) = merged.last_mut() {
            if segment.start.t <= last.end.t {
                if segment.end.t > last.end.t {
                    last.end = segment.end;
                }
                continue;
            }
        }
        merged.push(segment);
    }

    *segments = merged;
}

pub fn csg_and(a: &[RaySegment], b: &[RaySegment], c: &mut Vec<RaySegment>) {
    c.clear();
    for seg_a in a {
        for seg_b in b {
            let mut segment = *seg_a;
            segment.intersect(seg_b);
            if !segment.is_empty() {
                c.push(segment);
            }
        }
    }
    merge_overlapping_segments(c);
}

pub fn csg_or(a: &[RaySegment], b: &[RaySegment], c: &mut Vec<RaySegment>) {
    c.clear();
    c.extend_from_slice(a);
    c.extend_from_slice(b);
    merge_overlapping_segments(c);
}

pub fn csg_sub(a: &[RaySegment], b: &[RaySegment], c: &mut Vec<RaySegment>) {
    c.clear();
    let mut right_segments = b.to_vec();
    sort_segments(&mut right_segments);

    for seg_a in a {
        let mut current_segments = vec![*seg_a];
        for seg_b in &right_segments {
            let mut next_segments = Vec::new();
            for segment in current_segments {
                if !segment.intersects(seg_b) {
                    next_segments.push(segment);
                } else {
                    if segment.start.t < seg_b.start.t {
                        let mut left = segment;
                        left.end = seg_b.start;
                        left.end.invert = !left.end.invert;
                        if !left.is_empty() {
                            next_segments.push(left);
                        }
                    }
                    if segment.end.t > seg_b.end.t {
                        let mut right = segment;
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
    sort_segments(c);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start: f32, end: f32) -> RaySegment {
        let material = MaterialProps::default();
        let mut start_hit = HitData::new_blank(start, material);
        start_hit.normal = -Vec3::X;
        let mut end_hit = HitData::new_blank(end, material);
        end_hit.normal = Vec3::X;
        RaySegment {
            start: Endpoint {
                t: start,
                invert: false,
                hit: start_hit,
            },
            end: Endpoint {
                t: end,
                invert: false,
                hit: end_hit,
            },
        }
    }

    #[test]
    fn csg_or_merges_intervals_that_bridge_multiple_segments() {
        let left = [segment(1.0, 2.0), segment(5.0, 6.0)];
        let bridge = [segment(1.5, 5.5)];
        let mut result = Vec::new();

        csg_or(&left, &bridge, &mut result);

        assert_eq!(result.len(), 1);
        assert_eq!((result[0].start.t, result[0].end.t), (1.0, 6.0));
    }

    #[test]
    fn csg_subtraction_inverts_normal_at_inner_entry_boundary() {
        let material = MaterialProps::default();
        let object = CsgObject::Composite {
            op1: Box::new(CsgObject::Sphere {
                center: Vec3::ZERO,
                radius: 3.0,
                radius_sq: 9.0,
                material,
            }),
            op2: Box::new(CsgObject::Sphere {
                center: Vec3::ZERO,
                radius: 1.0,
                radius_sq: 1.0,
                material,
            }),
            operation: CsgOperation::Sub,
        };
        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::X,
        };

        let hit = object.intersect(&ray, None).unwrap();

        assert!((hit.t - 1.0).abs() < 1e-6);
        assert_eq!(hit.normal, -Vec3::X);
    }

    #[test]
    fn csg_box_handles_parallel_ray_on_box_face() {
        let mut result = Vec::new();
        let ray = Ray3d {
            origin: Vec3::new(-1.0, 0.0, -2.0),
            direction: Vec3::Z,
        };

        assert!(csg_box_all_intersections(
            Vec3::splat(-1.0),
            Vec3::splat(1.0),
            MaterialProps::default(),
            &ray,
            1e-4,
            &mut result,
        ));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start.hit.normal, -Vec3::Z);
    }

    #[test]
    fn csg_sphere_intersection_supports_non_unit_ray_direction() {
        let mut result = Vec::new();
        let ray = Ray3d {
            origin: Vec3::new(-3.0, 0.0, 0.0),
            direction: Vec3::new(2.0, 0.0, 0.0),
        };

        assert!(csg_sphere_all_intersections(
            Vec3::ZERO,
            1.0,
            1.0,
            MaterialProps::default(),
            &ray,
            1e-4,
            &mut result,
        ));
        assert!((result[0].start.t - 1.0).abs() < 1e-6);
        assert_eq!(result[0].start.hit.position, Vec3::new(-1.0, 0.0, 0.0));
    }
}
