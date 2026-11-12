//! Буфер трафарета и тест глубины

use gl::{
    ALWAYS, DECR, DECR_WRAP, DepthFunc, DepthMask, EQUAL, FALSE, GEQUAL, GREATER, INCR, INCR_WRAP,
    INVERT, KEEP, LEQUAL, LESS, NEVER, NOTEQUAL, REPLACE, StencilFunc, StencilMask, StencilOp,
    TRUE, ZERO,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Операции сравнения для трафарета и теста глубины
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr, Default)]
#[repr(u32)]
pub enum CompareOperation {
    /// Тест никогда не выполнен
    Never = NEVER,
    /// Тест всегда выполнен (значение по-умолчанию)
    #[default]
    Always = ALWAYS,
    /// Тест выполнен, если эталонное значение равно значению в буфере
    Equal = EQUAL,
    /// Тест выполнен, если эталонное значение не равно значению в буфере
    NotEqual = NOTEQUAL,
    /// Тест выполнен, если эталонное значение меньше значения в буфере
    Less = LESS,
    /// Тест выполнен, если эталонное значение меньше или равно значению в буфере
    LessEqual = LEQUAL,
    /// Тест выполнен, если эталонное значение больше значения в буфере
    Greater = GREATER,
    /// Тест выполнен, если эталонное значение больше или равно значению в буфере
    GreaterEqual = GEQUAL,
}
impl PartialEq<u32> for CompareOperation {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}
impl TryFrom<u32> for CompareOperation {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"CompareOperation\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr, Default)]
#[repr(u32)]
pub enum StencilOperation {
    /// Оставляет значение в буфере трафарета без изменений (значение по-умолчанию)
    #[default]
    Keep = KEEP,
    /// Обнуляет значение в буфере трафарета
    Zero = ZERO,
    /// Заменяет значение в буфере трафарета эталонным значением
    Replace = REPLACE,
    /// Инкрементирует значение в буфере трафарета с насыщением
    Incr = INCR,
    /// Инкрементирует значение в буфере трафарета с заворачиванием
    IncrWrap = INCR_WRAP,
    /// Декрементирует значение в буфере трафарета с насыщением
    Decr = DECR,
    /// Декрементирует значение в буфере трафарета с заворачиванием
    DecrWrap = DECR_WRAP,
    /// Инвертирует биты значения в буфере трафарета
    Invert = INVERT,
}
impl PartialEq<u32> for StencilOperation {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}
impl TryFrom<u32> for StencilOperation {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"StencilOperation\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Состояние настройки буфера трафарета
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct StencilState {
    /// Функция сравнения для теста трафарета
    pub compare: CompareOperation,
    /// Эталонное значение для теста трафарета
    pub reference: i32,
    /// Маска чтения (применяется к эталону и значению в буфере при сравнении)
    pub read_mask: u8,
    /// Маска записи (определяет, какие биты можно изменять в буфере)
    pub write_mask: u8,
    /// Операция при провале теста трафарета
    pub on_stencil_fail: StencilOperation,
    /// Операция при провале теста глубины
    pub on_depth_fail: StencilOperation,
    /// Операция при успешном прохождении обоих тестов
    pub on_pass: StencilOperation,
}

impl Default for StencilState {
    /// Дефолтные значения строго по спецификации OpenGL
    fn default() -> Self {
        Self {
            compare: CompareOperation::Always,
            reference: 0,
            read_mask: 0xFF,  // В OpenGL по умолчанию все биты активны (1)
            write_mask: 0xFF, // В OpenGL по умолчанию запись разрешена во все биты
            on_stencil_fail: StencilOperation::Keep,
            on_depth_fail: StencilOperation::Keep,
            on_pass: StencilOperation::Keep,
        }
    }
}

impl StencilState {
    /// Начать настройку с дефолтных значений OpenGL
    pub const fn new() -> Self {
        Self {
            compare: CompareOperation::Always,
            reference: 0,
            read_mask: 0xFF,
            write_mask: 0xFF,
            on_stencil_fail: StencilOperation::Keep,
            on_depth_fail: StencilOperation::Keep,
            on_pass: StencilOperation::Keep,
        }
    }

    /// Настроить функцию сравнения (аналог glStencilFunc)
    pub const fn with_func(
        mut self,
        compare: CompareOperation,
        reference: i32,
        read_mask: u8,
    ) -> Self {
        self.compare = compare;
        self.reference = reference;
        self.read_mask = read_mask;
        self
    }

    /// Настроить действия при прохождении/провале тестов (аналог glStencilOp)
    pub const fn with_ops(
        mut self,
        on_stencil_fail: StencilOperation,
        on_depth_fail: StencilOperation,
        on_pass: StencilOperation,
    ) -> Self {
        self.on_stencil_fail = on_stencil_fail;
        self.on_depth_fail = on_depth_fail;
        self.on_pass = on_pass;
        self
    }

    /// Установить маску записи (аналог glStencilMask)
    pub const fn with_write_mask(mut self, write_mask: u8) -> Self {
        self.write_mask = write_mask;
        self
    }

    /// Применить данную конфигурацию к текущему контексту OpenGL
    pub fn apply(&self) {
        unsafe {
            StencilFunc(self.compare as u32, self.reference, self.read_mask as u32);
            StencilOp(
                self.on_stencil_fail as u32,
                self.on_depth_fail as u32,
                self.on_pass as u32,
            );
            StencilMask(self.write_mask as u32);
        }
    }
}

/// Состояние настройки теста глубины (Depth Test)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct DepthState {
    /// Функция сравнения глубин (аналог glDepthFunc)
    pub compare: CompareOperation,
    /// Разрешена ли запись в буфер глубины (аналог glDepthMask)
    pub write_enabled: bool,
}

impl Default for DepthState {
    /// Дефолтные значения строго по спецификации OpenGL
    fn default() -> Self {
        Self {
            // В OpenGL по умолчанию используется GL_LESS, а не GL_ALWAYS
            compare: CompareOperation::Less,
            // Запись в буфер глубины по умолчанию включена (GL_TRUE)
            write_enabled: true,
        }
    }
}

impl DepthState {
    /// Начать настройку с дефолтных значений OpenGL
    pub const fn new() -> Self {
        Self {
            compare: CompareOperation::Less,
            write_enabled: true,
        }
    }

    /// Задать функцию сравнения глубин (аналог glDepthFunc)
    pub const fn with_compare(mut self, compare: CompareOperation) -> Self {
        self.compare = compare;
        self
    }

    /// Разрешить или запретить запись в буфер глубины (аналог glDepthMask)
    pub const fn with_write(mut self, enabled: bool) -> Self {
        self.write_enabled = enabled;
        self
    }

    /// Применить настройки к текущему контексту OpenGL
    pub fn apply(&self) {
        let mask = if self.write_enabled { TRUE } else { FALSE };
        unsafe {
            DepthFunc(self.compare as u32);
            DepthMask(mask);
        }
    }
}
