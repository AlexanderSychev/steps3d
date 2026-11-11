//! Базовые состояния OpenGL ("Capabilities")

use gl::{Disable, Enable};
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::{EnumCount, EnumIter, FromRepr, IntoEnumIterator};

/// Различные флаги состояний OpenGL, которые можно включить или выключить
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr, EnumCount, EnumIter)]
#[repr(u32)]
pub enum Capability {
    Blend = gl::BLEND,
    ClipDistance0 = gl::CLIP_DISTANCE0,
    ClipDistance1 = gl::CLIP_DISTANCE1,
    ClipDistance2 = gl::CLIP_DISTANCE2,
    ClipDistance3 = gl::CLIP_DISTANCE3,
    ClipDistance4 = gl::CLIP_DISTANCE4,
    ClipDistance5 = gl::CLIP_DISTANCE5,
    ClipDistance6 = gl::CLIP_DISTANCE6,
    ClipDistance7 = gl::CLIP_DISTANCE7,
    ColorLogicOp = gl::COLOR_LOGIC_OP,
    /// Отсечение нелицевых граней
    CullFace = gl::CULL_FACE,
    /// Отладочные логи
    DebugOutput = gl::DEBUG_OUTPUT,
    /// Отладочные логи (синхронный вывод)
    DebugOutputSynchronous = gl::DEBUG_OUTPUT_SYNCHRONOUS,
    DepthClamp = gl::DEPTH_CLAMP,
    DepthTest = gl::DEPTH_TEST,
    Dither = gl::DITHER,
    FramebufferSrgb = gl::FRAMEBUFFER_SRGB,
    LineSmooth = gl::LINE_SMOOTH,
    Multisample = gl::MULTISAMPLE,
    PolygonOffsetFill = gl::POLYGON_OFFSET_FILL,
    PolygonOffsetLine = gl::POLYGON_OFFSET_LINE,
    PolygonOffsetPoint = gl::POLYGON_OFFSET_POINT,
    PolygonSmooth = gl::POLYGON_SMOOTH,
    PrimitiveRestart = gl::PRIMITIVE_RESTART,
    PrimitiveRestartFixedIndex = gl::PRIMITIVE_RESTART_FIXED_INDEX,
    RasterizerDiscard = gl::RASTERIZER_DISCARD,
    SampleAlphaToCoverage = gl::SAMPLE_ALPHA_TO_COVERAGE,
    SampleAlphaToOne = gl::SAMPLE_ALPHA_TO_ONE,
    SampleCoverage = gl::SAMPLE_COVERAGE,
    SampleShading = gl::SAMPLE_SHADING,
    SampleMask = gl::SAMPLE_MASK,
    ScissorTest = gl::SCISSOR_TEST,
    StencilTest = gl::STENCIL_TEST,
    TextureCubeMapSeamless = gl::TEXTURE_CUBE_MAP_SEAMLESS,
    ProgramPointSize = gl::PROGRAM_POINT_SIZE,
}

impl PartialEq<u32> for Capability {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for Capability {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"Capability\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

impl Capability {
    /// Получить индекс элемента перечисления (только для внутренних нужд)
    #[inline]
    fn get_value_index(self) -> usize {
        Capability::iter()
            .position(|cap| cap == self)
            .expect("Capability must be part of its own iterator")
    }
}

/// Менеджер для управления флагами состояний OpenGL с кэшированием
pub struct CapabilityManager {
    /// Флаги включенных возможностей OpenGL
    flags: [bool; Capability::COUNT],
}

impl CapabilityManager {
    pub fn new(hardware: bool) -> Self {
        // Инициализируем массив значениями по умолчанию
        let mut flags = [false; Capability::COUNT];

        for cap in Capability::iter() {
            let index = cap.get_value_index();

            flags[index] = if hardware {
                // Если флаг "hardware" включен, то заполняем по данным из GPU.
                // Это довольно "тяжелая" операция.
                unsafe { gl::IsEnabled(cap as u32) != 0 }
            } else {
                // Если флаг "hardware" выключен, то заполняем согласно спецификации OpenGL.
                match cap {
                    Capability::Multisample | Capability::Dither => true,
                    _ => false,
                }
            };
        }

        Self { flags }
    }

    /// Проверяем флаг состояния
    pub fn is_capability_enabled(&self, cap: Capability) -> bool {
        self.flags[cap.get_value_index()]
    }

    /// Включить флаг состояния
    pub fn enable(&mut self, cap: Capability) {
        self.set_capability(cap, true);
    }

    /// Выключить флаг состояния
    pub fn disable(&mut self, cap: Capability) {
        self.set_capability(cap, false);
    }

    /// Безопасно переключает флаг состояни ТОЛЬКО если он реально изменился
    pub fn set_capability(&mut self, cap: Capability, enable: bool) {
        let index = cap.get_value_index();

        // Проверяем кэш на CPU. Если состояние уже совпадает — игнорируем вызов!
        if self.flags[index] == enable {
            return;
        }

        // Если не совпадает — обновляем кэш и отправляем ОДНУ команду "на запись" в GPU
        self.flags[index] = enable;
        if enable {
            unsafe { Enable(cap as u32) }
        } else {
            unsafe { Disable(cap as u32) }
        }
    }
}
