//! Работа с буферами OpenGL.

use super::typings::ValueType;
use bytemuck::Pod;
use gl::{
    ARRAY_BUFFER, ARRAY_BUFFER_BINDING, ATOMIC_COUNTER_BUFFER, ATOMIC_COUNTER_BUFFER_BINDING,
    BindBuffer, BindVertexArray, COPY_READ_BUFFER, COPY_READ_BUFFER_BINDING, COPY_WRITE_BUFFER,
    COPY_WRITE_BUFFER_BINDING, DISPATCH_INDIRECT_BUFFER, DISPATCH_INDIRECT_BUFFER_BINDING,
    DRAW_INDIRECT_BUFFER, DRAW_INDIRECT_BUFFER_BINDING, DYNAMIC_COPY, DYNAMIC_DRAW, DYNAMIC_READ,
    DeleteBuffers, DeleteVertexArrays, ELEMENT_ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER_BINDING,
    EnableVertexAttribArray, GenBuffers, GenVertexArrays, GetIntegerv, MAP_READ_BIT, MAP_WRITE_BIT,
    MAX_VERTEX_ATTRIBS, MapBufferRange, PIXEL_PACK_BUFFER, PIXEL_PACK_BUFFER_BINDING,
    PIXEL_UNPACK_BUFFER, PIXEL_UNPACK_BUFFER_BINDING, QUERY_BUFFER, QUERY_BUFFER_BINDING,
    SHADER_STORAGE_BUFFER, SHADER_STORAGE_BUFFER_BINDING, STATIC_COPY, STATIC_DRAW, STATIC_READ,
    STREAM_COPY, STREAM_DRAW, STREAM_READ, TEXTURE_BUFFER, TEXTURE_BUFFER_BINDING,
    TRANSFORM_FEEDBACK_BUFFER, TRANSFORM_FEEDBACK_BUFFER_BINDING, UNIFORM_BUFFER,
    UNIFORM_BUFFER_BINDING, UnmapBuffer, VertexAttribPointer,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::marker::PhantomData;
use std::num::NonZeroU32;

/// Получить максимально допустимое число атрибутов вершины в текущем контексте OpenGL.
pub fn get_max_vertex_attribs() -> i32 {
    let mut max_attribs = 0;
    unsafe {
        GetIntegerv(MAX_VERTEX_ATTRIBS, &mut max_attribs);
    }
    max_attribs
}

/// Типы буферов (*BO)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
        match value {
            ARRAY_BUFFER => Ok(Self::ArrayBuffer),
            ATOMIC_COUNTER_BUFFER => Ok(Self::AtomicCounterBuffer),
            COPY_READ_BUFFER => Ok(Self::CopyReadBuffer),
            COPY_WRITE_BUFFER => Ok(Self::CopyWriteBuffer),
            DISPATCH_INDIRECT_BUFFER => Ok(Self::DispatchIndirectBuffer),
            DRAW_INDIRECT_BUFFER => Ok(Self::DrawIndirectBuffer),
            ELEMENT_ARRAY_BUFFER => Ok(Self::ElementArrayBuffer),
            PIXEL_PACK_BUFFER => Ok(Self::PixelPackBuffer),
            PIXEL_UNPACK_BUFFER => Ok(Self::PixelUnpackBuffer),
            QUERY_BUFFER => Ok(Self::QueryBuffer),
            SHADER_STORAGE_BUFFER => Ok(Self::ShaderStorageBuffer),
            TEXTURE_BUFFER => Ok(Self::TextureBuffer),
            TRANSFORM_FEEDBACK_BUFFER => Ok(Self::TransformFeedbackBuffer),
            UNIFORM_BUFFER => Ok(Self::UniformBuffer),
            _ => Err(simple_error!(
                "Cannot create \"VertexBufferObjectType\" value from \"u32\" value: {}",
                value
            )),
        }
    }
}

/// Привязки буферов (VBO)
///
/// В основном нужны для получения текущего буфера
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
        match value {
            ARRAY_BUFFER_BINDING => Ok(Self::ArrayBufferBinding),
            ATOMIC_COUNTER_BUFFER_BINDING => Ok(Self::AtomicCounterBufferBinding),
            COPY_READ_BUFFER_BINDING => Ok(Self::CopyReadBufferBinding),
            COPY_WRITE_BUFFER_BINDING => Ok(Self::CopyWriteBufferBinding),
            DISPATCH_INDIRECT_BUFFER_BINDING => Ok(Self::DispatchIndirectBufferBinding),
            DRAW_INDIRECT_BUFFER_BINDING => Ok(Self::DrawIndirectBufferBinding),
            ELEMENT_ARRAY_BUFFER_BINDING => Ok(Self::ElementArrayBufferBinding),
            PIXEL_PACK_BUFFER_BINDING => Ok(Self::PixelPackBufferBinding),
            PIXEL_UNPACK_BUFFER_BINDING => Ok(Self::PixelUnpackBufferBinding),
            QUERY_BUFFER_BINDING => Ok(Self::QueryBufferBinding),
            SHADER_STORAGE_BUFFER_BINDING => Ok(Self::ShaderStorageBufferBinding),
            TEXTURE_BUFFER_BINDING => Ok(Self::TextureBufferBinding),
            TRANSFORM_FEEDBACK_BUFFER_BINDING => Ok(Self::TransformFeedbackBufferBinding),
            UNIFORM_BUFFER_BINDING => Ok(Self::UniformBufferBinding),
            _ => Err(simple_error!(
                "Cannot create \"VertexBufferObjectType\" value from \"u32\" value: {}",
                value
            )),
        }
    }
}

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

/// Стратегия использования буфера данных
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BufferDataUsageStrategy {
    /// Данные задаются один раз и будут использованы **немного** раз
    Stream,
    /// Данные задаются один раз и будут использованы **много** раз
    Static,
    /// Данные задаются часто и будут использованы **много** раз
    Dynamic,
}

/// Назначение буфера данных
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BufferDataUsageType {
    /// Данные будут использоваться рендерингом
    Draw,
    /// Данные будут читаться приложением
    Read,
    /// Данные будут использоваться и рендерингом, и приложением
    Copy,
}

/// Правила использования буфера данных
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct BufferDataUsage {
    // Стратегия использования буфера данных
    pub strategy: BufferDataUsageStrategy,
    // Назначение буфера данных
    pub usage_type: BufferDataUsageType,
}

impl BufferDataUsage {
    /// Создать новые правила использования буфера данных
    pub fn new(strategy: BufferDataUsageStrategy, usage_type: BufferDataUsageType) -> Self {
        Self {
            strategy,
            usage_type,
        }
    }
}

impl Into<u32> for BufferDataUsage {
    fn into(self) -> u32 {
        match (self.strategy, self.usage_type) {
            (BufferDataUsageStrategy::Stream, BufferDataUsageType::Draw) => STREAM_DRAW,
            (BufferDataUsageStrategy::Stream, BufferDataUsageType::Read) => STREAM_READ,
            (BufferDataUsageStrategy::Stream, BufferDataUsageType::Copy) => STREAM_COPY,
            (BufferDataUsageStrategy::Static, BufferDataUsageType::Draw) => STATIC_DRAW,
            (BufferDataUsageStrategy::Static, BufferDataUsageType::Read) => STATIC_READ,
            (BufferDataUsageStrategy::Static, BufferDataUsageType::Copy) => STATIC_COPY,
            (BufferDataUsageStrategy::Dynamic, BufferDataUsageType::Draw) => DYNAMIC_DRAW,
            (BufferDataUsageStrategy::Dynamic, BufferDataUsageType::Read) => DYNAMIC_READ,
            (BufferDataUsageStrategy::Dynamic, BufferDataUsageType::Copy) => DYNAMIC_COPY,
        }
    }
}

/// Буфер OpenGL (*BO)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferObject {
    /// Идентификатор ("handle") буфера в OpenGL
    opengl_id: NonZeroU32,
    /// Тип буфера
    target: BufferObjectTarget,
}

impl BufferObject {
    /// Сбросить текущий буфер по типу
    pub fn unbind_current(target: BufferObjectTarget) {
        unsafe { BindBuffer(target as u32, 0) }
    }

    /// Получить текущий буфер по типу
    pub fn get_current(target: BufferObjectTarget) -> Option<BufferObject> {
        let parameter_name: BufferObjectBinding = target.into();
        let mut opengl_id: i32 = 0;
        unsafe {
            GetIntegerv(parameter_name as u32, &mut opengl_id);
        };
        match NonZeroU32::new(opengl_id as u32) {
            Some(opengl_id) => Some(BufferObject { opengl_id, target }),
            None => None,
        }
    }

    /// Создать новый VBO
    pub fn new(buffer_type: BufferObjectTarget) -> SimpleResult<BufferObject> {
        let mut opengl_id = 0;
        unsafe {
            GenBuffers(1, &mut opengl_id);
        }

        match NonZeroU32::new(opengl_id) {
            Some(opengl_id) => Ok(BufferObject {
                opengl_id,
                target: buffer_type,
            }),
            None => Err(simple_error!(
                "OpenGL failed to create buffer object for unknown reason"
            )),
        }
    }

    pub fn is_bound(&self) -> bool {
        match BufferObject::get_current(self.target) {
            Some(current) => current.opengl_id == self.opengl_id,
            None => false,
        }
    }

    /// Назначить VBO текущим
    pub fn bind(&self) {
        unsafe { BindBuffer(self.target as u32, self.opengl_id.get()) }
    }

    pub fn unbind(&self) {
        if self.is_bound() {
            BufferObject::unbind_current(self.target);
        }
    }

    /// Удалить буфер из OpenGL
    pub fn delete(&self) {
        self.unbind();
        unsafe { DeleteBuffers(1, &self.opengl_id.get()) };
    }

    /// Заменить содержимое буфера байтовым представлением среза `Pod`-значений.
    ///
    /// Можно передавать разные типы данных в разных вызовах, например `&[f32]`
    /// или `&[Vertex]`. Все элементы одного среза должны иметь один тип.
    /// Функция возвращает ошибку, если размер данных не помещается в `GLsizeiptr`.
    pub fn fill_buffer<T: Pod>(&self, data: &[T], usage: BufferDataUsage) -> SimpleResult<()> {
        let byte_len = data
            .len()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| simple_error!("Buffer data size overflow"))?;
        let size = isize::try_from(byte_len)
            .map_err(|_| simple_error!("Buffer data is too large for GLsizeiptr"))?;
        let bytes: &[u8] = bytemuck::cast_slice(data);
        let data_ptr = if bytes.is_empty() {
            std::ptr::null()
        } else {
            bytes.as_ptr().cast()
        };

        if !self.is_bound() {
            self.bind();
        }

        unsafe {
            gl::BufferData(self.target as u32, size, data_ptr, usage.into());
        }

        Ok(())
    }

    pub fn vertex_attrib_pointer(
        &self,
        location: u32,
        size: usize,
        value_type: ValueType,
        normalized: bool,
        stride: usize,
        offset: usize,
    ) {
        if !self.is_bound() {
            self.bind();
        }

        unsafe {
            VertexAttribPointer(
                location,
                size as i32,
                value_type as u32,
                normalized as u8,
                stride as i32,
                offset as *const std::ffi::c_void,
            );
        }
    }

    pub fn enable_vertex_attrib_array(&self, location: u32) {
        if !self.is_bound() {
            self.bind();
        }
        unsafe { EnableVertexAttribArray(location) };
    }
}

/// Объект состояния вершинных буферов (VAO)
#[derive(Debug, PartialEq)]
pub struct ArrayObject {
    /// Идентификатор VAO в OpenGL
    opengl_id: u32,
}

impl ArrayObject {
    /// Создать новый VAO
    pub fn new() -> ArrayObject {
        let mut opengl_id = 0;
        unsafe {
            GenVertexArrays(1, &mut opengl_id);
        }
        ArrayObject { opengl_id }
    }

    /// Сделать этот VAO текущим
    pub fn bind(&self) {
        unsafe { BindVertexArray(self.opengl_id) }
    }
}

impl Drop for ArrayObject {
    fn drop(&mut self) {
        if self.opengl_id != 0 {
            unsafe {
                DeleteVertexArrays(1, &self.opengl_id);
            }
        }
    }
}

/// Интересная штука, разберем позже
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u32)]
pub enum BufferMapperFlag {
    Read = MAP_READ_BIT,
    Write = MAP_WRITE_BIT,
}

impl PartialEq<u32> for BufferMapperFlag {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for BufferMapperFlag {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        match value {
            MAP_READ_BIT => Ok(Self::Read),
            MAP_WRITE_BIT => Ok(Self::Write),
            _ => Err(simple_error!(
                "Cannot create \"VertexBufferObjectType\" value from \"u32\" value: {}",
                value
            )),
        }
    }
}

/// Структура для отображения данных GPU в память приложения
pub struct BufferMapper<T> {
    /// Буфер, который отображается
    buffer: BufferObject,
    _marker: PhantomData<T>,
}

impl<T> BufferMapper<T> {
    /// Создать новый маппер для указанного буфера
    pub fn new(buffer: BufferObject) -> Self {
        Self {
            buffer,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for BufferMapper<T> {
    fn drop(&mut self) {
        self.buffer.bind();
        unsafe {
            UnmapBuffer(self.buffer.target as u32);
        }
    }
}
