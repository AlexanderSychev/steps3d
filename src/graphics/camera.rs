//! Камера

use super::super::constraints::{AngleRadians, Viewport};
use glam::Vec3;
use simple_error::{SimpleResult, simple_error};
use std::f32::consts::PI;

/// Базис камеры
#[derive(Debug, Clone, Copy, PartialEq)]
struct CameraBasis {
    /// Вектор направления взгляда
    view: Vec3,
    /// Вектор направления вверх
    up: Vec3,
    /// Вектор направления вправо
    right: Vec3,
}
impl Default for CameraBasis {
    fn default() -> Self {
        // Стандартный базис камеры
        let view = Vec3::new(0.0, 0.0, 1.0);
        let view = view / view.length();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let right = up.cross(view).normalize();
        let up = view.cross(right).normalize();
        Self { view, up, right }
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCamera {
    /// Позиция камеры
    position: Vec3,
    /// Базис камеры
    basis: CameraBasis,
    /// Область просмотра камеры
    viewport: Viewport,
    /// Угол обзора по оси X (в радианах)
    field_of_view_x: f32,
    /// Угол обзора по оси Y (в радианах)
    field_of_view_y: f32,
}
impl PerspectiveCamera {
    /// Попробовать создать камеру со стандартным базисом
    pub fn try_new(
        position: Vec3,
        viewport: Viewport,
        field_of_view_x: AngleRadians,
    ) -> SimpleResult<Self> {
        let basis = CameraBasis::default();

        if field_of_view_x.get() >= PI {
            return Err(simple_error!("Field of view X must be less than PI"));
        }

        // Вычисляем угол обзора по оси Y
        let aspect_ratio_inversed = viewport.get_aspect_ratio_inversed();
        let field_of_view_y =
            2.0 * ((field_of_view_x.get() * 0.5).tan() * aspect_ratio_inversed).atan();

        Ok(Self {
            position,
            basis,
            viewport,
            field_of_view_x: field_of_view_x.get(),
            field_of_view_y,
        })
    }

    /// Получить позицию камеры
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    /// Задать позицию камеры
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn get_viewort(&self) -> Viewport {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;

        // Пересчитываем угол обзора по оси Y при изменении области просмотра
        self.field_of_view_y = 2.0
            * ((self.field_of_view_x * 0.5).tan() * self.viewport.get_aspect_ratio_inversed())
                .atan();
    }

    pub fn get_field_of_view_x(&self) -> f32 {
        self.field_of_view_x
    }

    pub fn set_field_of_view_x(&mut self, field_of_view_x: AngleRadians) {
        self.field_of_view_x = field_of_view_x.get();

        // Пересчитываем угол обзора по оси Y при изменении области просмотра
        self.field_of_view_y = 2.0
            * ((self.field_of_view_x * 0.5).tan() * self.viewport.get_aspect_ratio_inversed())
                .atan();
    }
}
