//! Подсистема логгирования для OpenGL

use super::capability::{Capability, CapabilityManager};
pub use super::extensions::{Extension, get_extensions};
use gl;
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::{
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    sync::Arc,
};
use strum::FromRepr;

/// Тип сообщения логгера OpenGL
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum DebugLogMessageType {
    DebugTypeError = gl::DEBUG_TYPE_ERROR,
    DebugTypeDeprecatedBehavior = gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR,
    DebugTypeUndefinedBehavior = gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR,
    DebugTypePortability = gl::DEBUG_TYPE_PORTABILITY,
    DebugTypePerformance = gl::DEBUG_TYPE_PERFORMANCE,
    DebugTypeMarker = gl::DEBUG_TYPE_MARKER,
    DebugTypePushGroup = gl::DEBUG_TYPE_PUSH_GROUP,
    DebugTypePopGroup = gl::DEBUG_TYPE_POP_GROUP,
    DebugTypeOther = gl::DEBUG_TYPE_OTHER,
}

impl PartialEq<u32> for DebugLogMessageType {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for DebugLogMessageType {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"DebugLogMessageType\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Источник сообщения логгера OpenGL
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum DebugLogSource {
    DebugSourceApi = gl::DEBUG_SOURCE_API,
    DebugSourceWindowSystem = gl::DEBUG_SOURCE_WINDOW_SYSTEM,
    DebugSourceShaderCompiler = gl::DEBUG_SOURCE_SHADER_COMPILER,
    DebugSourceThirdParty = gl::DEBUG_SOURCE_THIRD_PARTY,
    DebugSourceApplication = gl::DEBUG_SOURCE_APPLICATION,
    DebugSourceOther = gl::DEBUG_SOURCE_OTHER,
}

impl PartialEq<u32> for DebugLogSource {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for DebugLogSource {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"DebugLogSource\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Уровень сообщения логгера OpenGL
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum DebugLogSeverity {
    DebugSeverityHigh = gl::DEBUG_SEVERITY_HIGH,
    DebugSeverityMedium = gl::DEBUG_SEVERITY_MEDIUM,
    DebugSeverityLow = gl::DEBUG_SEVERITY_LOW,
    DebugSeverityNotification = gl::DEBUG_SEVERITY_NOTIFICATION,
}

impl PartialEq<u32> for DebugLogSeverity {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for DebugLogSeverity {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"DebugLogSeverity\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Коллбэк для сообщений логгера OpenGL.
type LogCallback = dyn Fn(DebugLogSource, DebugLogMessageType, u32, DebugLogSeverity, &str)
    + Send
    + Sync
    + 'static;

/// Владение зарегистрированным OpenGL callback'ом.
///
/// Экземпляр следует держать, пока нужен логгер, и уничтожать при текущем
/// контексте OpenGL, в котором он был создан. Пока guard жив, не устанавливайте
/// другой `DebugMessageCallback` в этом контексте.
pub struct DebugLogger {
    _callback_data: Box<Arc<LogCallback>>,
    capability_manager: CapabilityManager,
    previous_debug_output: bool,
    previous_synchronous_output: bool,
    // OpenGL-контекст привязан к потоку; guard не должен перемещаться между потоками.
    _thread_affinity: PhantomData<Rc<()>>,
}

/// Нативный callback OpenGL. Паника не должна пересекать границу FFI.
extern "system" fn opengl_debug_log(
    source: u32,
    message_type: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    user_param: *mut std::ffi::c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if user_param.is_null() || length < 0 {
            return;
        }

        // SAFETY: the pointer refers to the boxed Arc kept alive by DebugLogger;
        // synchronous output and the guard's thread affinity protect it until detach.
        let Some(raw_message) = (unsafe { user_param.cast::<Arc<LogCallback>>().as_ref() }) else {
            return;
        };
        // Clone immediately so dropping the logger from user code cannot invalidate
        // the callback currently being invoked.
        let callback = Arc::clone(raw_message);

        let length = length as usize;
        let bytes = if length == 0 {
            &[]
        } else {
            if message.is_null() {
                return;
            }
            // SAFETY: OpenGL supplies `length` readable bytes for a non-null message.
            unsafe { std::slice::from_raw_parts(message.cast::<u8>(), length) }
        };

        let (Ok(source), Ok(message_type), Ok(severity)) = (
            DebugLogSource::try_from(source),
            DebugLogMessageType::try_from(message_type),
            DebugLogSeverity::try_from(severity),
        ) else {
            return;
        };
        let message = String::from_utf8_lossy(bytes);
        callback(source, message_type, id, severity, &message);
    }));
}

impl Drop for DebugLogger {
    fn drop(&mut self) {
        unsafe {
            // Synchronous output is enabled while installed, so no callback can
            // still be running on another thread after this unregisters it.
            gl::DebugMessageCallback(None, std::ptr::null_mut());
        }

        if self.previous_debug_output {
            self.capability_manager.enable(Capability::DebugOutput);
        } else {
            self.capability_manager.disable(Capability::DebugOutput);
        }
        if self.previous_synchronous_output {
            self.capability_manager
                .enable(Capability::DebugOutputSynchronous);
        } else {
            self.capability_manager
                .disable(Capability::DebugOutputSynchronous);
        }
    }
}

/// Установить callback отладочных сообщений OpenGL.
///
/// Возвращённый guard владеет callback'ом: при его уничтожении регистрация
/// снимается и восстанавливаются прежние состояния `DEBUG_OUTPUT` и
/// `DEBUG_OUTPUT_SYNCHRONOUS`. У OpenGL нет API для чтения ранее установленного
/// callback, поэтому этот вызов заменяет его и требует исключительного владения
/// callback-слотом. Для безопасного владения данными callback принудительно
/// переводится в синхронный режим.
pub fn attach_log<F>(callback: F) -> SimpleResult<DebugLogger>
where
    F: Fn(DebugLogSource, DebugLogMessageType, u32, DebugLogSeverity, &str) + Send + Sync + 'static,
{
    if !supports_debug_output() {
        return Err(simple_error!("OpenGL debug output is not supported"));
    }

    // Нужна актуальная информация по флагам, берем с GPU: большой оверхед, но логгер инициируется обычно один раз
    let mut capability_manager = CapabilityManager::new(true);

    let previous_debug_output = capability_manager.is_capability_enabled(Capability::DebugOutput);
    let previous_synchronous_output =
        capability_manager.is_capability_enabled(Capability::DebugOutputSynchronous);

    // A synchronous callback runs on the thread issuing the GL command, which
    // makes the user-data lifetime manageable by this guard.
    capability_manager.enable(Capability::DebugOutputSynchronous);

    let callback_data: Box<Arc<LogCallback>> = Box::new(Arc::new(callback));
    let user_param = callback_data.as_ref() as *const Arc<LogCallback> as *mut std::ffi::c_void;

    unsafe {
        gl::DebugMessageCallback(Some(opengl_debug_log), user_param);
    }
    capability_manager.enable(Capability::DebugOutput);

    Ok(DebugLogger {
        _callback_data: callback_data,
        previous_debug_output,
        capability_manager,
        previous_synchronous_output,
        _thread_affinity: PhantomData,
    })
}

/// Проверить, поддерживаются ли отладочные сообщения OpenGL.
///
/// Требует загруженных функций OpenGL и текущего контекста версии 3.3 или выше.
pub fn supports_debug_output() -> bool {
    let (mut major, mut minor) = (0, 0);
    unsafe {
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
    }

    let api_supported = (major, minor) >= (4, 3) || get_extensions().contains(&Extension::KhrDebug);
    api_supported && gl::DebugMessageCallback::is_loaded()
}
