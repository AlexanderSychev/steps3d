//! Источники освещения

use palette::Srgb;

/// Точечный источник освещения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointLight {
    /// Цвет
    color: Srgb<u8>,
}
impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Srgb::new(255, 255, 255),
        }
    }
}
