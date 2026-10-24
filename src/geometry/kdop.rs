//! k-дискретно ориентированный многогранник (k-DOP)

use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::plane::Plane;
use super::ray_3d::Ray3d;
use glam::Vec3;

/// Базис нормалей для k-DOP. Задает фиксированные направления пространственного сжатия.
#[derive(Debug, Clone, PartialEq)]
pub struct KDiscreteOrientedPolytopeBase {
    normals: Vec<Vec3>,
}

impl KDiscreteOrientedPolytopeBase {
    /// Создать новый базис из набора нормалей.
    /// Все нормали автоматически приводятся к единичной длине.
    pub fn new(normals: &[Vec3]) -> Self {
        let normalized = normals.iter().map(|n| n.normalize()).collect();
        Self {
            normals: normalized,
        }
    }

    /// Получить количество фиксированных направлений (нормалей)
    pub fn count(&self) -> usize {
        self.normals.len()
    }

    /// Получить ссылку на нормаль по её индексу
    pub fn get_normal(&self, index: usize) -> Vec3 {
        self.normals[index]
    }
}

/// Экземпляр ограничивающего k-DOP многогранника, построенного на определенном базисе
#[derive(Debug, Clone, PartialEq)]
pub struct KDiscreteOrientedPolytope {
    /// Минимальные расстояния проекций точек на соответствующие нормали базиса
    pub d_min: Vec<f32>,
    /// Максимальные расстояния проекций точек на соответствующие нормали базиса
    pub d_max: Vec<f32>,
}

impl KDiscreteOrientedPolytope {
    /// Создать пустой (вывернутый наизнанку) k-DOP на основе заданного базиса
    pub fn new(base: &KDiscreteOrientedPolytopeBase) -> Self {
        let count = base.count();
        Self {
            d_min: vec![f32::MAX; count],
            d_max: vec![f32::MIN; count],
        }
    }

    /// Построить плотный k-DOP вокруг набора трехмерных вершин на заданном базисе.
    /// Заменяет мутирующий метод addVertices из C++ на чистый конструктор ассоциированной функции.
    pub fn from_vertices(base: &KDiscreteOrientedPolytopeBase, vertices: &[Vec3]) -> Self {
        let mut kdop = Self::new(base);
        if vertices.is_empty() {
            return kdop;
        }

        for &vertex in vertices {
            for j in 0..base.count() {
                // Вычисляем проекцию вершины на фиксированную нормаль j
                let d = vertex.dot(base.get_normal(j));

                kdop.d_min[j] = kdop.d_min[j].min(d);
                kdop.d_max[j] = kdop.d_max[j].max(d);
            }
        }
        kdop
    }

    /// Слияние (поглощение) другого k-DOP.
    /// В отличие от C++ assert, в Rust мы возвращаем предикат валидности операции.
    pub fn merge(&mut self, other: &Self) -> bool {
        if self.d_min.len() != other.d_min.len() {
            return false; // Многогранники построены на несовпадающих базисах
        }

        for i in 0..self.d_min.len() {
            self.d_min[i] = self.d_min[i].min(other.d_min[i]);
            self.d_max[i] = self.d_max[i].max(other.d_max[i]);
        }
        true
    }

    /// Проверить высокоскоростное пересечение двух k-DOP многогранников (AABB-стиль)
    pub fn intersects(&self, other: &Self) -> bool {
        if self.d_min.len() != other.d_min.len() {
            return false;
        }

        for i in 0..self.d_min.len() {
            // Если интервалы проекций хотя бы по одной оси не пересекаются — коллизии нет
            if other.d_max[i] < self.d_min[i] || self.d_max[i] < other.d_min[i] {
                return false;
            }
        }
        true
    }

    /// Найти пересечение луча с k-DOP (Обобщенный метод параллельных плоскостей).
    /// Принимает базис, на котором построен данный k-DOP.
    /// Возвращает Some((t_near, t_far)) с параметрами точек входа и выхода,
    /// или None, если луч пролетает мимо многогранника.
    pub fn ray_intersection(
        &self,
        base: &KDiscreteOrientedPolytopeBase,
        ray: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Option<(f32, f32)> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        if self.d_min.len() != base.count() {
            return None;
        }

        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;

        for i in 0..base.count() {
            let normal = base.get_normal(i);

            // Проекция направления скорости луча на нормаль плоскости пары
            let velocity_projection = ray.direction.dot(normal);

            // Проекция точки старта луча на нормаль
            let origin_projection = ray.origin.dot(normal);

            if velocity_projection.abs() > epsilon {
                // Расчет параметров входа и выхода для текущего слоя плоскостей
                let mut t_enter = -(self.d_min[i] + origin_projection) / velocity_projection;
                let mut t_exit = -(self.d_max[i] + origin_projection) / velocity_projection;

                if t_enter > t_exit {
                    std::mem::swap(&mut t_enter, &mut t_exit);
                }

                t_near = t_near.max(t_enter);
                t_far = t_far.min(t_exit);

                if t_near > t_far {
                    return None; // Отрезки пересечения не наложились, луч промахнулся
                }
            } else {
                // Если луч идет параллельно плоскостям слоя, проверяем, лежит ли он внутри них
                if -origin_projection < self.d_min[i] || -origin_projection > self.d_max[i] {
                    return None;
                }
            }
        }

        if t_far < epsilon {
            return None; // Многогранник остался позади луча
        }

        Some((t_near.max(0.0), t_far))
    }

    /// Вспомогательный метод классификации относительно плоскости (заглушка автора)
    pub fn classify(&self, _plane: &Plane) -> i32 {
        0
    }
}
