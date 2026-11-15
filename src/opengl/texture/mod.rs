//! Текстурирование

mod guards;
mod pixel_type;
mod regular;
mod target;
mod texture_format;

pub use pixel_type::*;
pub use regular::*;
pub use target::*;
pub use texture_format::*;

use gl::{BindTexture, DeleteTextures, GenTextures};
use simple_error::{SimpleResult, simple_error};
use std::num::NonZeroU32;

/// Дескриптор текстуры в OpenGL
#[derive(Debug, PartialEq)]
pub struct Texture {
    /// Идентификатор текстуры в OpenGL
    opengl_id: NonZeroU32,
    /// Назначение текстуры
    target: TextureTarget,
}

impl Texture {
    /// Попробовать создать новый дескриптор текстуры
    pub fn try_new(target: TextureTarget) -> SimpleResult<Texture> {
        let mut opengl_id = 0;
        unsafe {
            GenTextures(1, &mut opengl_id);
        }
        match NonZeroU32::new(opengl_id) {
            Some(opengl_id) => Ok(Texture { opengl_id, target }),
            None => Err(simple_error!("Failed to create texture descriptor")),
        }
    }

    /// Сделать текстурный дескриптор текущим
    pub fn bind(&self) {
        unsafe { BindTexture(self.target as u32, self.opengl_id.get()) }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            DeleteTextures(1, &self.opengl_id.get());
        }
    }
}
