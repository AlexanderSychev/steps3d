//! Проверка и подключение возможностей OpenGL (capabilities)

use gl;
use simple_error::{SimpleError, SimpleResult, simple_error};

/// Различные возможности OpenGL, которые можно включить или выключить
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
        match value {
            gl::BLEND => Ok(Capability::Blend),
            gl::CLIP_DISTANCE0 => Ok(Capability::ClipDistance0),
            gl::CLIP_DISTANCE1 => Ok(Capability::ClipDistance1),
            gl::CLIP_DISTANCE2 => Ok(Capability::ClipDistance2),
            gl::CLIP_DISTANCE3 => Ok(Capability::ClipDistance3),
            gl::CLIP_DISTANCE4 => Ok(Capability::ClipDistance4),
            gl::CLIP_DISTANCE5 => Ok(Capability::ClipDistance5),
            gl::CLIP_DISTANCE6 => Ok(Capability::ClipDistance6),
            gl::CLIP_DISTANCE7 => Ok(Capability::ClipDistance7),
            gl::COLOR_LOGIC_OP => Ok(Capability::ColorLogicOp),
            gl::CULL_FACE => Ok(Capability::CullFace),
            gl::DEBUG_OUTPUT => Ok(Capability::DebugOutput),
            gl::DEBUG_OUTPUT_SYNCHRONOUS => Ok(Capability::DebugOutputSynchronous),
            gl::DEPTH_CLAMP => Ok(Capability::DepthClamp),
            gl::DEPTH_TEST => Ok(Capability::DepthTest),
            gl::DITHER => Ok(Capability::Dither),
            gl::FRAMEBUFFER_SRGB => Ok(Capability::FramebufferSrgb),
            gl::LINE_SMOOTH => Ok(Capability::LineSmooth),
            gl::MULTISAMPLE => Ok(Capability::Multisample),
            gl::POLYGON_OFFSET_FILL => Ok(Capability::PolygonOffsetFill),
            gl::POLYGON_OFFSET_LINE => Ok(Capability::PolygonOffsetLine),
            gl::POLYGON_OFFSET_POINT => Ok(Capability::PolygonOffsetPoint),
            gl::POLYGON_SMOOTH => Ok(Capability::PolygonSmooth),
            gl::PRIMITIVE_RESTART => Ok(Capability::PrimitiveRestart),
            gl::PRIMITIVE_RESTART_FIXED_INDEX => Ok(Capability::PrimitiveRestartFixedIndex),
            gl::RASTERIZER_DISCARD => Ok(Capability::RasterizerDiscard),
            gl::SAMPLE_ALPHA_TO_COVERAGE => Ok(Capability::SampleAlphaToCoverage),
            gl::SAMPLE_ALPHA_TO_ONE => Ok(Capability::SampleAlphaToOne),
            gl::SAMPLE_COVERAGE => Ok(Capability::SampleCoverage),
            gl::SAMPLE_SHADING => Ok(Capability::SampleShading),
            gl::SAMPLE_MASK => Ok(Capability::SampleMask),
            gl::SCISSOR_TEST => Ok(Capability::ScissorTest),
            gl::STENCIL_TEST => Ok(Capability::StencilTest),
            gl::TEXTURE_CUBE_MAP_SEAMLESS => Ok(Capability::TextureCubeMapSeamless),
            gl::PROGRAM_POINT_SIZE => Ok(Capability::ProgramPointSize),
            _ => Err(simple_error!(
                "Cannot create \"Capability\" value from \"u32\" value: {}",
                value
            )),
        }
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
