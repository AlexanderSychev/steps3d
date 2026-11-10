use super::super::typings::ValueType;
use std::num::NonZeroU32;
use gl::{
    BindVertexArray,
    DeleteVertexArrays,
    EnableVertexAttribArray, GenVertexArrays,
    VertexAttribPointer,
};

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

    pub fn vertex_attrib_pointer(
        &self,
        location: u32,
        size: usize,
        value_type: ValueType,
        normalized: bool,
        stride: usize,
        offset: usize,
    ) {
        // self.bind();

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
        self.bind();
        unsafe { EnableVertexAttribArray(location) };
    }
}

impl Drop for ArrayObject {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.opengl_id.get());
        }
    }
}
