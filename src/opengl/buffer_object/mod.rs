//! Работа с буферами OpenGL.

mod buffer_object_target;
mod buffer_data_usage;
mod vbo;
mod vao;

pub use buffer_object_target::*;
pub use buffer_data_usage::*;
pub use vbo::*;
pub use vao::*;

use super::typings::ValueType;
use bytemuck::Pod;
use gl::{
    BindBuffer, BindVertexArray,
    DeleteBuffers, DeleteVertexArrays,
    EnableVertexAttribArray, GenBuffers, GenVertexArrays, GetIntegerv, MAP_READ_BIT, MAP_WRITE_BIT,
    MAX_VERTEX_ATTRIBS, UnmapBuffer, VertexAttribPointer,
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
