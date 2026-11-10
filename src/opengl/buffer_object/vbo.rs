use super::super::typings::{GlScalar, ValueType};
use super::buffer_data_usage::BufferDataUsage;
use super::buffer_object_target::{BufferObjectBinding, BufferObjectTarget};
use bytemuck::Pod;
use gl::{
    BindBuffer, DeleteBuffers, GenBuffers, GetIntegerv, MAP_READ_BIT, MAP_WRITE_BIT,
    MAX_VERTEX_ATTRIBS, UnmapBuffer,
};
use glam::Vec3;
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::marker::PhantomData;
use std::mem::size_of;
use std::num::NonZeroU32;

/// Получить максимально допустимое число атрибутов вершины в текущем контексте OpenGL.
pub fn get_max_vertex_attribs() -> u32 {
    let mut max_attribs = 0;
    unsafe {
        GetIntegerv(MAX_VERTEX_ATTRIBS, &mut max_attribs);
    }
    max_attribs as u32
}

/// Интерфейс для вывода описания атрибута в OpenGL
pub trait AttributeFormat {
    /// Тип значения в OpenGL
    const VALUE_TYPE: ValueType;
    /// Количество компонентов
    const COMPONENTS: u8;
}

impl<S: GlScalar> AttributeFormat for S {
    const VALUE_TYPE: ValueType = S::VALUE_TYPE;
    const COMPONENTS: u8 = 1;
}

impl<S: GlScalar, const N: usize> AttributeFormat for [S; N] {
    const VALUE_TYPE: ValueType = S::VALUE_TYPE;
    const COMPONENTS: u8 = N as u8;
}

impl AttributeFormat for Vec3 {
    const VALUE_TYPE: ValueType = ValueType::Float;
    const COMPONENTS: u8 = 3;
}

/// Атрибут вершины
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    /// Позиция атрибута в шейдере
    location: u32,
    /// Количество компонентов
    components: u8,
    /// Тип значения
    value_type: ValueType,
    /// Нормализованы ли значения
    pub normalized: bool,
    /// Смещение в байтах (сколько байт надо "пропустить", чтобы "попасть" в начало атрибута)
    pub offset: usize,
}

impl VertexAttribute {
    /// Создать описание атрибута из типа
    pub fn from_type<T: AttributeFormat>(
        location: u32,
        offset: usize,
        normalized: bool,
    ) -> SimpleResult<VertexAttribute> {
        let components = T::COMPONENTS;
        if !(1..=4).contains(&components) {
            return Err(simple_error!("Components count must be value from 1 to 4"));
        }

        let max_vertex_attribs = get_max_vertex_attribs();
        if location >= max_vertex_attribs {
            return Err(simple_error!(
                "Location is too large: {:?}. Max allowed value is {:?}",
                &location,
                &max_vertex_attribs
            ));
        }

        Ok(VertexAttribute {
            location,
            offset,
            normalized,
            components,
            value_type: T::VALUE_TYPE,
        })
    }

    /// Получить позицию атрибута в шейдере
    pub fn get_location(&self) -> u32 {
        self.location
    }

    /// Получить количество компонентов
    pub fn get_components(&self) -> u8 {
        self.components
    }

    /// Получить тип значения
    pub fn get_value_type(&self) -> ValueType {
        self.value_type
    }
}

/// Описание атрибутов вершин для VAO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexLayout<T: Pod> {
    /// Размер одного элемента в байтах
    stride: usize,
    /// Атрибуты вершин
    pub attributes: Vec<VertexAttribute>,
    /// "Метка" для компилятора
    _marker: PhantomData<T>,
}

impl<T: Pod> VertexLayout<T> {
    /// Создать новое описание атрибута вершин
    pub fn new(attributes: &[VertexAttribute]) -> SimpleResult<VertexLayout<T>> {
        let stride = size_of::<T>();
        i32::try_from(stride).map_err(|_| simple_error!("Stride is too large for GLsizeiptr"))?;

        Ok(VertexLayout {
            stride,
            attributes: Vec::from(attributes),
            _marker: PhantomData,
        })
    }

    // Получить размер одного элемента в байтах
    pub fn get_stride(&self) -> usize {
        self.stride
    }
}

/// Буфер OpenGL (VBO)
#[derive(Debug, PartialEq)]
pub struct VertexBufferObject<T: Pod> {
    /// Идентификатор ("handle") буфера в OpenGL
    opengl_id: NonZeroU32,
    /// Тип буфера
    target: BufferObjectTarget,
    /// Размер элемента в байтах
    element_size: usize,
    /// "Метка" для компилятора
    _marker: PhantomData<T>,
}

impl<T: Pod> VertexBufferObject<T> {
    /// Сбросить текущий VBO по типу
    pub fn unbind_current(target: BufferObjectTarget) {
        unsafe { BindBuffer(target as u32, 0) }
    }

    /// Получить ID текущего VBO по типу
    pub fn get_current(target: BufferObjectTarget) -> Option<NonZeroU32> {
        let parameter_name: BufferObjectBinding = target.into();
        let mut opengl_id: i32 = 0;
        unsafe {
            GetIntegerv(parameter_name as u32, &mut opengl_id);
        };
        match NonZeroU32::new(opengl_id as u32) {
            Some(opengl_id) => Some(opengl_id),
            None => None,
        }
    }

    /// Создать новый VBO
    pub fn new(buffer_type: BufferObjectTarget) -> SimpleResult<VertexBufferObject<T>> {
        let mut opengl_id = 0;
        unsafe {
            GenBuffers(1, &mut opengl_id);
        }

        match NonZeroU32::new(opengl_id) {
            Some(opengl_id) => Ok(VertexBufferObject::<T> {
                opengl_id,
                target: buffer_type,
                element_size: size_of::<T>(),
                _marker: PhantomData,
            }),
            None => Err(simple_error!(
                "OpenGL failed to create buffer object for unknown reason"
            )),
        }
    }

    /// Получить тип буфера
    pub fn get_target(&self) -> BufferObjectTarget {
        self.target
    }

    /// Проверить, является ли данный VBO текущим
    pub fn is_bound(&self) -> bool {
        match VertexBufferObject::<T>::get_current(self.target) {
            Some(current) => current == self.opengl_id,
            None => false,
        }
    }

    /// Назначить VBO текущим
    pub fn bind(&self) {
        unsafe { BindBuffer(self.target as u32, self.opengl_id.get()) }
    }

    /// Отвязать VBO от текущего состояния
    pub fn unbind(&self) {
        if self.is_bound() {
            VertexBufferObject::<T>::unbind_current(self.target);
        }
    }

    /// Заменить содержимое буфера байтовым представлением среза `Pod`-значений.
    ///
    /// Функция возвращает ошибку, если размер данных не помещается в `GLsizeiptr`.
    pub fn fill_buffer(&self, data: &[T], usage: BufferDataUsage) -> SimpleResult<()> {
        let byte_len = data
            .len()
            .checked_mul(self.element_size)
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

impl<T: Pod> Drop for VertexBufferObject<T> {
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
pub struct BufferMapper<T: Pod> {
    /// Буфер, который отображается
    buffer: VertexBufferObject<T>,
    /// "Метка" для компилятора
    _marker: PhantomData<T>,
}

impl<T: Pod> BufferMapper<T> {
    /// Создать новый маппер для указанного буфера
    pub fn new(buffer: VertexBufferObject<T>) -> Self {
        Self {
            buffer,
            _marker: PhantomData,
        }
    }
}

impl<T: Pod> Drop for BufferMapper<T> {
    fn drop(&mut self) {
        self.buffer.bind();
        unsafe {
            UnmapBuffer(self.buffer.target as u32);
        }
    }
}
