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
        if !deltas.is_finite() {
            return Err(simple_error!("Position deltas must contain finite values"));
        }

        let position = self.position + self.orientation * deltas;
        if !position.is_finite() {
            return Err(simple_error!("Result position must contain finite values"));
        }

        self.position = position;
        Ok(())
    }

    /// Переместить объект на заданный вектор в глобальной системе координат (т.е. без учета ориентации)
    pub fn translate_global(&mut self, deltas: Vec3) -> SimpleResult<()> {
        if !deltas.is_finite() {
            return Err(simple_error!("Position deltas must contain finite values"));
        }

        let position = self.position + deltas;
        if !position.is_finite() {
            return Err(simple_error!("Result position must contain finite values"));
        }

        self.position = position;
        Ok(())
    }

    /// Переместить объект в глобальной системе координат (совместимый псевдоним `translate_global`)
    pub fn translate(&mut self, deltas: Vec3) -> SimpleResult<()> {
        self.translate_global(deltas)
    }

    /// Задать углы поворота объекта
    pub fn rotate(&mut self, angles: Vec3) -> SimpleResult<()> {
        if !angles.is_finite() {
            return Err(simple_error!("Rotation angles must contain finite values"));
        }

        let orientation = Quat::from_euler(
            EulerRot::YXZ,
            -angles.y, // Рыскание ("Yaw")
            -angles.x, // Тангаж ("Pitch")
            -angles.z, // Крен ("Roll")
        );
        if !orientation.is_finite() {
            return Err(simple_error!("Result rotation must contain finite values"));
        }

        self.orientation = orientation;
        Ok(())
    }

    /// Повернуть объект на заданные углы относительно текущей ориентации
    pub fn turn(&mut self, angle_deltas: Vec3) -> SimpleResult<()> {
        if !angle_deltas.is_finite() {
            return Err(simple_error!(
                "Rotation angle deltas must contain finite values"
            ));
        }

        let delta_rot = Quat::from_euler(
            EulerRot::YXZ,
            -angle_deltas.y, // Рыскание ("Yaw")
            -angle_deltas.x, // Тангаж ("Pitch")
            -angle_deltas.z, // Крен ("Roll")
        );
        if !delta_rot.is_finite() {
            return Err(simple_error!("Rotation delta must contain finite values"));
        }

        let orientation = (self.orientation * delta_rot).normalize();
        if !orientation.is_finite() {
            return Err(simple_error!("Result rotation must contain finite values"));
        }

        self.orientation = orientation;
        Ok(())
    }

    /// Задать масштаб объекта по осям локальной системы координат
    pub fn set_scale(&mut self, scale: Vec3) -> SimpleResult<()> {
        if scale.is_finite() && scale.x != 0.0 && scale.y != 0.0 && scale.z != 0.0 {
            self.scale = scale;
            Ok(())
        } else {
            Err(simple_error!("Scale must contain finite non-zero values"))
        }
    }

    /// Задать масштаб (совместимый псевдоним `set_scale`)
    pub fn scale(&mut self, scale: Vec3) -> SimpleResult<()> {
        self.set_scale(scale)
    }

    /// Увеличить масштаб по осям локальной системы координат сложением
    pub fn increase(&mut self, scale_deltas: Vec3) -> SimpleResult<()> {
        if !scale_deltas.is_finite()
            || (scale_deltas.x == 0.0 && scale_deltas.y == 0.0 && scale_deltas.z == 0.0)
        {
            return Err(simple_error!(
                "Scale changes must be finite and not all zero"
            ));
        }
        self.set_scale(self.scale + scale_deltas)
    }

    /// Увеличить масштаб по осям локальной системы координат умножением
    pub fn grow(&mut self, scale_muls: Vec3) -> SimpleResult<()> {
        if !scale_muls.is_finite()
            || (scale_muls.x == 0.0 && scale_muls.y == 0.0 && scale_muls.z == 0.0)
        {
            return Err(simple_error!(
                "Scale factors must be finite and not all zero"
            ));
        }
        self.set_scale(self.scale * scale_muls)
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

#[cfg(test)]
mod tests {
    use super::Transform;
    use glam::{Mat4, Vec3};

    #[test]
    fn scaling_methods_apply_additive_and_multiplicative_changes() {
        let mut transform = Transform::default();
        transform.increase(Vec3::X).unwrap();
        transform.grow(Vec3::new(2.0, 3.0, 4.0)).unwrap();

        assert_eq!(
            transform.get_model_matrix(),
            Mat4::from_scale(Vec3::new(4.0, 3.0, 4.0))
        );
    }

    #[test]
    fn invalid_transform_results_are_rejected_without_mutating_state() {
        let mut transform = Transform::default();
        transform.set_scale(Vec3::splat(f32::MAX)).unwrap();
        let previous_matrix = transform.get_model_matrix();

        assert!(transform.grow(Vec3::splat(2.0)).is_err());
        assert_eq!(transform.get_model_matrix(), previous_matrix);

        transform.set_position(Vec3::splat(f32::MAX)).unwrap();
        let previous_position = transform.get_position();
        assert!(transform.translate_global(Vec3::splat(f32::MAX)).is_err());
        assert_eq!(transform.get_position(), previous_position);
    }

    #[test]
    fn legacy_translate_and_scale_methods_remain_available() {
        let mut transform = Transform::default();
        transform.translate(Vec3::X).unwrap();
        transform.scale(Vec3::splat(2.0)).unwrap();

        assert_eq!(transform.get_position(), Vec3::X);
        assert_eq!(
            transform.get_model_matrix(),
            Mat4::from_scale_rotation_translation(Vec3::splat(2.0), glam::Quat::IDENTITY, Vec3::X)
        );
    }
}
