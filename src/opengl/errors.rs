//! Обработка ошибок OpenGL

use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ErrorType {
    InvalidEnum = gl::INVALID_ENUM,
    InvalidValue = gl::INVALID_VALUE,
    InvalidOperation = gl::INVALID_OPERATION,
    InvalidFramebufferOperation = gl::INVALID_FRAMEBUFFER_OPERATION,
    OutOfMemory = gl::OUT_OF_MEMORY,
    StackUnderflow = gl::STACK_UNDERFLOW,
    StackOverflow = gl::STACK_OVERFLOW,
}

impl PartialEq<u32> for ErrorType {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for ErrorType {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| simple_error!(
            "Cannot create \"ErrorType\" value from \"u32\" value: {}",
            value
        ))
    }
}

/// Забрать все накопленные ошибки OpenGL и объединить их в одно сообщение.
///
/// Каждый вызов `glGetError` очищает одну ошибку из очереди текущего контекста.
/// Поэтому эта функция потребляет ошибки, накопленные с предыдущей проверки.
/// Требует загруженных функций OpenGL и текущего контекста.
pub fn get_error() -> Option<SimpleError> {
    let mut codes = Vec::new();
    loop {
        let code = unsafe { gl::GetError() };
        if code == gl::NO_ERROR {
            break;
        }

        let description = match ErrorType::try_from(code) {
            Ok(error) => format!("{error:?}"),
            Err(_) => format!("Unknown(0x{code:04X})"),
        };
        codes.push(description);
    }

    if codes.is_empty() {
        None
    } else {
        Some(simple_error!(
            "OpenGL error(s) occurred: {}",
            codes.join(", ")
        ))
    }
}

/// Проверить ошибки, накопленные со времени предыдущей проверки.
///
/// При наличии хотя бы одной ошибки возвращает её как `Err`.
pub fn check_errors() -> SimpleResult<()> {
    match get_error() {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
