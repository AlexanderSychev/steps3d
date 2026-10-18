use super::circle_2d::Circle2d;
use super::constants::DEFAULT_GEOMETRY_EPSILON;
use super::line_segment_2d::LineSegment2d;
use glam::Vec2;
use simple_error::{SimpleResult, simple_error};

/// Вспомогательная структура для отслеживания ребер в алгоритме Боуера-Ватсона.
/// Для нее важен только сам факт связи между двумя вершинами, без учета направления.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StructuralEdge {
    pub v0: usize,
    pub v1: usize,
}

impl StructuralEdge {
    pub fn new(v0: usize, v1: usize) -> Self {
        // Всегда сортируем индексы, чтобы ребро (1, 2) и (2, 1) считались абсолютно одинаковыми
        if v0 < v1 {
            StructuralEdge { v0, v1 }
        } else {
            StructuralEdge { v0: v1, v1: v0 }
        }
    }
}

/// Треугольник для структуры Делоне, хранящий индексы вершин
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IndexedTriangle {
    pub vertices: [usize; 3],
}

impl IndexedTriangle {
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        // Для стабильности поиска ребер полезно хранить индексы отсортированными
        let mut v = [a, b, c];
        v.sort_unstable();
        IndexedTriangle { vertices: v }
    }

    /// Получить три ребра треугольника в виде пар индексов вершин
    pub fn edges(&self) -> [(usize, usize); 3] {
        [
            (self.vertices[0], self.vertices[1]),
            (self.vertices[1], self.vertices[2]),
            (self.vertices[0], self.vertices[2]),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Polygon2d {
    /// Вершины многоугольника
    vertices: Vec<Vec2>,
}

impl Polygon2d {
    /// Создать многоугольник
    pub fn new(vertices: &Vec<Vec2>) -> SimpleResult<Polygon2d> {
        if vertices.len() < 3 {
            return Err(simple_error!("At least three vertices are required"));
        }

        Ok(Polygon2d {
            vertices: vertices.clone(),
        })
    }

    /// Создать треугольник
    pub fn triangle(a: &Vec2, b: &Vec2, c: &Vec2) -> Polygon2d {
        Polygon2d {
            vertices: vec![*a, *b, *c],
        }
    }

    /// ФУНДАМЕНТАЛЬНАЯ ТРИАНГУЛЯЦИЯ ДЕЛОНЕ (Алгоритм Боуера-Ватсона)
    /// Принимает любое произвольное облако точек и гарантированно разбивает его на треугольники.
    pub fn triangulate_vertices(
        vertices: &[Vec2],
        maybe_epsilon: Option<f32>,
    ) -> SimpleResult<Vec<Polygon2d>> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);
        let n = vertices.len();

        if n < 3 {
            return Err(simple_error!(
                "Для триангуляции требуется минимум 3 вершины"
            ));
        }

        // 1. Создаем рабочий массив точек, куда скопируем исходные точки и добавим вершины супер-треугольника
        let mut pts = vertices.to_vec();

        // Находим границы облака точек для построения супер-треугольника
        let mut min_x = pts[0].x;
        let mut max_x = pts[0].x;
        let mut min_y = pts[0].y;
        let mut max_y = pts[0].y;

        for p in &pts {
            if p.x < min_x {
                min_x = p.x;
            }
            if p.x > max_x {
                max_x = p.x;
            }
            if p.y < min_y {
                min_y = p.y;
            }
            if p.y > max_y {
                max_y = p.y;
            }
        }

        let dx = max_x - min_x;
        let dy = max_y - min_y;
        let delta_max = dx.max(dy);
        let mid_x = (min_x + max_x) * 0.5;
        let mid_y = (min_y + max_y) * 0.5;

        // Расчет координат супер-треугольника, который заведомо перекрывает всё пространство с запасом
        let st_v0 = Vec2::new(mid_x - 20.0 * delta_max, mid_y - delta_max);
        let st_v1 = Vec2::new(mid_x, mid_y + 20.0 * delta_max);
        let st_v2 = Vec2::new(mid_x + 20.0 * delta_max, mid_y - delta_max);

        // Индексы вершин супер-треугольника в нашем рабочем массиве 'pts'
        let st_idx0 = pts.len();
        let st_idx1 = pts.len() + 1;
        let st_idx2 = pts.len() + 2;

        pts.push(st_v0);
        pts.push(st_v1);
        pts.push(st_v2);

        // Список активных индексированных треугольников сетки
        let mut mesh: Vec<IndexedTriangle> = Vec::new();
        // Начинаем с добавления супер-треугольника
        mesh.push(IndexedTriangle::new(st_idx0, st_idx1, st_idx2));

        // 2. ИНКРЕМЕНТНЫЙ ОБХОД: Вставляем каждую реальную точку по очереди
        for i in 0..n {
            let current_point = pts[i];

            // Временный список невалидных треугольников, чья описанная окружность содержит текущую точку
            let mut bad_triangles: Vec<IndexedTriangle> = Vec::new();

            for t in &mesh {
                if let Some(circumcircle) = Circle2d::circumcircle_for_triangle(
                    &pts[t.vertices[0]],
                    &pts[t.vertices[1]],
                    &pts[t.vertices[2]],
                    Some(epsilon),
                ) {
                    if circumcircle.contains_point_inclusive(&current_point, Some(epsilon)) {
                        bad_triangles.push(*t);
                    }
                }
            }

            // Находим ребра контура будущей полости.
            // Ребро является граничным для полости, если оно принадлежит ТОЛЬКО ОДНОМУ невалидному треугольнику.
            let mut polygon_edges: Vec<StructuralEdge> = Vec::new();

            for bt in &bad_triangles {
                let edges = [
                    StructuralEdge::new(bt.vertices[0], bt.vertices[1]),
                    StructuralEdge::new(bt.vertices[1], bt.vertices[2]),
                    StructuralEdge::new(bt.vertices[2], bt.vertices[0]),
                ];

                for edge in edges {
                    // Проверяем, сколько раз это ребро встречается среди всех невалидных треугольников
                    let mut count = 0;
                    for check_t in &bad_triangles {
                        if check_t.vertices.contains(&edge.v0)
                            && check_t.vertices.contains(&edge.v1)
                        {
                            count += 1;
                        }
                    }
                    // Если ребро уникально для этого контура разрушения — оно формирует внешнюю границу полости
                    if count == 1 {
                        polygon_edges.push(edge);
                    }
                }
            }

            // Удаляем невалидные треугольники из общей сетки
            mesh.retain(|t| !bad_triangles.contains(t));

            // Заполняем полость: соединяем граничные ребра с новой точкой `i`
            for edge in polygon_edges {
                mesh.push(IndexedTriangle::new(edge.v0, edge.v1, i));
            }
        }

        // 3. ФИЛЬТРАЦИЯ: Удаляем треугольники, связанные с супер-треугольником, и переводим в структуру Polygon
        let mut final_polygons = Vec::new();

        for t in mesh {
            let shares_vertex_with_super_triangle = t.vertices.contains(&st_idx0)
                || t.vertices.contains(&st_idx1)
                || t.vertices.contains(&st_idx2);

            if !shares_vertex_with_super_triangle {
                let poly = Polygon2d::triangle(
                    &pts[t.vertices[0]],
                    &pts[t.vertices[1]],
                    &pts[t.vertices[2]],
                );
                final_polygons.push(poly);
            }
        }

        if final_polygons.is_empty() {
            return Err(simple_error!("Критическая ошибка триангуляции сетки"));
        }

        Ok(final_polygons)
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

    /// Проверить, лежит ли точка внутри треугольника (более оптимальный алгоритм).
    /// Если это не треугольник, то вызывается "contains_point"
    pub fn point_in_triangle(&self, point: &Vec2, maybe_epsilon: Option<f32>) -> bool {
        if self.vertices.len() != 3 {
            return self.contains_point(point);
        }

        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Даем вершинам понятные геометрические имена
        let a = self.vertices[0];
        let b = self.vertices[1];
        let c = self.vertices[2];

        // Направленные векторы ребер, исходящие из базовой вершины 'c'
        let edge_ca = a - c;
        let edge_cb = b - c;

        // Вектор от базовой вершины к целевой точке
        let to_point = point - c;

        // Вычисляем скалярные произведения векторов (метрики длин и проекций)
        let dot_aa = edge_ca.dot(edge_ca);
        let dot_ab = edge_ca.dot(edge_cb);
        let dot_bb = edge_cb.dot(edge_cb);

        let dot_pa = to_point.dot(edge_ca);
        let dot_pb = to_point.dot(edge_cb);

        // Главный определитель системы линейных уравнений (детерминант)
        let det = dot_aa * dot_bb - dot_ab * dot_ab;

        // Защита от вырожденного треугольника (если все 3 точки лежат на одной прямой)
        if det.abs() < epsilon {
            // Если точка лежит прямо на линии вырожденного треугольника, считаем это попаданием
            return true;
        }

        // Вычисляем барицентрические координаты u и v через правило Крамера
        let u = (dot_pa * dot_bb - dot_pb * dot_ab) / det;
        if u < -epsilon || u > 1.0 + epsilon {
            return false;
        }

        let v = (dot_aa * dot_pb - dot_ab * dot_pa) / det;
        if v < -epsilon || v > 1.0 + epsilon {
            return false;
        }

        // Точка находится внутри треугольника, если сумма барицентрических координат <= 1
        u + v <= 1.0 + epsilon
    }

    /// Проверить, лежит ли точка внутри многоугольника
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

    pub fn is_line_segment_intersects(
        &self,
        line: &LineSegment2d,
        maybe_epsilon: Option<f32>,
    ) -> bool {
        self.clip_line_segment(line, maybe_epsilon).is_some()
    }

    pub fn clip_line_segment(
        &self,
        line: &LineSegment2d,
        maybe_epsilon: Option<f32>,
    ) -> Option<LineSegment2d> {
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
        let test_index = (start_index + 2) % n;

        let first_vertex = self.vertices[start_index];
        let end_vertex = self.vertices[end_index];
        let test_vertex = self.vertices[test_index];

        // Первичное построение перпендикуляра к ребру
        let mut n_vec = Vec2::new(first_vertex.y - end_vertex.y, end_vertex.x - first_vertex.x);

        // Если нормаль смотрит внутрь (в сторону тестовой вершины end_vertex), разворачиваем её
        if (test_vertex - first_vertex).dot(n_vec) > 0.0 {
            n_vec = -n_vec;
        }

        Some(n_vec)
    }

    /// Построить выпуклую оболочку вокруг набора точек методом Грэхема.
    /// Возвращает ошибку, если из точек невозможно собрать многоугольник (например, их меньше 3
    /// или они все лежат на одной прямой).
    pub fn convex_hull_graham(points: &[Vec2], maybe_epsilon: Option<f32>) -> SimpleResult<Self> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        if points.len() < 3 {
            return Err(simple_error!(
                "Для построения оболочки требуется минимум 3 точки"
            ));
        }

        let mut pts = points.to_vec();

        // ШАГ 1: Находим базовую (самую нижнюю и левую) точку
        // Ищем индекс элемента с минимальным Y, а при равенстве — с минимальным X
        let pivot_idx = pts
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.y.partial_cmp(&b.y)
                    .unwrap()
                    .then_with(|| a.x.partial_cmp(&b.x).unwrap())
            })
            .map(|(idx, _)| idx)
            .unwrap();

        // Перемещаем базовую точку в самое начало вектора
        pts.swap(0, pivot_idx);
        let pivot = pts[0];

        // ШАГ 2: Полярная сортировка остальных точек относительно `pivot`
        // Мы берем срез [1..] и сортируем его
        pts[1..].sort_by(|&a, &b| {
            // Векторы от базовой точки к двум тестируемым
            let va = a - pivot;
            let vb = b - pivot;

            // Косое произведение векторов (аналог определителя 2x2)
            let cross_product = va.x * vb.y - va.y * vb.x;

            if cross_product.abs() < epsilon {
                // Если точки лежат на одном луче, более близкую к pivot ставим первой
                va.length_squared()
                    .partial_cmp(&vb.length_squared())
                    .unwrap()
            } else if cross_product > 0.0 {
                // va находится правее vb (поворот влево от va к vb), значит a должен быть раньше b
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        // ШАГ 3: Обход по точкам с использованием стека (вектора)
        let mut hull: Vec<Vec2> = Vec::with_capacity(pts.len());

        for p in pts {
            // Пока в стеке есть хотя бы 2 точки, проверяем характер поворота.
            // Нам нужен только ЛЕВЫЙ поворот. Если поворот правый (или точки коллинеарны),
            // предыдущая вершина внутри «впадины» — удаляем её.
            while hull.len() >= 2 {
                let last = hull[hull.len() - 1];
                let prev = hull[hull.len() - 2];

                // Векторы ребер: из prev в last, и из last в текущую точку p
                let v1 = last - prev;
                let v2 = p - last;
                let area = v1.x * v2.y - v2.x * v1.y;

                // Если area <= epsilon, поворот правый или точки выстроены в линию.
                // Выбрасываем точку 'last' из стека.
                if area > epsilon {
                    break;
                }
                hull.pop();
            }
            hull.push(p);
        }

        // Финальная проверка: если все точки оказались на одной линии,
        // у нас не получится полноценного многоугольника.
        if hull.len() < 3 {
            return Err(simple_error!(
                "Все точки коллинеарны, невозможно построить выпуклый многоугольник"
            ));
        }

        Ok(Polygon2d { vertices: hull })
    }
}
