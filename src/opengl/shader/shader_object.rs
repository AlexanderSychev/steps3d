//! Базовые типы для шейдеров

use super::super::typings::impl_gl_enum_traits;
use simple_error::{SimpleResult, simple_error};
use std::fs::read_to_string;
use std::num::NonZeroU32;
use std::path::Path;
use strum::FromRepr;

/// Тип шейдера
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ShaderType {
    /// Вычислительный шейдер
    ComputeShader = gl::COMPUTE_SHADER,
    /// Вершинный шейдер
    VertexShader = gl::VERTEX_SHADER,
    /// Шейдер управления тесселяцией
    TessControlShader = gl::TESS_CONTROL_SHADER,
    /// Шейдер оценки тесселяции
    TessEvaluationShader = gl::TESS_EVALUATION_SHADER,
    /// Геометрический шейдер
    GeometryShader = gl::GEOMETRY_SHADER,
    /// Фрагментный шейдер
    FragmentShader = gl::FRAGMENT_SHADER,
}
impl_gl_enum_traits!(ShaderType);

/// Шейдер
#[derive(Debug, Eq, PartialEq)]
pub struct Shader {
    /// Тип шейдера
    shader_type: ShaderType,
    /// Идентификатор шейдера OpenGL
    opengl_id: NonZeroU32,
}
impl Shader {
    /// Создать шейдер из текстового файла
    pub fn try_new_from_file<P: AsRef<Path>>(
        shader_type: ShaderType,
        filepath: P,
    ) -> SimpleResult<Self> {
        let source = read_to_string(filepath).map_err(|err| simple_error!(err.to_string()))?;
        Self::try_new_from_source(shader_type, &source)
    }

    /// Создать шейдер из исходного кода
    pub fn try_new_from_source(shader_type: ShaderType, source: &str) -> SimpleResult<Self> {
        let source_len = i32::try_from(source.len())
            .map_err(|err| simple_error!("Source code length overflow: {err}"))?;

        // Создаем дескриптор шейдера
        let opengl_id = unsafe { gl::CreateShader(shader_type as u32) };
        let opengl_id =
            NonZeroU32::new(opengl_id).ok_or_else(|| simple_error!("Failed to create shader"))?;

        // Создаем объект (автоматически удалит дескриптор шейдера при ошибке)
        let shader = Self {
            opengl_id,
            shader_type,
        };

        // Загружаем исходный код
        let source_ptr = source.as_ptr().cast::<gl::types::GLchar>();
        unsafe {
            gl::ShaderSource(shader.opengl_id.get(), 1, &source_ptr, &source_len);
        }

        // Компилируем шейдер
        let mut compile_status = 0;
        unsafe {
            gl::CompileShader(shader.opengl_id.get());
            gl::GetShaderiv(
                shader.opengl_id.get(),
                gl::COMPILE_STATUS,
                &mut compile_status,
            );
        }

        // Если компиляция не прошла успешно, то получаем ошибку
        if compile_status == 0 {
            let mut log_length = 0;
            unsafe {
                gl::GetShaderiv(shader.opengl_id.get(), gl::INFO_LOG_LENGTH, &mut log_length);
            }
            let log_capacity = usize::try_from(log_length)
                .map_err(|_| simple_error!("Invalid shader info log length: {log_length}"))?;
            if log_capacity == 0 {
                return Err(simple_error!("Failed to compile shader for unknown reason"));
            }

            let mut log = vec![0u8; log_capacity];
            let mut written_length = 0;
            unsafe {
                gl::GetShaderInfoLog(
                    shader.opengl_id.get(),
                    log_length,
                    &mut written_length,
                    log.as_mut_ptr().cast::<gl::types::GLchar>(),
                );
            }
            let written_length = usize::try_from(written_length)
                .map_err(|_| simple_error!("Invalid written shader info log length"))?
                .min(log.len());
            let log = String::from_utf8_lossy(&log[..written_length]).into_owned();

            return Err(simple_error!(log));
        }

        Ok(shader)
    }

    /// Получить идентификатор шейдера в OpenGL
    pub fn get_opengl_id(&self) -> NonZeroU32 {
        self.opengl_id
    }

    /// Получить тип шейдера
    pub fn get_shader_type(&self) -> ShaderType {
        self.shader_type
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.opengl_id.get());
        }
    }
}
