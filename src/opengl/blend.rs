//! Смешивание цветов

use gl::{
    BlendFunc, CONSTANT_ALPHA, CONSTANT_COLOR, DST_ALPHA, DST_COLOR, ONE, ONE_MINUS_CONSTANT_ALPHA,
    ONE_MINUS_CONSTANT_COLOR, ONE_MINUS_DST_ALPHA, ONE_MINUS_DST_COLOR, ONE_MINUS_SRC_ALPHA,
    ONE_MINUS_SRC_COLOR, SRC_ALPHA, SRC_ALPHA_SATURATE, SRC_COLOR, ZERO,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Источник или приёмник цвета при смешивании
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BlendFactor {
    Zero = ZERO,
    One = ONE,
    SrcColor = SRC_COLOR,
    OneMinusSrcColor = ONE_MINUS_SRC_COLOR,
    SrcAlpha = SRC_ALPHA,
    OneMinusSrcAlpha = ONE_MINUS_SRC_ALPHA,
    DstColor = DST_COLOR,
    OneMinusDstColor = ONE_MINUS_DST_COLOR,
    DstAlpha = DST_ALPHA,
    OneMinusDstAlpha = ONE_MINUS_DST_ALPHA,
    ConstantColor = CONSTANT_COLOR,
    OneMinusConstantColor = ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = CONSTANT_ALPHA,
    OneMinusConstantAlpha = ONE_MINUS_CONSTANT_ALPHA,
    SrcAlphaSaturate = SRC_ALPHA_SATURATE,
}
impl PartialEq<u32> for BlendFactor {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}
impl TryFrom<u32> for BlendFactor {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"BlendColor\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Настройки смешивания цветов
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct BlendColorsState {
    /// Исходный цвет (цвет фрагмента)
    pub source: BlendFactor,
    /// Цвет приемника (цвет буфера)
    pub destination: BlendFactor,
}
impl Default for BlendColorsState {
    fn default() -> Self {
        Self {
            source: BlendFactor::One,
            destination: BlendFactor::Zero,
        }
    }
}
impl BlendColorsState {
    /// Создать состояние со значениями OpenGL по умолчанию
    pub const fn new() -> Self {
        Self {
            source: BlendFactor::One,
            destination: BlendFactor::Zero,
        }
    }

    /// Готовый пресет для классической альфа-прозрачности
    pub const fn alpha_blending() -> Self {
        Self {
            source: BlendFactor::SrcAlpha,
            destination: BlendFactor::OneMinusSrcAlpha,
        }
    }

    /// Готовый пресет для аддитивного смешивания (эффекты огня, света, частиц)
    pub const fn additive() -> Self {
        Self {
            source: BlendFactor::SrcAlpha,
            destination: BlendFactor::One,
        }
    }

    /// Установить исходный цвет
    pub const fn with_source(mut self, source: BlendFactor) -> Self {
        self.source = source;
        self
    }

    /// Установить цвет приемника
    pub const fn with_destination(mut self, destination: BlendFactor) -> Self {
        self.destination = destination;
        self
    }

    /// Применить данную конфигурацию к текущему контексту OpenGL
    pub fn apply(&self) {
        unsafe {
            BlendFunc(self.source as u32, self.destination as u32);
        }
    }
}
