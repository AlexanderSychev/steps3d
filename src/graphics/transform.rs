//! Модуль трансформаций: позиция, поворот, масштаб и матрица модели.

use glam::camera::lh::view::look_to_mat4;
use glam::{EulerRot, Mat4, Quat, Vec3};
use simple_error::{SimpleResult, simple_error};

/// Трансформации объекта в мировом пространстве
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Положение объекта в глобальной системе координат
    position: Vec3,
    /// Ориентация объекта в виде кватерниона
    orientation: Quat,
    /// Масштаб объекта по осям
    scale: Vec3,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
impl Transform {
    /// Базис: получить вектор направления вправо
    pub fn get_right(&self) -> Vec3 {
        self.orientation * Vec3::X
    }

    /// Базис: получить вектор направления вверх
    pub fn get_up(&self) -> Vec3 {
        self.orientation * Vec3::Y
    }

    /// Базис: получить вектор направления взгляда
    pub fn get_view(&self) -> Vec3 {
        self.orientation * Vec3::Z
    }

    /// Получить позицию в глобальной системе координат
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    /// Задать позицию в глобальной системе координат
    pub fn set_position(&mut self, position: Vec3) -> SimpleResult<()> {
        if position.is_finite() {
            self.position = position;
            Ok(())
        } else {
            Err(simple_error!("Position must contain finite values"))
        }
    }

    /// Переместить объект на заданный вектор в локальной системе координат (т.е. с учетом ориентации)
    pub fn translate_local(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if deltas.is_finite() {
            self.position += self.orientation * deltas;
            Ok(())
        } else {
            Err(simple_error!("Position deltas must contain finite values"))
        }
    }

    /// Переместить объект на заданный вектор в глобальной системе координат (т.е. без учета ориентации)
    pub fn translate(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if deltas.is_finite() {
            self.position += deltas;
            Ok(())
        } else {
            Err(simple_error!("Position deltas must contain finite values"))
        }
    }

    /// Задать углы поворота объекта
    pub fn rotate(&mut self, angles: Vec3) -> SimpleResult<()> {
        if angles.is_finite() {
            self.orientation = Quat::from_euler(
                EulerRot::YXZ,
                -angles.y, // Раскание ("Yaw")
                -angles.x, // Тангаж ("Pitch")
                -angles.z, // Крен ("Roll")
            );
            Ok(())
        } else {
            Err(simple_error!("Rotation angles must contain finite values"))
        }
    }

    /// Повернуть объект на заданные углы относительно текущей ориентации
    pub fn turn(&mut self, angle_deltas: Vec3) -> SimpleResult<()> {
        if angle_deltas.is_finite() {
            let delta_rot = Quat::from_euler(
                EulerRot::YXZ,
                -angle_deltas.y, // Раскание ("Yaw")
                -angle_deltas.x, // Тангаж ("Pitch")
                -angle_deltas.z, // Крен ("Roll")
            );
            self.orientation = (self.orientation * delta_rot).normalize();
            Ok(())
        } else {
            Err(simple_error!(
                "Rotation angle deltas must contain finite values"
            ))
        }
    }

    /// Масштабировать объект по осям локальной системы координат
    pub fn scale(&mut self, scale: Vec3) -> SimpleResult<()> {
        if scale.is_finite() && scale.x != 0.0 && scale.y != 0.0 && scale.z != 0.0 {
            self.scale = scale;
            Ok(())
        } else {
            Err(simple_error!("Scale must contain finite non-zero values"))
        }
    }

    /// Получить матрицу модели для передачи в шейдер
    pub fn get_model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.orientation, self.position)
    }

    /// Получить матрицу вида (View Matrix) для левостороннего мира
    pub fn get_view_matrix(&self) -> Mat4 {
        // self.get_view() и self.get_up() уже нормализованы
        look_to_mat4(self.position, self.get_view(), self.get_up())
    }
}
