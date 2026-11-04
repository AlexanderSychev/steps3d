use glam::Vec3;
use palette::Srgb;

/// Основные оптические свойства отражающей среды.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpticalMedium {
    /// Коэффициент преломления ("nRefr" в оригинальном коде на C++)
    pub refraction: f32,
    /// Коэффициент поглощения ("extinct" в оригинальном коде на C++)
    pub absorption: f32,
}

impl OpticalMedium {
    /// Создает новую оптическую среду
    pub const fn new(refraction: f32, absorption: f32) -> Self {
        Self {
            refraction,
            absorption,
        }
    }
}

impl Default for OpticalMedium {
    /// По умолчанию возвращает свойства вакуума/воздуха
    fn default() -> Self {
        AIR
    }
}

/// Отражающие свойства воздуха
pub const AIR: OpticalMedium = OpticalMedium {
    refraction: 1.0,
    absorption: 0.0,
};

/// Отражающие свойства стекла
pub const GLASS: OpticalMedium = OpticalMedium {
    refraction: 1.5,
    absorption: 0.0,
};

/// Оптические свойства материала поверхности (модель Блинна-Фонга + Рейтрейсинг)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialProps {
    /// Собственный цвет материала (диффузный)
    pub color: Srgb<f32>,
    /// Собственная светимость (для источников света)
    pub emission: Srgb<f32>,
    /// Экспонента блеска для зеркального блика (мощность блика)
    pub shininess: f32,

    /// Коэффициент фонового (ambient) освещения или <code>k<sub>a</sub></code>
    pub ambient_coeff: f32,
    /// Коэффициент диффузного (diffuse) рассеивания или <code>k<sub>d</sub></code>
    pub diffuse_coeff: f32,
    /// Коэффициент зеркального (specular) блика или <code>k<sub>s</sub></code>
    pub specular_coeff: f32,
    /// Коэффициент идеального зеркального отражения (reflection) или <code>k<sub>r</sub></code>
    pub reflection_coeff: f32,
    /// Коэффициент светопропускания / прозрачности (transmission) или <code>k<sub>t</sub></code>
    pub transmission_coeff: f32,

    /// Оптическая среда внутри объекта (индекс преломления и поглощения)
    pub medium: OpticalMedium,
}

/// Параметры освещаемой поверхности в точке
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceData {
    /// Позиция точки пересечения в мировых координатах
    pub position: Vec3,
    /// Нормаль к поверхности в этой точке (обычно нормализованная)
    pub normal: Vec3,
    /// Свойства материала в данной конкретной точке
    pub material: MaterialProps,
}
