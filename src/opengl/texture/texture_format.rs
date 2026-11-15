//! Форматы текстур

use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Базовые ("безразмерные") форматы текстур
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BaseTextureFormat {
    Red = gl::RED,
    RedGreen = gl::RG,
    RGB = gl::RGB,
    BGR = gl::BGR,
    RGBA = gl::RGBA,
    BGRA = gl::BGRA,
    RedInteger = gl::RED_INTEGER,
    RedGreebInteger = gl::RG_INTEGER,
    RgbInteger = gl::RGB_INTEGER,
    BgrInteger = gl::BGR_INTEGER,
    RgbaInteger = gl::RGBA_INTEGER,
    BgraInteger = gl::BGRA_INTEGER,
    StencilIndex = gl::STENCIL_INDEX,
    DepthComponent = gl::DEPTH_COMPONENT,
    DepthStencil = gl::DEPTH_STENCIL,
}
impl PartialEq<u32> for BaseTextureFormat {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}
impl TryFrom<u32> for BaseTextureFormat {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"TextureTarget\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Размерные форматы текстур
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum SizedTextureFormat {
    /// Красный канал в формате целого 8-битного числа
    Red8 = gl::R8,
    /// Красный и синий каналы в формате целых 8-битных чисел
    RedGreen8 = gl::RG8,
    /// RGB-цвет в формате целых 8-битных чисел
    RGB8 = gl::RGB8,
    /// 8-битный RGB-цвет с альфа-каналом в формате целых 8-битных чисел
    RGBA8 = gl::RGBA8,
    /// Красный канал в формате 32-битного числа с плавающей точкой
    R32F = gl::R32F,
    /// RGB-цвет в формате 32-битных чисел с плавающей точкой
    RGB32F = gl::RGB32F,
    /// RGB-цвет с альфа-каналом в формате 32-битных чисел с плавающей точкой
    RGBA32F = gl::RGBA32F,
    /// Глубина
    DepthComponent24 = gl::DEPTH_COMPONENT24,
    /// Трафарет
    Depth24Stencil8 = gl::DEPTH24_STENCIL8,
}

/// Внутренний формат текстур для OpenGL
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum InternalTextureFormat {
    /// Базовый формат
    Base(BaseTextureFormat),
    /// Размерный формат
    Sized(SizedTextureFormat),
}
// Позволяет писать: InternalTextureFormat::from(BaseTextureFormat::RGBA)
impl From<BaseTextureFormat> for InternalTextureFormat {
    fn from(format: BaseTextureFormat) -> Self {
        InternalTextureFormat::Base(format)
    }
}
// Позволяет писать: InternalTextureFormat::from(SizedTextureFormat::RGBA8)
impl From<SizedTextureFormat> for InternalTextureFormat {
    fn from(format: SizedTextureFormat) -> Self {
        InternalTextureFormat::Sized(format)
    }
}
// Позволяет сравнивать напрямую с u32 (например, при проверках драйвера)
impl PartialEq<u32> for InternalTextureFormat {
    fn eq(&self, other: &u32) -> bool {
        u32::from(*self) == *other
    }
}
// Полезно при чтении формата из существующей текстуры через glGetTexLevelParameteriv
impl TryFrom<u32> for InternalTextureFormat {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        if let Some(base) = BaseTextureFormat::from_repr(value) {
            Ok(InternalTextureFormat::Base(base))
        } else if let Some(sized) = SizedTextureFormat::from_repr(value) {
            Ok(InternalTextureFormat::Sized(sized))
        } else {
            Err(simple_error!(
                "Cannot create \"InternalTextureFormat\" from \"u32\" value: {}",
                value
            ))
        }
    }
}
// Приведение к "родному" формату OpenGL
impl From<InternalTextureFormat> for u32 {
    fn from(value: InternalTextureFormat) -> Self {
        match value {
            InternalTextureFormat::Base(format) => format as u32,
            InternalTextureFormat::Sized(format) => format as u32,
        }
    }
}
