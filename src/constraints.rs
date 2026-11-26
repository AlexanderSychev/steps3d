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
    pub fn set_width(&mut self, width: u32) -> SimpleResult<()> {
        if width == 0 {
            return Err(simple_error!("Width cannot be zero"));
        }
        self.width = width;
        Ok(())
    }

    /// Получить высоту в пикселях
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Задать высоту в пикселях
    pub fn set_height(&mut self, height: u32) -> SimpleResult<()> {
        if height == 0 {
            return Err(simple_error!("Height cannot be zero"));
        }
        self.height = height;
        Ok(())
    }

    /// Задать новые размеры области видимости
    pub fn resize(&mut self, width: u32, height: u32) -> SimpleResult<()> {
        if width == 0 {
            return Err(simple_error!("Width cannot be zero"));
        }
        if height == 0 {
            return Err(simple_error!("Height cannot be zero"));
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Получить соотношение сторон (ширина деленная на высоту)
    pub fn get_aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Получить обратное соотношение сторон (высота деленная на ширину)
    pub fn get_aspect_ratio_inversed(&self) -> f32 {
        self.height as f32 / self.width as f32
    }
}
