//! Модуль источников освещения.

use palette::Srgb;
use glam::Vec3;

/// Глобальное диффузное освещение
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffuseLight {
    /// Вектор направления лучей
    pub direction: Vec3,
    /// Цвет освещения
    pub main_color: Srgb<f32>,
    /// Фоновой цвет (для неосвещённых участков)
    pub ambient_color: Srgb<f32>,
}

impl DiffuseLight {
    /// Создать новое глобальное диффузное освещение
    pub fn new(direction: Vec3, main_color: Srgb<f32>, ambient_color: Srgb<f32>) -> Self {
        Self { direction, main_color, ambient_color }
    }
}

/// Точечный источник света
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    /// Расположение источника в глобальной системе координат
    pub position: Vec3,
    /// Цвет освещения
    pub color: Srgb<f32>,
}

impl PointLight {
    /// Создать новый точечный источник света
    pub fn new(position: Vec3, color: Srgb<f32>) -> Self {
        Self { position, color }
    }
}
