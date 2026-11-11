//! Различные утилиты для типов OpenGL

use gl::{
    BYTE, DOUBLE, FIXED, FLOAT, HALF_FLOAT, INT, SHORT, UNSIGNED_BYTE, UNSIGNED_INT, UNSIGNED_SHORT,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Типы значений OpenGL
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ValueType {
    /// 8-битное целое со знаком
    Byte = BYTE,
    /// 8-битное целое без знака
    UnsignedByte = UNSIGNED_BYTE,
    /// 16-битное целое со знаком
    Short = SHORT,
    /// 16-битное целое без знака
    UnsignedShort = UNSIGNED_SHORT,
    /// 32-битное целое со знаком
    Int = INT,
    /// 32-битное целое без знака
    UnsignedInt = UNSIGNED_INT,
    /// 16.16 фиксированное
    Fixed = FIXED,
    /// 16-битное число с плавающей точкой
    HalfFloat = HALF_FLOAT,
    /// 32-битное число с плавающей точкой
    Float = FLOAT,
    /// 64-битное число с плавающей точкой
    Double = DOUBLE,
}

impl PartialEq<u32> for ValueType {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for ValueType {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"ValueType\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Трейт для скалярных типов OpenGL, которые можно использовать в качестве значений атрибутов вершин
pub trait GlScalar {
    /// Соответствующий скалярный тип OpenGL
    const VALUE_TYPE: ValueType;
}

impl GlScalar for i8 {
    const VALUE_TYPE: ValueType = ValueType::Byte;
}
impl GlScalar for u8 {
    const VALUE_TYPE: ValueType = ValueType::UnsignedByte;
}
impl GlScalar for i16 {
    const VALUE_TYPE: ValueType = ValueType::Short;
}
impl GlScalar for u16 {
    const VALUE_TYPE: ValueType = ValueType::UnsignedShort;
}
impl GlScalar for i32 {
    const VALUE_TYPE: ValueType = ValueType::Int;
}
impl GlScalar for u32 {
    const VALUE_TYPE: ValueType = ValueType::UnsignedInt;
}
impl GlScalar for f32 {
    const VALUE_TYPE: ValueType = ValueType::Float;
}
impl GlScalar for f64 {
    const VALUE_TYPE: ValueType = ValueType::Double;
}
