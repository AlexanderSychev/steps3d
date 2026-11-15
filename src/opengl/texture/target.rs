//! Назначнение текстур

use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Назначение текстуры
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureTarget {
    Texture1d = gl::TEXTURE_1D,
    Texture2d = gl::TEXTURE_2D,
    Texture3d = gl::TEXTURE_3D,
    Texture1dArray = gl::TEXTURE_1D_ARRAY,
    Texture2dArray = gl::TEXTURE_2D_ARRAY,
    TextureRectangle = gl::TEXTURE_RECTANGLE,
    TextureCubeMap = gl::TEXTURE_CUBE_MAP,
    TextureCubeMapArray = gl::TEXTURE_CUBE_MAP_ARRAY,
    TextureBuffer = gl::TEXTURE_BUFFER,
    Texture2dMultisample = gl::TEXTURE_2D_MULTISAMPLE,
    Texture2dMultisampleArray = gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
}
impl PartialEq<u32> for TextureTarget {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}
impl TryFrom<u32> for TextureTarget {
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
impl TextureTarget {
    /// Можно ли тестировать текстуру без выделения памяти
    pub fn can_test(&self) -> bool {
        ProxyTextureTarget::try_from(*self).is_ok()
    }

    /// Получить "альтернативную" версию соответствующей константы OpenGL
    /// (для проверки текущей привязанной текстуры)
    pub fn to_binding(&self) -> u32 {
        match self {
            TextureTarget::Texture1d => gl::TEXTURE_BINDING_1D,
            TextureTarget::Texture2d => gl::TEXTURE_BINDING_2D,
            TextureTarget::Texture3d => gl::TEXTURE_BINDING_3D,
            TextureTarget::Texture1dArray => gl::TEXTURE_BINDING_1D_ARRAY,
            TextureTarget::Texture2dArray => gl::TEXTURE_BINDING_2D_ARRAY,
            TextureTarget::TextureRectangle => gl::TEXTURE_BINDING_RECTANGLE,
            TextureTarget::TextureCubeMap => gl::TEXTURE_BINDING_CUBE_MAP,
            TextureTarget::TextureCubeMapArray => gl::TEXTURE_BINDING_CUBE_MAP_ARRAY,
            TextureTarget::TextureBuffer => gl::TEXTURE_BINDING_BUFFER,
            TextureTarget::Texture2dMultisample => gl::TEXTURE_BINDING_2D_MULTISAMPLE,
            TextureTarget::Texture2dMultisampleArray => gl::TEXTURE_BINDING_2D_MULTISAMPLE_ARRAY,
        }
    }
}

/// Специальные таргеты для тестирования возможностей GPU без выделения памяти
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ProxyTextureTarget {
    Texture1d = gl::PROXY_TEXTURE_1D,
    Texture2d = gl::PROXY_TEXTURE_2D,
    Texture3d = gl::PROXY_TEXTURE_3D,
    Texture1dArray = gl::PROXY_TEXTURE_1D_ARRAY,
    Texture2dArray = gl::PROXY_TEXTURE_2D_ARRAY,
    TextureRectangle = gl::PROXY_TEXTURE_RECTANGLE,
    TextureCubeMap = gl::PROXY_TEXTURE_CUBE_MAP,
}
impl PartialEq<u32> for ProxyTextureTarget {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}
impl TryFrom<u32> for ProxyTextureTarget {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"ProxyTextureTarget\" value from \"u32\" value: {}",
                value
            )
        })
    }
}
impl TryFrom<TextureTarget> for ProxyTextureTarget {
    type Error = SimpleError;

    fn try_from(value: TextureTarget) -> SimpleResult<Self> {
        match value {
            TextureTarget::Texture1d => Ok(ProxyTextureTarget::Texture1d),
            TextureTarget::Texture2d => Ok(ProxyTextureTarget::Texture2d),
            TextureTarget::Texture3d => Ok(ProxyTextureTarget::Texture3d),
            TextureTarget::Texture1dArray => Ok(ProxyTextureTarget::Texture1dArray),
            TextureTarget::Texture2dArray => Ok(ProxyTextureTarget::Texture2dArray),
            TextureTarget::TextureRectangle => Ok(ProxyTextureTarget::TextureRectangle),
            TextureTarget::TextureCubeMap => Ok(ProxyTextureTarget::TextureCubeMap),
            _ => Err(simple_error!(
                "Texture target {:?} has no corresponding proxy target",
                &value
            )),
        }
    }
}
