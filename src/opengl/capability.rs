//! Проверка и подключение возможностей OpenGL (capabilities)

use gl;
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Различные возможности OpenGL, которые можно включить или выключить
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
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
        Self::from_repr(value).ok_or_else(|| simple_error!(
            "Cannot create \"Capability\" value from \"u32\" value: {}",
            value
        ))
    }
}

impl Capability {
    /// Проверить, включина ли возможность OpenGL
    pub fn is_enabled(self) -> bool {
        unsafe { gl::IsEnabled(self as u32) != 0 }
    }

    /// Включить возможность OpenGL
    pub fn enable(self) {
        unsafe {
            gl::Enable(self as u32);
        }
    }

    /// Отключить возможность OpenGL
    pub fn disable(self) {
        unsafe { gl::Disable(self as u32) }
    }
}
