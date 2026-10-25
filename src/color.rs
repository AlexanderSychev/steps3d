//! Утилиты для "связки" библиотек "palette" (работа с цветами) и "glam" (векторная алгебра для игр)

use glam::{Vec3, Vec4};
use palette::{IntoColor, LinSrgb};

pub trait IntoGlam {
    /// Конвертирует цвет в Линейный RGB f32 и возвращает glam::Vec3
    fn to_glam3(&self) -> Vec3;

    /// Конвертирует цвет в Линейный RGB f32 и возвращает glam::Vec4 с указанной альфой
    fn to_glam4(&self, alpha: f32) -> Vec4 {
        self.to_glam3().extend(alpha)
    }
}

// Реализуем этот трейт для любого типа, который palette умеет
// конвертировать в линейный sRGB на базе f32
impl<C> IntoGlam for C
where
    C: IntoColor<LinSrgb<f32>> + Clone,
{
    fn to_glam3(&self) -> Vec3 {
        let linear: LinSrgb<f32> = self.clone().into_color();
        Vec3::from_array(linear.into_components().into())
    }
}
