//! Расширения OpenGL

use gl::{EXTENSIONS, GetIntegerv, GetStringi, NUM_EXTENSIONS};
use std::ffi::CStr;
use std::str::FromStr;
use strum::EnumString;

#[derive(Debug, EnumString, Clone, PartialEq, Eq)]
pub enum Extension {
    /// Расширение "KHR_debug"
    #[strum(serialize = "GL_KHR_debug")]
    KhrDebug,
    /// Неизвестное расширение
    #[strum(default)]
    Unknown(String),
}

/// Получить список всех доступных расширений OpenGL
pub fn get_extensions() -> Vec<Extension> {
    let mut extensions = Vec::new();

    unsafe {
        let mut num_extensions = 0;
        GetIntegerv(NUM_EXTENSIONS, &mut num_extensions);
        for i in 0..num_extensions {
            // GetStringi возвращает сырой указатель *const GLubyte (C-строку)
            let name_ptr = GetStringi(EXTENSIONS, i as u32); //
            if name_ptr.is_null() {
                continue;
            }

            // Безопасно преобразуем указатель в Rust-строку &str
            if let Ok(c_str) = CStr::from_ptr(name_ptr as *const std::os::raw::c_char).to_str() {
                // 3. Преобразуем строку в наше перечисление Extension
                let Ok(ext) = Extension::from_str(c_str);
                extensions.push(ext);
            }
        }
    }

    extensions
}
