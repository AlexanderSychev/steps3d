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
            gl::Uniform1fv(location, 2, self.to_array().as_ptr());
        }
    }
}

// Передача трехмерного вектора
impl UniformData for Vec3 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform1fv(location, 3, self.to_array().as_ptr());
        }
    }
}

// Передача четырехмерного вектора
impl UniformData for Vec4 {
    fn set_uniform_value(&self, location: i32) {
        unsafe {
            gl::Uniform1fv(location, 4, self.to_array().as_ptr());
        }
    }
}

// -------------------
// - Передача цветов -
// -------------------

use palette::{LinSrgb, Srgb, LinSrgba, Srgba};

impl UniformData for LinSrgb<f32> {
    fn set_uniform_value(&self, location: i32) {
        let flat: [f32; 3] = (*self).into();
        unsafe {
            gl::Uniform1fv(location, 3, flat.as_ptr());
        }
    }
}

impl UniformData for Srgb<f32> {
    fn set_uniform_value(&self, location: i32) {
        let linear = self.into_linear();
        linear.set_uniform_value(location);
    }
}

impl UniformData for LinSrgba<f32> {
    fn set_uniform_value(&self, location: i32) {
        let flat: [f32; 4] = (*self).into();
        unsafe {
            gl::Uniform1fv(location, 4, flat.as_ptr());
        }
    }
}

impl UniformData for Srgba<f32> {
    fn set_uniform_value(&self, location: i32) {
        let linear = self.into_linear();
        linear.set_uniform_value(location);
    }
}

// TODO: Добавить другие реализации "UniformData" по необходимости
