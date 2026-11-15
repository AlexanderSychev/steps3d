//! "Обычные" текстуры

use super::super::errors::check_errors;
use super::super::limits::Limits;
use super::guards::{PixelUnpackAlignmentGuard, TextureBindingGuard};
use super::target::TextureTarget;
use crate::graphics::pictures::{MipMappedTexture, Picture};
use simple_error::{SimpleResult, simple_error};
use std::num::NonZeroU32;

/// "Обычная" текстура: реализуется простой алгоритм загрузки в OpenGL
#[derive(Debug)]
pub struct RegularTexture {
    /// Ширина текстуры в пикселях
    width: usize,
    /// Высота текстуры в пикселях
    height: usize,
    /// Назначение текстуры
    target: TextureTarget,
    /// Идентификатор текстуры в OpenGL
    opengl_id: NonZeroU32,
}

impl Drop for RegularTexture {
    // Деструктор: автоматически удаляем текстуру из контекста OpenGL при удалении объекта текстуры
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.opengl_id.get());
        }
    }
}
impl RegularTexture {
    /// Попробовать создать и загрузить в GPU текстуру из полученного изображения
    pub fn try_from_picture(limits: &Limits, pic: &Picture) -> SimpleResult<RegularTexture> {
        // Преобразование и валидация размеров текстуры
        let width = pic.get_width();
        let height = pic.get_height();
        let gl_width = i32::try_from(width).map_err(|err| simple_error!(err.to_string()))?;
        if gl_width > limits.get_max_texture_size() {
            return Err(simple_error!(
                "Texture width is too large: {}px. Max allowed texture width is {}px",
                gl_width,
                limits.get_max_texture_size()
            ));
        }
        let gl_height = i32::try_from(height).map_err(|err| simple_error!(err.to_string()))?;
        if gl_height > limits.get_max_texture_size() {
            return Err(simple_error!(
                "Texture height is too large: {}px. Max allowed texture height is {}px",
                gl_height,
                limits.get_max_texture_size()
            ));
        }

        // Форматы текстуры
        let gl_gpu_format =
            i32::try_from(gl::SRGB8_ALPHA8).map_err(|err| simple_error!(err.to_string()))?;
        let gl_app_format = gl::RGBA;

        // Назначение текстуры
        let target = TextureTarget::Texture2d;

        // Конвертируем пиксели в "сырой" набор байт
        let pixels: &[u8] = bytemuck::try_cast_slice(pic.get_pixels())
            .map_err(|err| simple_error!(err.to_string()))?;

        // Включаем выравнивание данных пикселя (на время выполнения метода)
        let _unpack_alignment_guard = PixelUnpackAlignmentGuard::new(4);

        // Создаем дескриптор текстуры в OpenGL
        let mut opengl_id = 0;
        unsafe {
            gl::GenTextures(1, &mut opengl_id);
        }
        let opengl_id = NonZeroU32::new(opengl_id)
            .ok_or_else(|| simple_error!("Failed to create texture descriptor"))?;

        // Создаем объект текстуры
        let result = RegularTexture {
            width,
            height,
            opengl_id,
            target,
        };

        // Делаем текстуру текущей (на время выполнения метода)
        let _binding_guard = TextureBindingGuard::new(target, opengl_id);

        // Загружаем пиксели в текстуру
        unsafe {
            gl::TexImage2D(
                target as u32,
                0,
                gl_gpu_format,
                gl_width,
                gl_height,
                0,
                gl_app_format,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
        }

        // Проверяем на ошибки OpenGL: если при загрузке произошли ошибки,
        // то дескриптор текстуры должен автоматически удалиться
        check_errors()?;

        Ok(result)
    }

    pub fn try_from_mip_mapped(
        limits: &Limits,
        mipmapped: &MipMappedTexture,
    ) -> SimpleResult<RegularTexture> {
        // Преобразование и валидация размеров текстуры
        let width = mipmapped.get_width();
        let height = mipmapped.get_height();
        let gl_width = i32::try_from(width).map_err(|err| simple_error!(err.to_string()))?;
        if gl_width > limits.get_max_texture_size() {
            return Err(simple_error!(
                "Texture width is too large: {}px. Max allowed texture width is {}px",
                gl_width,
                limits.get_max_texture_size()
            ));
        }
        let gl_height = i32::try_from(height).map_err(|err| simple_error!(err.to_string()))?;
        if gl_height > limits.get_max_texture_size() {
            return Err(simple_error!(
                "Texture height is too large: {}px. Max allowed texture height is {}px",
                gl_height,
                limits.get_max_texture_size()
            ));
        }

        // Форматы текстуры
        let gl_gpu_format =
            i32::try_from(gl::SRGB8_ALPHA8).map_err(|err| simple_error!(err.to_string()))?;
        let gl_app_format = gl::RGBA;

        // Назначение текстуры
        let target = TextureTarget::Texture2d;

        // Конвертируем пиксели каждого уровня в "сырые" наборы байт
        let mut raw_levels: Vec<&[u8]> = Vec::with_capacity(mipmapped.get_mip_levels_count());
        for i in 0..mipmapped.get_mip_levels_count() {
            let pixels = mipmapped.get_pixels(i)?;
            let level: &[u8] =
                bytemuck::try_cast_slice(pixels).map_err(|err| simple_error!(err.to_string()))?;
            raw_levels.push(level);
        }

        // Включаем выравнивание данных пикселя (на время выполнения метода)
        let _unpack_alignment_guard = PixelUnpackAlignmentGuard::new(4);

        // Создаем дескриптор текстуры в OpenGL
        let mut opengl_id = 0;
        unsafe {
            gl::GenTextures(1, &mut opengl_id);
        }
        let opengl_id = NonZeroU32::new(opengl_id)
            .ok_or_else(|| simple_error!("Failed to create texture descriptor"))?;

        // Создаем объект текстуры
        let result = RegularTexture {
            width,
            height,
            opengl_id,
            target,
        };

        // Делаем текстуру текущей (на время выполнения метода)
        let _binding_guard = TextureBindingGuard::new(target, opengl_id);

        // Загружаем пиксели в текстуру
        let (mut level_width, mut level_height) = (gl_width, gl_height);
        for (level, pixels) in raw_levels.iter().enumerate() {
            unsafe {
                gl::TexImage2D(
                    target as u32,
                    level as i32,
                    gl_gpu_format,
                    level_width,
                    level_height,
                    0,
                    gl_app_format,
                    gl::UNSIGNED_BYTE,
                    pixels.as_ptr().cast(),
                );
            }

            level_width = (level_width / 2).max(1);
            level_height = (level_height / 2).max(1);
        }

        // Проверяем на ошибки OpenGL: если при загрузке произошли ошибки,
        // то дескриптор текстуры должен автоматически удалиться
        check_errors()?;

        Ok(result)
    }

    /// Сделать текстуру текущей
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(self.target as u32, self.opengl_id.get());
        }
    }

    /// Получить ширину текстуры
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Получить высоту текстуры
    pub fn get_height(&self) -> usize {
        self.height
    }
}
