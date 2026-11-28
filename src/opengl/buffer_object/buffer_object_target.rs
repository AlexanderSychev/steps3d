use strum::FromRepr;
use super::super::typings::impl_gl_enum_traits;

/// Типы буферов
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BufferObjectTarget {
    ArrayBuffer = gl::ARRAY_BUFFER,
    AtomicCounterBuffer = gl::ATOMIC_COUNTER_BUFFER,
    CopyReadBuffer = gl::COPY_READ_BUFFER,
    CopyWriteBuffer = gl::COPY_WRITE_BUFFER,
    DispatchIndirectBuffer = gl::DISPATCH_INDIRECT_BUFFER,
    DrawIndirectBuffer = gl::DRAW_INDIRECT_BUFFER,
    ElementArrayBuffer = gl::ELEMENT_ARRAY_BUFFER,
    PixelPackBuffer = gl::PIXEL_PACK_BUFFER,
    PixelUnpackBuffer = gl::PIXEL_UNPACK_BUFFER,
    QueryBuffer = gl::QUERY_BUFFER,
    ShaderStorageBuffer = gl::SHADER_STORAGE_BUFFER,
    TextureBuffer = gl::TEXTURE_BUFFER,
    TransformFeedbackBuffer = gl::TRANSFORM_FEEDBACK_BUFFER,
    UniformBuffer = gl::UNIFORM_BUFFER,
}

/// Привязки буферов
///
/// В основном нужны для получения текущего буфера
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BufferObjectBinding {
    ArrayBufferBinding = gl::ARRAY_BUFFER_BINDING,
    AtomicCounterBufferBinding = gl::ATOMIC_COUNTER_BUFFER_BINDING,
    CopyReadBufferBinding = gl::COPY_READ_BUFFER_BINDING,
    CopyWriteBufferBinding = gl::COPY_WRITE_BUFFER_BINDING,
    DispatchIndirectBufferBinding = gl::DISPATCH_INDIRECT_BUFFER_BINDING,
    DrawIndirectBufferBinding = gl::DRAW_INDIRECT_BUFFER_BINDING,
    ElementArrayBufferBinding = gl::ELEMENT_ARRAY_BUFFER_BINDING,
    PixelPackBufferBinding = gl::PIXEL_PACK_BUFFER_BINDING,
    PixelUnpackBufferBinding = gl::PIXEL_UNPACK_BUFFER_BINDING,
    QueryBufferBinding = gl::QUERY_BUFFER_BINDING,
    ShaderStorageBufferBinding = gl::SHADER_STORAGE_BUFFER_BINDING,
    TextureBufferBinding = gl::TEXTURE_BUFFER_BINDING,
    TransformFeedbackBufferBinding = gl::TRANSFORM_FEEDBACK_BUFFER_BINDING,
    UniformBufferBinding = gl::UNIFORM_BUFFER_BINDING,
}

impl_gl_enum_traits!(BufferObjectTarget, BufferObjectBinding);

// Конверсия между перечислениями

impl Into<BufferObjectBinding> for BufferObjectTarget {
    fn into(self) -> BufferObjectBinding {
        match self {
            Self::ArrayBuffer => BufferObjectBinding::ArrayBufferBinding,
            Self::AtomicCounterBuffer => BufferObjectBinding::AtomicCounterBufferBinding,
            Self::CopyReadBuffer => BufferObjectBinding::CopyReadBufferBinding,
            Self::CopyWriteBuffer => BufferObjectBinding::CopyWriteBufferBinding,
            Self::DispatchIndirectBuffer => BufferObjectBinding::DispatchIndirectBufferBinding,
            Self::DrawIndirectBuffer => BufferObjectBinding::DrawIndirectBufferBinding,
            Self::ElementArrayBuffer => BufferObjectBinding::ElementArrayBufferBinding,
            Self::PixelPackBuffer => BufferObjectBinding::PixelPackBufferBinding,
            Self::PixelUnpackBuffer => BufferObjectBinding::PixelUnpackBufferBinding,
            Self::QueryBuffer => BufferObjectBinding::QueryBufferBinding,
            Self::ShaderStorageBuffer => BufferObjectBinding::ShaderStorageBufferBinding,
            Self::TextureBuffer => BufferObjectBinding::TextureBufferBinding,
            Self::TransformFeedbackBuffer => BufferObjectBinding::TransformFeedbackBufferBinding,
            Self::UniformBuffer => BufferObjectBinding::UniformBufferBinding,
        }
    }
}

impl Into<BufferObjectTarget> for BufferObjectBinding {
    fn into(self) -> BufferObjectTarget {
        match self {
            Self::ArrayBufferBinding => BufferObjectTarget::ArrayBuffer,
            Self::AtomicCounterBufferBinding => BufferObjectTarget::AtomicCounterBuffer,
            Self::CopyReadBufferBinding => BufferObjectTarget::CopyReadBuffer,
            Self::CopyWriteBufferBinding => BufferObjectTarget::CopyWriteBuffer,
            Self::DispatchIndirectBufferBinding => BufferObjectTarget::DispatchIndirectBuffer,
            Self::DrawIndirectBufferBinding => BufferObjectTarget::DrawIndirectBuffer,
            Self::ElementArrayBufferBinding => BufferObjectTarget::ElementArrayBuffer,
            Self::PixelPackBufferBinding => BufferObjectTarget::PixelPackBuffer,
            Self::PixelUnpackBufferBinding => BufferObjectTarget::PixelUnpackBuffer,
            Self::QueryBufferBinding => BufferObjectTarget::QueryBuffer,
            Self::ShaderStorageBufferBinding => BufferObjectTarget::ShaderStorageBuffer,
            Self::TextureBufferBinding => BufferObjectTarget::TextureBuffer,
            Self::TransformFeedbackBufferBinding => BufferObjectTarget::TransformFeedbackBuffer,
            Self::UniformBufferBinding => BufferObjectTarget::UniformBuffer,
        }
    }
}
