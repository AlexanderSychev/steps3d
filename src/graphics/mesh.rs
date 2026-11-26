//! Базовые трехмерные "примитивы".
use glam::{Vec2, Vec3, Quat};

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

/// Сетка, состоящая из вершин и граней. Базовый "примитив" трехмерной модели/
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Позиция сетки в глобальной системе координат
    position: Vec3,
    /// Ориентация сетки в пространстве
    orientation: Quat,
    /// Геометрия вершины
    geometry: MeshGeometry,
}
impl Mesh {
    /// Создать куб
    pub fn new_cube() -> Self {
        todo!()
    }
}
