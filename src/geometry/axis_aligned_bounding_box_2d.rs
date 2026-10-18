use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::line_segment_2d::LineSegment2d;
use glam::Vec2;

/// Прямогольник, параллельный осям координат (AABB)
#[derive(Debug, Clone, Copy)]
pub struct AxisAlignedBoundingBox2d {
    /// Минимальная точка
    pub min: Vec2,
    /// Максимальная точка
    pub max: Vec2,
}

impl AxisAlignedBoundingBox2d {
    /// Расстояние до прямоугольника до точки
    pub fn distance_to_point(&self, point: &Vec2) -> f32 {
        let delta_x = if point.x < self.min.x {
            self.min.x - point.x
        } else if point.x > self.max.x {
            point.x - self.max.x
        } else {
            0.0
        };

        let delta_y = if point.y < self.min.y {
            self.min.y - point.y
        } else if point.y > self.max.y {
            point.y - self.max.y
        } else {
            0.0
        };

        (delta_x * delta_x + delta_y * delta_y).sqrt()
    }

    /// Отсекает отрезок по границам прямоугольника (Алгоритм Лянга-Барски).
    /// Возвращает Some(LineSegment2d) с новыми координатами, если отрезок виден,
    /// или None, если отрезок находится полностью снаружи.
    pub fn clip_line_segment(
        &self,
        line: &LineSegment2d,
        maybe_epsilon: Option<f32>,
    ) -> Option<LineSegment2d> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let d = line.direction_not_normalized();

        // t_in собирает максимум (начинаем с 0.0 — старт отрезка)
        let mut t_in = 0.0f32;
        // t_out собирает минимум (начинаем с 1.0 — конец отрезка)
        let mut t_out = 1.0f32;

        // Вспомогательное замыкание (лямбда), чтобы не дублировать код для 4-х сторон.
        // p — проекция направления (знаменатель), q — расстояние до границы (числитель).
        let mut clip_edge = |p: f32, q: f32| -> bool {
            if p.abs() < epsilon {
                // Отрезок параллелен границе.
                // Если q < 0, то начальная точка находится с внешней стороны от границы -> отрезок невидим.
                return q >= 0.0;
            }

            let t = q / p;
            if p < 0.0 {
                // Отрезок направлен снаружи внутрь этой границы (линия входа)
                if t > t_in {
                    t_in = t;
                }
            } else {
                // Отрезок направлен изнутри наружу этой границы (линия выхода)
                if t < t_out {
                    t_out = t;
                }
            }

            // Если в любой момент точка входа оказалась позже точки выхода,
            // отрезок целиком лежит снаружи прямоугольника.
            t_in <= t_out
        };

        // Проверяем все 4 границы по очереди:
        // Граница 1: Левая (X min)
        if !clip_edge(-d.x, line.a.x - self.min.x) {
            return None;
        }
        // Граница 2: Правая (X max)
        if !clip_edge(d.x, self.max.x - line.a.x) {
            return None;
        }
        // Граница 3: Нижняя (Y min)
        if !clip_edge(-d.y, line.a.y - self.min.y) {
            return None;
        }
        // Граница 4: Верхняя (Y max)
        if !clip_edge(d.y, self.max.y - line.a.y) {
            return None;
        }

        // Если интервал [t_in, t_out] остался валидным, строим отсеченный отрезок
        Some(LineSegment2d {
            a: line.a + t_in * d,
            b: line.a + t_out * d,
        })
    }
}
