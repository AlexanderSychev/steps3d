use super::axis_aligned_bounding_box_2d::AxisAlignedBoundingBox2d;
use super::circle_2d::Circle2d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::enums::{LineArea, PointArea};
use glam::Vec2;

/// Отрезок прямой на плоскости
#[derive(Debug, Clone, Copy)]
pub struct LineSegment2d {
    /// Начальная точка
    pub a: Vec2,
    /// Конечная точка
    pub b: Vec2,
}

impl LineSegment2d {
    /// Получить _ненормализованный_ вектор направления
    pub fn direction_not_normalized(&self) -> Vec2 {
        self.b - self.a
    }

    /// Получить _нормализованный_ вектор направления, если это возможно
    pub fn direction(&self) -> Option<Vec2> {
        self.direction_not_normalized().try_normalize()
    }

    /// Получить нормаль отрезка
    pub fn normal(&self) -> Option<Vec2> {
        let maybe_l = self.direction();
        if let Some(l) = maybe_l {
            Some(Vec2::new(l.y, -l.x))
        } else {
            None
        }
    }

    /// Определить положение точки относительно отрезка с погрешностью
    pub fn classify_point(&self, point: &Vec2, maybe_epsilon: Option<f32>) -> PointArea {
        // Если не передана конкретная погрешность, то берем стандартную для типа "f32"
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // От "line_a" к "line_b"
        let a2b = self.b - self.a;
        // От "line_a" к "point"
        let a2p = point - self.a;
        // Площадь полученного треугольника
        let area = a2b.x * a2p.y - a2p.x * a2b.y;

        if area > epsilon {
            return PointArea::Left;
        }

        if area < -epsilon {
            return PointArea::Right;
        }

        if (point - self.a).length() < epsilon {
            return PointArea::Start;
        }

        if (point - self.b).length() < epsilon {
            return PointArea::End;
        }

        if (a2b.x * a2p.x < 0.0) || (a2b.y * a2p.y < 0.0) {
            return PointArea::Behind;
        }

        if a2b.length() < a2p.length() {
            return PointArea::Beyond;
        }

        PointArea::Between
    }

    /// Определить положение окружности относительно отрезка с погрешностью
    pub fn classify_circle(
        &self,
        circle: &Circle2d,
        maybe_epsilon: Option<f32>,
    ) -> Option<LineArea> {
        let epsilon: f32 = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let maybe_n = self.normal();
        if let Some(n) = maybe_n {
            let d = -self.a.dot(n);
            let signed_distance = circle.center.dot(n) + d;

            let area = if signed_distance >= circle.radius + epsilon {
                LineArea::InPositive
            } else if signed_distance <= -circle.radius - epsilon {
                LineArea::InNegative
            } else {
                LineArea::InBoth
            };
            Some(area)
        } else {
            None
        }
    }

    /// Определить положение прямоугольника относительно прямой с погрешностью
    pub fn classify_aabb(
        &self,
        bx: &AxisAlignedBoundingBox2d,
        maybe_epsilon: Option<f32>,
    ) -> Option<LineArea> {
        let epsilon: f32 = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let maybe_n = self.normal();
        if let Some(n) = maybe_n {
            let d = -self.a.dot(n);

            // 1. Ищем n-вершину (наиболее удаленную НАЗАД против вектора нормали)
            let n_vertex = Vec2::new(
                if n.x >= 0.0 { bx.min.x } else { bx.max.x },
                if n.y >= 0.0 { bx.min.y } else { bx.max.y },
            );

            // Если даже самая "ближняя" к линии точка ушла в положительную зону,
            // значит весь прямоугольник гарантированно находится в положительной полуплоскости
            if n.dot(n_vertex) + d > epsilon {
                return Some(LineArea::InPositive);
            }

            // 2. Ищем p-вершину (наиболее удаленную ВПЕРЕД по вектору нормали)
            let p_vertex = Vec2::new(
                if n.x >= 0.0 { bx.max.x } else { bx.min.x },
                if n.y >= 0.0 { bx.max.y } else { bx.min.y },
            );

            // Если даже самая "дальняя" по нормали точка находится в отрицательной зоне,
            // значит весь прямоугольник гарантированно находится в отрицательной полуплоскости
            if n.dot(p_vertex) + d < -epsilon {
                return Some(LineArea::InNegative);
            }

            // Если n-вершина в негативе, а p-вершина в позитиве — прямоугольник пересекает прямую
            Some(LineArea::InBoth)
        } else {
            None
        }
    }

    /// Найти точку пересечения двух отрезков
    pub fn intersect(&self, other: &LineSegment2d, maybe_epsilon: Option<f32>) -> Option<Vec2> {
        let epsilon = maybe_epsilon.unwrap_or(1e-5); // Немного больший эпсилон для f32

        let r = self.b - self.a;
        let s = other.b - other.a;

        // Косое произведение векторов направлений (аналог det)
        let r_cross_s = r.x * s.y - r.y * s.x;
        let q_minus_p = other.a - self.a;
        let q_minus_p_cross_r = q_minus_p.x * r.y - q_minus_p.y * r.x;

        // Случай 1: Отрезки параллельны или лежат на одной прямой
        if r_cross_s.abs() < epsilon {
            // Если они на одной прямой и перекрываются, можно вернуть точку начала наложения
            // (Для простоты базовых коллизий здесь обычно возвращают None или special случай)
            return None;
        }

        // Вычисляем коэффициенты пересечения прямых
        // t — параметр для self (от 0 до 1), u — параметр для other (от 0 до 1)
        let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / r_cross_s;
        let u = q_minus_p_cross_r / r_cross_s;

        // Строгая проверка: точка пересечения лежит ВНУТРИ обоих отрезков
        if t >= -epsilon && t <= 1.0 + epsilon && u >= -epsilon && u <= 1.0 + epsilon {
            // Возвращаем точную точку на отрезке
            Some(self.a + t.clamp(0.0, 1.0) * r)
        } else {
            None
        }
    }

    pub fn distance_to_point(&self, point: &Vec2, maybe_epsilon: Option<f32>) -> f32 {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let ab = self.b - self.a; // Вектор направления отрезка
        let ap = point - self.a; // Вектор от начала отрезка к целевой точке

        // Квадрат длины отрезка
        let ab_len_sq = ab.length_squared();

        // Защита от вырожденного отрезка (если точки A и B совпадают)
        if ab_len_sq < epsilon {
            return self.a.distance(*point);
        }

        // Находим коэффициент проекции t и строго ограничиваем его в рамках [0.0, 1.0]
        let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);

        // Вычисляем реальную ближайшую точку на отрезке
        let closest_point = self.a + t * ab;

        // Возвращаем расстояние до неё
        point.distance(closest_point)
    }
}
