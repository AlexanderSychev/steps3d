//! Иерархия ограничивающих объемов (Bounding Volume Hierarchy, BVH)
//!
//! В C++-версии узел хранил AABB, флаг `leaf` и `union` из вектора потомков
//! и вектора объектов. В Rust этот набор полей естественно превращается в
//! `enum`: вариант сам говорит, лист это или ветвь, поэтому флаг `leaf` и
//! `union` не нужны, а невалидное состояние (лист без объекта или ветвь без
//! детей) оказывается непредставимым.
//!
//! В отличие от C++-версии, где лист хранит вектор объектов, здесь лист
//! содержит ровно один объект. Группа объектов выражается поддеревом, поэтому
//! каждый лист всегда имеет корректные границы.

use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::oriented_bounding_box::OrientedBoundingBox;
use super::ray_3d::Ray3d;
use super::sphere::Sphere;
use super::triangle_3d::Triangle3d;

/// Объект, вокруг которого можно построить ограничивающий AABB.
pub trait Bounded {
    /// Вернуть плотный AABB объекта в мировых координатах.
    fn bounds(&self) -> AxisAlignedBoundingBox3d;
}

/// Объект, с которым можно найти пересечение луча.
pub trait RayIntersectable {
    /// Вернуть расстояние `t` вдоль луча до ближайшей точки пересечения
    /// или `None`, если луч не пересекает объект.
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32>;
}

/// Узел иерархии ограничивающих объемов (BVH).
///
/// Каждый узел хранит собственный AABB: для листа — границы объекта,
/// для ветви — объединение границ всех потомков. Инвариант поддерживается
/// конструкторами [`BoundingVolumeHierarchyNode::leaf`],
/// [`BoundingVolumeHierarchyNode::branch`] и
/// [`BoundingVolumeHierarchyNode::build`].
#[derive(Debug, Clone, PartialEq)]
pub enum BoundingVolumeHierarchyNode<Shape> {
    /// Лист: ровно один объект и его AABB.
    Leaf {
        /// Объект, хранящийся в листе.
        shape: Shape,
        /// AABB объекта.
        bounds: AxisAlignedBoundingBox3d,
    },
    /// Внутренний узел: потомки и AABB, охватывающий их все.
    Branch {
        /// AABB, объединяющий границы всех потомков.
        bounds: AxisAlignedBoundingBox3d,
        /// Непустой набор потомков.
        children: Vec<BoundingVolumeHierarchyNode<Shape>>,
    },
}

impl<Shape> BoundingVolumeHierarchyNode<Shape> {
    /// AABB узла: границы объекта для листа или объединение границ
    /// потомков для ветви.
    pub fn bounds(&self) -> &AxisAlignedBoundingBox3d {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => bounds,
        }
    }

    /// Является ли узел листом.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Является ли узел внутренним (ветвью).
    pub fn is_branch(&self) -> bool {
        matches!(self, Self::Branch { .. })
    }

    /// Объект листа или `None`, если узел — ветвь.
    pub fn shape(&self) -> Option<&Shape> {
        match self {
            Self::Leaf { shape, .. } => Some(shape),
            Self::Branch { .. } => None,
        }
    }

    /// Потомки ветви. Для листа возвращается пустой срез, что позволяет
    /// единообразно обходить дерево без `match`.
    pub fn children(&self) -> &[Self] {
        match self {
            Self::Leaf { .. } => &[],
            Self::Branch { children, .. } => children,
        }
    }

    /// Обход всех узлов дерева в глубину (сначала узел, затем потомки).
    pub fn iter(&self) -> impl Iterator<Item = &Self> + '_ {
        let mut stack = vec![self];

        std::iter::from_fn(move || {
            let node = stack.pop()?;

            if let Self::Branch { children, .. } = node {
                stack.extend(children.iter());
            }

            Some(node)
        })
    }

    /// Обход всех объектов, хранящихся в листьях.
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> + '_ {
        let mut stack = vec![self];

        std::iter::from_fn(move || {
            while let Some(node) = stack.pop() {
                match node {
                    Self::Leaf { shape, .. } => return Some(shape),
                    Self::Branch { children, .. } => stack.extend(children.iter()),
                }
            }

            None
        })
    }

    /// Высота поддерева: лист имеет высоту 1, ветвь — 1 + максимальная
    /// высота потомка.
    pub fn height(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Branch { children, .. } => {
                1 + children
                    .iter()
                    .map(|child| child.height())
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    /// Количество листьев (объектов) в поддереве.
    pub fn leaves_count(&self) -> usize {
        self.shapes().count()
    }

    /// Создать ветвь по набору потомков.
    ///
    /// Возвращает `None` для пустого набора: ветвь без детей не имеет смысла
    /// и нарушила бы инвариант дерева. Для непустого набора AABB ветви
    /// автоматически вычисляется как объединение границ потомков.
    pub fn branch(children: Vec<Self>) -> Option<Self> {
        if children.is_empty() {
            return None;
        }

        let mut bounds = AxisAlignedBoundingBox3d::default();

        for child in &children {
            bounds.merge(child.bounds());
        }

        Some(Self::Branch { bounds, children })
    }

    /// Найти все объекты, AABB которых пересекается с заданной областью.
    pub fn find_shapes_in_box(&self, area: &AxisAlignedBoundingBox3d) -> Vec<&Shape> {
        let mut result = Vec::new();
        self.collect_shapes_in_box(area, &mut result);
        result
    }

    fn collect_shapes_in_box<'a>(
        &'a self,
        area: &AxisAlignedBoundingBox3d,
        result: &mut Vec<&'a Shape>,
    ) {
        // Отсечение целого поддерева одним тестом AABB.
        if !self.bounds().intersects(area) {
            return;
        }

        match self {
            Self::Leaf { shape, bounds } => {
                if bounds.intersects(area) {
                    result.push(shape);
                }
            }
            Self::Branch { children, .. } => {
                for child in children {
                    child.collect_shapes_in_box(area, result);
                }
            }
        }
    }
}

impl<Shape: Bounded> BoundingVolumeHierarchyNode<Shape> {
    /// Создать лист. Границы вычисляются по объекту, поэтому лист не может
    /// остаться без корректного AABB.
    pub fn leaf(shape: Shape) -> Self {
        let bounds = shape.bounds();
        Self::Leaf { shape, bounds }
    }

    /// Построить BVH по набору объектов (top-down, медианное разбиение).
    ///
    /// Объекты забираются во владение. На каждом шаге выбирается ось
    /// наибольшего размера общего AABB, объекты сортируются по центрам
    /// своих AABB и делятся пополам. Так дерево получается сбалансированным
    /// по количеству объектов, а не по объему.
    ///
    /// Возвращает `None`, если набор пуст.
    pub fn build<I>(shapes: I) -> Option<Self>
    where
        I: IntoIterator<Item = Shape>,
    {
        Self::build_from_vec(shapes.into_iter().collect())
    }

    fn build_from_vec(mut shapes: Vec<Shape>) -> Option<Self> {
        if shapes.is_empty() {
            return None;
        }

        if shapes.len() == 1 {
            return shapes.pop().map(Self::leaf);
        }

        // Общий AABB группы: по его размерам выбираем ось разбиения.
        let mut bounds = AxisAlignedBoundingBox3d::default();

        for shape in &shapes {
            bounds.merge(&shape.bounds());
        }

        let size = bounds.get_size();
        let axis = if size.x >= size.y && size.x >= size.z {
            0
        } else if size.y >= size.z {
            1
        } else {
            2
        };

        // Медианное разбиение: половина объектов слева, половина справа.
        // `total_cmp` корректно упорядочивает и NaN, не требуя `unwrap`.
        shapes.sort_by(|a, b| {
            let a_center = a.bounds().get_center().to_array()[axis];
            let b_center = b.bounds().get_center().to_array()[axis];
            a_center.total_cmp(&b_center)
        });

        let right_shapes = shapes.split_off(shapes.len() / 2);

        let children = vec![
            Self::build_from_vec(shapes)?,
            Self::build_from_vec(right_shapes)?,
        ];
        Self::branch(children)
    }

    /// Проверить инвариант дерева: границы каждого узла совпадают с
    /// границами объекта (для листа) или объединением границ потомков
    /// (для ветви), а каждая ветвь непуста.
    pub fn is_consistent(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        match self {
            Self::Leaf { shape, bounds } => {
                Self::bounds_are_close(bounds, &shape.bounds(), epsilon)
            }
            Self::Branch { bounds, children } => {
                if children.is_empty()
                    || !children
                        .iter()
                        .all(|child| child.is_consistent(Some(epsilon)))
                {
                    return false;
                }

                let mut merged = AxisAlignedBoundingBox3d::default();

                for child in children {
                    merged.merge(child.bounds());
                }

                Self::bounds_are_close(bounds, &merged, epsilon)
            }
        }
    }

    fn bounds_are_close(
        a: &AxisAlignedBoundingBox3d,
        b: &AxisAlignedBoundingBox3d,
        epsilon: f32,
    ) -> bool {
        (a.min_point - b.min_point).abs().max_element() <= epsilon
            && (a.max_point - b.max_point).abs().max_element() <= epsilon
    }
}

impl<Shape: RayIntersectable> BoundingVolumeHierarchyNode<Shape> {
    /// Найти ближайшее к началу луча пересечение среди всех объектов дерева.
    ///
    /// Возвращает расстояние `t` вдоль луча и ссылку на объект. Обход идёт
    /// от ближнего потомка к дальнему, а поддеревья, начинающиеся дальше
    /// уже найденного попадания, отсекаются.
    pub fn find_closest_ray_hit(
        &self,
        ray: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Option<(f32, &Shape)> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let mut best = None;

        self.search_ray(ray, epsilon, &mut best);

        best
    }

    fn search_ray<'a>(&'a self, ray: &Ray3d, epsilon: f32, best: &mut Option<(f32, &'a Shape)>) {
        let Some((t_enter, t_exit)) = self.bounds().ray_intersection_interval(ray, Some(epsilon))
        else {
            return;
        };

        // AABB целиком за началом луча.
        if t_exit < 0.0 {
            return;
        }

        // Ближняя граница узла дальше уже найденного попадания — всё
        // поддерево можно пропустить.
        if let Some((best_t, _)) = best.as_ref()
            && t_enter > *best_t
        {
            return;
        }

        match self {
            Self::Leaf { shape, .. } => {
                let Some(t) = shape.ray_intersection(ray, Some(epsilon)) else {
                    return;
                };

                let is_closer = match best.as_ref() {
                    None => true,
                    Some((best_t, _)) => t < *best_t,
                };

                if is_closer {
                    *best = Some((t, shape));
                }
            }
            Self::Branch { children, .. } => {
                // Сначала посещаем потомка, в чей AABB луч входит раньше.
                let mut ordered: Vec<(f32, &'a Self)> = children
                    .iter()
                    .filter_map(|child| {
                        let (enter, exit) = child
                            .bounds()
                            .ray_intersection_interval(ray, Some(epsilon))?;
                        (exit >= 0.0).then_some((enter, child))
                    })
                    .collect();

                ordered.sort_by(|a, b| a.0.total_cmp(&b.0));

                for (_, child) in ordered {
                    child.search_ray(ray, epsilon, best);
                }
            }
        }
    }
}

impl Bounded for AxisAlignedBoundingBox3d {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        *self
    }
}

impl Bounded for Sphere {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        let radius = glam::Vec3::splat(self.radius);
        AxisAlignedBoundingBox3d::new(self.center - radius, self.center + radius)
    }
}

impl Bounded for Triangle3d {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        AxisAlignedBoundingBox3d::from_vertices(&[self.a, self.b, self.c])
    }
}

impl Bounded for OrientedBoundingBox {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        // Полуразмер AABB по каждой мировой оси — сумма модулей проекций
        // локальных полуосей OBB на эту ось.
        let half_extents = self.orientation.x_axis.abs() * self.extents.x
            + self.orientation.y_axis.abs() * self.extents.y
            + self.orientation.z_axis.abs() * self.extents.z;

        AxisAlignedBoundingBox3d::new(self.center - half_extents, self.center + half_extents)
    }
}

impl RayIntersectable for AxisAlignedBoundingBox3d {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        let (t_enter, t_exit) = self.ray_intersection_interval(ray, maybe_epsilon)?;

        (t_exit >= 0.0).then(|| t_enter.max(0.0))
    }
}

impl RayIntersectable for Sphere {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        Sphere::ray_intersection(self, ray, maybe_epsilon)
    }
}

impl RayIntersectable for Triangle3d {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        self.get_intersection_barycentric(ray, maybe_epsilon)
            .map(|intersection| intersection.t)
    }
}

impl RayIntersectable for OrientedBoundingBox {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        OrientedBoundingBox::ray_intersection(self, ray, maybe_epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    type BvhNode = BoundingVolumeHierarchyNode<Sphere>;

    #[test]
    fn leaf_bounds_are_derived_from_shape() {
        let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 2.0);
        let node = BvhNode::leaf(sphere);

        assert!(node.is_leaf());
        assert_eq!(
            *node.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::new(-1.0, 0.0, 1.0), Vec3::new(3.0, 4.0, 5.0))
        );
        assert_eq!(node.shape(), Some(&sphere));
        assert!(node.children().is_empty());
    }

    #[test]
    fn branch_bounds_cover_all_children() {
        let left = BvhNode::leaf(Sphere::new(Vec3::new(-5.0, 0.0, 0.0), 1.0));
        let right = BvhNode::leaf(Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0));
        let branch = BvhNode::branch(vec![left, right]).unwrap();

        assert!(branch.is_branch());
        assert_eq!(
            *branch.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::new(-6.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0))
        );
        assert!(branch.is_consistent(None));
    }

    #[test]
    fn branch_requires_at_least_one_child() {
        let empty: Option<BvhNode> = BvhNode::branch(Vec::new());
        assert!(empty.is_none());
    }

    #[test]
    fn build_returns_none_for_empty_input() {
        let empty: Option<BvhNode> = BvhNode::build(Vec::new());
        assert!(empty.is_none());
    }

    #[test]
    fn build_splits_objects_into_balanced_tree() {
        let spheres: Vec<Sphere> = (0..10)
            .map(|i| Sphere::new(Vec3::new(i as f32 * 3.0, 0.0, 0.0), 0.5))
            .collect();

        let root = BvhNode::build(spheres).unwrap();

        assert_eq!(root.leaves_count(), 10);
        assert_eq!(root.shapes().count(), 10);
        assert_eq!(root.height(), 5);
        assert_eq!(root.iter().filter(|node| node.is_leaf()).count(), 10);
        assert!(
            root.iter()
                .filter(|node| node.is_branch())
                .all(|node| node.children().len() >= 2)
        );
        assert!(root.is_consistent(None));
    }

    #[test]
    fn find_shapes_in_box_returns_only_intersecting_shapes() {
        let center = Sphere::new(Vec3::ZERO, 1.0);
        let root = BvhNode::build(vec![
            Sphere::new(Vec3::new(-10.0, 0.0, 0.0), 1.0),
            center,
            Sphere::new(Vec3::new(10.0, 0.0, 0.0), 1.0),
        ])
        .unwrap();

        let area = AxisAlignedBoundingBox3d::new(Vec3::splat(-1.5), Vec3::splat(1.5));
        let found = root.find_shapes_in_box(&area);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].center, center.center);
    }

    #[test]
    fn find_closest_ray_hit_skips_farther_shapes() {
        let root = BvhNode::build(vec![
            Sphere::new(Vec3::new(0.0, 0.0, 5.0), 0.5),
            Sphere::new(Vec3::new(0.0, 0.0, 2.0), 0.5),
            Sphere::new(Vec3::new(0.0, 0.0, 9.0), 0.5),
        ])
        .unwrap();

        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        };

        let (t, shape) = root.find_closest_ray_hit(&ray, None).unwrap();

        assert!((t - 1.5).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert_eq!(shape.center, Vec3::new(0.0, 0.0, 2.0));

        // Перебор по всем листьям должен дать тот же результат.
        let brute_force = root
            .shapes()
            .filter_map(|sphere| sphere.ray_intersection(&ray, None).map(|t| (t, sphere)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();

        assert_eq!(brute_force.0, t);
        assert_eq!(brute_force.1.center, shape.center);
    }

    #[test]
    fn find_closest_ray_hit_returns_none_when_ray_misses() {
        let root = BvhNode::build(vec![Sphere::new(Vec3::new(10.0, 0.0, 0.0), 1.0)]).unwrap();
        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        };

        assert!(root.find_closest_ray_hit(&ray, None).is_none());
    }

    #[test]
    fn inconsistent_bounds_are_detected() {
        let sphere = Sphere::new(Vec3::new(10.0, 0.0, 0.0), 1.0);
        let bogus = BvhNode::Branch {
            bounds: AxisAlignedBoundingBox3d::new(Vec3::ZERO, Vec3::ONE),
            children: vec![BvhNode::leaf(sphere)],
        };

        assert!(!bogus.is_consistent(None));
    }

    #[test]
    fn aabb_ray_interval_handles_entry_exit_and_misses() {
        let bounds = AxisAlignedBoundingBox3d::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let ray = Ray3d {
            origin: Vec3::new(0.0, 0.0, -5.0),
            direction: Vec3::Z,
        };

        let (enter, exit) = bounds.ray_intersection_interval(&ray, None).unwrap();
        assert!((enter - 4.0).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((exit - 6.0).abs() < DEFAULT_GEOMETRY_EPSILON);

        // Луч, параллельный оси Z, но проходящий мимо AABB.
        let parallel_miss = Ray3d {
            origin: Vec3::new(5.0, 0.0, -5.0),
            direction: Vec3::Z,
        };
        assert!(
            bounds
                .ray_intersection_interval(&parallel_miss, None)
                .is_none()
        );

        // AABB остался позади луча.
        let behind = Ray3d {
            origin: Vec3::new(0.0, 0.0, 5.0),
            direction: Vec3::Z,
        };
        assert!(bounds.ray_intersection_interval(&behind, None).is_none());
    }

    #[test]
    fn oriented_box_and_triangle_bounds_cover_geometry() {
        let triangle = Triangle3d {
            a: Vec3::new(1.0, 0.0, 0.0),
            b: Vec3::new(0.0, 2.0, 0.0),
            c: Vec3::new(0.0, 0.0, 3.0),
        };

        assert_eq!(
            triangle.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0))
        );

        // Квадрат 2x2x2, повернутый на 45 градусов вокруг Z: по X и Y
        // мировой AABB вырастает до sqrt(2), по Z остается прежним.
        let obb = OrientedBoundingBox::new(
            Vec3::ZERO,
            Vec3::ONE,
            glam::Mat3::from_rotation_z(std::f32::consts::FRAC_PI_4),
        );
        let obb_bounds = obb.bounds();
        let sqrt_2 = std::f32::consts::SQRT_2;

        assert!((obb_bounds.max_point.x - sqrt_2).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((obb_bounds.max_point.y - sqrt_2).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((obb_bounds.max_point.z - 1.0).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((obb_bounds.min_point.x + sqrt_2).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((obb_bounds.min_point.y + sqrt_2).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert!((obb_bounds.min_point.z + 1.0).abs() < DEFAULT_GEOMETRY_EPSILON);
    }

    #[test]
    fn aabb_ray_interval_reports_negative_entry_inside_box() {
        let bounds = AxisAlignedBoundingBox3d::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        };

        let (enter, exit) = bounds.ray_intersection_interval(&ray, None).unwrap();

        assert!(enter < 0.0);
        assert!((exit - 1.0).abs() < DEFAULT_GEOMETRY_EPSILON);
    }
}
