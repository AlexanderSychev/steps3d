//! BSP-дерево (Binary Space Partitioning)
//!
//! BSP-дерево рекурсивно делит пространство плоскостями, взятыми из
//! многоугольников сцены. Каждый узел хранит:
//!
//! - многоугольник-рассекатель (`splitter`);
//! - его плоскость (`split_plane`);
//! - два поддерева: со стороны нормали (`positive`) и с противоположной
//!   стороны (`negative`).
//!
//! В отличие от C++-версии книги, где `pos` и `neg` — сырые указатели и могут
//! быть `nullptr`, здесь это `Option<Box<BspNode>>`. Один потомок может
//! отсутствовать: если все оставшиеся многоугольники оказались по одну
//! сторону плоскости, дерево продолжается только в эту сторону. Так ни один
//! многоугольник не теряется (в книжном примере ветка с пустой стороной
//! просто не строилась, а многоугольники из непустой стороны пропадали).
//!
//! Основное применение — обход в порядке от дальних многоугольников к близким
//! (алгоритм художника), который использовался в Doom и Quake. Для этого
//! достаточно знать, с какой стороны плоскости находится наблюдатель:
//! дальнюю сторону обходим первой, затем рисуем рассекатель, затем ближнюю.

use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::plane::{Plane, PlaneSide};
use super::polygon_3d::{Polygon3d, PolygonPlaneSide};
use glam::Vec3;

/// Узел BSP-дерева.
#[derive(Debug, Clone, PartialEq)]
pub struct BspNode {
    /// AABB всех многоугольников поддерева, включая рассекатель.
    bounds: AxisAlignedBoundingBox3d,
    /// Многоугольник, плоскость которого делит пространство в этом узле.
    splitter: Polygon3d,
    /// Плоскость рассекателя (хранится отдельно, как `plane p` в C++).
    split_plane: Plane,
    /// Поддерево со стороны нормали плоскости.
    positive: Option<Box<BspNode>>,
    /// Поддерево с противоположной стороны.
    negative: Option<Box<BspNode>>,
}

impl BspNode {
    /// Создать узел по рассекателю и готовым поддеревьям.
    ///
    /// Плоскость берётся из многоугольника, а AABB вычисляется как объединение
    /// границ рассекателя и обоих поддеревьев.
    pub fn new(splitter: Polygon3d, positive: Option<BspNode>, negative: Option<BspNode>) -> Self {
        let mut bounds = splitter.bounds();

        if let Some(child) = &positive {
            bounds.merge(child.bounds());
        }

        if let Some(child) = &negative {
            bounds.merge(child.bounds());
        }

        Self {
            bounds,
            split_plane: *splitter.plane(),
            splitter,
            positive: positive.map(Box::new),
            negative: negative.map(Box::new),
        }
    }

    /// Построить BSP-дерево по набору многоугольников.
    ///
    /// Возвращает `None` для пустого набора. Эвристика выбора рассекателя
    /// простая и наглядная: минимизируем число разрезаний, затем избегаем
    /// односторонних разбиений, затем балансируем количество многоугольников.
    /// Это не оптимизированный вариант Doom, а учебная версия.
    pub fn build(polygons: Vec<Polygon3d>) -> Option<Self> {
        Self::build_recursive(polygons, DEFAULT_GEOMETRY_EPSILON)
    }

    fn build_recursive(polygons: Vec<Polygon3d>, epsilon: f32) -> Option<Self> {
        if polygons.is_empty() {
            return None;
        }

        let splitter_index = Self::choose_splitter(&polygons, epsilon);
        let mut remaining = polygons;
        let splitter = remaining.swap_remove(splitter_index);
        let split_plane = *splitter.plane();

        let mut positive = Vec::new();
        let mut negative = Vec::new();

        for polygon in remaining {
            match polygon.classify(&split_plane, Some(epsilon)) {
                // Как и в книге, многоугольники, лежащие в плоскости
                // рассекателя, уходят в положительное поддерево.
                PolygonPlaneSide::InPositive | PolygonPlaneSide::InPlane => positive.push(polygon),
                PolygonPlaneSide::InNegative => negative.push(polygon),
                PolygonPlaneSide::Intersecting => {
                    let split = polygon.split(&split_plane, Some(epsilon));

                    // На всякий случай не теряем геометрию, если разрезание
                    // выродилось в пустые части.
                    if split.positive.is_none() && split.negative.is_none() {
                        positive.push(polygon);
                    } else {
                        if let Some(fragment) = split.positive {
                            positive.push(fragment);
                        }

                        if let Some(fragment) = split.negative {
                            negative.push(fragment);
                        }
                    }
                }
            }
        }

        let positive = Self::build_recursive(positive, epsilon);
        let negative = Self::build_recursive(negative, epsilon);

        Some(Self::new(splitter, positive, negative))
    }

    /// Выбрать многоугольник-рассекатель.
    ///
    /// Оценка: каждое разрезание штрафуется сильнее всего, одностороннее
    /// разбиение — сильнее дисбаланса. При равных оценках побеждает первый
    /// кандидат, поэтому результат детерминирован.
    fn choose_splitter(polygons: &[Polygon3d], epsilon: f32) -> usize {
        let mut best_index = 0;
        let mut best_score = usize::MAX;

        for (index, candidate) in polygons.iter().enumerate() {
            let plane = *candidate.plane();
            let mut positive: usize = 0;
            let mut negative: usize = 0;
            let mut spanning: usize = 0;

            for (other_index, polygon) in polygons.iter().enumerate() {
                if index == other_index {
                    continue;
                }

                match polygon.classify(&plane, Some(epsilon)) {
                    PolygonPlaneSide::InPositive | PolygonPlaneSide::InPlane => positive += 1,
                    PolygonPlaneSide::InNegative => negative += 1,
                    PolygonPlaneSide::Intersecting => spanning += 1,
                }
            }

            let one_sided = positive == 0 || negative == 0;
            let score =
                spanning * 10 + if one_sided { 100 } else { 0 } + positive.abs_diff(negative);

            if score < best_score {
                best_score = score;
                best_index = index;
            }
        }

        best_index
    }

    /// AABB всего поддерева.
    pub fn bounds(&self) -> &AxisAlignedBoundingBox3d {
        &self.bounds
    }

    /// Многоугольник-рассекатель узла.
    pub fn splitter(&self) -> &Polygon3d {
        &self.splitter
    }

    /// Плоскость рассекателя.
    pub fn split_plane(&self) -> &Plane {
        &self.split_plane
    }

    /// Положительное поддерево (со стороны нормали плоскости).
    pub fn positive(&self) -> Option<&BspNode> {
        self.positive.as_deref()
    }

    /// Отрицательное поддерево.
    pub fn negative(&self) -> Option<&BspNode> {
        self.negative.as_deref()
    }

    /// Узел без потомков.
    pub fn is_leaf(&self) -> bool {
        self.positive.is_none() && self.negative.is_none()
    }

    /// Высота поддерева: узел без потомков имеет высоту 1.
    pub fn height(&self) -> usize {
        let positive_height = self.positive.as_ref().map_or(0, |node| node.height());
        let negative_height = self.negative.as_ref().map_or(0, |node| node.height());

        1 + positive_height.max(negative_height)
    }

    /// Обход узлов в глубину (сначала узел, затем потомки).
    pub fn iter(&self) -> impl Iterator<Item = &Self> + '_ {
        let mut stack = vec![self];

        std::iter::from_fn(move || {
            let node = stack.pop()?;

            if let Some(positive) = &node.positive {
                stack.push(positive.as_ref());
            }

            if let Some(negative) = &node.negative {
                stack.push(negative.as_ref());
            }

            Some(node)
        })
    }

    /// Обход всех многоугольников, включая части, полученные при разрезании.
    pub fn polygons(&self) -> impl Iterator<Item = &Polygon3d> + '_ {
        self.iter().map(|node| node.splitter())
    }

    /// Количество листьев (узлов без потомков).
    pub fn leaves_count(&self) -> usize {
        self.iter().filter(|node| node.is_leaf()).count()
    }

    /// Найти узел, в чью область попадает точка.
    ///
    /// Спускаемся по стороне плоскости, в которой лежит точка, пока есть
    /// подходящий потомок. Если с этой стороны потомка нет, возвращаем
    /// текущий узел: точка находится в пустой половине пространства.
    pub fn find_leaf_for_point(&self, point: Vec3) -> &BspNode {
        let mut node = self;
        let epsilon = DEFAULT_GEOMETRY_EPSILON;

        loop {
            let distance = node.split_plane.signed_distance_to_point(point);

            let next = if distance > epsilon {
                node.positive.as_deref()
            } else if distance < -epsilon {
                node.negative.as_deref()
            } else {
                // Точка лежит на плоскости — подходит любая существующая сторона.
                node.positive.as_deref().or(node.negative.as_deref())
            };

            match next {
                Some(child) => node = child,
                None => return node,
            }
        }
    }

    /// Обход от дальних многоугольников к ближним (алгоритм художника).
    ///
    /// Если наблюдатель находится со стороны нормали, сначала обходится
    /// отрицательное поддерево (оно дальше), затем рисуется рассекатель и
    /// только потом положительное поддерево. Если наблюдатель с другой
    /// стороны — порядок зеркальный.
    pub fn traverse_back_to_front<F>(&self, observer: Vec3, visit: &mut F)
    where
        F: FnMut(&Polygon3d),
    {
        let observer_is_positive = self.split_plane.signed_distance_to_point(observer) >= 0.0;

        let (near, far) = if observer_is_positive {
            (self.positive(), self.negative())
        } else {
            (self.negative(), self.positive())
        };

        if let Some(far) = far {
            far.traverse_back_to_front(observer, &mut *visit);
        }

        visit(&self.splitter);

        if let Some(near) = near {
            near.traverse_back_to_front(observer, &mut *visit);
        }
    }

    /// Обход от ближних многоугольников к дальним.
    ///
    /// Полезен для отсечения: если ближний многоугольник перекрыл экран,
    /// дальние можно не рисовать.
    pub fn traverse_front_to_back<F>(&self, observer: Vec3, visit: &mut F)
    where
        F: FnMut(&Polygon3d),
    {
        let observer_is_positive = self.split_plane.signed_distance_to_point(observer) >= 0.0;

        let (near, far) = if observer_is_positive {
            (self.positive(), self.negative())
        } else {
            (self.negative(), self.positive())
        };

        if let Some(near) = near {
            near.traverse_front_to_back(observer, &mut *visit);
        }

        visit(&self.splitter);

        if let Some(far) = far {
            far.traverse_front_to_back(observer, &mut *visit);
        }
    }

    /// Проверить инвариант узла и всех потомков.
    ///
    /// Проверяется, что плоскость совпадает с плоскостью рассекателя,
    /// AABB равен объединению границ, а поддеревья не лежат целиком не с
    /// той стороны плоскости.
    pub fn is_consistent(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        if (self.split_plane.normal.length() - 1.0).abs() > epsilon
            || !planes_are_close(&self.split_plane, self.splitter.plane(), epsilon)
        {
            return false;
        }

        let mut merged = self.splitter.bounds();

        if let Some(positive) = &self.positive {
            merged.merge(positive.bounds());
        }

        if let Some(negative) = &self.negative {
            merged.merge(negative.bounds());
        }

        if !bounds_are_close(&self.bounds, &merged, epsilon) {
            return false;
        }

        if let Some(positive) = &self.positive
            && (self
                .split_plane
                .classify_aabb(positive.bounds(), Some(epsilon))
                == PlaneSide::InNegative
                || !positive.is_consistent(Some(epsilon)))
        {
            return false;
        }

        if let Some(negative) = &self.negative
            && (self
                .split_plane
                .classify_aabb(negative.bounds(), Some(epsilon))
                == PlaneSide::InPositive
                || !negative.is_consistent(Some(epsilon)))
        {
            return false;
        }

        true
    }
}

fn planes_are_close(a: &Plane, b: &Plane, epsilon: f32) -> bool {
    (a.normal - b.normal).abs().max_element() <= epsilon && (a.d - b.d).abs() <= epsilon
}

fn bounds_are_close(
    a: &AxisAlignedBoundingBox3d,
    b: &AxisAlignedBoundingBox3d,
    epsilon: f32,
) -> bool {
    (a.min_point - b.min_point).abs().max_element() <= epsilon
        && (a.max_point - b.max_point).abs().max_element() <= epsilon
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn quad(z: f32) -> Polygon3d {
        Polygon3d::new(
            vec![
                Vec3::new(-3.0, -2.0, z),
                Vec3::new(3.0, -2.0, z),
                Vec3::new(3.0, 2.0, z),
                Vec3::new(-3.0, 2.0, z),
            ],
            None,
        )
        .unwrap()
    }

    /// Наклонный многоугольник, пересекающий плоскость z = 0.
    fn slanted() -> Polygon3d {
        Polygon3d::new(
            vec![
                Vec3::new(-2.0, -2.0, -4.0),
                Vec3::new(2.0, -2.0, 4.0),
                Vec3::new(2.0, 2.0, 4.0),
                Vec3::new(-2.0, 2.0, -4.0),
            ],
            None,
        )
        .unwrap()
    }

    #[test]
    fn single_polygon_becomes_leaf() {
        let tree = BspNode::build(vec![quad(0.0)]).unwrap();

        assert!(tree.is_leaf());
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.polygons().count(), 1);
        assert_eq!(tree.leaves_count(), 1);
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn back_to_front_order_depends_on_observer_side() {
        let tree = BspNode::build(vec![quad(0.0), quad(5.0)]).unwrap();

        let mut from_behind = Vec::new();
        tree.traverse_back_to_front(Vec3::new(0.0, 0.0, -10.0), &mut |polygon| {
            from_behind.push(polygon.center().z);
        });

        // Дальний многоугольник (z = 5) должен быть нарисован первым.
        assert_eq!(from_behind, vec![5.0, 0.0]);

        let mut from_front = Vec::new();
        tree.traverse_back_to_front(Vec3::new(0.0, 0.0, 10.0), &mut |polygon| {
            from_front.push(polygon.center().z);
        });

        // Наблюдатель спереди: дальним стал z = 0.
        assert_eq!(from_front, vec![0.0, 5.0]);
    }

    #[test]
    fn front_to_back_is_reverse_of_back_to_front() {
        let tree = BspNode::build(vec![quad(0.0), quad(5.0)]).unwrap();
        let observer = Vec3::new(0.0, 0.0, -10.0);

        let mut back_to_front = Vec::new();
        tree.traverse_back_to_front(observer, &mut |polygon| {
            back_to_front.push(polygon.center().z);
        });

        let mut front_to_back = Vec::new();
        tree.traverse_front_to_back(observer, &mut |polygon| {
            front_to_back.push(polygon.center().z);
        });

        back_to_front.reverse();
        assert_eq!(back_to_front, front_to_back);
    }

    #[test]
    fn spanning_polygon_is_split_into_fragments() {
        let tree = BspNode::build(vec![quad(0.0), slanted()]).unwrap();

        // Исходно было 2 многоугольника, наклонный разрезан на две части.
        assert!(tree.polygons().count() >= 3);
        assert!(tree.is_consistent(None));

        let mut visited = 0;
        tree.traverse_back_to_front(Vec3::ZERO, &mut |_| visited += 1);
        assert_eq!(visited, tree.polygons().count());
    }

    #[test]
    fn coplanar_polygons_are_kept() {
        let small = Polygon3d::new(
            vec![
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(-1.0, 1.0, 0.0),
            ],
            None,
        )
        .unwrap();

        let tree = BspNode::build(vec![quad(0.0), small]).unwrap();

        assert_eq!(tree.polygons().count(), 2);
        assert_eq!(tree.height(), 2);
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn find_leaf_for_point_descends_to_the_right_side() {
        let tree = BspNode::build(vec![quad(0.0), quad(5.0)]).unwrap();

        let front_leaf = tree.find_leaf_for_point(Vec3::new(0.0, 0.0, 10.0));
        assert_eq!(front_leaf.splitter().center().z, 5.0);

        let back_leaf = tree.find_leaf_for_point(Vec3::new(0.0, 0.0, -10.0));
        assert_eq!(back_leaf.splitter().center().z, 0.0);
    }

    #[test]
    fn empty_input_returns_none() {
        let tree: Option<BspNode> = BspNode::build(Vec::new());
        assert!(tree.is_none());
    }
}
