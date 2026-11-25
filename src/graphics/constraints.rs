//! Дополнительные структуры данных с валидацией

use simple_error::{SimpleResult, simple_error};
use std::f32::consts::PI;

/// Область просмотра
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Viewport {
    /// Ширина в пикселях
    width: u32,
    /// Высота в пикселях
    height: u32,
}
impl Viewport {
    /// Создаёт новую область просмотра с проверкой размеров
    pub fn try_new(width: u32, height: u32) -> SimpleResult<Viewport> {
        if width == 0 {
            return Err(simple_error!("Width cannot be zero"));
        }
        if height == 0 {
            return Err(simple_error!("Height cannot be zero"));
        }
        Ok(Viewport { width, height })
    }

    /// Получить ширину в пикселях
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Задать ширину в пикселях
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }

    /// Получить высоту в пикселях
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Задать высоту в пикселях
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }

    /// Получить соотношение сторон
    pub fn get_aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Получить обратное соотношение сторон
    pub fn get_aspect_ratio_inversed(&self) -> f32 {
        self.height as f32 / self.width as f32
    }
}

/// Угол в радианах
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AngleRadians {
    /// Значение угла
    angle: f32,
}
impl AngleRadians {
    /// Создаёт новый угол в радианах с проверкой значения
    pub fn try_new(angle: f32) -> SimpleResult<AngleRadians> {
        if !angle.is_finite() || (0.0..2.0 * PI).contains(&angle) {
            return Err(simple_error!("Angle must be in range [0, 2π]"));
        }
        Ok(AngleRadians { angle })
    }

    /// Получить значение угла в радианах
    pub fn get(&self) -> f32 {
        self.angle
    }

    /// Задать значение угла с проверкой
    pub fn set(&mut self, angle: f32) -> SimpleResult<()> {
        if !angle.is_finite() || (0.0..2.0 * PI).contains(&angle) {
            return Err(simple_error!("Angle must be in range [0, 2π]"));
        }
        self.angle = angle;
        Ok(())
    }
}
