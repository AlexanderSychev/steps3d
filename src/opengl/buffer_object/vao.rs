use super::super::limits::Limits;
use super::super::typings::ValueType;
use super::buffer_object_target::BufferObjectTarget;
use super::vbo::{VertexAttribute, VertexBufferObject, VertexLayout};
use bytemuck::Pod;
use gl::{
    BindVertexArray, DeleteVertexArrays, DrawArrays, EnableVertexAttribArray, GenVertexArrays,
    LINE_LOOP, LINE_STRIP, LINE_STRIP_ADJACENCY, LINES, LINES_ADJACENCY, PATCHES, POINTS,
    TRIANGLE_FAN, TRIANGLE_STRIP, TRIANGLE_STRIP_ADJACENCY, TRIANGLES, TRIANGLES_ADJACENCY,
    VertexAttribIPointer, VertexAttribLPointer, VertexAttribPointer,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::num::NonZeroU32;
use strum::FromRepr;

/// Режимы отрисовки
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ArrayDrawMode {
    /// Точки
    Points = POINTS,
    /// Отрезки
    LineStrip = LINE_STRIP,
    /// Многоугольники из отрезков
    LineLoop = LINE_LOOP,
    /// Ломаные
    Lines = LINES,
    LineStripAdjacency = LINE_STRIP_ADJACENCY,
    LinesAdjacency = LINES_ADJACENCY,
    TriangleStrip = TRIANGLE_STRIP,
    /// Полигоны трегуольников
    TriangleFan = TRIANGLE_FAN,
    /// Треугольники
    Triangles = TRIANGLES,
    TriangleStripAdjacency = TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = TRIANGLES_ADJACENCY,
    Patches = PATCHES,
}

impl PartialEq<u32> for ArrayDrawMode {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for ArrayDrawMode {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"ArrayDrawMode\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Объект состояния вершинных буферов (VAO)
#[derive(Debug, PartialEq)]
pub struct ArrayObject {
    /// Идентификатор VAO в OpenGL
    opengl_id: NonZeroU32,
}

impl ArrayObject {
    /// Создать новый VAO
    pub fn new() -> Option<ArrayObject> {
        let mut opengl_id = 0;
        unsafe {
            GenVertexArrays(1, &mut opengl_id);
        }
        match NonZeroU32::new(opengl_id as u32) {
            Some(opengl_id) => Some(ArrayObject { opengl_id }),
            None => None,
        }
    }

    /// Сделать этот VAO текущим
    pub fn bind(&self) {
        unsafe { BindVertexArray(self.opengl_id.get()) }
    }

    pub fn configure_layout<T: Pod>(
        &self,
        vbo: &VertexBufferObject<T>,
        layout: &VertexLayout<T>,
        limits: &Limits,
    ) -> SimpleResult<()> {
        if vbo.get_target() != BufferObjectTarget::ArrayBuffer {
            return Err(simple_error!(
                "Invalid buffer target {:?}. Buffer must have target {:?}",
                vbo.get_target(),
                BufferObjectTarget::ArrayBuffer
            ));
        }

        // Преобразование типа "stride" с проверкой
        let stride = i32::try_from(layout.get_stride())
            .map_err(|_| simple_error!("Stride is too large for \"GLsizei\""))?;

        // Валидация атрибутов
        for attribute in layout.get_attributes() {
            if attribute.get_location() >= limits.get_max_vertex_attribs() {
                return Err(simple_error!(
                    "Location is too large: {:?}. Max allowed value is {:?}",
                    &attribute.get_location(),
                    &limits.get_max_vertex_attribs()
                ));
            }
        }

        // !! Вся работа с OpenGL - строго после проверок !!

        // Сделать буферы текущими
        self.bind();
        vbo.bind();

        // Описать атрибуты для OpenGL
        for attribute in layout.get_attributes() {
            match attribute.get_value_type() {
                ValueType::Byte
                | ValueType::UnsignedByte
                | ValueType::Short
                | ValueType::UnsignedShort
                | ValueType::Int
                | ValueType::UnsignedInt => {
                    self.vertex_attribute_integer_pointer(stride, attribute)
                }
                ValueType::Double => self.vertex_attribute_long_pointer(stride, attribute),
                _ => self.vertex_attribute_pointer(stride, attribute),
            }

            unsafe {
                EnableVertexAttribArray(attribute.get_location());
            }
        }

        Ok(())
    }

    /// "Нарисовать" примитивы по VAO
    pub fn draw_arrays(&self, mode: ArrayDrawMode, first: u32, count: u32) -> SimpleResult<()> {
        let first = i32::try_from(first)
            .map_err(|_| simple_error!("First vertex is too large for GLint"))?;
        let count = i32::try_from(count)
            .map_err(|_| simple_error!("Vertex count is too large for GLsizei"))?;

        self.bind();
        unsafe {
            DrawArrays(mode as u32, first, count);
        }

        Ok(())
    }

    fn vertex_attribute_integer_pointer(&self, stride: i32, attribute: &VertexAttribute) {
        unsafe {
            VertexAttribIPointer(
                attribute.get_location(),
                i32::from(attribute.get_components()),
                attribute.get_value_type() as u32,
                stride,
                attribute.get_offset() as *const std::ffi::c_void,
            );
        }
    }

    fn vertex_attribute_long_pointer(&self, stride: i32, attribute: &VertexAttribute) {
        unsafe {
            VertexAttribLPointer(
                attribute.get_location(),
                i32::from(attribute.get_components()),
                attribute.get_value_type() as u32,
                stride,
                attribute.get_offset() as *const std::ffi::c_void,
            );
        }
    }

    fn vertex_attribute_pointer(&self, stride: i32, attribute: &VertexAttribute) {
        unsafe {
            VertexAttribPointer(
                attribute.get_location(),
                i32::from(attribute.get_components()),
                attribute.get_value_type() as u32,
                attribute.get_normalized() as u8,
                stride,
                attribute.get_offset() as *const std::ffi::c_void,
            );
        }
    }
}

impl Drop for ArrayObject {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.opengl_id.get());
        }
    }
}
