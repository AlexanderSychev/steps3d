use bytemuck::Pod;
use gl::{
    BindBuffer,
    DeleteBuffers,
    GenBuffers, GetIntegerv,
    UnmapBuffer,
    MAP_READ_BIT,
    MAP_WRITE_BIT,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::num::NonZeroU32;
use super::buffer_object_target::{BufferObjectBinding, BufferObjectTarget};
use super::buffer_data_usage::BufferDataUsage;
use std::marker::PhantomData;

/// Буфер OpenGL (VBO)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexBufferObject {
    /// Идентификатор ("handle") буфера в OpenGL
    opengl_id: NonZeroU32,
    /// Тип буфера
    target: BufferObjectTarget,
}

impl VertexBufferObject {
    /// Сбросить текущий буфер по типу
    pub fn unbind_current(target: BufferObjectTarget) {
        unsafe { BindBuffer(target as u32, 0) }
    }

    /// Получить текущий буфер по типу
    pub fn get_current(target: BufferObjectTarget) -> Option<VertexBufferObject> {
        let parameter_name: BufferObjectBinding = target.into();
        let mut opengl_id: i32 = 0;
        unsafe {
            GetIntegerv(parameter_name as u32, &mut opengl_id);
        };
        match NonZeroU32::new(opengl_id as u32) {
            Some(opengl_id) => Some(VertexBufferObject { opengl_id, target }),
            None => None,
        }
    }

    /// Создать новый VBO
    pub fn new(buffer_type: BufferObjectTarget) -> SimpleResult<VertexBufferObject> {
        let mut opengl_id = 0;
        unsafe {
            GenBuffers(1, &mut opengl_id);
        }

        match NonZeroU32::new(opengl_id) {
            Some(opengl_id) => Ok(VertexBufferObject {
                opengl_id,
                target: buffer_type,
            }),
            None => Err(simple_error!(
                "OpenGL failed to create buffer object for unknown reason"
            )),
        }
    }

    pub fn is_bound(&self) -> bool {
        match VertexBufferObject::get_current(self.target) {
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
            VertexBufferObject::unbind_current(self.target);
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

        self.bind();

        unsafe {
            gl::BufferData(self.target as u32, size, data_ptr, usage.into());
        }

        Ok(())
    }
}

impl Drop for VertexBufferObject {
    fn drop(&mut self) {
        self.unbind();
        unsafe { DeleteBuffers(1, &self.opengl_id.get()) };
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
    buffer: VertexBufferObject,
    _marker: PhantomData<T>,
}

impl<T> BufferMapper<T> {
    /// Создать новый маппер для указанного буфера
    pub fn new(buffer: VertexBufferObject) -> Self {
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
