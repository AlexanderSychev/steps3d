//! Пример трассировки лучей из учебника.
//! Возможно, потом часть логики уйдет в модуль "light".

use super::canvas::Canvas;
use super::geometry::DEFAULT_GEOMETRY_EPSILON;
use super::geometry::Ray3d;
use glam::Vec3;
use palette::Srgb;
use std::io::Result;
use std::path::Path;

mod camera;
mod light;
mod material;
mod object;
mod scene;

pub use camera::*;
pub use light::*;
pub use material::*;
pub use object::*;
pub use scene::*;
