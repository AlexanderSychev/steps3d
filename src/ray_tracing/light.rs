//! Примитивы освещения

use super::super::geometry::DEFAULT_GEOMETRY_EPSILON;
use super::super::geometry::Ray3d;
use super::scene::Scene;
use glam::Vec3;
use palette::Srgb;

/// Базовый Интерфейс фонового освещения
pub trait BackgroundLight {
    /// Получить цвет фонового освещения по лучу
    fn get_background(&self, ray: &Ray3d) -> Srgb<f32>;
}

/// Фоновое освещение с постоянным цветом, независящим от луча
pub struct SolidBackgroundLight {
    color: Srgb<f32>,
}

impl SolidBackgroundLight {
    pub fn new(color: Srgb<f32>) -> Self {
        Self { color }
    }
}

impl BackgroundLight for SolidBackgroundLight {
    fn get_background(&self, _ray: &Ray3d) -> Srgb<f32> {
        self.color
    }
}

pub struct HorizontalGradientBackgroundLight {
    /// Верхний цвет градиента
    up_color: Srgb<f32>,
    /// Нижний цвет градиента
    down_color: Srgb<f32>,
}

impl BackgroundLight for HorizontalGradientBackgroundLight {
    fn get_background(&self, ray: &Ray3d) -> Srgb<f32> {
        let t = 0.5 * (ray.direction.y + 1.0);

        return self.down_color * (1.0 - t) + self.up_color * t;
    }
}

/// Конкретные виды источников света
#[derive(Debug, Clone, Copy)]
pub enum LightSourceKind {
    /// Точечный источник света
    Point {
        /// Позиция источника в пространстве
        position: Vec3,
    },
    /// Сферический источник света
    Spheric {
        /// Позиция источника в пространстве
        position: Vec3,
        /// Радиус сферы
        radius: f32,
    },
}

/// Источник света
#[derive(Debug, Clone, Copy)]
pub struct LightSource {
    /// Цвет света
    pub color: Srgb<f32>,
    /// Конкретные характеристики источника
    pub kind: LightSourceKind,
}

impl LightSource {
    /// Вычисляет тень и направление к источнику света.
    /// Возвращает кортеж: (коэффициент_освещенности, вектор_направления_на_свет)
    pub fn shadow(&self, scene: &Scene, point: Vec3, maybe_epsilon: Option<f32>) -> (f32, Vec3) {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        match &self.kind {
            LightSourceKind::Point { position } => {
                // Вектор от точки на поверхности к источнику света
                let mut direction = *position - point;

                // Расстояние до источника света
                let distance = direction.length();

                if distance < epsilon {
                    return (1.0, Vec3::ZERO);
                }

                // Нормализуем вектор направления
                direction = direction / distance;

                // Важнейший нюанс против самозатенения (Shadow Acne):
                // Пускаем теневой луч не из самой точки, а слегка сместив ее старт
                // вдоль направления к свету на величину epsilon.
                let shadow_ray = Ray3d {
                    origin: point + direction * epsilon,
                    direction,
                };

                // Проверяем пересечение теневого луча со сценой
                if let Some(hit) = scene.intersect(&shadow_ray, Some(epsilon)) {
                    // Если на пути луча есть объект и он ближе, чем источник света,
                    // значит точка находится в тени.
                    // Учитываем epsilon, так как мы сместили точку старта луча.
                    if hit.t + epsilon < distance {
                        return (0.0, direction); // Полная тень
                    }
                }

                // Препятствий нет — точка освещена
                (1.0, direction)
            }

            LightSourceKind::Spheric { position, radius } => {
                // Пока возвращаем дефолтное значение для сферического источника
                // (Его можно будет расширить для Distributed Ray Tracing / Soft Shadows)
                (0.0, Vec3::ZERO)
            }
        }
    }
}
