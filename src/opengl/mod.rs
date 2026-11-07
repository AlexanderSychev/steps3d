//! Различная "обвязка" для OpenGL

mod buffer;
mod capability;
pub mod configuration;
pub mod errors;
mod extensions;
pub mod log;

pub use buffer::Buffer;
pub use capability::Capability;

/// Загрузить адреса функций OpenGL от выбранного оконного backend'а.
///
/// Сам модуль OpenGL не зависит от SDL: backend передаёт сюда функцию поиска
/// адресов, предоставленную созданным и текущим контекстом.
pub fn load_with<F>(loader: F)
where
    F: FnMut(&'static str) -> *const std::ffi::c_void,
{
    gl::load_with(loader);
}
