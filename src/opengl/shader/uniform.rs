//! Uniform-переменные для шейдеров

/// Типаж, описывающий объект, который может передавать свои данные в uniform-переменную шейдера
pub trait UniformData {
    /// Установить значение uniform-переменной по её расположению
    fn set_uniform_value(&self, location: i32);
}

// -----------------------
// - Передача примитивов -
// -----------------------

// Передача 32-битного целого числа со знаком
impl UniformData for i32 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform1i(location, *self);
        }
    }
}

// Передача 32-битного целого числа без знака
impl UniformData for u32 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform1ui(location, *self);
        }
    }
}

// Передача 32-битного числа с плавающей точкой
impl UniformData for f32 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform1f(location, *self);
        }
    }
}

// ---------------------
// - Передача векторов -
// ---------------------

use glam::{Vec2, Vec3, Vec4};

// Передача двухмерного вектора
impl UniformData for Vec2 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform2fv(location, 1, self.to_array().as_ptr());
        }
    }
}

// Передача трехмерного вектора
impl UniformData for Vec3 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform3fv(location, 1, self.to_array().as_ptr());
        }
    }
}

// Передача четырехмерного вектора
impl UniformData for Vec4 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform4fv(location, 1, self.to_array().as_ptr());
        }
    }
}

// -------------------
// - Передача цветов -
// -------------------

use palette::{LinSrgb, LinSrgba, Srgb, Srgba};

impl UniformData for LinSrgb<f32> {
    fn set_uniform_value(&self, location: i32) {
        let flat: [f32; 3] = (*self).into();
        unsafe {
            gl::Uniform3fv(location, 1, flat.as_ptr());
        }
    }
}

impl UniformData for Srgb<f32> {
    fn set_uniform_value(&self, location: i32) {
        let flat: [f32; 3] = (*self).into();
        unsafe {
            gl::Uniform3fv(location, 1, flat.as_ptr());
        }
    }
}

impl UniformData for LinSrgba<f32> {
    fn set_uniform_value(&self, location: i32) {
        let flat: [f32; 4] = (*self).into();
        unsafe {
            gl::Uniform4fv(location, 1, flat.as_ptr());
        }
    }
}

impl UniformData for Srgba<f32> {
    fn set_uniform_value(&self, location: i32) {
        let flat: [f32; 4] = (*self).into();
        unsafe {
            gl::Uniform4fv(location, 1, flat.as_ptr());
        }
    }
}

// TODO: Добавить другие реализации "UniformData" по необходимости
