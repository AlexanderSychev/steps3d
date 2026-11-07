//! SDL-специфичная обвязка для окна с OpenGL-контекстом.
//!
//! Этот модуль отвечает только за создание SDL-окна/контекста и платформенные
//! операции. Загрузчик OpenGL и GL API находятся в модуле `opengl`.

use sdl2::{
    EventPump, Sdl, VideoSubsystem, init,
    video::{GLContext, GLProfile, Window},
};
use simple_error::{SimpleResult, simple_error};
use std::ffi::c_void;

/// SDL-окно, связанное с OpenGL-контекстом.
pub struct SdlGlWindow {
    _gl_context: GLContext,
    window: Window,
    video: VideoSubsystem,
    _sdl: Sdl,
}

impl SdlGlWindow {
    /// Создать SDL-окно и контекст OpenGL Core Profile 3.3.
    pub fn try_new(title: &str, width: u32, height: u32) -> SimpleResult<Self> {
        let sdl = init().map_err(|message| simple_error!(message))?;
        let video = sdl.video().map_err(|message| simple_error!(message))?;

        let gl_attributes = video.gl_attr();
        gl_attributes.set_context_profile(GLProfile::Core);
        gl_attributes.set_context_version(3, 3);

        let window = video
            .window(title, width, height)
            .opengl()
            .resizable()
            .position_centered()
            .build()
            .map_err(|error| simple_error!(error.to_string()))?;
        let gl_context = window
            .gl_create_context()
            .map_err(|message| simple_error!(message))?;

        Ok(Self {
            _sdl: sdl,
            video,
            window,
            _gl_context: gl_context,
        })
    }

    /// Получить адрес OpenGL-функции через SDL.
    ///
    /// Передавайте этот метод загрузчику из `opengl`, например:
    /// `opengl::load_with(|name| window.gl_get_proc_address(name))`.
    pub fn gl_get_proc_address(&self, name: &str) -> *const c_void {
        self.video.gl_get_proc_address(name) as *const c_void
    }

    /// Проверить, является ли контекст окна текущим для данного потока.
    pub fn is_context_current(&self) -> bool {
        self._gl_context.is_current()
    }

    /// Поменять местами передний и задний буферы окна.
    pub fn swap_buffers(&self) {
        self.window.gl_swap_window();
    }

    /// Получить обработчик событий SDL.
    pub fn event_pump(&self) -> SimpleResult<EventPump> {
        self._sdl
            .event_pump()
            .map_err(|message| simple_error!(message))
    }
}
