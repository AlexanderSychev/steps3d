//! Различная "обвязка" для OpenGL

mod capability;
mod extensions;
pub mod log;

pub use capability::Capability;
pub use extensions::{Extension, get_extensions};

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
