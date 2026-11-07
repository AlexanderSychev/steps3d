//! Работа с глобальными буферами OpenGL

use gl::{
    COLOR_BUFFER_BIT, Clear, ClearColor, ClearDepth, ClearStencil, ColorMask, DEPTH_BUFFER_BIT,
    DepthMask, STENCIL_BUFFER_BIT, StencilMask,
};
use glam::BVec4;
use palette::LinSrgba;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferValues<M, C> {
    /// Маска
    pub mask_value: M,
    /// Очищающее значение
    pub clear_value: C,
}

impl<M, C> BufferValues<M, C> {
    /// Заменить маску в описании буфера; состояние OpenGL не меняется до её применения.
    pub fn set_mask_value(&mut self, mask_value: M) {
        self.mask_value = mask_value;
    }

    /// Заменить очищающее значение в описании буфера; состояние OpenGL не меняется до его применения.
    pub fn set_clear_value(&mut self, clear_value: C) {
        self.clear_value = clear_value;
    }
}

/// Типы буферов
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Buffer {
    /// Буфер цветов
    Color(BufferValues<BVec4, LinSrgba>),
    /// Буфер глубины
    Depth(BufferValues<bool, f64>),
    /// Буфер трафарета
    Stencil(BufferValues<u32, i32>),
}

impl Buffer {
    /// Получить изменяемые настройки цветового буфера, если это `Color`.
    pub fn color_values_mut(&mut self) -> Option<&mut BufferValues<BVec4, LinSrgba>> {
        match self {
            Self::Color(values) => Some(values),
            _ => None,
        }
    }

    /// Получить изменяемые настройки буфера глубины, если это `Depth`.
    pub fn depth_values_mut(&mut self) -> Option<&mut BufferValues<bool, f64>> {
        match self {
            Self::Depth(values) => Some(values),
            _ => None,
        }
    }

    /// Получить изменяемые настройки буфера трафарета, если это `Stencil`.
    pub fn stencil_values_mut(&mut self) -> Option<&mut BufferValues<u32, i32>> {
        match self {
            Self::Stencil(values) => Some(values),
            _ => None,
        }
    }

    /// Применить значения очистки и маски, затем очистить перечисленные буферы.
    pub fn clear_buffers(buffers: &[Buffer]) {
        let mut mask = 0;
        for buffer in buffers {
            buffer.apply_clear_value();
            buffer.apply_mask();
            mask |= buffer.get_buffer_bit();
        }
        unsafe {
            Clear(mask);
        }
    }

    /// Создать буфер цвета со значениями по-умолчанию
    pub fn default_color() -> Self {
        Self::Color(BufferValues {
            // В OpenGL маска цвета изначально включена для всех каналов (GL_TRUE)
            mask_value: BVec4::new(true, true, true, true),
            // Цвет очистки по умолчанию — полностью черный и прозрачный (0.0, 0.0, 0.0, 0.0)
            clear_value: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
        })
    }

    /// Создать буфер глубины со значениями по умолчанию
    pub fn default_depth() -> Self {
        Buffer::Depth(BufferValues {
            // Запись в буфер глубины изначально включена
            mask_value: true,
            // Изначально буфер очищается в самую дальнюю плоскость (1.0)
            clear_value: 1.0,
        })
    }

    /// Создать буфер трафарета со значениями по умолчанию
    pub fn default_stencil() -> Self {
        Buffer::Stencil(BufferValues {
            // Маска трафарета по умолчанию — все единицы (0xFFFFFFFF)
            mask_value: u32::MAX,
            // Буфер трафарета изначально очищается в 0
            clear_value: 0,
        })
    }

    /// Задать значение, которым будет очищен буфер.
    pub fn apply_clear_value(&self) {
        match self {
            Buffer::Color(BufferValues { clear_value, .. }) => unsafe {
                ClearColor(
                    clear_value.red,
                    clear_value.green,
                    clear_value.blue,
                    clear_value.alpha,
                );
            },
            Buffer::Depth(BufferValues { clear_value, .. }) => unsafe {
                ClearDepth(*clear_value);
            },
            Buffer::Stencil(BufferValues { clear_value, .. }) => unsafe {
                ClearStencil(*clear_value);
            },
        }
    }

    /// Задать маску записи соответствующего буфера.
    pub fn apply_mask(&self) {
        match self {
            Buffer::Color(BufferValues { mask_value, .. }) => unsafe {
                ColorMask(
                    mask_value.x as u8,
                    mask_value.y as u8,
                    mask_value.z as u8,
                    mask_value.w as u8,
                );
            },
            Buffer::Depth(BufferValues { mask_value, .. }) => unsafe {
                DepthMask(*mask_value as u8);
            },
            Buffer::Stencil(BufferValues { mask_value, .. }) => unsafe {
                StencilMask(*mask_value);
            },
        }
    }

    /// Преобразовать буфер в значение для битовой маски очистки.
    fn get_buffer_bit(&self) -> u32 {
        match self {
            Buffer::Color(_) => COLOR_BUFFER_BIT,
            Buffer::Depth(_) => DEPTH_BUFFER_BIT,
            Buffer::Stencil(_) => STENCIL_BUFFER_BIT,
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::default_color()
    }
}

#[cfg(test)]
mod tests {
    use super::{BVec4, Buffer, LinSrgba};

    #[test]
    fn buffer_values_can_be_changed_through_typed_accessor() {
        let mut buffer = Buffer::default_color();
        let values = buffer.color_values_mut().unwrap();
        values.set_mask_value(BVec4::new(true, false, true, false));
        values.set_clear_value(LinSrgba::new(0.1, 0.2, 0.3, 0.4));

        let values = buffer.color_values_mut().unwrap();
        assert_eq!(values.mask_value, BVec4::new(true, false, true, false));
        assert_eq!(values.clear_value, LinSrgba::new(0.1, 0.2, 0.3, 0.4));
        assert!(buffer.depth_values_mut().is_none());
    }
}
