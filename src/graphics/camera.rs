//! Камера

use super::super::constraints::Viewport;
use super::transform::MovableEntity;
use glam::camera::lh::proj::opengl::perspective;
use glam::camera::lh::view::look_to_mat4;
use glam::{EulerRot, Mat4, Quat, Vec3};
use simple_error::{SimpleResult, simple_error};
use std::f32::consts::PI;

/// Плоскости отсечения (по оси Z)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippingPlanes {
    /// Ближняя
    near: f32,
    /// Дальняя
    far: f32,
}
impl ClippingPlanes {
    pub fn try_new(near: f32, far: f32) -> SimpleResult<Self> {
        if !near.is_finite() {
            return Err(simple_error!("Near clipping plane value must be finite"));
        }
        if near <= 0.0 {
            return Err(simple_error!("Near clipping plane value must be positive"));
        }
        if !far.is_finite() {
            return Err(simple_error!("Far clipping plane value must be finite"));
        }
        if far <= near {
            return Err(simple_error!(
                "Far clipping plane value must be greater than near"
            ));
        }

        Ok(Self { near, far })
    }

    /// Получить ближнюю плоскость отсечения
    pub fn get_near(&self) -> f32 {
        self.near
    }

    /// Задать ближнюю плоскость отсечения
    pub fn set_near(&mut self, near: f32) -> SimpleResult<()> {
        if !near.is_finite() {
            return Err(simple_error!("Near clipping plane value must be finite"));
        }
        if near <= 0.0 {
            return Err(simple_error!("Near clipping plane value must be positive"));
        }
        if self.far <= near {
            return Err(simple_error!(
                "Far clipping plane value must be greater than near"
            ));
        }
        self.near = near;

        Ok(())
    }

    /// Получить дальнюю плоскость отсечения
    pub fn get_far(&self) -> f32 {
        self.far
    }

    /// Задать дальнюю плоскость отсечения
    pub fn set_far(&mut self, far: f32) -> SimpleResult<()> {
        if !far.is_finite() {
            return Err(simple_error!("Far clipping plane value must be finite"));
        }
        if far <= self.near {
            return Err(simple_error!(
                "Far clipping plane value must be greater than near"
            ));
        }

        self.far = far;

        Ok(())
    }
}
impl Default for ClippingPlanes {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Перспективная камера
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCamera {
    /// Позиция камеры
    position: Vec3,
    /// Ориентация камеры в пространстве
    orientation: Quat,
    /// Область просмотра камеры
    viewport: Viewport,
    /// Плоскости отсечения
    clipping: ClippingPlanes,
    /// Угол обзора по оси X (в радианах)
    field_of_view_x: f32,
    /// Угол обзора по оси Y (в радианах)
    field_of_view_y: f32,
}
impl PerspectiveCamera {
    /// Попробовать создать камеру со стандартным базисом
    pub fn try_new(
        field_of_view_x: f32,
        position: Vec3,
        viewport: Viewport,
        clipping: ClippingPlanes,
    ) -> SimpleResult<Self> {
        if !(0.0..PI).contains(&field_of_view_x) {
            return Err(simple_error!(
                "X-axis field of view angle must be in range (0, π)"
            ));
        }
        if !position.is_finite() {
            return Err(simple_error!(
                "Position point must contain only finite values"
            ));
        }

        // Вычисляем угол обзора по оси Y
        let aspect_ratio_inversed = viewport.get_aspect_ratio_inversed();
        let field_of_view_y = 2.0 * ((field_of_view_x * 0.5).tan() * aspect_ratio_inversed).atan();

        Ok(Self {
            position,
            orientation: Quat::IDENTITY,
            viewport,
            clipping,
            field_of_view_x,
            field_of_view_y,
        })
    }

    // Получение векторов базиса

    /// Получить направление вправо для базиса камеры
    pub fn get_right(&self) -> Vec3 {
        self.orientation * Vec3::X
    }

    /// Получить направление вверх для базиса камеры
    pub fn get_up(&self) -> Vec3 {
        self.orientation * Vec3::Y
    }

    /// Получить направление взгляда для базиса камеры
    pub fn get_view(&self) -> Vec3 {
        self.orientation * Vec3::Z
    }

    /// Получить область просмотра
    pub fn get_viewport(&self) -> Viewport {
        self.viewport
    }

    /// Изменить область просмотра
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;

        // Пересчитываем угол обзора по оси Y при изменении области просмотра
        self.field_of_view_y = 2.0
            * ((self.field_of_view_x * 0.5).tan() * self.viewport.get_aspect_ratio_inversed())
                .atan();
    }

    /// Получить угол обзора по оси X (в радианах)
    pub fn get_field_of_view_x(&self) -> f32 {
        self.field_of_view_x
    }

    /// Задать угол обзора по оси X (в радианах)
    pub fn set_field_of_view_x(&mut self, field_of_view_x: f32) -> SimpleResult<()> {
        if !(0.0..PI).contains(&field_of_view_x) {
            return Err(simple_error!(
                "X-axis field of view angle must be in range (0, π)"
            ));
        }
        self.field_of_view_x = field_of_view_x;

        // Пересчитываем угол обзора по оси Y при изменении области просмотра
        self.field_of_view_y = 2.0
            * ((self.field_of_view_x * 0.5).tan() * self.viewport.get_aspect_ratio_inversed())
                .atan();

        Ok(())
    }

    /// Получить отсекающие плоскости для чтения
    pub fn get_clipping(&self) -> ClippingPlanes {
        self.clipping
    }

    /// Получить отсекающие плоскости для записи
    pub fn get_clipping_mut(&mut self) -> &mut ClippingPlanes {
        &mut self.clipping
    }

    /// Задать новые отсекающие плоскости
    pub fn set_clipping(&mut self, clipping: ClippingPlanes) {
        self.clipping = clipping
    }

    /// Получить позицию камеры
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    /// Возвращает матрицу вида (View Matrix) для левостороннего мира
    pub fn get_view_matrix(&self) -> Mat4 {
        // self.view() и self.up() уже нормализованы
        look_to_mat4(self.position, self.get_view(), self.get_up())
    }

    /// Возвращает матрицу проекции (Projection Matrix) для OpenGL
    pub fn get_projection_matrix(&self) -> Mat4 {
        perspective(
            self.field_of_view_y,
            self.viewport.get_aspect_ratio(),
            self.clipping.near,
            self.clipping.far,
        )
    }
}
impl MovableEntity for PerspectiveCamera {
    fn position_entity(&mut self, position: Vec3) -> SimpleResult<()> {
        if !position.is_finite() {
            return Err(simple_error!(
                "Position point must contain only finite values"
            ));
        }
        self.position = position;
        Ok(())
    }

    fn move_entity(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if !deltas.is_finite() {
            return Err(simple_error!(
                "Position deltas must contain only finite values"
            ));
        }
        self.position += self.orientation * deltas;
        Ok(())
    }

    fn translate_entity(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if !deltas.is_finite() {
            return Err(simple_error!(
                "Position deltas must contain only finite values"
            ));
        }
        self.position += deltas;
        Ok(())
    }

    fn rotate_entity(&mut self, angles: Vec3) -> SimpleResult<()> {
        if !angles.is_finite() {
            return Err(simple_error!(
                "Rotation angles must contain only finite values"
            ));
        }
        self.orientation = Quat::from_euler(
            EulerRot::YXZ,
            -angles.y, // Yaw
            -angles.x, // Pitch
            -angles.z, // Roll
        );
        Ok(())
    }

    fn turn_entity(&mut self, angle_deltas: Vec3) -> SimpleResult<()> {
        if !angle_deltas.is_finite() {
            return Err(simple_error!(
                "Rotation angles must contain only finite values"
            ));
        }
        let delta_rot = Quat::from_euler(
            EulerRot::YXZ,
            -angle_deltas.y,
            -angle_deltas.x,
            -angle_deltas.z,
        );
        self.orientation = (self.orientation * delta_rot).normalize();
        Ok(())
    }
}
