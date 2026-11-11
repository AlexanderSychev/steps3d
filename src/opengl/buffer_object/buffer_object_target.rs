use gl::{
    ARRAY_BUFFER, ARRAY_BUFFER_BINDING, ATOMIC_COUNTER_BUFFER, ATOMIC_COUNTER_BUFFER_BINDING,
    COPY_READ_BUFFER, COPY_READ_BUFFER_BINDING, COPY_WRITE_BUFFER, COPY_WRITE_BUFFER_BINDING,
    DISPATCH_INDIRECT_BUFFER, DISPATCH_INDIRECT_BUFFER_BINDING, DRAW_INDIRECT_BUFFER,
    DRAW_INDIRECT_BUFFER_BINDING, ELEMENT_ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER_BINDING,
    PIXEL_PACK_BUFFER, PIXEL_PACK_BUFFER_BINDING, PIXEL_UNPACK_BUFFER, PIXEL_UNPACK_BUFFER_BINDING,
    QUERY_BUFFER, QUERY_BUFFER_BINDING, SHADER_STORAGE_BUFFER, SHADER_STORAGE_BUFFER_BINDING,
    TEXTURE_BUFFER, TEXTURE_BUFFER_BINDING, TRANSFORM_FEEDBACK_BUFFER,
    TRANSFORM_FEEDBACK_BUFFER_BINDING, UNIFORM_BUFFER, UNIFORM_BUFFER_BINDING,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Типы буферов
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BufferObjectTarget {
    ArrayBuffer = ARRAY_BUFFER,
    AtomicCounterBuffer = ATOMIC_COUNTER_BUFFER,
    CopyReadBuffer = COPY_READ_BUFFER,
    CopyWriteBuffer = COPY_WRITE_BUFFER,
    DispatchIndirectBuffer = DISPATCH_INDIRECT_BUFFER,
    DrawIndirectBuffer = DRAW_INDIRECT_BUFFER,
    ElementArrayBuffer = ELEMENT_ARRAY_BUFFER,
    PixelPackBuffer = PIXEL_PACK_BUFFER,
    PixelUnpackBuffer = PIXEL_UNPACK_BUFFER,
    QueryBuffer = QUERY_BUFFER,
    ShaderStorageBuffer = SHADER_STORAGE_BUFFER,
    TextureBuffer = TEXTURE_BUFFER,
    TransformFeedbackBuffer = TRANSFORM_FEEDBACK_BUFFER,
    UniformBuffer = UNIFORM_BUFFER,
}

impl PartialEq<u32> for BufferObjectTarget {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for BufferObjectTarget {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"BufferObjectTarget\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Привязки буферов
///
/// В основном нужны для получения текущего буфера
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BufferObjectBinding {
    ArrayBufferBinding = ARRAY_BUFFER_BINDING,
    AtomicCounterBufferBinding = ATOMIC_COUNTER_BUFFER_BINDING,
    CopyReadBufferBinding = COPY_READ_BUFFER_BINDING,
    CopyWriteBufferBinding = COPY_WRITE_BUFFER_BINDING,
    DispatchIndirectBufferBinding = DISPATCH_INDIRECT_BUFFER_BINDING,
    DrawIndirectBufferBinding = DRAW_INDIRECT_BUFFER_BINDING,
    ElementArrayBufferBinding = ELEMENT_ARRAY_BUFFER_BINDING,
    PixelPackBufferBinding = PIXEL_PACK_BUFFER_BINDING,
    PixelUnpackBufferBinding = PIXEL_UNPACK_BUFFER_BINDING,
    QueryBufferBinding = QUERY_BUFFER_BINDING,
    ShaderStorageBufferBinding = SHADER_STORAGE_BUFFER_BINDING,
    TextureBufferBinding = TEXTURE_BUFFER_BINDING,
    TransformFeedbackBufferBinding = TRANSFORM_FEEDBACK_BUFFER_BINDING,
    UniformBufferBinding = UNIFORM_BUFFER_BINDING,
}

impl PartialEq<u32> for BufferObjectBinding {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for BufferObjectBinding {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"BufferObjectBinding\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

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
