//! Базовые трехмерные "примитивы".
//!
//! Не зависит от конкретного графического API

use glam::{Vec2, Vec3};

/// Данные для условного треугольника <code>ABC</code>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleData<T: Clone + Copy> {
    /// Данные для точки <code>A</code>
    pub a: T,
    /// Данные для точки <code>B</code>
    pub b: T,
    /// Данные для точки <code>C</code>
    pub c: T,
}

/// Сетка, состоящая из вершин и граней.
///
/// Оформлена в "компактном" виде:
///   - Уникальный список вершин: одна вершина может использоваться несколькими треугольниками;
///   - Полигоны определяются тройками индексов вершин в списке;
///   - Текстурные координаты определяются для каждой вершины полигона;
///
/// Базовый "примитив" трехмерной модели
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Все вершины сетки
    vertices: Vec<Vec3>,
    /// Все грани сетки: "тройки" индексов используемых вершин
    triangles: Vec<TriangleData<usize>>,
    /// Текстурные координаты вершин
    texture_coords: Vec<TriangleData<Vec2>>,
}

impl Mesh {
    // Конструкторы

    /// Создать треугольник
    pub fn triangle(a: Vec3, b: Vec3, c: Vec3) -> Mesh {
        let vertices = vec![a, b, c];
        let triangles = vec![TriangleData { a: 0, b: 1, c: 2 }];
        let texture_coords = vec![TriangleData {
            a: Vec2::new(0.0, 0.0),
            b: Vec2::new(0.5, 1.0),
            c: Vec2::new(1.0, 0.0),
        }];
        Mesh {
            vertices,
            triangles,
            texture_coords,
        }
    }

    // Методы

    /// Получить вершину для чтения по индексу
    pub fn get_vertex(&self, index: usize) -> Option<&Vec3> {
        self.vertices.get(index)
    }

    /// Получить вершину для записи по индексу
    pub fn get_vertex_mut(&mut self, index: usize) -> Option<&mut Vec3> {
        self.vertices.get_mut(index)
    }

    /// Получить количество вершин
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Получить полигон для чтения по индексу
    pub fn get_triangle(&self, index: usize) -> Option<&TriangleData<usize>> {
        self.triangles.get(index)
    }

    /// Получить полигон для записи по индексу
    pub fn get_triangle_mut(&mut self, index: usize) -> Option<&mut TriangleData<usize>> {
        self.triangles.get_mut(index)
    }

    /// Получить количество треугольников
    pub fn count_triangles(&self) -> usize {
        self.triangles.len()
    }

    /// Получить текстурные координаты для чтения по индексу
    pub fn get_texture_coords(&self, index: usize) -> Option<&TriangleData<Vec2>> {
        self.texture_coords.get(index)
    }

    /// Получить текстурные координаты для записи по индексу
    pub fn get_texture_coords_mut(&mut self, index: usize) -> Option<&mut TriangleData<Vec2>> {
        self.texture_coords.get_mut(index)
    }

    /// Получить количество текстурных координат
    pub fn texture_coords_count(&self) -> usize {
        self.texture_coords.len()
    }
}
