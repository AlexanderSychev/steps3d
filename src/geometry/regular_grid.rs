//! Регулярная сетка (uniform grid) для пространственного индекса
//!
//! Сетка делит общий AABB сцены на `n1 × n2 × n3` одинаковых ячеек. Объект
//! записывается во все ячейки, которые пересекает его AABB. В отличие от
//! BVH/kD-дерева/BSP, здесь нет иерархии: ячейки лежат в плоском массиве, а
//! поиск сводится к вычислению диапазона ячеек.
//!
//! # Владение объектами
//!
//! В C++-версии книги ячейки хранят `Shape*`, а объекты принадлежат сцене.
//! В Rust один объект нельзя положить в несколько `Vec` без `Rc` или
//! клонирования. Здесь сетка владеет ареной `Vec<Shape>`, а ячейки хранят
//! только индексы в этой арене. Поэтому:
//!
//! - объект хранится ровно один раз;
//! - попадание в несколько ячеек не дублирует данные;
//! - запросы дедуплицируют результаты по индексу.
//!
//! # Исправления относительно C++
//!
//! В книжном `grid.h` индекс ячейки считается как `(i*n1 + j)*n2 + k`, а
//! также есть опечатка `k3` вместо `k2`. Здесь используется корректная
//! формула `(i*n2 + j)*n3 + k` для размеров `[n1, n2, n3]`.
//!
//! Функция `findIntersection` в книге только объявлена; здесь реализован
//! обход ячеек лучом по алгоритму 3D-DDA (Amanatides–Woo), который посещает
//! ячейки в порядке возрастания `t`.

use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::ray_3d::Ray3d;
use super::traits::{Bounded, RayIntersectable};
use glam::Vec3;

/// Равномерная сетка для пространственного индекса объектов с AABB.
#[derive(Debug, Clone, PartialEq)]
pub struct RegularGrid<Shape> {
    dimensions: [usize; 3],
    bounds: AxisAlignedBoundingBox3d,
    cell_size: Vec3,
    cells: Vec<Vec<usize>>,
    shapes: Vec<Shape>,
    shape_bounds: Vec<AxisAlignedBoundingBox3d>,
}

impl<Shape> RegularGrid<Shape> {
    /// Количество объектов в сетке.
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Пуста ли сетка.
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Размеры сетки по осям.
    pub fn dimensions(&self) -> [usize; 3] {
        self.dimensions
    }

    /// Общий AABB сетки.
    pub fn bounds(&self) -> &AxisAlignedBoundingBox3d {
        &self.bounds
    }

    /// Размер одной ячейки.
    pub fn cell_size(&self) -> Vec3 {
        self.cell_size
    }

    /// Общее количество ячеек.
    pub fn cell_count(&self) -> usize {
        self.dimensions[0] * self.dimensions[1] * self.dimensions[2]
    }

    /// Обход всех объектов.
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> + '_ {
        self.shapes.iter()
    }

    /// Координаты ячейки, в которую попадает точка.
    ///
    /// Точки вне сетки прижимаются к крайним ячейкам.
    pub fn cell_coords(&self, point: Vec3) -> [usize; 3] {
        let point = point.to_array();
        let min = self.bounds.min_point.to_array();
        let cell_size = self.cell_size.to_array();
        let mut coords = [0; 3];

        for axis in 0..3 {
            if cell_size[axis] > 0.0 {
                let value = ((point[axis] - min[axis]) / cell_size[axis]).floor();
                coords[axis] = value.clamp(0.0, (self.dimensions[axis] - 1) as f32) as usize;
            }
        }

        coords
    }

    /// AABB ячейки по её координатам.
    pub fn cell_bounds(&self, cell: [usize; 3]) -> Option<AxisAlignedBoundingBox3d> {
        self.cell_index_checked(cell)?;

        let min = self.bounds.min_point
            + Vec3::new(cell[0] as f32, cell[1] as f32, cell[2] as f32) * self.cell_size;
        let mut max = min + self.cell_size;

        // Прижимаем внешнюю границу последних ячеек к общему AABB, чтобы
        // избежать щелей из-за округления f32.
        if cell[0] + 1 == self.dimensions[0] {
            max.x = self.bounds.max_point.x;
        }
        if cell[1] + 1 == self.dimensions[1] {
            max.y = self.bounds.max_point.y;
        }
        if cell[2] + 1 == self.dimensions[2] {
            max.z = self.bounds.max_point.z;
        }

        Some(AxisAlignedBoundingBox3d::new(min, max))
    }

    /// Объекты, записанные в конкретную ячейку.
    pub fn shapes_in_cell(&self, cell: [usize; 3]) -> Option<Vec<&Shape>> {
        let index = self.cell_index_checked(cell)?;

        Some(
            self.cells[index]
                .iter()
                .map(|&shape_index| &self.shapes[shape_index])
                .collect(),
        )
    }

    /// Количество занятых ячеек.
    pub fn occupied_cells(&self) -> usize {
        self.cells.iter().filter(|cell| !cell.is_empty()).count()
    }

    /// Максимальное число объектов в одной ячейке.
    pub fn max_cell_occupancy(&self) -> usize {
        self.cells.iter().map(|cell| cell.len()).max().unwrap_or(0)
    }

    fn cell_index(&self, cell: [usize; 3]) -> usize {
        (cell[0] * self.dimensions[1] + cell[1]) * self.dimensions[2] + cell[2]
    }

    fn cell_index_checked(&self, cell: [usize; 3]) -> Option<usize> {
        if cell[0] < self.dimensions[0]
            && cell[1] < self.dimensions[1]
            && cell[2] < self.dimensions[2]
        {
            Some(self.cell_index(cell))
        } else {
            None
        }
    }

    fn cell_range_for_bounds(&self, bounds: &AxisAlignedBoundingBox3d) -> ([usize; 3], [usize; 3]) {
        (
            self.cell_coords(bounds.min_point),
            self.cell_coords(bounds.max_point),
        )
    }
}

impl<Shape: Bounded> RegularGrid<Shape> {
    /// Построить сетку по набору объектов.
    ///
    /// `dimensions` — число ячеек по X, Y и Z. Нулевой размер или пустой
    /// набор объектов дают `None`.
    pub fn new(dimensions: [usize; 3], shapes: Vec<Shape>) -> Option<Self> {
        if dimensions.contains(&0) || shapes.is_empty() {
            return None;
        }

        let shape_bounds: Vec<_> = shapes.iter().map(|shape| shape.bounds()).collect();
        let mut bounds = AxisAlignedBoundingBox3d::default();

        for shape_bounds in &shape_bounds {
            bounds.merge(shape_bounds);
        }

        let cell_size = bounds.get_size()
            / Vec3::new(
                dimensions[0] as f32,
                dimensions[1] as f32,
                dimensions[2] as f32,
            );
        let cell_count = dimensions[0] * dimensions[1] * dimensions[2];

        let mut grid = Self {
            dimensions,
            bounds,
            cell_size,
            cells: vec![Vec::new(); cell_count],
            shapes,
            shape_bounds,
        };

        for shape_index in 0..grid.shapes.len() {
            let shape_bounds = grid.shape_bounds[shape_index];
            let (min_cell, max_cell) = grid.cell_range_for_bounds(&shape_bounds);

            for i in min_cell[0]..=max_cell[0] {
                for j in min_cell[1]..=max_cell[1] {
                    for k in min_cell[2]..=max_cell[2] {
                        let cell_index = grid.cell_index([i, j, k]);
                        grid.cells[cell_index].push(shape_index);
                    }
                }
            }
        }

        Some(grid)
    }

    /// Построить сетку из итератора объектов.
    pub fn build<I>(dimensions: [usize; 3], shapes: I) -> Option<Self>
    where
        I: IntoIterator<Item = Shape>,
    {
        Self::new(dimensions, shapes.into_iter().collect())
    }

    /// Найти все объекты, AABB которых пересекается с заданной областью.
    ///
    /// Объект, попавший в несколько ячеек, возвращается один раз.
    pub fn find_shapes_in_box(&self, area: &AxisAlignedBoundingBox3d) -> Vec<&Shape> {
        let mut result = Vec::new();

        if !self.bounds.intersects(area) {
            return result;
        }

        let (min_cell, max_cell) = self.cell_range_for_bounds(area);
        let mut seen = vec![false; self.shapes.len()];

        for i in min_cell[0]..=max_cell[0] {
            for j in min_cell[1]..=max_cell[1] {
                for k in min_cell[2]..=max_cell[2] {
                    let cell_index = self.cell_index([i, j, k]);

                    for &shape_index in &self.cells[cell_index] {
                        if seen[shape_index] {
                            continue;
                        }

                        seen[shape_index] = true;

                        if self.shape_bounds[shape_index].intersects(area) {
                            result.push(&self.shapes[shape_index]);
                        }
                    }
                }
            }
        }

        result
    }

    /// Проверить инварианты сетки.
    ///
    /// Проверяется, что каждый объект записан во все ячейки, которые
    /// пересекает его AABB, индексы не выходят за пределы арены, а
    /// сохранённые границы совпадают с границами объектов.
    pub fn is_consistent(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        if self.shapes.len() != self.shape_bounds.len() || self.cells.len() != self.cell_count() {
            return false;
        }

        let mut merged = AxisAlignedBoundingBox3d::default();

        for shape_bounds in &self.shape_bounds {
            merged.merge(shape_bounds);
        }

        if !bounds_are_close(&self.bounds, &merged, epsilon) {
            return false;
        }

        for (shape_index, (shape, shape_bounds)) in
            self.shapes.iter().zip(&self.shape_bounds).enumerate()
        {
            if !bounds_are_close(shape_bounds, &shape.bounds(), epsilon) {
                return false;
            }

            let (min_cell, max_cell) = self.cell_range_for_bounds(shape_bounds);

            for i in min_cell[0]..=max_cell[0] {
                for j in min_cell[1]..=max_cell[1] {
                    for k in min_cell[2]..=max_cell[2] {
                        let cell_index = self.cell_index([i, j, k]);

                        if !self.cells[cell_index].contains(&shape_index) {
                            return false;
                        }
                    }
                }
            }
        }

        // Индексы должны быть валидны и не повторяться внутри ячейки.
        for cell in &self.cells {
            for (position, &shape_index) in cell.iter().enumerate() {
                if shape_index >= self.shapes.len() || cell[..position].contains(&shape_index) {
                    return false;
                }
            }
        }

        true
    }
}

impl<Shape: RayIntersectable> RegularGrid<Shape> {
    /// Найти ближайшее пересечение луча с объектами сетки.
    ///
    /// Ячейки посещаются в порядке возрастания `t` алгоритмом 3D-DDA.
    /// Как только ближайшее попадание оказывается не дальше входа в
    /// следующую ячейку, обход прекращается.
    pub fn find_closest_ray_hit(
        &self,
        ray: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Option<(f32, &Shape)> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let (t_enter, t_exit) = self.bounds.ray_intersection_interval(ray, Some(epsilon))?;

        if t_exit < 0.0 {
            return None;
        }

        let start_t = t_enter.max(0.0);

        if start_t > t_exit {
            return None;
        }

        let start_point = ray.origin + ray.direction * start_t;
        let mut cell = self.cell_coords(start_point);

        let origin = ray.origin.to_array();
        let direction = ray.direction.to_array();
        let cell_size = self.cell_size.to_array();
        let grid_min = self.bounds.min_point.to_array();

        let mut step = [0i32; 3];
        let mut t_max = [f32::INFINITY; 3];
        let mut t_delta = [f32::INFINITY; 3];

        for axis in 0..3 {
            if direction[axis].abs() <= epsilon || cell_size[axis] <= 0.0 {
                continue;
            }

            step[axis] = if direction[axis] > 0.0 { 1 } else { -1 };
            t_delta[axis] = (cell_size[axis] / direction[axis]).abs();

            let boundary = if step[axis] > 0 {
                grid_min[axis] + (cell[axis] as f32 + 1.0) * cell_size[axis]
            } else {
                grid_min[axis] + cell[axis] as f32 * cell_size[axis]
            };

            t_max[axis] = (boundary - origin[axis]) / direction[axis];

            if t_max[axis] < 0.0 {
                t_max[axis] = 0.0;
            }
        }

        let mut seen = vec![false; self.shapes.len()];
        let mut best = None;

        loop {
            let cell_index = self.cell_index(cell);

            for &shape_index in &self.cells[cell_index] {
                if seen[shape_index] {
                    continue;
                }

                seen[shape_index] = true;
                Self::consider_hit(&self.shapes[shape_index], ray, epsilon, &mut best);
            }

            let next_t = t_max[0].min(t_max[1]).min(t_max[2]);

            // Попадание не дальше входа в следующую ячейку — дальше искать
            // ближе уже бессмысленно.
            if let Some((best_t, _)) = best.as_ref()
                && *best_t <= next_t
            {
                break;
            }

            if !next_t.is_finite() || next_t > t_exit {
                break;
            }

            let mut left_grid = false;

            for axis in 0..3 {
                if t_max[axis] <= next_t + epsilon {
                    let next = cell[axis] as isize + step[axis] as isize;

                    if next < 0 || next >= self.dimensions[axis] as isize {
                        left_grid = true;
                        break;
                    }

                    cell[axis] = next as usize;
                    t_max[axis] += t_delta[axis];
                }
            }

            if left_grid {
                break;
            }
        }

        best
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

    fn sphere(x: f32, y: f32, z: f32, radius: f32) -> Sphere {
        Sphere::new(Vec3::new(x, y, z), radius)
    }

    #[test]
    fn empty_or_zero_dimensions_return_none() {
        let empty: Option<RegularGrid<Sphere>> = RegularGrid::new([2, 2, 2], Vec::new());
        let zero: Option<RegularGrid<Sphere>> = RegularGrid::new([0, 2, 2], Vec::new());

        assert!(empty.is_none());
        assert!(zero.is_none());
    }

    #[test]
    fn builds_correct_cell_count_and_bounds() {
        let shapes = vec![
            sphere(-5.0, 0.0, 0.0, 1.0),
            sphere(0.0, 0.0, 0.0, 1.0),
            sphere(5.0, 0.0, 0.0, 1.0),
        ];

        let grid = RegularGrid::new([2, 2, 2], shapes).unwrap();

        assert_eq!(grid.dimensions(), [2, 2, 2]);
        assert_eq!(grid.cell_count(), 8);
        assert_eq!(grid.len(), 3);
        assert_eq!(
            *grid.bounds(),
            AxisAlignedBoundingBox3d::new(Vec3::new(-6.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0))
        );
        assert!(grid.is_consistent(None));
    }

    #[test]
    fn find_shapes_in_box_matches_brute_force() {
        let shapes: Vec<Sphere> = (0..20)
            .map(|i| sphere((i % 5) as f32 * 2.0, (i / 5) as f32 * 2.0, 0.0, 0.75))
            .collect();

        let grid = RegularGrid::new([4, 4, 4], shapes.clone()).unwrap();

        let area =
            AxisAlignedBoundingBox3d::new(Vec3::new(1.0, 1.0, -1.0), Vec3::new(5.0, 5.0, 1.0));
        let mut found: Vec<Vec3> = grid
            .find_shapes_in_box(&area)
            .iter()
            .map(|shape| shape.center)
            .collect();
        let mut expected: Vec<Vec3> = shapes
            .iter()
            .filter(|shape| shape.bounds().intersects(&area))
            .map(|shape| shape.center)
            .collect();

        found.sort_by(|a, b| a.x.total_cmp(&b.x));
        expected.sort_by(|a, b| a.x.total_cmp(&b.x));

        assert_eq!(found, expected);
    }

    #[test]
    fn object_spanning_many_cells_is_returned_once() {
        let big = sphere(0.0, 0.0, 0.0, 10.0);
        let grid = RegularGrid::new([4, 4, 4], vec![big]).unwrap();

        let area = AxisAlignedBoundingBox3d::new(Vec3::splat(-20.0), Vec3::splat(20.0));
        let found = grid.find_shapes_in_box(&area);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].center, Vec3::ZERO);
    }

    #[test]
    fn ray_query_matches_brute_force() {
        let shapes: Vec<Sphere> = (0..20)
            .map(|i| {
                sphere(
                    (i % 5) as f32 * 2.0 - 4.0,
                    (i / 5) as f32 * 2.0 - 4.0,
                    (i % 3) as f32 * 2.0,
                    0.75,
                )
            })
            .collect();

        let grid = RegularGrid::new([4, 4, 4], shapes.clone()).unwrap();

        for ray in [
            Ray3d {
                origin: Vec3::new(0.0, 0.0, -20.0),
                direction: Vec3::Z,
            },
            Ray3d {
                origin: Vec3::new(0.0, 0.0, 20.0),
                direction: -Vec3::Z,
            },
            Ray3d {
                origin: Vec3::new(-20.0, 0.0, 0.0),
                direction: Vec3::X,
            },
            Ray3d {
                origin: Vec3::ZERO,
                direction: Vec3::new(0.2, 0.1, 1.0).normalize(),
            },
        ] {
            let hit = grid.find_closest_ray_hit(&ray, None);
            let brute_force = shapes
                .iter()
                .filter_map(|shape| shape.ray_intersection(&ray, None).map(|t| (t, shape)))
                .min_by(|a, b| a.0.total_cmp(&b.0));

            match (hit, brute_force) {
                (Some((t, shape)), Some((brute_t, brute_shape))) => {
                    assert!((t - brute_t).abs() < DEFAULT_GEOMETRY_EPSILON);
                    assert_eq!(shape.center, brute_shape.center);
                }
                (None, None) => {}
                _ => panic!("DDA и полный перебор дали разные результаты"),
            }
        }
    }

    #[test]
    fn degenerate_axis_is_supported() {
        let shapes = vec![sphere(-1.0, -1.0, 0.0, 0.0), sphere(1.0, 1.0, 0.0, 0.0)];

        let grid = RegularGrid::new([4, 4, 4], shapes).unwrap();

        assert_eq!(grid.cell_size().z, 0.0);
        assert!(grid.is_consistent(None));

        let area = AxisAlignedBoundingBox3d::new(Vec3::splat(-2.0), Vec3::splat(2.0));
        assert_eq!(grid.find_shapes_in_box(&area).len(), 2);
    }

    #[test]
    fn cell_lookup_and_contents() {
        let grid = RegularGrid::new(
            [2, 1, 1],
            vec![sphere(-5.0, 0.0, 0.0, 1.0), sphere(5.0, 0.0, 0.0, 1.0)],
        )
        .unwrap();

        let left = grid.cell_coords(Vec3::new(-5.0, 0.0, 0.0));
        let right = grid.cell_coords(Vec3::new(5.0, 0.0, 0.0));

        assert_eq!(left, [0, 0, 0]);
        assert_eq!(right, [1, 0, 0]);
        assert_eq!(grid.shapes_in_cell(left).unwrap().len(), 1);
        assert_eq!(grid.shapes_in_cell(right).unwrap().len(), 1);
        assert!(grid.cell_bounds([2, 0, 0]).is_none());
    }
}
