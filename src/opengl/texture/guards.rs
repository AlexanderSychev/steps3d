//! Структуры-"защитники" для текстур
//!
//! Их задача: "проконтролировать" некоторое состояние OpenGL и, при необходимости,
//! откатить его при выходе из области видимости

use super::target::TextureTarget;
use std::num::NonZeroU32;

/// Защитник для выравнивания данных пикселя при распаковке
pub struct PixelUnpackAlignmentGuard {
    /// Предыдущее выравнивание данных пикселя
    previous: i32,
    /// Следующее выравнивание данных пикселя
    next: i32,
}
impl PixelUnpackAlignmentGuard {
    pub fn new(next: i32) -> Self {
        // Получаем текущее выравнивание
        let mut previous = 0;
        unsafe {
            gl::GetIntegerv(gl::UNPACK_ALIGNMENT, &mut previous);
        };

        // Задаем новое выравнивание, если оно отличается от текущего
        if next != previous {
            unsafe {
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, next);
            }
        }

        Self { previous, next }
    }
}
impl Drop for PixelUnpackAlignmentGuard {
    fn drop(&mut self) {
        // Возвращаем старое выравнивание, если оно отличалось
        if self.previous != self.next {
            unsafe {
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, self.previous);
            }
        }
    }
}

/// Защитник для привязки текстуры
pub struct TextureBindingGuard {
    /// Назначение текстуры
    target: TextureTarget,
    /// Предыдущий идентификатор привязанной текстуры (может быть 0)
    previous_id: u32,
    /// Следующий идентификатор привязанной текстуры
    next_id: u32,
}

impl TextureBindingGuard {
    pub fn new(target: TextureTarget, next_id: NonZeroU32) -> Self {
        let next_id_raw = next_id.get();
        let mut previous_id: i32 = 0;

        unsafe {
            gl::GetIntegerv(target.to_binding(), &mut previous_id);
        };
        let previous_id_raw = previous_id as u32;

        // Привязываем новую текстуру, только если она отличается от текущей
        if previous_id_raw != next_id_raw {
            unsafe {
                gl::BindTexture(target as u32, next_id_raw);
            }
        }

        Self {
            target,
            previous_id: previous_id_raw,
            next_id: next_id_raw,
        }
    }
}

impl Drop for TextureBindingGuard {
    fn drop(&mut self) {
        // Возвращаем предыдущее состояние (даже если это был 0)
        if self.previous_id != self.next_id {
            unsafe {
                gl::BindTexture(self.target as u32, self.previous_id);
            }
        }
    }
}
