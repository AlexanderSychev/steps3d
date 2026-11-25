//! Преобразования объектов в трехмерном пространстве

use glam::Vec3;
use simple_error::SimpleResult;

/// Типаж объекта, который можно различным образом перемещать в трехмерном пространстве
pub trait MovableEntity {
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
}

/// Типаж объекта, который можно масштабировать в трехмерном пространстве
pub trait Scalable {
    /// Масштабировать объект в трехмерном пространстве
    fn scale_entity(&mut self, scales: Vec3) -> SimpleResult<()>;
}
