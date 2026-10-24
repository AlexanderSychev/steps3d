//! R-дерево — динамическая пространственная иерархия AABB
//!
//! R-дерево похоже на BVH, но строится не один раз по готовому набору, а
//! поддерживает вставку объектов. Каждый узел хранит от `min_entries` до
//! `max_entries` записей:
//!
//! - лист — объекты и их AABB;
//! - внутренний узел — дочерние узлы и AABB каждого из них.
//!
//! В C++-версии книги узел использует `union` из указателя на узел и указателя
//! на объект, массив `Record*` и `parent` для подъёма при разбиении. В Rust
//! вместо `union` используются варианты [`RTreeNode`], вместо сырых указателей
//! — владеющие `Box`/`Vec`, а `parent` не нужен: рекурсивная вставка возвращает
//! нового соседа наверх по стеку вызовов.
//!
//! # Вставка
//!
//! 1. Спускаемся к листу, выбирая на каждом уровне ребёнка, для которого
//!    расширение AABB минимально (при равенстве — с меньшей площадью).
//! 2. Добавляем объект в лист.
//! 3. Если лист переполнен, делим записи на две группы квадратичным
//!    разбиением Гуттмана и возвращаем новую группу наверх.
//! 4. Родитель добавляет новую ветвь; если переполнен он — разбивается сам.
//! 5. Если разбился корень, создаётся новый корень с двумя детьми.
//!
//! Удаление объектов в этой учебной версии не реализовано — как и в
//! C++-наброске книги, где показана только вставка.

use std::cmp::Ordering;

use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::ray_3d::Ray3d;
use super::traits::{Bounded, RayIntersectable};

/// Ёмкость узла R-дерева по умолчанию.
pub const DEFAULT_R_TREE_MAX_ENTRIES: usize = 4;

#[derive(Debug, Clone, PartialEq)]
struct LeafEntry<Shape> {
    shape: Shape,
    bounds: AxisAlignedBoundingBox3d,
}

#[derive(Debug, Clone, PartialEq)]
struct BranchEntry<Shape> {
    node: Box<RTreeNode<Shape>>,
    bounds: AxisAlignedBoundingBox3d,
}

impl<Shape: Bounded> BranchEntry<Shape> {
    fn new(node: Box<RTreeNode<Shape>>) -> Self {
        let bounds = node.bounds();
        Self { node, bounds }
    }
}

/// Внутренний узел R-дерева.
///
/// Тип намеренно не публичный: пользователю не нужно вручную поддерживать
/// инварианты `min_entries`/`max_entries` и пересчитывать AABB.
#[derive(Debug, Clone, PartialEq)]
enum RTreeNode<Shape> {
    Leaf(Vec<LeafEntry<Shape>>),
    Branch(Vec<BranchEntry<Shape>>),
}

impl<Shape> RTreeNode<Shape> {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        let mut bounds = AxisAlignedBoundingBox3d::default();

        match self {
            Self::Leaf(entries) => {
                for entry in entries {
                    bounds.merge(&entry.bounds);
                }
            }
            Self::Branch(entries) => {
                for entry in entries {
                    bounds.merge(&entry.bounds);
                }
            }
        }

        bounds
    }

    fn entry_count(&self) -> usize {
        match self {
            Self::Leaf(entries) => entries.len(),
            Self::Branch(entries) => entries.len(),
        }
    }
}

/// R-дерево для пространственного индекса объектов с AABB.
#[derive(Debug, Clone, PartialEq)]
pub struct RTree<Shape> {
    root: Option<Box<RTreeNode<Shape>>>,
    max_entries: usize,
    min_entries: usize,
    size: usize,
}

impl<Shape> RTree<Shape> {
    /// Создать пустое R-дерево с заданной максимальной ёмкостью узла.
    ///
    /// Ёмкость меньше двух приводится к двум: иначе узел нельзя разбить на
    /// две непустые группы. Минимальная ёмкость — половина максимальной.
    pub fn new(max_entries: usize) -> Self {
        let max_entries = max_entries.max(2);

        Self {
            root: None,
            max_entries,
            min_entries: max_entries / 2,
            size: 0,
        }
    }

    /// Максимальное число записей в узле.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Минимальное число записей в узле (кроме корня).
    pub fn min_entries(&self) -> usize {
        self.min_entries
    }

    /// Количество объектов в дереве.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Пусто ли дерево.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Высота дерева: пустое — 0, один лист — 1.
    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |root| Self::node_height(root))
    }

    /// AABB всего дерева.
    pub fn bounds(&self) -> Option<AxisAlignedBoundingBox3d> {
        self.root.as_ref().map(|root| root.bounds())
    }

    /// Обход всех объектов дерева.
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> + '_ {
        let mut nodes: Vec<&RTreeNode<Shape>> = self.root.as_deref().into_iter().collect();
        let mut leaves: Vec<&LeafEntry<Shape>> = Vec::new();

        std::iter::from_fn(move || {
            loop {
                if let Some(entry) = leaves.pop() {
                    return Some(&entry.shape);
                }

                match nodes.pop()? {
                    RTreeNode::Leaf(entries) => leaves.extend(entries.iter().rev()),
                    RTreeNode::Branch(entries) => {
                        for entry in entries.iter().rev() {
                            nodes.push(entry.node.as_ref());
                        }
                    }
                }
            }
        })
    }

    /// Удалить все объекты.
    pub fn clear(&mut self) {
        self.root = None;
        self.size = 0;
    }

    fn node_height(node: &RTreeNode<Shape>) -> usize {
        match node {
            RTreeNode::Leaf(_) => 1,
            RTreeNode::Branch(entries) => {
                1 + entries
                    .iter()
                    .map(|entry| Self::node_height(&entry.node))
                    .max()
                    .unwrap_or(0)
            }
        }
    }
}

impl<Shape> Default for RTree<Shape> {
    fn default() -> Self {
        Self::new(DEFAULT_R_TREE_MAX_ENTRIES)
    }
}

impl<Shape: Bounded> RTree<Shape> {
    /// Вставить объект в дерево.
    pub fn insert(&mut self, shape: Shape) {
        let entry = LeafEntry {
            bounds: shape.bounds(),
            shape,
        };

        let split = match self.root.as_mut() {
            None => {
                self.root = Some(Box::new(RTreeNode::Leaf(vec![entry])));
                self.size += 1;
                return;
            }
            Some(root) => Self::insert_into_node(root, entry, self.max_entries, self.min_entries),
        };

        self.size += 1;

        if let Some(sibling) = split {
            let old_root = self.root.take().expect("корень только что существовал");
            let entries = vec![
                BranchEntry::new(old_root),
                BranchEntry::new(Box::new(sibling)),
            ];

            self.root = Some(Box::new(RTreeNode::Branch(entries)));
        }
    }

    /// Построить дерево из набора объектов последовательными вставками.
    pub fn from_shapes<I>(shapes: I, max_entries: usize) -> Self
    where
        I: IntoIterator<Item = Shape>,
    {
        let mut tree = Self::new(max_entries);

        for shape in shapes {
            tree.insert(shape);
        }

        tree
    }

    fn insert_into_node(
        node: &mut RTreeNode<Shape>,
        entry: LeafEntry<Shape>,
        max_entries: usize,
        min_entries: usize,
    ) -> Option<RTreeNode<Shape>> {
        match node {
            RTreeNode::Leaf(entries) => {
                entries.push(entry);

                if entries.len() > max_entries {
                    Some(Self::split_leaf(entries, max_entries, min_entries))
                } else {
                    None
                }
            }
            RTreeNode::Branch(entries) => {
                let index = Self::choose_child_index(entries, &entry.bounds);
                let sibling = Self::insert_into_node(
                    entries[index].node.as_mut(),
                    entry,
                    max_entries,
                    min_entries,
                );

                // После вставки AABB ребёнка мог вырасти.
                entries[index].bounds = entries[index].node.bounds();

                if let Some(sibling) = sibling {
                    entries.push(BranchEntry::new(Box::new(sibling)));
                }

                if entries.len() > max_entries {
                    Some(Self::split_branch(entries, max_entries, min_entries))
                } else {
                    None
                }
            }
        }
    }

    /// Выбрать ребёнка с минимальным расширением AABB.
    ///
    /// При равенстве предпочитаем меньшую площадь. Это аналог `leafForShape`
    /// из C++-версии.
    fn choose_child_index(
        entries: &[BranchEntry<Shape>],
        bounds: &AxisAlignedBoundingBox3d,
    ) -> usize {
        let mut best_index = 0;
        let mut best_enlargement = f32::INFINITY;
        let mut best_area = f32::INFINITY;

        for (index, entry) in entries.iter().enumerate() {
            let enlargement = enlargement(&entry.bounds, bounds);
            let area = entry.bounds.area();

            let order = enlargement.total_cmp(&best_enlargement);

            if order.is_lt() || (order.is_eq() && area < best_area) {
                best_index = index;
                best_enlargement = enlargement;
                best_area = area;
            }
        }

        best_index
    }

    fn split_leaf(
        entries: &mut Vec<LeafEntry<Shape>>,
        max_entries: usize,
        min_entries: usize,
    ) -> RTreeNode<Shape> {
        let all = std::mem::take(entries);
        let (left, right) = split_entries(all, max_entries, min_entries, |entry| entry.bounds);

        *entries = left;

        RTreeNode::Leaf(right)
    }

    fn split_branch(
        entries: &mut Vec<BranchEntry<Shape>>,
        max_entries: usize,
        min_entries: usize,
    ) -> RTreeNode<Shape> {
        let all = std::mem::take(entries);
        let (left, right) = split_entries(all, max_entries, min_entries, |entry| entry.bounds);

        *entries = left;

        RTreeNode::Branch(right)
    }

    /// Найти все объекты, AABB которых пересекается с заданной областью.
    pub fn find_shapes_in_box(&self, area: &AxisAlignedBoundingBox3d) -> Vec<&Shape> {
        let mut result = Vec::new();

        if let Some(root) = &self.root {
            Self::collect_shapes_in_box(root, area, &mut result);
        }

        result
    }

    fn collect_shapes_in_box<'a>(
        node: &'a RTreeNode<Shape>,
        area: &AxisAlignedBoundingBox3d,
        result: &mut Vec<&'a Shape>,
    ) {
        if !node.bounds().intersects(area) {
            return;
        }

        match node {
            RTreeNode::Leaf(entries) => {
                for entry in entries {
                    if entry.bounds.intersects(area) {
                        result.push(&entry.shape);
                    }
                }
            }
            RTreeNode::Branch(entries) => {
                for entry in entries {
                    if entry.bounds.intersects(area) {
                        Self::collect_shapes_in_box(&entry.node, area, result);
                    }
                }
            }
        }
    }

    /// Проверить инварианты R-дерева.
    ///
    /// Проверяются:
    /// - количество записей в каждом узле;
    /// - совпадение AABB записи с AABB объекта/дочернего узла;
    /// - количество объектов совпадает с `len()`;
    /// - все листья находятся на одной глубине.
    pub fn is_consistent(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let Some(root) = &self.root else {
            return self.size == 0;
        };

        if self.size == 0 || self.shapes().count() != self.size {
            return false;
        }

        let (valid, _, _) =
            Self::node_is_consistent(root, true, epsilon, self.max_entries, self.min_entries);

        valid
    }

    /// Возвращает (валиден ли узел, глубина листьев, количество объектов).
    fn node_is_consistent(
        node: &RTreeNode<Shape>,
        is_root: bool,
        epsilon: f32,
        max_entries: usize,
        min_entries: usize,
    ) -> (bool, usize, usize) {
        let count = node.entry_count();
        let minimum = if is_root { 1 } else { min_entries };

        if count == 0 || count < minimum || count > max_entries {
            return (false, 0, 0);
        }

        match node {
            RTreeNode::Leaf(entries) => {
                let bounds_ok = entries
                    .iter()
                    .all(|entry| bounds_are_close(&entry.bounds, &entry.shape.bounds(), epsilon));

                (bounds_ok, 1, count)
            }
            RTreeNode::Branch(entries) => {
                let mut all_valid = true;
                let mut leaf_depth = None;
                let mut total_entries = 0;

                for entry in entries {
                    if !bounds_are_close(&entry.bounds, &entry.node.bounds(), epsilon) {
                        all_valid = false;
                    }

                    let (valid, depth, child_entries) = Self::node_is_consistent(
                        &entry.node,
                        false,
                        epsilon,
                        max_entries,
                        min_entries,
                    );

                    if !valid {
                        all_valid = false;
                    }

                    if let Some(expected_depth) = leaf_depth {
                        if expected_depth != depth {
                            all_valid = false;
                        }
                    } else {
                        leaf_depth = Some(depth);
                    }

                    total_entries += child_entries;
                }

                (all_valid, 1 + leaf_depth.unwrap_or(0), total_entries)
            }
        }
    }
}

impl<Shape: Bounded + RayIntersectable> RTree<Shape> {
    /// Найти ближайшее к началу луча пересечение среди объектов дерева.
    pub fn find_closest_ray_hit(
        &self,
        ray: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Option<(f32, &Shape)> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let mut best = None;

        if let Some(root) = &self.root {
            Self::search_ray(root, ray, epsilon, &mut best);
        }

        best
    }

    fn search_ray<'a>(
        node: &'a RTreeNode<Shape>,
        ray: &Ray3d,
        epsilon: f32,
        best: &mut Option<(f32, &'a Shape)>,
    ) {
        let Some((t_enter, t_exit)) = node.bounds().ray_intersection_interval(ray, Some(epsilon))
        else {
            return;
        };

        if t_exit < 0.0 {
            return;
        }

        if let Some((best_t, _)) = best.as_ref()
            && t_enter > *best_t
        {
            return;
        }

        match node {
            RTreeNode::Leaf(entries) => {
                for entry in entries {
                    Self::consider_hit(&entry.shape, ray, epsilon, best);
                }
            }
            RTreeNode::Branch(entries) => {
                let mut order: Vec<(f32, usize)> = entries
                    .iter()
                    .enumerate()
                    .filter_map(|(index, entry)| {
                        let (enter, exit) =
                            entry.bounds.ray_intersection_interval(ray, Some(epsilon))?;
                        (exit >= 0.0).then_some((enter, index))
                    })
                    .collect();

                order.sort_by(|a, b| a.0.total_cmp(&b.0));

                for (_, index) in order {
                    Self::search_ray(entries[index].node.as_ref(), ray, epsilon, best);
                }
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

fn merge_bounds(
    a: &AxisAlignedBoundingBox3d,
    b: &AxisAlignedBoundingBox3d,
) -> AxisAlignedBoundingBox3d {
    let mut merged = *a;
    merged.merge(b);
    merged
}

fn enlargement(current: &AxisAlignedBoundingBox3d, addition: &AxisAlignedBoundingBox3d) -> f32 {
    merge_bounds(current, addition).volume() - current.volume()
}

fn group_bounds<T, F>(group: &[T], bounds_of: &F) -> AxisAlignedBoundingBox3d
where
    F: Fn(&T) -> AxisAlignedBoundingBox3d,
{
    let mut bounds = AxisAlignedBoundingBox3d::default();

    for entry in group {
        bounds.merge(&bounds_of(entry));
    }

    bounds
}

/// Квадратичное разбиение Гуттмана для переполненного набора записей.
fn split_entries<T, F>(
    entries: Vec<T>,
    max_entries: usize,
    min_entries: usize,
    bounds_of: F,
) -> (Vec<T>, Vec<T>)
where
    F: Fn(&T) -> AxisAlignedBoundingBox3d,
{
    debug_assert!(entries.len() > max_entries);

    let mut remaining = entries;
    let (seed_a, seed_b) = choose_seed_pair(&remaining, &bounds_of);

    // Удаляем больший индекс первым, чтобы не сдвинуть меньший.
    let (first_index, second_index) = if seed_a < seed_b {
        (seed_a, seed_b)
    } else {
        (seed_b, seed_a)
    };

    let second = remaining.swap_remove(second_index);
    let first = remaining.swap_remove(first_index);

    let mut group_a = vec![first];
    let mut group_b = vec![second];
    let mut bounds_a = bounds_of(&group_a[0]);
    let mut bounds_b = bounds_of(&group_b[0]);

    while let Some(entry) = remaining.pop() {
        let entry_bounds = bounds_of(&entry);
        let entries_left = remaining.len() + 1;

        // Если иначе группа не наберёт минимум, все оставшиеся идут в неё.
        let must_go_a = group_a.len() + entries_left == min_entries;
        let must_go_b = group_b.len() + entries_left == min_entries;

        let to_a = if must_go_a {
            true
        } else if must_go_b {
            false
        } else {
            let enlarge_a = enlargement(&bounds_a, &entry_bounds);
            let enlarge_b = enlargement(&bounds_b, &entry_bounds);

            match enlarge_a.total_cmp(&enlarge_b) {
                Ordering::Less => true,
                Ordering::Greater => false,
                Ordering::Equal => {
                    if group_a.len() != group_b.len() {
                        group_a.len() < group_b.len()
                    } else {
                        merge_bounds(&bounds_a, &entry_bounds).volume()
                            <= merge_bounds(&bounds_b, &entry_bounds).volume()
                    }
                }
            }
        };

        if to_a {
            bounds_a = merge_bounds(&bounds_a, &entry_bounds);
            group_a.push(entry);
        } else {
            bounds_b = merge_bounds(&bounds_b, &entry_bounds);
            group_b.push(entry);
        }
    }

    // Страховка для вырожденных случаев: обе группы должны быть не меньше min.
    while group_a.len() < min_entries {
        move_best_entry(&mut group_b, &mut group_a, &mut bounds_a, &bounds_of);
        bounds_b = group_bounds(&group_b, &bounds_of);
    }

    while group_b.len() < min_entries {
        move_best_entry(&mut group_a, &mut group_b, &mut bounds_b, &bounds_of);
    }

    (group_a, group_b)
}

/// Выбрать пару записей, дающую максимальный «мусор» при объединении AABB.
fn choose_seed_pair<T, F>(entries: &[T], bounds_of: &F) -> (usize, usize)
where
    F: Fn(&T) -> AxisAlignedBoundingBox3d,
{
    debug_assert!(entries.len() >= 2);

    let mut best = (0, 1);
    let mut best_waste = f32::NEG_INFINITY;

    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let first = bounds_of(&entries[i]);
            let second = bounds_of(&entries[j]);
            let waste = merge_bounds(&first, &second).volume() - first.volume() - second.volume();

            if waste > best_waste {
                best = (i, j);
                best_waste = waste;
            }
        }
    }

    best
}

/// Перенести запись, которая минимально расширит AABB принимающей группы.
fn move_best_entry<T, F>(
    from: &mut Vec<T>,
    to: &mut Vec<T>,
    to_bounds: &mut AxisAlignedBoundingBox3d,
    bounds_of: &F,
) where
    F: Fn(&T) -> AxisAlignedBoundingBox3d,
{
    if from.len() <= 1 {
        return;
    }

    let mut best_index = 0;
    let mut best_enlargement = f32::INFINITY;

    for (index, entry) in from.iter().enumerate() {
        let value = enlargement(to_bounds, &bounds_of(entry));

        if value < best_enlargement {
            best_index = index;
            best_enlargement = value;
        }
    }

    let entry = from.swap_remove(best_index);
    *to_bounds = merge_bounds(to_bounds, &bounds_of(&entry));
    to.push(entry);
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
    use super::super::sphere::Sphere;
    use super::*;
    use glam::Vec3;

    fn sphere(x: f32, y: f32, z: f32) -> Sphere {
        Sphere::new(Vec3::new(x, y, z), 0.5)
    }

    #[test]
    fn empty_tree_has_no_bounds() {
        let tree: RTree<Sphere> = RTree::new(4);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.height(), 0);
        assert!(tree.bounds().is_none());
        assert_eq!(tree.shapes().count(), 0);
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn insert_single_object() {
        let object = sphere(1.0, 2.0, 3.0);
        let mut tree = RTree::new(4);
        tree.insert(object);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.shapes().count(), 1);
        assert_eq!(
            tree.bounds().unwrap(),
            AxisAlignedBoundingBox3d::new(Vec3::new(0.5, 1.5, 2.5), Vec3::new(1.5, 2.5, 3.5))
        );
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn many_insertions_keep_all_objects_and_invariants() {
        let mut tree = RTree::new(3);

        for i in 0..20 {
            tree.insert(sphere(i as f32, (i % 3) as f32, (i % 5) as f32));
        }

        assert_eq!(tree.len(), 20);
        assert_eq!(tree.shapes().count(), 20);
        assert!(tree.height() > 1);
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn small_capacity_forces_root_split() {
        let mut tree = RTree::new(2);

        for i in 0..5 {
            tree.insert(sphere(i as f32 * 2.0, 0.0, 0.0));
        }

        assert_eq!(tree.max_entries(), 2);
        assert_eq!(tree.min_entries(), 1);
        assert_eq!(tree.len(), 5);
        assert!(tree.height() >= 2);
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn find_shapes_in_box_returns_intersecting_objects() {
        let mut tree = RTree::new(3);

        for object in [
            sphere(-10.0, 0.0, 0.0),
            sphere(0.0, 0.0, 0.0),
            sphere(10.0, 0.0, 0.0),
        ] {
            tree.insert(object);
        }

        let area = AxisAlignedBoundingBox3d::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let found = tree.find_shapes_in_box(&area);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].center, Vec3::ZERO);
    }

    #[test]
    fn ray_query_matches_brute_force() {
        let tree = RTree::from_shapes(
            [
                sphere(0.0, 0.0, 5.0),
                sphere(0.0, 0.0, 2.0),
                sphere(0.0, 0.0, 9.0),
            ],
            2,
        );

        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        };

        let (t, shape) = tree.find_closest_ray_hit(&ray, None).unwrap();
        let brute_force = tree
            .shapes()
            .filter_map(|object| object.ray_intersection(&ray, None).map(|t| (t, object)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();

        assert!((t - 1.5).abs() < DEFAULT_GEOMETRY_EPSILON);
        assert_eq!(shape.center, Vec3::new(0.0, 0.0, 2.0));
        assert_eq!(brute_force.0, t);
        assert_eq!(brute_force.1.center, shape.center);
    }

    #[test]
    fn clear_removes_all_objects() {
        let mut tree = RTree::new(3);
        tree.insert(sphere(0.0, 0.0, 0.0));
        tree.insert(sphere(1.0, 0.0, 0.0));

        tree.clear();

        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
        assert!(tree.bounds().is_none());
        assert!(tree.is_consistent(None));
    }

    #[test]
    fn stress_insert_keeps_all_invariants() {
        let mut tree = RTree::new(4);

        for i in 0..120 {
            tree.insert(sphere(
                (i % 12) as f32,
                (i / 12) as f32,
                (i % 7) as f32 * 0.25,
            ));
        }

        assert_eq!(tree.len(), 120);
        assert_eq!(tree.shapes().count(), 120);
        assert!(tree.is_consistent(None));
    }
}
