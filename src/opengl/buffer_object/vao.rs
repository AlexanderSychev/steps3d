use super::super::typings::ValueType;
use super::buffer_object_target::BufferObjectTarget;
use super::vbo::{VertexBufferObject, VertexLayout};
use bytemuck::Pod;
use gl::{
    BindVertexArray, DeleteVertexArrays, EnableVertexAttribArray, GenVertexArrays,
    VertexAttribPointer,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::num::NonZeroU32;

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
    ) -> SimpleResult<()> {
        if vbo.get_target() != BufferObjectTarget::ArrayBuffer {
            return Err(simple_error!(
                "Invalid buffer target {:?}. Buffer must have target {:?}",
                vbo.get_target(),
                BufferObjectTarget::ArrayBuffer
            ));
        }

        // Сделать буферы текущими
        self.bind();
        vbo.bind();

        for attribute in &layout.attributes {
            unsafe {
                VertexAttribPointer(
                    attribute.get_location(),
                    i32::from(attribute.get_components()),
                    attribute.get_value_type() as u32,
                    attribute.normalized as u8,
                    i32::try_from(layout.get_stride()).unwrap(),
                    attribute.offset as *const std::ffi::c_void,
                );
                EnableVertexAttribArray(attribute.get_location());
            }
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
