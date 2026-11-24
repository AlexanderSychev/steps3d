//! Шейдерная программа

use super::shader_object::Shader;
use super::uniform::UniformData;
use simple_error::{SimpleResult, simple_error};
use std::marker::PhantomData;
use std::num::NonZeroU32;

/// Защитник для автоматической привязки и отвязки шейдерной программы
pub struct CurrentShaderProgramGuard<'a> {
    previous_program: u32,
    next_program: u32,
    _program: PhantomData<&'a ShaderProgram>,
}
impl<'a> CurrentShaderProgramGuard<'a> {
    fn new(next_program: &'a ShaderProgram) -> Self {
        let next_program_raw = next_program.opengl_id.get();
        let mut previous_program = 0;
        unsafe {
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut previous_program);
        }
        let previous_program_raw = previous_program as u32;

        if previous_program_raw != next_program_raw {
            unsafe {
                gl::UseProgram(next_program_raw);
            }
        }

        Self {
            previous_program: previous_program_raw,
            next_program: next_program_raw,
            _program: PhantomData,
        }
    }
}
impl Drop for CurrentShaderProgramGuard<'_> {
    fn drop(&mut self) {
        if self.previous_program != self.next_program {
            unsafe {
                gl::UseProgram(self.previous_program);
            }
        }
    }
}

/// Шейдерная программа
#[derive(Debug, Eq, PartialEq)]
pub struct ShaderProgram {
    /// Идентификатор программы в OpenGL
    opengl_id: NonZeroU32,
}
impl ShaderProgram {
    /// Создать шейдерную программу
    pub fn try_new(shaders: &[Shader]) -> SimpleResult<Self> {
        if shaders.is_empty() {
            return Err(simple_error!("Program must contain at least one shader"));
        }

        // Создаем дескриптор шейдерной программы
        let opengl_id = unsafe { gl::CreateProgram() };
        let opengl_id = NonZeroU32::new(opengl_id)
            .ok_or_else(|| simple_error!("Failed to create shader program"))?;

        // Создаем объект шейдерной программы (автоматически удалит ее при ошибке)
        let result = Self { opengl_id };

        // Добавляем шейдеры в программу
        for shader in shaders {
            unsafe {
                gl::AttachShader(result.opengl_id.get(), shader.get_opengl_id().get());
            }
        }

        // Линкуем программу
        let mut link_status = 0;
        unsafe {
            gl::LinkProgram(result.opengl_id.get());
            gl::GetProgramiv(result.opengl_id.get(), gl::LINK_STATUS, &mut link_status);
        }

        // Если линковка не прошла успешно, то получаем ошибку
        if link_status == 0 {
            let mut log_length = 0;
            unsafe {
                gl::GetProgramiv(result.opengl_id.get(), gl::INFO_LOG_LENGTH, &mut log_length);
            }
            let log_capacity = usize::try_from(log_length).map_err(|_| {
                simple_error!("Invalid shader program info log length: {log_length}")
            })?;
            if log_capacity == 0 {
                return Err(simple_error!(
                    "Failed to link shader program for unknown reason"
                ));
            }

            let mut log = vec![0u8; log_capacity];
            let mut written_length = 0;
            unsafe {
                gl::GetProgramInfoLog(
                    result.opengl_id.get(),
                    log_length,
                    &mut written_length,
                    log.as_mut_ptr().cast::<gl::types::GLchar>(),
                );
            }
            let written_length = usize::try_from(written_length)
                .map_err(|_| simple_error!("Invalid written shader program info log length"))?
                .min(log.len());
            let log = String::from_utf8_lossy(&log[..written_length]).into_owned();

            return Err(simple_error!(log));
        }

        // Открепляем шейдеры от программы
        for shader in shaders {
            unsafe {
                gl::DetachShader(result.opengl_id.get(), shader.get_opengl_id().get());
            }
        }

        Ok(result)
    }

    /// Привязать программу к контексту OpenGL с автоматическим восстановлением
    /// предыдущей привязанной программы при выходе из области видимости
    pub fn bind(&self) -> CurrentShaderProgramGuard<'_> {
        CurrentShaderProgramGuard::new(&self)
    }

    /// Установить значение uniform-переменной
    pub fn set_uniform<T: UniformData>(&self, name: &str, value: &T) -> SimpleResult<()> {
        let raw_name = std::ffi::CString::new(name)
            .map_err(|_| simple_error!("Uniform name contains an interior NUL byte"))?;

        let _use_program_guard = self.bind();
        let location = unsafe { gl::GetUniformLocation(self.opengl_id.get(), raw_name.as_ptr()) };
        if location >= 0 {
            value.set_uniform_value(location);
        }
        Ok(())
    }
}
impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.opengl_id.get());
        }
    }
}
