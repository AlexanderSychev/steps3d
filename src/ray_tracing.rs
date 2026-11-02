//! Пример трассировки лучей из учебника.
//! Возможно, потом часть логики уйдет в модуль "light".

use glam::Vec3;
use palette::Srgb;
use super::geometry::DEFAULT_GEOMETRY_EPSILON;
use super::geometry::Ray3d;

const MAX_DEPTH: i32 = 7;

/// Интерфейс геометрического объекта сцены
pub trait Object {
    /// Проверить, пересекает ли луч объект
    fn intersect(&self, ray: &Ray3d) -> Option<HitData>;
    /// Получить данные поверхности
    fn get_surface(&self, hit_data: &HitData, surface_data: &SurfaceData);
}

/// Основные свойства отражающей среды.
/// PS: Название взято из книги, но мне кажется каким-то совсем неинформативным, может лучше назвать "LightProps"?
#[derive(Debug, Clone, Copy)]
pub struct Medium {
    /// Коэффициент преломления ("nRefr" в оригинальном коде на C++)
    pub refraction: f32,
    /// Коэффициент поглощения ("extinct" в оригинальном коде на C++)
    pub absorption: f32,
}

/// Параметры освещаемой поверхности в точке
#[derive(Debug, Clone, Copy)]
pub struct SurfaceData {
    /// Позиция точки на поверхности
    pub position: Vec3,
    /// Нормаль к поверхности в этой точке
    pub normal: Vec3,
    /// Цвет в точке
    pub color: Srgb<f32>,
    /// Собственная светимость в точке
    pub emission: Srgb<f32>,
    /// Степень для бликовой модели ("p" в оригинальном коде на C++)
    pub shininess: f32,
    /// Характеризующий коэффициент <code>k<sub>a</sub></code> (нужно более "читаемое название")
    pub ka: f32,
    /// Характеризующий коэффициент <code>k<sub>d</sub></code> (нужно более "читаемое название")
    pub kd: f32,
    /// Характеризующий коэффициент <code>k<sub>s</sub></code> (нужно более "читаемое название")
    pub ks: f32,
    /// Характеризующий коэффициент <code>k<sub>r</sub></code> (нужно более "читаемое название")
    pub kr: f32,
    /// Характеризующий коэффициент <code>k<sub>t</sub></code> (нужно более "читаемое название")
    pub kt: f32,
    /// Световые характеристики материала
    pub medium: Medium,
}

/// Кэшируемые данные пересечений
#[derive(Debug, Clone, Copy)]
pub enum HitCache {
    Sphere {
        t1: f32,
        t2: f32,
    },
    Plane {
        vd: f32,
    },
    Triangle,
    Box,
}

/// Информация о пересечении с кэшированными данными
#[derive(Debug, Clone, Copy)]
pub struct HitData {
    /// Точка пересечения
    pub position: Vec3,
    /// Параметр вдоль луча
    pub t: f32,
    /// Нужно ли перевернуть нормаль
    pub invert_normal: bool,
    /// Кэшированные значения
    pub cache: HitCache,
}

/// Сцена
#[derive(Default)]
pub struct Scene {
    /// Источники света
    lights: Vec<LightSource>,
    /// Объекты на сцене
    objects: Vec<Box<dyn Object>>,
    /// Цвет фонового освещения
    ambient: Srgb<f32>,
}

impl Scene {
    /// Добавить объект на сцену
    pub fn add_object(&mut self, object: Box<dyn Object>) {
        self.objects.push(object);
    }

    /// Добавить источник освещения на сцену
    pub fn add_light(&mut self, ray: &Ray3d, light: LightSource) {
        self.lights.push(light);
    }

    pub fn intersect(&self, r: &Ray3d) -> Option<HitData> {
        self.objects
            .iter()
            // Вызываем intersect для каждого объекта.
            // Каждый объект сам возвращает Option<HitData> в случае успеха
            .filter_map(|obj| obj.intersect(r))
            // Ищем пересечение с минимальной дистанцией t
            .min_by(|hit_a, hit_b| {
                hit_a.t
                    .partial_cmp(&hit_b.t)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn trace(&self, cur_medium: Medium, weight: f32, depth: i32, r: &Ray3d) -> Srgb<f32> {
        if depth > MAX_DEPTH {
            return self.get_background(r);
        }

        // Просто проверяем через if let
        if let Some(hit) = self.intersect(r) {
            // Извлекаем свойства поверхности у того объекта, в который попали
            let mut surface = hit.object.get_surface(&hit);

            // Переходим к расчету освещения (shade)
            let mut color = self.shade(cur_medium, weight, depth, r, &hit, &mut surface);

            if cur_medium.extinct > 1e-4 {
                color = color * f32::exp(-hit.t * cur_medium.extinct);
            }
            color
        } else {
            self.get_background(r)
        }
    }

    pub fn get_background(&self, r: &Ray3d) -> Srgb<f32> {
        Srgb::new(0.0, 0.5, 0.5)
    }
}

/// Конкретные виды источников света
#[derive(Debug, Clone, Copy)]
pub enum LightSourceKind {
    /// Точечный источник света
    Point {
        /// Позиция источника в пространстве
        position: Vec3
    },
    /// Сферический источник света
    Spheric {
        /// Позиция источника в пространстве
        position: Vec3,
        /// Радиус сферы
        radius: f32,
    }
}

/// Источник света
#[derive(Debug, Clone, Copy)]
pub struct LightSource {
    /// Цвет света
    pub color: Srgb,
    /// Конкретные характеристики источника
    pub kind: LightSourceKind,
}

impl LightSource {
    pub fn shadow(&self, point: Vec3, maybe_epsilon: Option<f32>) -> (f32, Vec3) {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        match &self.kind {
            LightSourceKind::Point { position } => {
                let direction = position - point;
                let distance = direction.length();
            },
            LightSourceKind::Spheric { position, radius } => (0.0, Vec3::ZERO)
        }
    }
}
