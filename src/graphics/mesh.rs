//! Базовые трехмерные "примитивы".

use glam::{Vec2, Vec3};
use simple_error::{SimpleResult, simple_error};
use std::f32::consts::{PI, TAU};

/// Вершина
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// Позиция вершины в системе координат сетки
    pub position: Vec3,
    /// Текстурные координаты вершины
    pub texture_coords: Vec2,
    /// Нормаль
    pub normal: Vec3,
    /// Касательный вектор
    pub tangent: Vec3,
    /// Бинормаль
    pub binormal: Vec3,
}

/// Сетка вершин: базовый трехмерный объект
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Вершины
    pub vertices: Vec<Vertex>,
    /// Индексы вершин
    pub indices: Vec<u32>,
}
impl Mesh {
    /// Создать куб заданного размера
    pub fn try_new_cube(maybe_size: Option<Vec3>) -> SimpleResult<Self> {
        let size = maybe_size.unwrap_or(Vec3::ONE);
        if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
            return Err(simple_error!("Cube size must be finite and positive"));
        }

        let hx = size.x * 0.5;
        let hy = size.y * 0.5;
        let hz = size.z * 0.5;

        // Вспомогательная функция для сборки одной грани из 4 углов (по часовой стрелке снаружи)
        let make_face =
            |p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, n: Vec3, t: Vec3, b: Vec3| -> [Vertex; 4] {
                [
                    Vertex {
                        position: p0,
                        texture_coords: Vec2::new(0.0, 0.0),
                        normal: n,
                        tangent: t,
                        binormal: b,
                    },
                    Vertex {
                        position: p1,
                        texture_coords: Vec2::new(0.0, 1.0),
                        normal: n,
                        tangent: t,
                        binormal: b,
                    },
                    Vertex {
                        position: p2,
                        texture_coords: Vec2::new(1.0, 1.0),
                        normal: n,
                        tangent: t,
                        binormal: b,
                    },
                    Vertex {
                        position: p3,
                        texture_coords: Vec2::new(1.0, 0.0),
                        normal: n,
                        tangent: t,
                        binormal: b,
                    },
                ]
            };

        let mut vertices = Vec::with_capacity(24);

        // 1. Ближняя грань (-Z, смотрит на камеру по умолчанию)
        vertices.extend_from_slice(&make_face(
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(-hx, hy, -hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(hx, -hy, -hz),
            Vec3::NEG_Z,
            Vec3::X,
            Vec3::Y,
        ));

        // 2. Дальняя грань (+Z)
        vertices.extend_from_slice(&make_face(
            Vec3::new(hx, -hy, hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(-hx, hy, hz),
            Vec3::new(-hx, -hy, hz),
            Vec3::Z,
            Vec3::NEG_X,
            Vec3::Y,
        ));

        // 3. Левая грань (-X)
        vertices.extend_from_slice(&make_face(
            Vec3::new(-hx, -hy, hz),
            Vec3::new(-hx, hy, hz),
            Vec3::new(-hx, hy, -hz),
            Vec3::new(-hx, -hy, -hz),
            Vec3::NEG_X,
            Vec3::NEG_Z,
            Vec3::Y,
        ));

        // 4. Правая грань (+X)
        vertices.extend_from_slice(&make_face(
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(hx, -hy, hz),
            Vec3::X,
            Vec3::Z,
            Vec3::Y,
        ));

        // 5. Верхняя грань (+Y)
        vertices.extend_from_slice(&make_face(
            Vec3::new(-hx, hy, -hz),
            Vec3::new(-hx, hy, hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(hx, hy, -hz),
            Vec3::Y,
            Vec3::X,
            Vec3::Z,
        ));

        // 6. Нижняя грань (-Y)
        vertices.extend_from_slice(&make_face(
            Vec3::new(-hx, -hy, hz),
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, -hy, hz),
            Vec3::NEG_Y,
            Vec3::X,
            Vec3::NEG_Z,
        ));

        // Индексы: по 2 треугольника (6 индексов) на каждую из 6 граней
        let mut indices = Vec::with_capacity(36);
        for face in 0..6u32 {
            let base = face * 4;
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }

        Ok(Self { vertices, indices })
    }

    /// Создать UV-сферу с центром в (0, 0, 0)
    /// * `radius` — радиус сферы (> 0)
    /// * `rings` — количество сегментов по вертикали (широта, >= 3)
    /// * `sectors` — количество сегментов по горизонтали (долгота, >= 3)
    pub fn try_new_sphere(radius: f32, rings: u32, sectors: u32) -> SimpleResult<Self> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(simple_error!("Sphere radius must be finite and positive"));
        }
        if rings < 3 || sectors < 3 {
            return Err(simple_error!("Sphere rings and sectors must be at least 3"));
        }

        let num_vertices = ((rings + 1) * (sectors + 1)) as usize;
        let num_indices = (rings * sectors * 6) as usize;
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut indices = Vec::with_capacity(num_indices);

        let inv_rings = 1.0 / rings as f32;
        let inv_sectors = 1.0 / sectors as f32;

        for i in 0..=rings {
            let v = i as f32 * inv_rings;
            // phi от 0 (верхний полюс +Y) до PI (нижний полюс -Y)
            let phi = v * PI;
            let (sin_phi, cos_phi) = phi.sin_cos();

            for j in 0..=sectors {
                let u = j as f32 * inv_sectors;
                let theta = u * TAU;
                let (sin_theta, cos_theta) = theta.sin_cos();

                // Нормаль в левосторонней системе (+Y вверх, +Z вглубь)
                let normal = Vec3::new(sin_phi * cos_theta, cos_phi, sin_phi * sin_theta);
                let position = normal * radius;

                // Касательный вектор вдоль увеличения долготы (u)
                let tangent = Vec3::new(-sin_theta, 0.0, cos_theta);
                // Бинормаль (левосторонняя: cross(normal, tangent) направлен вниз к +v)
                let binormal = normal.cross(tangent);

                vertices.push(Vertex {
                    position,
                    texture_coords: Vec2::new(u, v),
                    normal,
                    tangent,
                    binormal,
                });
            }
        }

        let stride = sectors + 1;
        for i in 0..rings {
            for j in 0..sectors {
                let a = i * stride + j;
                let b = (i + 1) * stride + j;
                let c = (i + 1) * stride + (j + 1);
                let d = i * stride + (j + 1);

                // Обход по часовой стрелке снаружи (левая система координат)
                indices.extend_from_slice(&[a, d, c, c, b, a]);
            }
        }

        Ok(Self { vertices, indices })
    }

    /// Создать тор в плоскости XZ вокруг оси +Y с центром в (0, 0, 0)
    /// * `major_radius` — расстояние от центра тора до центра трубки (> 0)
    /// * `minor_radius` — радиус самой трубки (> 0)
    /// * `rings` — количество сегментов основного кольца (>= 3)
    /// * `sides` — количество сегментов сечения трубки (>= 3)
    pub fn try_new_torus(
        major_radius: f32,
        minor_radius: f32,
        rings: u32,
        sides: u32,
    ) -> SimpleResult<Self> {
        if !major_radius.is_finite() || major_radius <= 0.0 {
            return Err(simple_error!(
                "Torus major radius must be finite and positive"
            ));
        }
        if !minor_radius.is_finite() || minor_radius <= 0.0 {
            return Err(simple_error!(
                "Torus minor radius must be finite and positive"
            ));
        }
        if rings < 3 || sides < 3 {
            return Err(simple_error!("Torus rings and sides must be at least 3"));
        }

        let num_vertices = ((rings + 1) * (sides + 1)) as usize;
        let num_indices = (rings * sides * 6) as usize;
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut indices = Vec::with_capacity(num_indices);

        let inv_rings = 1.0 / rings as f32;
        let inv_sides = 1.0 / sides as f32;

        for i in 0..=rings {
            let u = i as f32 * inv_rings;
            let theta = u * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            // Касательный вектор вдоль основного кольца
            let tangent = Vec3::new(-sin_theta, 0.0, cos_theta);

            for j in 0..=sides {
                let v = j as f32 * inv_sides;
                let phi = v * TAU;
                let (sin_phi, cos_phi) = phi.sin_cos();

                let dist = major_radius + minor_radius * cos_phi;

                let position =
                    Vec3::new(cos_theta * dist, minor_radius * sin_phi, sin_theta * dist);
                let normal = Vec3::new(cos_theta * cos_phi, sin_phi, sin_theta * cos_phi);
                let binormal = normal.cross(tangent);

                vertices.push(Vertex {
                    position,
                    texture_coords: Vec2::new(u, v),
                    normal,
                    tangent,
                    binormal,
                });
            }
        }

        let stride = sides + 1;
        for i in 0..rings {
            for j in 0..sides {
                let a = i * stride + j;
                let b = (i + 1) * stride + j;
                let c = (i + 1) * stride + (j + 1);
                let d = i * stride + (j + 1);

                // Обход по часовой стрелке снаружи
                indices.extend_from_slice(&[a, b, c, c, d, a]);
            }
        }

        Ok(Self { vertices, indices })
    }

    /// Создать конус вдоль оси +Y с центром в (0, 0, 0) (основание на -height/2, вершина на +height/2)
    /// * `radius` — радиус основания (> 0)
    /// * `height` — высота конуса (> 0)
    /// * `sectors` — количество сегментов окружности (>= 3)
    pub fn try_new_cone(radius: f32, height: f32, sectors: u32) -> SimpleResult<Self> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(simple_error!("Cone radius must be finite and positive"));
        }
        if !height.is_finite() || height <= 0.0 {
            return Err(simple_error!("Cone height must be finite and positive"));
        }
        if sectors < 3 {
            return Err(simple_error!("Cone sectors must be at least 3"));
        }

        let half_h = height * 0.5;
        let inv_sectors = 1.0 / sectors as f32;

        // Наклон образующей конуса для расчета боковых нормалей
        let slope_len = (radius * radius + height * height).sqrt();
        let ny = radius / slope_len;
        let nr = height / slope_len;

        let mut vertices = Vec::with_capacity((sectors as usize + 1) * 3 + 1);
        let mut indices = Vec::with_capacity(sectors as usize * 6);

        // 1. Боковая поверхность: пары вершин (основание -> вершина) для каждого сектора
        for j in 0..=sectors {
            let u = j as f32 * inv_sectors;
            let theta = u * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            let normal = Vec3::new(nr * cos_theta, ny, nr * sin_theta);
            let tangent = Vec3::new(-sin_theta, 0.0, cos_theta);
            let binormal = normal.cross(tangent);

            // Вершина на окружности основания (индекс j * 2)
            vertices.push(Vertex {
                position: Vec3::new(radius * cos_theta, -half_h, radius * sin_theta),
                texture_coords: Vec2::new(u, 0.0),
                normal,
                tangent,
                binormal,
            });

            // Вершина на пике конуса (индекс j * 2 + 1)
            vertices.push(Vertex {
                position: Vec3::new(0.0, half_h, 0.0),
                texture_coords: Vec2::new(u, 1.0),
                normal,
                tangent,
                binormal,
            });
        }

        for j in 0..sectors {
            let b0 = j * 2;
            let t0 = j * 2 + 1;
            let b1 = (j + 1) * 2;

            // Обход по часовой стрелке снаружи
            indices.extend_from_slice(&[b0, b1, t0]);
        }

        // 2. Нижняя крышка (основание на y = -half_h)
        let center_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: Vec3::new(0.0, -half_h, 0.0),
            texture_coords: Vec2::new(0.5, 0.5),
            normal: Vec3::NEG_Y,
            tangent: Vec3::X,
            binormal: Vec3::NEG_Z,
        });

        let ring_start = vertices.len() as u32;
        for j in 0..=sectors {
            let u = j as f32 * inv_sectors;
            let theta = u * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            vertices.push(Vertex {
                position: Vec3::new(radius * cos_theta, -half_h, radius * sin_theta),
                texture_coords: Vec2::new(0.5 + 0.5 * cos_theta, 0.5 + 0.5 * sin_theta),
                normal: Vec3::NEG_Y,
                tangent: Vec3::X,
                binormal: Vec3::NEG_Z,
            });
        }

        for j in 0..sectors {
            let v0 = ring_start + j;
            let v1 = ring_start + j + 1;
            // Обход по часовой стрелке при взгляде снизу (со стороны -Y)
            indices.extend_from_slice(&[center_idx, v0, v1]);
        }

        Ok(Self { vertices, indices })
    }

    /// Создать цилиндр вдоль оси +Y с центром в (0, 0, 0)
    /// * `radius` — радиус цилиндра (> 0)
    /// * `height` — высота цилиндра (> 0)
    /// * `sectors` — количество сегментов окружности (>= 3)
    pub fn try_new_cylinder(radius: f32, height: f32, sectors: u32) -> SimpleResult<Self> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(simple_error!("Cylinder radius must be finite and positive"));
        }
        if !height.is_finite() || height <= 0.0 {
            return Err(simple_error!("Cylinder height must be finite and positive"));
        }
        if sectors < 3 {
            return Err(simple_error!("Cylinder sectors must be at least 3"));
        }

        let half_h = height * 0.5;
        let inv_sectors = 1.0 / sectors as f32;

        // Боковая стенка: 2 * (sectors + 1), две крышки: 2 * (1 + sectors + 1)
        let num_vertices = (sectors as usize + 1) * 4 + 2;
        // Боковая стенка: sectors * 6, две крышки: 2 * sectors * 3
        let num_indices = sectors as usize * 12;

        let mut vertices = Vec::with_capacity(num_vertices);
        let mut indices = Vec::with_capacity(num_indices);

        // 1. Боковая поверхность
        for j in 0..=sectors {
            let u = j as f32 * inv_sectors;
            let theta = u * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            let normal = Vec3::new(cos_theta, 0.0, sin_theta);
            let tangent = Vec3::new(-sin_theta, 0.0, cos_theta);
            let binormal = Vec3::Y;

            let x = radius * cos_theta;
            let z = radius * sin_theta;

            // Нижняя вершина стенки (индекс j * 2)
            vertices.push(Vertex {
                position: Vec3::new(x, -half_h, z),
                texture_coords: Vec2::new(u, 0.0),
                normal,
                tangent,
                binormal,
            });

            // Верхняя вершина стенки (индекс j * 2 + 1)
            vertices.push(Vertex {
                position: Vec3::new(x, half_h, z),
                texture_coords: Vec2::new(u, 1.0),
                normal,
                tangent,
                binormal,
            });
        }

        for j in 0..sectors {
            let b0 = j * 2;
            let t0 = j * 2 + 1;
            let b1 = (j + 1) * 2;
            let t1 = (j + 1) * 2 + 1;

            // Обход по часовой стрелке снаружи
            indices.extend_from_slice(&[b0, b1, t1, t1, t0, b0]);
        }

        // 2. Верхняя крышка (y = +half_h)
        let top_center_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: Vec3::new(0.0, half_h, 0.0),
            texture_coords: Vec2::new(0.5, 0.5),
            normal: Vec3::Y,
            tangent: Vec3::X,
            binormal: Vec3::Z,
        });

        let top_ring_start = vertices.len() as u32;
        for j in 0..=sectors {
            let u = j as f32 * inv_sectors;
            let theta = u * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            vertices.push(Vertex {
                position: Vec3::new(radius * cos_theta, half_h, radius * sin_theta),
                texture_coords: Vec2::new(0.5 + 0.5 * cos_theta, 0.5 + 0.5 * sin_theta),
                normal: Vec3::Y,
                tangent: Vec3::X,
                binormal: Vec3::Z,
            });
        }

        for j in 0..sectors {
            let v0 = top_ring_start + j;
            let v1 = top_ring_start + j + 1;
            // Обход по часовой стрелке при взгляде сверху (со стороны +Y)
            indices.extend_from_slice(&[top_center_idx, v1, v0]);
        }

        // 3. Нижняя крышка (y = -half_h)
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: Vec3::new(0.0, -half_h, 0.0),
            texture_coords: Vec2::new(0.5, 0.5),
            normal: Vec3::NEG_Y,
            tangent: Vec3::X,
            binormal: Vec3::NEG_Z,
        });

        let bottom_ring_start = vertices.len() as u32;
        for j in 0..=sectors {
            let u = j as f32 * inv_sectors;
            let theta = u * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            vertices.push(Vertex {
                position: Vec3::new(radius * cos_theta, -half_h, radius * sin_theta),
                texture_coords: Vec2::new(0.5 + 0.5 * cos_theta, 0.5 + 0.5 * sin_theta),
                normal: Vec3::NEG_Y,
                tangent: Vec3::X,
                binormal: Vec3::NEG_Z,
            });
        }

        for j in 0..sectors {
            let v0 = bottom_ring_start + j;
            let v1 = bottom_ring_start + j + 1;
            // Обход по часовой стрелке при взгляде снизу (со стороны -Y)
            indices.extend_from_slice(&[bottom_center_idx, v0, v1]);
        }

        Ok(Self { vertices, indices })
    }
}
