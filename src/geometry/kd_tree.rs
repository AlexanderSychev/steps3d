//! k-мерное дерево (kD-дерево) для трехмерных объектов
//!
//! # Где хранится секущая плоскость
//!
//! В C++-версии `plane p` — поле каждого `kDNode`, хотя осмысленно оно только
//! для внутренних узлов: у листа плоскость не используется. В Rust это
//! естественно выражается вариантом [`KdTree::Branch`]: плоскость есть ровно
//! там, где есть разбиение, а у [`KdTree::Leaf`] её невозможно даже прочитать.
//!
//! Плоскость хранится в родительском узле, а не в потомках. Потомки не должны
//! её копировать: принадлежность к положительной или отрицательной стороне
//! задаётся самим полем `positive` / `negative`. При обходе плоскость нужна
//! только в момент выбора, в какое поддерево идти раньше.
//!
//! # Объекты, пересекающие плоскость
//!
//! В книге такой объект кладётся указателем сразу в оба поддерева. В Rust
//! один объект нельзя одновременно владеть в двух местах: понадобился бы
//! `Rc`/`Arc` или клонирование. Здесь выбран вариант BSP-дерева: объект,
//! пересекающий плоскость, хранится в самом узле (`straddling_shapes`),
//! поэтому каждый объект принадлежит дереву ровно один раз, а результаты
//! поиска не содержат дубликатов.
//!
//! # Построение
//!
//! Как и в книге, плоскость проходит через центр AABB узла перпендикулярно
//! самой длинной его оси. Объекты классифицируются по своим AABB:
//! целиком положительные и целиком отрицательные уходят в соответствующие
//! поддеревья, пересекающие плоскость остаются в узле. Если разбиение
//! не разделило объекты, узел становится листом.

use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::plane::{Plane, PlaneSide};
use super::ray_3d::Ray3d;
use super::traits::{Bounded, RayIntersectable};
use glam::Vec3;

/// Сколько объектов по умолчанию может лежать в листе.
pub const DEFAULT_KD_LEAF_CAPACITY: usize = 4;

/// Узел kD-дерева.
#[derive(Debug, Clone, PartialEq)]
pub enum KdTree<Shape> {
    /// Лист: несколько объектов и AABB, охватывающий их все.
    Leaf {
        /// Объекты листа.
        shapes: Vec<Shape>,
        /// AABB всех объектов листа.
        bounds: AxisAlignedBoundingBox3d,
    },
    /// Внутренний узел: секущая плоскость и два поддерева.
    Branch {
        /// AABB всего поддерева.
        bounds: AxisAlignedBoundingBox3d,
        /// Секущая плоскость: `normal · point + d = 0`.
        split_plane: Plane,
        /// Объекты, которые пересекают плоскость и потому не могут быть
        /// отнесены ни к одной стороне.
        straddling_shapes: Vec<Shape>,
        /// Поддерево со стороны положительного полупространства (по нормали).
        positive: Box<KdTree<Shape>>,
        /// Поддерево со стороны отрицательного полупространства.
        negative: Box<KdTree<Shape>>,
    },
}

impl<Shape> KdTree<Shape> {
    /// AABB узла.
    pub fn bounds(&self) -> &AxisAlignedBoundingBox3d {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => bounds,
        }
    }

    /// Является ли узел листом.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Является ли узел внутренним.
    pub fn is_branch(&self) -> bool {
        matches!(self, Self::Branch { .. })
    }

    /// Секущая плоскость внутреннего узла.
    ///
    /// У листа плоскости нет, поэтому возвращается `None`.
    pub fn split_plane(&self) -> Option<&Plane> {
        match self {
            Self::Leaf { .. } => None,
            Self::Branch { split_plane, .. } => Some(split_plane),
        }
    }

    /// Объекты, пересекающие секущую плоскость этого узла.
    ///
    /// Для листа возвращается пустой срез.
    pub fn straddling_shapes(&self) -> &[Shape] {
        match self {
            Self::Leaf { .. } => &[],
            Self::Branch {
                straddling_shapes, ..
            } => straddling_shapes,
        }
    }

    /// Обход всех узлов дерева в глубину (сначала узел, затем потомки).
    pub fn iter(&self) -> impl Iterator<Item = &Self> + '_ {
        let mut stack = vec![self];

        std::iter::from_fn(move || {
            let node = stack.pop()?;

            if let Self::Branch {
                positive, negative, ..
            } = node
            {
                stack.push(negative.as_ref());
                stack.push(positive.as_ref());
            }

            Some(node)
        })
    }

    /// Обход всех объектов дерева.
    ///
    /// Объекты из `straddling_shapes` встречаются ровно один раз — в том узле,
    /// где они лежат.
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> + '_ {
        let mut nodes = vec![self];
        let mut shapes = Vec::new();

        std::iter::from_fn(move || {
            loop {
                if let Some(shape) = shapes.pop() {
                    return Some(shape);
                }

                match nodes.pop()? {
                    Self::Leaf {
                        shapes: leaf_shapes,
                        ..
                    } => {
                        shapes.extend(leaf_shapes.iter().rev());
                    }
                    Self::Branch {
                        straddling_shapes,
                        positive,
                        negative,
                        ..
                    } => {
                        shapes.extend(straddling_shapes.iter().rev());
                        nodes.push(negative.as_ref());
                        nodes.push(positive.as_ref());
                    }
                }
            }
        })
    }

    /// Высота поддерева: лист имеет высоту 1.
    pub fn height(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Branch {
                positive, negative, ..
            } => 1 + positive.height().max(negative.height()),
        }
    }

    /// Количество листьев в поддереве.
    pub fn leaves_count(&self) -> usize {
        self.iter().filter(|node| node.is_leaf()).count()
    }
}

impl<Shape: Bounded> KdTree<Shape> {
    /// Создать лист по непустому набору объектов.
    ///
    /// AABB листа вычисляется как объединение AABB объектов. Для пустого
    /// набора возвращается `None`: лист без объектов не имеет смысла.
    pub fn leaf(shapes: Vec<Shape>) -> Option<Self> {
        if shapes.is_empty() {
            return None;
        }

        Some(Self::Leaf {
            bounds: Self::merged_bounds(&shapes),
            shapes,
        })
    }

    /// Создать внутренний узел.
    ///
    /// `straddling_shapes` — объекты, пересекающие `split_plane`; они остаются
    /// в этом узле. AABB узла вычисляется автоматически как объединение
    /// границ объектов узла и обоих поддеревьев.
    pub fn branch(
        split_plane: Plane,
        straddling_shapes: Vec<Shape>,
        positive: Self,
        negative: Self,
    ) -> Self {
        let mut bounds = Self::merged_bounds(&straddling_shapes);
        bounds.merge(positive.bounds());
        bounds.merge(negative.bounds());

        Self::Branch {
            bounds,
            split_plane,
            straddling_shapes,
            positive: Box::new(positive),
            negative: Box::new(negative),
        }
    }

    /// Построить kD-дерево с ёмкостью листа по умолчанию.
    ///
    /// Возвращает `None`, если набор объектов пуст.
    pub fn build<I>(shapes: I) -> Option<Self>
    where
        I: IntoIterator<Item = Shape>,
    {
        Self::build_with_leaf_capacity(shapes, DEFAULT_KD_LEAF_CAPACITY)
    }

    /// Построить kD-дерево с заданной максимальной ёмкостью листа.
    ///
    /// Ёмкость меньше единицы приводится к единице.
    pub fn build_with_leaf_capacity<I>(shapes: I, max_shapes_per_leaf: usize) -> Option<Self>
    where
        I: IntoIterator<Item = Shape>,
    {
        Self::build_recursive(shapes.into_iter().collect(), max_shapes_per_leaf.max(1))
    }

    fn build_recursive(shapes: Vec<Shape>, max_shapes_per_leaf: usize) -> Option<Self> {
        if shapes.is_empty() {
            return None;
        }

        if shapes.len() <= max_shapes_per_leaf {
            return Self::leaf(shapes);
        }

        let mut bounds = AxisAlignedBoundingBox3d::default();

        for shape in &shapes {
            bounds.merge(&shape.bounds());
        }

        let split_plane = Self::split_plane_for_bounds(&bounds);

        let mut positive = Vec::new();
        let mut negative = Vec::new();
        let mut straddling_shapes = Vec::new();

        for shape in shapes {
            match split_plane.classify_aabb(&shape.bounds(), Some(DEFAULT_GEOMETRY_EPSILON)) {
                PlaneSide::InPositive => positive.push(shape),
                PlaneSide::InNegative => negative.push(shape),
                PlaneSide::Intersecting => straddling_shapes.push(shape),
            }
        }

        // Как и в книге: если плоскость не разделила объекты, узел остаётся
        // листом, даже если объектов больше ёмкости. Иначе получилось бы
        // бесконечное разбиение.
        if positive.is_empty() || negative.is_empty() {
            let mut all = positive;
            all.extend(negative);
            all.extend(straddling_shapes);
            return Self::leaf(all);
        }

        let positive = Self::build_recursive(positive, max_shapes_per_leaf)?;
        let negative = Self::build_recursive(negative, max_shapes_per_leaf)?;

        Some(Self::branch(
            split_plane,
            straddling_shapes,
            positive,
            negative,
        ))
    }

    /// Плоскость через центр AABB перпендикулярно самой длинной его оси.
    fn split_plane_for_bounds(bounds: &AxisAlignedBoundingBox3d) -> Plane {
        let size = bounds.get_size();
        let normal = if size.x >= size.y && size.x >= size.z {
            Vec3::X
        } else if size.y >= size.z {
            Vec3::Y
        } else {
            Vec3::Z
        };

        Plane::from_point_and_normal(bounds.get_center(), normal)
    }

    fn merged_bounds(shapes: &[Shape]) -> AxisAlignedBoundingBox3d {
        let mut bounds = AxisAlignedBoundingBox3d::default();

        for shape in shapes {
            bounds.merge(&shape.bounds());
        }

        bounds
    }

    /// Проверить инвариант дерева.
    ///
    /// Для листа: границы совпадают с объединением границ объектов.
    /// Для ветви: нормаль плоскости единичная, все `straddling_shapes`
    /// действительно пересекают плоскость, а границы узла совпадают с
    /// объединением границ объектов узла и обоих поддеревьев.
    pub fn is_consistent(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        match self {
            Self::Leaf { shapes, bounds } => {
                !shapes.is_empty()
                    && Self::bounds_are_close(bounds, &Self::merged_bounds(shapes), epsilon)
            }
            Self::Branch {
                bounds,
                split_plane,
                straddling_shapes,
                positive,
                negative,
            } => {
                if (split_plane.normal.length() - 1.0).abs() > epsilon {
                    return false;
                }

                let all_straddle = straddling_shapes.iter().all(|shape| {
                    split_plane.classify_aabb(&shape.bounds(), Some(epsilon))
                        == PlaneSide::Intersecting
                });

                if !all_straddle
                    || !positive.is_consistent(Some(epsilon))
                    || !negative.is_consistent(Some(epsilon))
                {
                    return false;
                }

                let mut merged = Self::merged_bounds(straddling_shapes);
                merged.merge(positive.bounds());
                merged.merge(negative.bounds());

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
        // Целиком не попавший в область узел можно не обходить.
        if !self.bounds().intersects(area) {
            return;
        }

        match self {
            Self::Leaf { shapes, .. } => {
                for shape in shapes {
                    if shape.bounds().intersects(area) {
                        result.push(shape);
                    }
                }
            }
            Self::Branch {
                straddling_shapes,
                positive,
                negative,
                ..
            } => {
                for shape in straddling_shapes {
                    if shape.bounds().intersects(area) {
                        result.push(shape);
                    }
                }

                positive.collect_shapes_in_box(area, result);
                negative.collect_shapes_in_box(area, result);
            }
        }
    }
}

impl<Shape: RayIntersectable> KdTree<Shape> {
    /// Найти ближайшее к началу луча пересечение среди всех объектов дерева.
    ///
    /// В ветви сначала проверяются объекты, пересекающие плоскость, затем
    /// поддерево с той стороны, в которой находится начало луча. Порядок
    /// важен для отсечения: если ближнее поддерево уже дало попадание,
    /// дальнее можно не обходить, когда его AABB начинается позже.
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

        if t_exit < 0.0 {
            return;
        }

        // Всё поддерево начинается дальше уже найденного попадания.
        if let Some((best_t, _)) = best.as_ref()
            && t_enter > *best_t
        {
            return;
        }

        match self {
            Self::Leaf { shapes, .. } => {
                for shape in shapes {
                    Self::consider_hit(shape, ray, epsilon, best);
                }
            }
            Self::Branch {
                split_plane,
                straddling_shapes,
                positive,
                negative,
                ..
            } => {
                for shape in straddling_shapes {
                    Self::consider_hit(shape, ray, epsilon, best);
                }

                // Ближе то поддерево, в котором находится начало луча.
                // Если луч стартует на плоскости, сторону выбираем по
                // направлению движения.
                let origin_side = split_plane.signed_distance_to_point(ray.origin);
                let positive_is_near = if origin_side.abs() > epsilon {
                    origin_side > 0.0
                } else {
                    ray.direction.dot(split_plane.normal) >= 0.0
                };

                let (near, far) = if positive_is_near {
                    (positive.as_ref(), negative.as_ref())
                } else {
                    (negative.as_ref(), positive.as_ref())
                };

                near.search_ray(ray, epsilon, best);
                far.search_ray(ray, epsilon, best);
            }
        }
    }

    fn consider_hit<'a>(
        shape: &'a Shape,
        ray: &Ray3d,
        epsilon: f32,
        best: &mut Option<(f32, &'a Shape)>,
    ) {
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
}

#[cfg(test)]
mod tests {
    use super::super::sphere::Sphere;
    use super::*;
    use glam::Vec3;

    fn sphere(x: f32, radius: f32) -> Sphere {
        Sphere::new(Vec3::new(x, 0.0, 0.0), radius)
    }

    #[test]
    fn leaf_has_no_split_plane() {
        let a = sphere(-2.0, 0.5);
        let b = sphere(2.0, 0.5);
        let tree = KdTree::leaf(vec![a, b]).unwrap();

        assert!(tree.is_leaf());
        assert!(tree.split_plane().is_none());
        assert!(tree.straddling_shapes().is_empty());
        assert_eq!(tree.shapes().count(), 2);
        assert_eq!(
            *tree.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::new(-2.5, -0.5, -0.5), Vec3::new(2.5, 0.5, 0.5))
        );
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn branch_stores_plane_and_children() {
        let plane = Plane::from_point_and_normal(Vec3::ZERO, Vec3::X);
        let straddler = sphere(0.0, 1.0);
        let positive = KdTree::leaf(vec![sphere(5.0, 1.0)]).unwrap();
        let negative = KdTree::leaf(vec![sphere(-5.0, 1.0)]).unwrap();

        let tree = KdTree::branch(plane, vec![straddler], positive, negative);

        assert!(tree.is_branch());
        assert_eq!(tree.split_plane(), Some(&plane));
        assert_eq!(tree.straddling_shapes(), &[straddler]);
        assert_eq!(
            *tree.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::new(-6.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0))
        );
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn empty_inputs_return_none() {
        let leaf: Option<KdTree<Sphere>> = KdTree::leaf(Vec::new());
        let tree: Option<KdTree<Sphere>> = KdTree::build_with_leaf_capacity(Vec::new(), 4);

        assert!(leaf.is_none());
        assert!(tree.is_none());
    }

    #[test]
    fn build_partitions_objects_and_creates_planes() {
        let spheres: Vec<Sphere> = (0..10)
            .map(|i| sphere(i as f32 * 3.0 - 13.5, 0.75))
            .collect();

        let tree = KdTree::build_with_leaf_capacity(spheres, 2).unwrap();

        // Каждый объект хранится ровно один раз.
        assert_eq!(tree.shapes().count(), 10);
        assert!(tree.height() > 1);
        assert!(tree.iter().any(|node| node.is_branch()));
        assert!(tree.iter().all(|node| node.is_consistent(None)));
        assert_eq!(
            tree.leaves_count(),
            tree.iter().filter(|n| n.is_leaf()).count()
        );
    }

    #[test]
    fn straddling_shape_is_stored_once_in_branch() {
        let big = sphere(0.0, 10.0);
        let tree =
            KdTree::build_with_leaf_capacity(vec![sphere(-20.0, 1.0), big, sphere(20.0, 1.0)], 1)
                .unwrap();

        // Большая сфера пересекает плоскость x = 0 и остаётся в корне.
        assert_eq!(tree.shapes().count(), 3);
        assert_eq!(tree.straddling_shapes(), &[big]);
        assert!(tree.split_plane().is_some());
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn find_shapes_in_box_finds_straddling_shape_once() {
        let big = sphere(0.0, 10.0);
        let tree =
            KdTree::build_with_leaf_capacity(vec![sphere(-20.0, 1.0), big, sphere(20.0, 1.0)], 1)
                .unwrap();

        let area = AxisAlignedBoundingBox3d::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let found = tree.find_shapes_in_box(&area);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].center, Vec3::ZERO);
    }

    #[test]
    fn find_closest_ray_hit_matches_brute_force() {
        let tree = KdTree::build_with_leaf_capacity(
            vec![
                sphere(-20.0, 1.0),
                sphere(0.0, 10.0),
                sphere(20.0, 1.0),
                sphere(0.0, 2.0),
            ],
            1,
        )
        .unwrap();

        let ray = Ray3d {
            origin: Vec3::new(0.0, 0.0, -50.0),
            direction: Vec3::Z,
        };

        let (t, shape) = tree.find_closest_ray_hit(&ray, None).unwrap();

        // Ближе всех сфера радиуса 10 с центром в начале координат.
        assert!((t - 40.0).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert_eq!(shape.center, Vec3::ZERO);

        let brute_force = tree
            .shapes()
            .filter_map(|shape| shape.ray_intersection(&ray, None).map(|t| (t, shape)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();

        assert_eq!(brute_force.0, t);
        assert_eq!(brute_force.1.center, shape.center);
    }

    #[test]
    fn find_closest_ray_hit_returns_none_when_ray_misses() {
        let tree = KdTree::build(vec![sphere(10.0, 1.0)]).unwrap();
        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        };

        assert!(tree.find_closest_ray_hit(&ray, None).is_none());
    }

    #[test]
    fn inconsistent_straddling_shape_is_detected() {
        let plane = Plane::from_point_and_normal(Vec3::ZERO, Vec3::X);
        let bogus = KdTree::<Sphere>::Branch {
            bounds: AxisAlignedBoundingBox3d::new(Vec3::splat(-10.0), Vec3::splat(10.0)),
            split_plane: plane,
            // Сфера целиком в положительной полуплоскости, а не пересекает её.
            straddling_shapes: vec![sphere(5.0, 1.0)],
            positive: Box::new(KdTree::leaf(vec![sphere(5.0, 1.0)]).unwrap()),
            negative: Box::new(KdTree::leaf(vec![sphere(-5.0, 1.0)]).unwrap()),
        };

        assert!(!bogus.is_consistent(None));
    }
}
