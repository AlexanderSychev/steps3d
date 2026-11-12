//! Различная "обвязка" для OpenGL

pub mod blend;
mod buffer;
pub mod buffer_object;
mod capability;
pub mod configuration;
pub mod errors;
mod extensions;
mod limits;
pub mod log;
pub mod provoking_vertex;
pub mod stencil_and_depth;
mod typings;

pub use buffer::Buffer;
pub use capability::{Capability, CapabilityManager};

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
