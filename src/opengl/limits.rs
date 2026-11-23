//! Различные лимиты GPU для OpenGL

/// Ограничения GPU
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Максимально допустимое число атрибутов вершины
    max_vertex_attribs: u32,
    /// Максимально допустимый размер текстуры
    max_texture_size: i32,
    /// Максимально допустимое смещение уровня детализации текстуры
    max_texture_lod_bias: f32,
}

impl Limits {
    ///
    pub fn new() -> Self {
        let mut result = Self {
            max_vertex_attribs: 0,
            max_texture_size: 0,
            max_texture_lod_bias: 0.0,
        };
        result.refresh();
        result
    }

    /// Получить максимально допустимое число атрибутов вершины
    pub fn get_max_vertex_attribs(&self) -> u32 {
        self.max_vertex_attribs
    }

    /// Получить максимально допустимый размер текстуры
    pub fn get_max_texture_size(&self) -> i32 {
        self.max_texture_size
    }

    /// Получить максимально допустимое смещение уровня детализации текстуры
    pub fn get_max_texture_lod_bias(&self) -> f32 {
        self.max_texture_lod_bias
    }

    /// "Опросить" GPU-устройство и обновить лимиты
    pub fn refresh(&mut self) {
        let mut max_vertex_attribs = 0;
        let mut max_texture_size = 0;
        let mut max_texture_lod_bias = 0.0;
        unsafe {
            gl::GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_vertex_attribs);
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_texture_size);
            gl::GetFloatv(gl::MAX_TEXTURE_LOD_BIAS, &mut max_texture_lod_bias);
        }

        self.max_vertex_attribs = max_vertex_attribs as u32;
        self.max_texture_size = max_texture_size as i32;
        self.max_texture_lod_bias = max_texture_lod_bias;
    }
}
