//! Различные лимиты GPU для OpenGL

use gl::{GetIntegerv, MAX_VERTEX_ATTRIBS};

/// Ограничения GPU
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Максимально допустимое число атрибутов вершины
    max_vertex_attribs: u32,
}

impl Limits {
    ///
    pub fn new() -> Self {
        let mut result = Self {
            max_vertex_attribs: 0,
        };
        result.refresh();
        result
    }

    /// Получить Максимально допустимое число атрибутов вершины
    pub fn get_max_vertex_attribs(&self) -> u32 {
        self.max_vertex_attribs
    }

    /// "Опросить" GPU-устройство и обновить лимиты
    pub fn refresh(&mut self) {
        let mut max_vertex_attribs = 0;
        unsafe {
            GetIntegerv(MAX_VERTEX_ATTRIBS, &mut max_vertex_attribs);
        }

        self.max_vertex_attribs = max_vertex_attribs as u32;
    }
}
