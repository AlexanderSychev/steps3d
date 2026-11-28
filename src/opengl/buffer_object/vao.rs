use super::super::limits::Limits;
use super::super::typings::ValueType;
use super::buffer_object_target::BufferObjectTarget;
use super::vbo::{VertexAttribute, VertexBufferObject, VertexLayout};
use bytemuck::Pod;
use simple_error::{SimpleResult, simple_error};
use std::num::NonZeroU32;
use strum::FromRepr;
use super::super::typings::impl_gl_enum_traits;
use std::marker::PhantomData;

/// Режимы отрисовки
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ArrayDrawMode {
    /// Точки
    Points = gl::POINTS,
    /// Отрезки
    LineStrip = gl::LINE_STRIP,
    /// Многоугольники из отрезков
    LineLoop = gl::LINE_LOOP,
    /// Ломаные
    Lines = gl::LINES,
    LineStripAdjacency = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency = gl::LINES_ADJACENCY,
    TriangleStrip = gl::TRIANGLE_STRIP,
    /// Полигоны трегуольников
    TriangleFan = gl::TRIANGLE_FAN,
    /// Треугольники
    Triangles = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY,
    Patches = gl::PATCHES,
}
impl_gl_enum_traits!(ArrayDrawMode);

/// Утилита-защитник для привязки VAO с последующей отвязкой при выходе из области видимости
pub struct CurrentArrayObjectGuard<'a> {
    /// Предыдущий привязанный VAO (`0`, если такого не было)
    previous: u32,
    /// Следующий привязанный VAO
    next: u32,
    /// "Метка" для компилятора
    _marker: PhantomData<&'a ArrayObject>,
}
impl <'a> CurrentArrayObjectGuard<'a> {
    fn new(next: &'a ArrayObject) -> Self {
        let next_raw = next.opengl_id.get();
        let mut previous = 0;
        unsafe {
            gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut previous);
        }
        let previous_raw = previous as u32;

        if previous_raw != next_raw {
            unsafe {
                gl::BindVertexArray(next_raw);
            }
        }

        Self {
            previous: previous_raw,
            next: next_raw,
            _marker: PhantomData
        }
    }
}
impl Drop for CurrentArrayObjectGuard<'_> {
    fn drop(&mut self) {
        if self.previous != self.next {
            unsafe {
                gl::BindVertexArray(self.previous);
            }
        }
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
            gl::GenVertexArrays(1, &mut opengl_id);
        }
        match NonZeroU32::new(opengl_id as u32) {
            Some(opengl_id) => Some(ArrayObject { opengl_id }),
            None => None,
        }
    }

    /// Сделать этот VAO текущим
    pub fn bind(&self) -> CurrentArrayObjectGuard<'_> {
        CurrentArrayObjectGuard::new(&self)
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
        let _vao_binding = self.bind();
        let _vbo_binding = vbo.bind();

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
                gl::EnableVertexAttribArray(attribute.get_location());
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

        let _vao_binding = self.bind();
        unsafe {
            gl::DrawArrays(mode as u32, first, count);
        }

        Ok(())
    }

    fn vertex_attribute_integer_pointer(&self, stride: i32, attribute: &VertexAttribute) {
        unsafe {
            gl::VertexAttribIPointer(
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
            gl::VertexAttribLPointer(
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
            gl::VertexAttribPointer(
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
            gl::DeleteVertexArrays(1, &self.opengl_id.get());
        }
    }
}
