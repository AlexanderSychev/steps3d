use glam::{Mat2, Vec2, Vec2Swizzles};
use num_complex::{Complex32, ComplexFloat};

mod geometry;

/// Проверка некоторые соответствия между библиотеками "GLM" (C++) и "glam" (Rust)
fn glm_glam_correspondences() {
    println!("Некоторые соответствие между библиотеками \"GLM\" (C++) и \"glam\" (Rust)\n");

    let a = Vec2::new(1.0, 1.0);
    println!("Вектор (1, 1)");
    println!("  GLM: glm::vec2 a(1.0f);");
    println!("  glam: let a = Vec2::new(1.0, 1.0)");
    println!("  результат: a = {:?}\n", &a);

    let b = a + Vec2::splat(1.0);
    println!("Вектор (2, 2)");
    println!("  GLM: glm::vec2 v = a + 1.0f;");
    println!("  glam: let b = a + Vec2::splat(1.0);");
    println!("  результат: b = {:?}\n", &b);

    let c = Vec2::new(2.0, -1.0);
    println!("Вектор (2, -1)");
    println!("  GLM: glm::vec2 c(2.0f, -1.0f);");
    println!("  glam: let c = Vec2::new(2.0, -1.0);");
    println!("  результат: c = {:?}\n", &c);

    let d: Vec2 = a + 2.0 * c;
    println!("Сложение и умножение на число");
    println!("  GLM: glm::vec2 d = a + 2.0 * f;");
    println!("  glam: let d = a + 2.0 * c;");
    println!("  результат: d = {:?}\n", &d);

    let e = -d;
    println!("Обратный вектор");
    println!("  GLM: glm::vec2 e = -d");
    println!("  glam: let e = -d;");
    println!("  результат: e = {:?}\n", &e);

    let f = e.normalize();
    println!("Нормализация вектора");
    println!("  GLM: glm::vec2 f = glm::normalize(e);");
    println!("  glam: let f = e.normalize();");
    println!("  результат: f = {:?}\n", &f);

    println!("Свиззлинг");
    let a2 = a.xy();
    println!("  GLM: glm::vec2 a2 = a.xy;");
    println!("  glam: let a2 = a.xy();");
    println!("  результат: a2 = {:?}", &a2);
    let b2 = a.xy();
    println!("  GLM: glm::vec2 b2 = a.xy();");
    println!("  glam: let b2 = a.xy();");
    println!("  результат: b2 = {:?}", &b2);
    println!("Легко видеть, что \"glam\" поддерживает только методы\n");

    let dot2 = Vec2::new(1.0, 1.0).dot(d);
    println!("Скалярное произведение");
    println!("  GLM: float dot2 = glm::dot(glm::vec2(1), d)");
    println!("  glam: let dot2 = Vec2::new(1.0, 1.0).dot(d);");
    println!("  результат: dot2 = {:?}\n", &dot2);

    let len = e.length();
    println!("Длина вектора");
    println!("  GLM: float len = glm::length(e);");
    println!("  glam: let len = e.length();");
    println!("  результат: len = {:?}\n", &len);

    let dist = b.distance(c);
    println!("Расстояние между векторами");
    println!("  GLM: float dist = glm::distabce(b, c);");
    println!("  glam: let dist = b.distance(c);");
    println!("  результат: dist = {:?}\n", &dist);

    let res = a2 == b2;
    println!("Равенство векторов");
    println!("  GLM: bool res = glm:all(glm::equal(a2, b2))");
    println!("  glam: let res = a2 == b2;");
    println!("  результат: res = {:?}\n", &res);

    let m = Mat2::from_cols_array(&[0.0, 1.0, 2.0, 3.0]);
    println!("Матрица");
    println!("  GLM: glm::mat2(0, 1, 2, 3)");
    println!("  glam: let m = Mat2::from_cols_array(&[0.0, 1.0, 2.0, 3.0]);");
    println!("  результат: m = {:?}\n", &m);

    let n = Mat2::from_cols(m.x_axis * m.x_axis, m.y_axis * m.y_axis);
    println!("Покомпонетное перемножение (оно же \"адамарово произведение\"");
    println!("  GLM: glm::mat2 n = matrixCompMult(m, m)");
    println!("  glam: let n = Mat2::from_cols(m.x_axis * m.x_axis, m.y_axis * m.y_axis);");
    println!("  результат: n = {:?}", &n);
    println!("Матрицы должны быть одинакового размера\n");

    let t = m.transpose();
    println!("Транспонирование");
    println!("  GLM: glm::mat2 n = transpose(m)");
    println!("  glam: let t = m.transpose();");
    println!("  результат: t = {:?}\n", &t);

    let mi = m.inverse();
    println!("Обратная матрица");
    println!("  GLM: glm::mat2 n = inverse(m)");
    println!("  glam: let mi = m.inverse();");
    println!("  результат: mi = {:?}\n", &mi);

    let u = mi * c + a;
    println!("Умножение матрицы на вектор");
    println!("  GLM: glm::vec2 u = mi * c + a;");
    println!("let u = mi * c + a;");
    println!("  результат: u = {:?}\n", &u);
}

/// Проверка комплексных чисел
fn complex_laws() {
    println!("Комплексные числа: проверяем математические операции и утверждения\n");

    let a = Complex32::new(1.0, 2.0);
    let b = Complex32::new(3.0, 7.0);

    print!("Сложение: a + b = ((ax + bx) + i * (ay + by)). ");
    let sum = a + b;
    assert_eq!(sum, Complex32::new(a.re + b.re, a.im + b.im));
    println!("✓");

    print!("Умножение: a * b = (ax * bx + ay * by) + i * (ax * by + ay * bx))). ");
    let mul = a * b;
    assert_eq!(
        mul,
        Complex32::new(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re)
    );
    println!("✓");

    print!("Сопряжение: a` = ax - i * ay. ");
    let a_conj = a.conj();
    assert_eq!(a_conj, Complex32::new(a.re, -a.im));
    println!("✓");

    print!("Модуль: |a| = sqrt(a * a`) = sqrt(ax^2 + ay^2). ");
    let abs_a = a.abs();
    assert_eq!(abs_a, (a.re * a.re + a.im * a.im).sqrt());
    println!("✓");
}

/// Прямая на плоскости
fn straight_line() {
    let a = 1.0f32;
    let b = 2.0f32;
    let c = -3.0f32;
    println!(
        "Есть прямая, определяемая уравнением \"ax + by + d = 0\", где a = {:?}, b = {:?} и c = {:?}",
        &a, &b, &c
    );

    let divider: f32 = (a * a + b * b).sqrt();
    let n = Vec2::new(a / divider, b / divider);
    let d = c / divider;
    println!(
        "Уравнение можно переопределить в векторной форме \"(n, u) + d = 0\", где u содежит x и y, n = {:?} и d = {:?}",
        &n, &d
    );
    print!("При этом длина вектора N равна единице. ");
    assert!((1.0 - n.length()).abs() < f32::EPSILON);
    println!("✓");
}

fn classify_point_2d_showcase() {
    use geometry::LineSegment2d;

    let line_seg = LineSegment2d {
        a: Vec2::new(1.0, 2.0),
        b: Vec2::new(3.0, 5.0),
    };
    let points: Vec<Vec2> = vec![
        Vec2::new(2.0, 4.0),
        Vec2::new(4.0, 5.0),
        Vec2::new(1.0, 2.0),
        Vec2::new(3.0, 5.0),
        Vec2::new(2.0, 3.5),
    ];
    for &point in &points {
        println!(
            "Вопрос: Где лежит точка {:?} относительно отрезка {:?}?",
            &point, &line_seg
        );
        let where_is_point = line_seg.classify_point(&point, None);
        println!("Ответ: {:?}", &where_is_point);
    }
}

fn classify_circle_showcase() {
    use geometry::{Circle2d, LineSegment2d};

    let line_seg = LineSegment2d {
        a: Vec2::new(1.0, 2.0),
        b: Vec2::new(3.0, 5.0),
    };
    let circles: Vec<Circle2d> = vec![
        Circle2d {
            radius: 1.0,
            center: Vec2::new(1.0, 4.0),
        },
        Circle2d {
            radius: 1.0,
            center: Vec2::new(3.0, 2.0),
        },
        Circle2d {
            radius: 1.0,
            center: Vec2::new(2.0, 3.5),
        },
    ];

    for &circle in &circles {
        println!(
            "Вопрос: Где лежит окружность {:?} относительно отрезка {:?}?",
            &circle, &line_seg
        );
        let where_is_point = line_seg.classify_circle(&circle, None);
        println!("Ответ: {:?}", &where_is_point);
    }
}

fn classify_aabb_showcase() {
    use geometry::{AxisAlignedBoundingBox2d, LineSegment2d};

    let line_seg = LineSegment2d {
        a: Vec2::new(1.0, 2.0),
        b: Vec2::new(3.0, 5.0),
    };
    let boxes: Vec<AxisAlignedBoundingBox2d> = vec![
        AxisAlignedBoundingBox2d {
            min: Vec2::new(0.0, 2.5),
            max: Vec2::new(1.0, 3.5),
        },
        AxisAlignedBoundingBox2d {
            min: Vec2::new(3.0, 3.0),
            max: Vec2::new(4.0, 4.0),
        },
        AxisAlignedBoundingBox2d {
            min: Vec2::new(1.5, 4.0),
            max: Vec2::new(3.0, 4.0),
        },
    ];
    for &bx in &boxes {
        println!(
            "Вопрос: Где лежит прямоугольник {:?} относительно отрезка {:?}?",
            &bx, &line_seg
        );
        let where_is_point = line_seg.classify_aabb(&bx, None);
        println!("Ответ: {:?}", &where_is_point);
    }
}

fn infinite_line_2d_showcase() {
    use geometry::{InfiniteLine2d, LineSegment2d};

    let infinite_line_a = InfiniteLine2d::try_from_segment(&LineSegment2d {
        a: Vec2::new(1.0, 2.0),
        b: Vec2::new(3.0, 5.0),
    })
    .unwrap();

    let infinite_line_b = InfiniteLine2d::try_from_segment(&LineSegment2d {
        a: Vec2::new(0.0, 2.0),
        b: Vec2::new(3.0, 4.0),
    })
    .unwrap();

    let angle_rad = InfiniteLine2d::angle(&infinite_line_a, &infinite_line_b);
    let angle_deg = angle_rad.to_degrees();

    println!(
        "Угол между линиями {:?} и {:?}: {:?} градусов ({:?} радиан)",
        &infinite_line_a, &infinite_line_b, &angle_deg, &angle_rad
    );
}

fn point_aabb_distance_showcase() {
    use geometry::AxisAlignedBoundingBox2d;

    let bx = AxisAlignedBoundingBox2d {
        min: Vec2::new(0.0, 2.5),
        max: Vec2::new(1.0, 3.5),
    };

    let points: Vec<Vec2> = vec![
        Vec2::new(2.0, 4.0),
        Vec2::new(1.0, 5.0),
        Vec2::new(-1.0, -4.0),
    ];
    for &point in &points {
        println!(
            "Вопрос: Каково расстояние от точки {:?} до прямоугольника {:?}?",
            &point, &bx
        );
        let distance = bx.distance_to_point(&point);
        println!("Ответ: {:?}", &distance);
    }
}

fn triangle_square_showcase() {
    use geometry::Polygon2d;

    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(2.0, 1.0);
    let c = Vec2::new(0.5, 1.0);
    let triangle = Polygon2d::triangle(&a, &b, &c);

    let square = triangle.signed_area();

    println!(
        "Площать треугольника ABC, где A = {:?}, B = {:?} и C = {:?}: {:?}",
        &a, &b, &c, &square
    );
    println!(
        "Расположены ли точки A, B и C по часовой стрелке? {:?}",
        triangle.is_clockwise()
    );
}

fn convex_polygon_showcase() {
    use geometry::Polygon2d;

    print!("Параллелограмм выпуклый. ");
    let parallelogram = Polygon2d::new(&vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 3.0),
        Vec2::new(4.0, 3.0),
        Vec2::new(3.0, 1.0),
    ])
    .unwrap();
    assert!(parallelogram.is_convex(None));
    println!("✓");

    print!("Неправильный пятиугольник не выпуклый. ");
    let strange = Polygon2d::new(&vec![
        Vec2::new(2.0, 4.0),
        Vec2::new(1.0, 6.0),
        Vec2::new(3.0, 5.0),
        Vec2::new(4.0, 6.0),
        Vec2::new(3.0, 4.0),
    ])
    .unwrap();
    assert!(!strange.is_convex(None));
    println!("✓");
}

fn triangles_showcase() {
    use geometry::Polygon2d;

    println!("Разбивка набора вершин на треугольники");
    let vertices = vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(2.5, 1.5),
        Vec2::new(2.0, 3.0),
        Vec2::new(4.0, 0.5),
        Vec2::new(4.5, 2.5),
        Vec2::new(5.0, 3.5),
    ];
    println!("  Вершины:");
    for vertex in &vertices {
        println!("    {:?}", vertex);
    }

    let triangles = Polygon2d::triangulate_vertices(&vertices, None).unwrap();
    println!("  Треугольники:");
    for triangle in &triangles {
        println!("    {:?}", triangle);
    }
}

fn cross_product_showcase() {
    use glam::Vec3;
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(2.5, 1.9, -4.3);
    println!("[a,b] = {:?}", a.cross(b));
}

fn main() {
    glm_glam_correspondences();
    complex_laws();
    straight_line();
    classify_point_2d_showcase();
    classify_circle_showcase();
    classify_aabb_showcase();
    point_aabb_distance_showcase();
    infinite_line_2d_showcase();
    triangle_square_showcase();
    convex_polygon_showcase();
    triangles_showcase();
    cross_product_showcase();
}
