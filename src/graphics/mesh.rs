//! Базовые трехмерные "примитивы".

use super::scene::SceneEntity;
use glam::{EulerRot, Quat, Vec2, Vec3};
use simple_error::{SimpleResult, simple_error};

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

#[derive(Debug, Clone, PartialEq)]
pub struct MeshGeometry {
    /// Вершины
    pub vertices: Vec<Vertex>,
    /// Индексы вершин
    pub indices: Vec<u32>,
}
impl MeshGeometry {
    pub fn new_cube(size: Vec3) -> Self {
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

        Self { vertices, indices }
    }
}

/// Сетка, состоящая из вершин и граней. Базовый "примитив" трехмерной модели/
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Позиция сетки в глобальной системе координат
    position: Vec3,
    /// Ориентация сетки в пространстве
    orientation: Quat,
    /// Масштабирование модели
    scale: Vec3,
    /// Геометрия сетки
    geometry: MeshGeometry,
}
impl Mesh {
    /// Получить позицию сетки
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    /// Получить геометрию сетки для чтения
    pub fn get_geometry(&self) -> &MeshGeometry {
        &self.geometry
    }

    /// Получить геометрию сетки для записи
    pub fn get_geometry_mut(&mut self) -> &mut MeshGeometry {
        &mut self.geometry
    }

    pub fn set_geometry(&mut self, geometry: MeshGeometry) {
        self.geometry = geometry;
    }
}
impl SceneEntity for Mesh {
    fn position_entity(&mut self, position: Vec3) -> SimpleResult<()> {
        if !position.is_finite() {
            return Err(simple_error!(
                "Position point must contain only finite values"
            ));
        }
        self.position = position;
        Ok(())
    }

    fn move_entity(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if !deltas.is_finite() {
            return Err(simple_error!(
                "Position deltas must contain only finite values"
            ));
        }
        self.position += self.orientation * deltas;
        Ok(())
    }

    fn translate_entity(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if !deltas.is_finite() {
            return Err(simple_error!(
                "Position deltas must contain only finite values"
            ));
        }
        self.position += deltas;
        Ok(())
    }

    fn rotate_entity(&mut self, angles: Vec3) -> SimpleResult<()> {
        if !angles.is_finite() {
            return Err(simple_error!(
                "Rotation angles must contain only finite values"
            ));
        }
        self.orientation = Quat::from_euler(
            EulerRot::YXZ,
            -angles.y, // Yaw
            -angles.x, // Pitch
            -angles.z, // Roll
        );
        Ok(())
    }

    fn turn_entity(&mut self, angle_deltas: Vec3) -> SimpleResult<()> {
        if !angle_deltas.is_finite() {
            return Err(simple_error!(
                "Rotation angles must contain only finite values"
            ));
        }
        let delta_rot = Quat::from_euler(
            EulerRot::YXZ,
            -angle_deltas.y,
            -angle_deltas.x,
            -angle_deltas.z,
        );
        self.orientation = (self.orientation * delta_rot).normalize();
        Ok(())
    }

    fn scale_entity(&mut self, scales: Vec3) -> SimpleResult<()> {
        if !scales.is_finite() {
            return Err(simple_error!("Scales must contain only finite values"));
        }
        self.scale = scales;
        Ok(())
    }
}
