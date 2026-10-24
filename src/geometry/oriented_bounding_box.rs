use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::eig3x3::{EigenSystem3x3, eigen_symmetric_3x3};
use super::ray_3d::Ray3d;
use glam::{Mat3, Vec3};

/// Ориентированный ограничивающий параллелепипед (OBB) в трехмерном пространстве
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientedBoundingBox {
    /// Центр параллелепипеда в мировых координатах
    pub center: Vec3,
    /// Половины размеров параллелепипеда вдоль его локальных осей (локальные радиусы)
    pub extents: Vec3,
    /// Локальная матрица ориентации (столбцы матрицы — это нормали граней параллелепипеда)
    pub orientation: Mat3,
}

impl OrientedBoundingBox {
    /// Создает новый ориентированный прямоугольник
    pub fn new(center: Vec3, extents: Vec3, orientation: Mat3) -> Self {
        Self {
            center,
            extents,
            orientation,
        }
    }

    /// ФУНДАМЕНТАЛЬНЫЙ КОНСТРУКТОР: Построить оптимальный OrientedBoundingBox (OBB)
    /// вокруг облака трехмерных точек с помощью анализа главных компонент (PCA).
    pub fn from_vertices(vertices: &[Vec3], maybe_epsilon: Option<f32>) -> Option<Self> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let n = vertices.len();
        if n < 3 {
            return None; // Для построения стабильного трехмерного объема нужно хотя бы 3 точки
        }

        // ШАГ 1: Находим средний геометрический центр масс (Mean Center)
        let sum_center: Vec3 = vertices.iter().sum();
        let mean_center = sum_center / n as f32;

        // ШАГ 2: Строим матрицу ковариации 3x3
        let mut cov_a = 0.0f32;
        let mut cov_ab = 0.0f32;
        let mut cov_ac = 0.0f32;
        let mut cov_b = 0.0f32;
        let mut cov_bc = 0.0f32;
        let mut cov_c = 0.0f32;

        for &vertex in vertices {
            let delta = vertex - mean_center;

            cov_a += delta.x * delta.x;
            cov_ab += delta.x * delta.y;
            cov_ac += delta.x * delta.z;
            cov_b += delta.y * delta.y;
            cov_bc += delta.y * delta.z;
            cov_c += delta.z * delta.z;
        }

        let inv_n = 1.0 / n as f32;
        let covariance_matrix = Mat3::from_cols(
            Vec3::new(cov_a * inv_n, cov_ab * inv_n, cov_ac * inv_n),
            Vec3::new(cov_ab * inv_n, cov_b * inv_n, cov_bc * inv_n),
            Vec3::new(cov_ac * inv_n, cov_bc * inv_n, cov_c * inv_n),
        );

        // ШАГ 3: Находим собственные векторы матрицы ковариации (это и есть оси нашей OBB коробки)
        let eigen_system = eigen_symmetric_3x3(&covariance_matrix, Some(epsilon))?;
        let orientation = eigen_system.vectors;

        // Извлекаем получившиеся нормали осей
        let axis_x = orientation.x_axis;
        let axis_y = orientation.y_axis;
        let axis_z = orientation.z_axis;

        // ШАГ 4: Проецируем точки на новые оси, чтобы найти экстремумы (размеры коробки)
        let mut min_extents = Vec3::splat(f32::MAX);
        let mut max_extents = Vec3::splat(f32::MIN);

        for &vertex in vertices {
            // Проекция на каждую из трех новых осей вращения
            let proj = Vec3::new(vertex.dot(axis_x), vertex.dot(axis_y), vertex.dot(axis_z));

            min_extents = min_extents.min(proj);
            max_extents = max_extents.max(proj);
        }

        // Полуразмеры (extents) — это половина расстояния между минимумом и максимумом
        let extents = (max_extents - min_extents) * 0.5;

        // Скорректированный центр в мировом пространстве с учетом поворота осей
        let local_center = (min_extents + max_extents) * 0.5;
        let center = axis_x * local_center.x + axis_y * local_center.y + axis_z * local_center.z;

        Some(OrientedBoundingBox {
            center,
            extents,
            orientation,
        })
    }

    /// Находит пересечение луча с OBB (Метод параллельных пластин).
    /// Возвращает Some(f32) с расстоянием `t` вдоль луча до точки входа,
    /// или None, если луч не пересекает коробку.
    pub fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Время входа (t_near) и время выхода (t_far)
        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;

        // Вектор от центра коробки к началу луча
        let delta = ray.origin - self.center;

        // Извлекаем локальные оси OBB (столбцы матрицы orientation)
        let axes = [
            self.orientation.x_axis,
            self.orientation.y_axis,
            self.orientation.z_axis,
        ];

        // Полуразмеры коробки в виде массива для удобства итерации по осям X, Y, Z
        let extents_arr = [self.extents.x, self.extents.y, self.extents.z];

        // Проверяем три пары параллельных плоскостей коробки
        for i in 0..3 {
            let axis = axes[i];

            // Проекция скорости луча на текущую локальную ось коробки (vd из книги)
            let velocity_projection = ray.direction.dot(axis);

            // Проекция расстояния от центра OBB до начала луча на эту же ось (vo из книги)
            let distance_projection = delta.dot(axis);

            let half_size = extents_arr[i];

            if velocity_projection.abs() > epsilon {
                // Вычисляем параметры t пересечения с ближней и дальней плоскостями оси
                let mut t1 = (-distance_projection - half_size) / velocity_projection;
                let mut t2 = (-distance_projection + half_size) / velocity_projection;

                // Гарантируем, что t1 — это всегда точка входа, а t2 — точка выхода
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }

                // Сужаем интервал видимости
                t_near = t_near.max(t1);
                t_far = t_far.min(t2);

                // Если точка входа оказалась дальше точки выхода — луч промахнулся
                if t_near > t_far {
                    return None;
                }
            } else {
                // Если луч параллелен текущей паре плоскостей, проверяем, находится ли он внутри них.
                // Если расстояние от луча до центра OBB по этой оси больше половины размера коробки — пересечения нет.
                if distance_projection.abs() > half_size {
                    return None;
                }
            }
        }

        // Если коробка находится позади луча (t_far < 0), пересечения нет
        if t_far < epsilon {
            return None;
        }

        // Если t_near < 0, значит источник луча находится прямо внутри OBB,
        // в таком случае ближайшая точка пересечения — это точка выхода или 0.0
        let t_result = if t_near < 0.0 { 0.0 } else { t_near };

        Some(t_result)
    }

    /// Находит абсолютные 3D-координаты точки входа луча в OBB.
    /// Возвращает Some(Vec3) с мировыми координатами, или None, если луч пролетает мимо.
    pub fn get_intersection_point(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<Vec3> {
        self.ray_intersection(ray, maybe_epsilon)
            .map(|t| ray.origin + t * ray.direction)
    }

    /// Просто проверяет факт пересечения луча с OBB (да/нет) без лишних вычислений.
    pub fn is_ray_intersects(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> bool {
        self.ray_intersection(ray, maybe_epsilon).is_some()
    }
}
