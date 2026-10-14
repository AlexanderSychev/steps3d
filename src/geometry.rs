use glam::Vec2;
use simple_error::{SimpleResult, simple_error};

/// Допустимая погрешность для геометрических рассчетов
pub const DEFAULT_GEOMETRY_EPSILON: f32 = 1e-5;

/// Расположение точки относительно отрезка
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointArea {
    /// Слева
    Left,
    /// Справа
    Right,
    /// В начале отрезка
    Start,
    /// В конце отрезка
    End,
    /// Позади отрезка
    Behind,
    /// Впереди отрезка
    Beyond,
    /// На отрезке
    Between,
}

/// Расположение относительно прямой
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineArea {
    InPositive,
    InNegative,
    InBoth,
}

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

/// Бесконечная прямая на плоскости
#[derive(Debug, Clone, Copy)]
pub struct InfiniteLine2d {
    /// Вектор направления
    pub direction: Vec2,
    /// Точка, которую пересекает прямая
    pub point: Vec2,
}

impl InfiniteLine2d {
    /// Попробовать создать прямую из отрезка
    pub fn try_from_segment(segment: &LineSegment2d) -> Option<InfiniteLine2d> {
        segment.direction().map(|direction| InfiniteLine2d {
            direction,
            point: segment.a,
        })
    }

    /// Получить косинус угла между двумя прямыми
    pub fn angle_cos(a: &InfiniteLine2d, b: &InfiniteLine2d) -> f32 {
        a.direction.dot(b.direction)
    }

    /// Получить угол между двумя прямыми (в радианах)
    pub fn angle(a: &InfiniteLine2d, b: &InfiniteLine2d) -> f32 {
        Self::angle_cos(a, b).abs().acos()
    }

    /// Получить нормаль прямой
    pub fn normal(&self) -> Vec2 {
        Vec2::new(self.direction.y, -self.direction.x)
    }

    /// Найти точку пересечения двух прямых
    pub fn intersect(&self, other: &InfiniteLine2d, maybe_epsilon: Option<f32>) -> Option<Vec2> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Косое произведение (определитель) векторов направлений
        let det = self.direction.x * other.direction.y - self.direction.y * other.direction.x;

        // Если определитель близок к нулю, прямые параллельны или совпадают
        if det.abs() < epsilon {
            return None;
        }

        let n_self = self.normal();
        let d_self = -self.point.dot(n_self);

        let n_other = other.normal();
        let d_other = -other.point.dot(n_other);

        // Решение системы линейных уравнений по правилу Крамера
        let x = (-d_self * n_other.y - (-d_other * n_self.y)) / det;
        let y = (n_self.x * (-d_other) - (n_other.x * (-d_self))) / det;

        Some(Vec2::new(x, y))
    }
}

/// Окружность
#[derive(Debug, Clone, Copy)]
pub struct Circle2d {
    /// Радиус
    pub radius: f32,
    /// Координаты центра окружности
    pub center: Vec2,
}

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
    pub fn clip_line_segment(&self, line: &LineSegment2d, maybe_epsilon: Option<f32>) -> Option<LineSegment2d> {
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
        if !clip_edge(-d.x, line.a.x - self.min.x) { return None; }
        // Граница 2: Правая (X max)
        if !clip_edge(d.x, self.max.x - line.a.x) { return None; }
        // Граница 3: Нижняя (Y min)
        if !clip_edge(-d.y, line.a.y - self.min.y) { return None; }
        // Граница 4: Верхняя (Y max)
        if !clip_edge(d.y, self.max.y - line.a.y) { return None; }

        // Если интервал [t_in, t_out] остался валидным, строим отсеченный отрезок
        Some(LineSegment2d {
            a: line.a + t_in * d,
            b: line.a + t_out * d,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    /// Вершины многоугольника
    vertices: Vec<Vec2>,
}

impl Polygon {
    /// Создать многоугольник
    pub fn new(vertices: &Vec<Vec2>) -> SimpleResult<Polygon> {
        if vertices.len() < 3 {
            return Err(simple_error!("At least three vertices are required"));
        }

        Ok(Polygon {
            vertices: vertices.clone(),
        })
    }

    /// Создать треугольник
    pub fn triangle(a: &Vec2, b: &Vec2, c: &Vec2) -> Polygon {
        Polygon {
            vertices: vec![*a, *b, *c],
        }
    }

    /// Получить количество вершин
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Площадь многоугольника (со знаком)
    pub fn signed_area(&self) -> f32 {
        let n = self.vertices.len();
        let mut sum = self.vertices[0].x * (self.vertices[1].y - self.vertices[n - 1].y)
            + self.vertices[n - 1].x * (self.vertices[0].y - self.vertices[n - 2].y);

        for i in 1..n - 1 {
            sum += self.vertices[i].x * (self.vertices[i + 1].y - self.vertices[i - 1].y);
        }

        0.5 * sum
    }

    /// Расположены ли вершины многоугольника по часовой стрелке?
    pub fn is_clockwise(&self) -> bool {
        self.signed_area() > 0.0
    }

    /// Является ли многоугольник выпуклым?
    pub fn is_convex(&self, maybe_epsilon: Option<f32>) -> bool {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let n = self.vertices.len();

        // Многоугольники с менее чем 3 вершинами не могут быть выпуклыми
        if n < 3 {
            return false;
        }

        let mut expected_sign = 0;

        // Обходим ВСЕ ребра (от 0 до n-1)
        for i in 0..n {
            let current_v = self.vertices[i];
            let next_v = self.vertices[(i + 1) % n]; // Замыкание на начало

            // Создаем направленный отрезок ребра
            let edge = LineSegment2d {
                a: current_v,
                b: next_v,
            };

            // Проверяем положение ВСЕХ остальных вершин относительно этого ребра
            for j in 0..n {
                // Пропускаем вершины, которые образуют само это ребро
                if j == i || j == (i + 1) % n {
                    continue;
                }

                let test_point = self.vertices[j];

                // Используем площадь треугольника (cross product) как в вашем classify_point
                let a2b = edge.b - edge.a;
                let a2p = test_point - edge.a;
                let area = a2b.x * a2p.y - a2p.x * a2b.y;

                // Если точка лежит строго на прямой ребра, это допустимо для выпуклого полигона
                if area.abs() < epsilon {
                    continue;
                }

                let current_sign = if area > 0.0 { 1 } else { -1 };

                if expected_sign == 0 {
                    // Запоминаем направление первого обнаруженного поворота
                    expected_sign = current_sign;
                } else if current_sign != expected_sign {
                    // Если нашли точку с противоположной стороны — полигон вогнутый
                    return false;
                }
            }
        }

        true
    }

    pub fn contains_point(&self, point: &Vec2) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }

        let mut inside = false;

        // Перебираем все ребра. j — это предыдущая вершина относительно i
        let mut j = n - 1;
        for i in 0..n {
            let p_i = self.vertices[i];
            let p_j = self.vertices[j];

            // 1. Проверяем, пересекает ли луч ребро по вертикали
            let intersects_y = ((p_i.y <= point.y) && (point.y < p_j.y))
                || ((p_j.y <= point.y) && (point.y < p_i.y));

            if intersects_y {
                // 2. Находим X-координату пересечения луча с ребром по формуле подобия треугольников
                let intersect_x = (p_j.x - p_i.x) * (point.y - p_i.y) / (p_j.y - p_i.y) + p_i.x;

                // Если точка левее пересечения — луч попал в ребро
                if point.x < intersect_x {
                    inside = !inside;
                }
            }

            j = i;
        }

        inside
    }

    pub fn is_line_segment_intersects(&self, line: &LineSegment2d, maybe_epsilon: Option<f32>) -> bool {
        self.clip_line_segment(line, maybe_epsilon).is_some()
    }

    pub fn clip_line_segment(&self, line: &LineSegment2d, maybe_epsilon: Option<f32>) -> Option<LineSegment2d> {
        let epsilon = maybe_epsilon.unwrap_or(1e-5);
        let n = self.vertices.len();
        if n < 3 {
            return None;
        }

        let direction = line.direction_not_normalized();
        let mut t_in = f32::NEG_INFINITY;
        let mut t_out = f32::INFINITY;

        // Обходим все грани многоугольника
        // j — начальная вершина ребра, i — конечная
        let mut j = n - 1;
        for i in 0..n {
            // Получаем внешнюю нормаль для текущего ребра (начинающегося в вершине j)
            let n_vec = self.get_outer_normal_for(j)?;
            let dn = direction.dot(n_vec);

            if dn.abs() < epsilon {
                // Отрезок параллелен текущему ребру
                // Если начальная точка отрезка находится снаружи от ребра, отрезок целиком невидим
                if (self.vertices[j] - line.a).dot(n_vec) > 0.0 {
                    return None;
                }
                // Иначе отрезок лежит во внутренней полуплоскости грани, продолжаем проверку
                j = i;
                continue;
            }

            let pn = (self.vertices[j] - line.a).dot(n_vec);
            let t = pn / dn;

            if dn > 0.0 {
                // Отрезок направлен изнутри многоугольника наружу
                t_out = t_out.min(t);
            } else {
                // Отрезок направлен снаружи внутрь многоугольника
                t_in = t_in.max(t);
            }

            // Если время входа превысило время выхода, пересечения с многоугольником нет
            if t_out <= t_in {
                return None;
            }

            j = i;
        }

        // Ограничиваем параметры t рамками исходного отрезка [0.0, 1.0]
        t_in = t_in.max(0.0);
        t_out = t_out.min(1.0);

        // Финальная проверка на валидность интервала видимости
        if t_out <= t_in {
            return None;
        }

        // Вычисляем новые (отсеченные) координаты точек
        Some(LineSegment2d {
            a: line.a + t_in * direction,
            b: line.a + t_out * direction,
        })
    }

    /// Получить внешнюю нормаль для отрезка многоугольника.
    /// Отрезок определяется по индексу его первой вершины.
    fn get_outer_normal_for(&self, start_index: usize) -> Option<Vec2> {
        let n = self.vertices.len();
        if start_index >= n {
            return None;
        }

        // Индексы следующей и тестовой (третьей) вершин с зацикливанием через остаток
        let end_index = (start_index + 1) % n;
        let test_index  = (start_index + 2) % n;

        let first_vertex  = self.vertices[start_index];
        let end_vertex = self.vertices[end_index];
        let test_vertex  = self.vertices[test_index];

        // Первичное построение перпендикуляра к ребру
        let mut n_vec = Vec2::new(first_vertex.y - end_vertex.y, end_vertex.x - first_vertex.x);

        // Если нормаль смотрит внутрь (в сторону тестовой вершины end_vertex), разворачиваем её
        if (test_vertex - first_vertex).dot(n_vec) > 0.0 {
            n_vec = -n_vec;
        }

        Some(n_vec)
    }
}
