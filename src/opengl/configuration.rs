//! Утилиты для базовой настройки OpenGL.
//!
//! Функции, вызывающие OpenGL API, требуют загруженных функций и текущего контекста.

use gl::{BACK, CCW, CW, CullFace, FRONT, FRONT_AND_BACK, FrontFace, Viewport};
use simple_error::{SimpleError, SimpleResult, simple_error};

/// Задать размеры области просмотра OpenGL.
pub fn set_viewport_size(width: u32, height: u32) -> SimpleResult<()> {
    // Нулевые размеры не допускаются.
    if width == 0 || height == 0 {
        return Err(simple_error!(
            "Invalid screen size: {}, {}",
            &width,
            &height
        ));
    }

    // Безопасное приведение к 32-битным знаковым целым.
    let width_signed = i32::try_from(width).map_err(|err| simple_error!(err.to_string()))?;
    let height_signed = i32::try_from(height).map_err(|err| simple_error!(err.to_string()))?;

    // Установка области просмотра OpenGL.
    unsafe {
        Viewport(0, 0, width_signed, height_signed);
    }

    Ok(())
}

/// Направление лицевых граней
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u32)]
pub enum FrontFaceDirection {
    /// По часовой стрелке
    Clockwise = CW,
    /// Против часовой стрелки
    CounterClockwise = CCW,
}

impl PartialEq<u32> for FrontFaceDirection {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for FrontFaceDirection {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        match value {
            CW => Ok(Self::Clockwise),
            CCW => Ok(Self::CounterClockwise),
            _ => Err(simple_error!(
                "Cannot create \"FrontFace\" value from \"u32\" value: {}",
                value
            )),
        }
    }
}

impl FrontFaceDirection {
    /// Задать направление обхода вершин лицевой грани.
    pub fn set(self) {
        unsafe {
            FrontFace(self as u32);
        }
    }
}

/// Режимы отсечения граней
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u32)]
pub enum CullFaceMode {
    /// Отсекать лицевые грани
    Front = FRONT,
    /// Отсекать нелицевые грани
    Back = BACK,
    /// Отсекать все грани
    FrontAndBack = FRONT_AND_BACK,
}

impl TryFrom<u32> for CullFaceMode {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        match value {
            FRONT => Ok(Self::Front),
            BACK => Ok(Self::Back),
            FRONT_AND_BACK => Ok(Self::FrontAndBack),
            _ => Err(simple_error!(
                "Cannot create \"CullFaceMode\" value from \"u32\" value: {}",
                value
            )),
        }
    }
}

impl PartialEq<u32> for CullFaceMode {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl CullFaceMode {
    /// Задать режим отсечения граней. Само отсечение включается через
    /// `Capability::CullFace`.
    pub fn set(self) {
        unsafe {
            CullFace(self as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::set_viewport_size;

    #[test]
    fn viewport_rejects_zero_dimensions() {
        assert!(set_viewport_size(0, 1).is_err());
        assert!(set_viewport_size(1, 0).is_err());
    }

    #[test]
    fn viewport_rejects_dimensions_that_do_not_fit_glsizei() {
        assert!(set_viewport_size(u32::MAX, 1).is_err());
        assert!(set_viewport_size(1, u32::MAX).is_err());
    }
}
