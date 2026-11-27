//! Преобразования объектов в трехмерном пространстве

use glam::{Quat, Vec3};
use simple_error::{SimpleError, SimpleResult, simple_error};

/// Типаж, описывающий стандартный объект трехмерной сцены
pub trait SceneEntity {
    /// Задать позицию в трехмерном пространстве
    fn position_entity(&mut self, position: Vec3) -> SimpleResult<()>;
    /// Переместить по всем трем осям с учетом вращения (т.е. вдоль "направляющего" вектора)
    fn move_entity(&mut self, deltas: Vec3) -> SimpleResult<()>;
    /// Переместить по всем трем осям без учета вращения
    fn translate_entity(&mut self, deltas: Vec3) -> SimpleResult<()>;
    /// Задать углы поворота объекта по всем трем осям (углы задаются в радианах)
    fn rotate_entity(&mut self, angles: Vec3) -> SimpleResult<()>;
    /// Повернуть объект относительно текущих углов поворота (углы задаются в радианах)
    fn turn_entity(&mut self, angle_deltas: Vec3) -> SimpleResult<()>;
    /// Масштабировать объект в трехмерном пространстве
    fn scale_entity(&mut self, scales: Vec3) -> SimpleResult<()>;
}

/// Опорная точка. В иерархии сцены выполняет роль центра локальной системы координат для группы объектов.
/// С её помощью можно, например, реализовать вращение различных фигур вокруг общего центра.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pivot {
    /// Положение в глобальной системе координат
    position: Vec3,
    /// Ориентация в пространстве
    orientation: Quat,
}
impl Default for Pivot {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
        }
    }
}
impl TryFrom<Vec3> for Pivot {
    type Error = SimpleError;

    fn try_from(position: Vec3) -> Result<Self, Self::Error> {
        if !position.is_finite() {
            return Err(simple_error!("Pivot position must be finite"));
        }

        Ok(Self {
            position,
            orientation: Quat::IDENTITY,
        })
    }
}
impl Pivot {
    pub fn try_new(position: Vec3, maybe_orientation: Option<Quat>) -> SimpleResult<Self> {
        if !position.is_finite() {
            return Err(simple_error!("Pivot position must be finite"));
        }
        let orientation = maybe_orientation.unwrap_or(Quat::IDENTITY);
        if !orientation.is_finite() {
            return Err(simple_error!("Pivot orientation must be finite"));
        }

        Ok(Self {
            position,
            orientation,
        })
    }
}
