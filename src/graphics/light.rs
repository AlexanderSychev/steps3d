//! Источники освещения

use palette::Srgb;
use simple_error::{SimpleResult, simple_error};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// Яркость по-умолчанию
pub const DEFAULT_INTENSITY: f32 = 1.0;

/// Радиус затухания по-умолчанию
pub const DEFAULT_RANGE: f32 = 10.0;

/// Цвет по-умолчанию
pub const DEFAULT_COLOR: Srgb<u8> = Srgb::new(255, 255, 255);

/// Точечный источник света
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    /// Яркость
    intensity: f32,
    /// Радиус затухания
    range: f32,
    /// Цвет
    color: Srgb<u8>,
}
impl Default for PointLight {
    fn default() -> Self {
        Self {
            intensity: DEFAULT_INTENSITY,
            range: DEFAULT_RANGE,
            color: DEFAULT_COLOR,
        }
    }
}
impl PointLight {
    /// Создать источник
    fn try_new(intensity: f32, range: f32, color: Srgb<u8>) -> SimpleResult<Self> {
        if !intensity.is_finite() || intensity <= 0.0 {
            return Err(simple_error!("Intensity value must be finite and positive"));
        }
        if !range.is_finite() || range <= 0.0 {
            return Err(simple_error!("Range value must be finite and positive"));
        }
        Ok(Self {
            intensity,
            range,
            color,
        })
    }

    /// Получить яркость источника
    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    /// Изменить яркость источника
    pub fn set_intensity(&mut self, intensity: f32) -> SimpleResult<()> {
        if intensity.is_finite() && intensity > 0.0 {
            self.intensity = intensity;
            Ok(())
        } else {
            Err(simple_error!("Intensity value must be finite and positive"))
        }
    }

    /// Получить радиус затухания
    pub fn get_range(&self) -> f32 {
        self.range
    }

    /// Изменить радиус затухания
    pub fn set_range(&mut self, range: f32) -> SimpleResult<()> {
        if range.is_finite() && range > 0.0 {
            self.range = range;
            Ok(())
        } else {
            Err(simple_error!("Range value must be finite and positive"))
        }
    }

    /// Получить цвет источника для чтения
    pub fn get_color(&self) -> Srgb<u8> {
        self.color
    }

    /// Получить цвет источника для записи
    pub fn get_color_mut(&mut self) -> &mut Srgb<u8> {
        &mut self.color
    }

    /// Задать цвет источника
    pub fn set_color(&mut self, color: Srgb<u8>) {
        self.color = color;
    }
}

/// Удаленный и направленный источник света
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalLight {
    /// Яркость
    intensity: f32,
    /// Цвет
    color: Srgb<u8>,
}
impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            intensity: DEFAULT_INTENSITY,
            color: DEFAULT_COLOR,
        }
    }
}
impl DirectionalLight {
    /// Создать источник
    fn try_new(intensity: f32, color: Srgb<u8>) -> SimpleResult<Self> {
        if !intensity.is_finite() || intensity <= 0.0 {
            return Err(simple_error!("Intensity value must be finite and positive"));
        }
        Ok(Self { intensity, color })
    }

    /// Получить яркость источника
    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    /// Изменить яркость источника
    pub fn set_intensity(&mut self, intensity: f32) -> SimpleResult<()> {
        if intensity.is_finite() && intensity > 0.0 {
            self.intensity = intensity;
            Ok(())
        } else {
            Err(simple_error!("Intensity value must be finite and positive"))
        }
    }

    /// Получить цвет источника для чтения
    pub fn get_color(&self) -> Srgb<u8> {
        self.color
    }

    /// Получить цвет источника для записи
    pub fn get_color_mut(&mut self) -> &mut Srgb<u8> {
        &mut self.color
    }

    /// Задать цвет источника
    pub fn set_color(&mut self, color: Srgb<u8>) {
        self.color = color;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotLight {
    /// Яркость
    intensity: f32,
    /// **Полный** угол внутреннего "конуса" (в радианах)
    inner_angle: f32,
    /// **Полный** угол внешнего "конуса" (в радианах)
    outer_angle: f32,
    /// Радиус затухания
    range: f32,
    /// Цвет
    color: Srgb<u8>,
}
impl Default for SpotLight {
    fn default() -> Self {
        Self {
            intensity: DEFAULT_INTENSITY,
            range: DEFAULT_RANGE,
            inner_angle: FRAC_PI_4,
            outer_angle: FRAC_PI_2,
            color: Srgb::new(255, 255, 255),
        }
    }
}
impl SpotLight {
    /// Создать источник
    fn try_new(
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
        range: f32,
        color: Srgb<u8>,
    ) -> SimpleResult<Self> {
        Self::validate_finite_and_positive(intensity, "Intensity")?;
        Self::validate_finite_and_positive(range, "Range")?;
        Self::validate_angles(inner_angle, outer_angle)?;

        Ok(Self {
            intensity,
            inner_angle,
            outer_angle,
            range,
            color,
        })
    }

    /// Получить яркость источника
    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    /// Изменить яркость источника
    pub fn set_intensity(&mut self, intensity: f32) -> SimpleResult<()> {
        Self::validate_finite_and_positive(intensity, "Intensity")?;
        self.intensity = intensity;
        Ok(())
    }

    /// Получить радиус затухания
    pub fn get_range(&self) -> f32 {
        self.range
    }

    /// Изменить радиус затухания
    pub fn set_range(&mut self, range: f32) -> SimpleResult<()> {
        Self::validate_finite_and_positive(range, "Range")?;
        self.range = range;
        Ok(())
    }

    /// Получить внутренний угол источника
    pub fn get_inner_angle(&self) -> f32 {
        self.inner_angle
    }

    /// Задать внутренний угол источника
    pub fn set_inner_angle(&mut self, inner_angle: f32) -> SimpleResult<()> {
        Self::validate_angles(inner_angle, self.outer_angle)?;
        self.inner_angle = inner_angle;
        Ok(())
    }

    /// Получить внешний угол источника
    pub fn get_outer_angle(&self) -> f32 {
        self.outer_angle
    }

    /// Задать внешний угол источника
    pub fn set_outer_angle(&mut self, outer_angle: f32) -> SimpleResult<()> {
        Self::validate_angles(self.inner_angle, outer_angle)?;
        self.outer_angle = outer_angle;
        Ok(())
    }

    /// Получить цвет источника для чтения
    pub fn get_color(&self) -> Srgb<u8> {
        self.color
    }

    /// Получить цвет источника для записи
    pub fn get_color_mut(&mut self) -> &mut Srgb<u8> {
        &mut self.color
    }

    /// Задать цвет источника
    pub fn set_color(&mut self, color: Srgb<u8>) {
        self.color = color;
    }

    /// Провалидировать углы:
    ///   - Оба угла должны быть конечными;
    ///   - Оба угла должны быть положительными;
    ///   - Оба угла должны быть в диапазоне (0, π);
    ///   - Внутренний угол должен быть меньше внешнего;
    fn validate_angles(inner_angle: f32, outer_angle: f32) -> SimpleResult<()> {
        if !inner_angle.is_finite() || inner_angle <= 0.0 || inner_angle >= PI {
            return Err(simple_error!(
                "Inner angle must be finite and in range (0, π)"
            ));
        }
        if !outer_angle.is_finite() || outer_angle <= 0.0 || outer_angle >= PI {
            return Err(simple_error!(
                "Outer angle must be finite and in range (0, π)"
            ));
        }
        if inner_angle >= outer_angle {
            return Err(simple_error!("Inner angle must be lower than outer angle"));
        }

        Ok(())
    }

    /// Провалидировать дробное значение: оно должно быть конечным и положительным
    fn validate_finite_and_positive(value: f32, value_name: &'static str) -> SimpleResult<()> {
        if !value.is_finite() || value <= 0.0 {
            Err(simple_error!("{} must be finite and positive", value_name))
        } else {
            Ok(())
        }
    }
}

/// Источник света
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightSource {
    /// Точечный источник цвета
    Point(PointLight),
    /// Направленный источник цвета
    Directional(DirectionalLight),
    /// Прожектор
    Spot(SpotLight),
}
impl Default for LightSource {
    fn default() -> Self {
        Self::Point(PointLight::default())
    }
}
impl LightSource {
    /// Создать точечный источник света
    pub fn try_new_point(intensity: f32, range: f32, color: Srgb<u8>) -> SimpleResult<Self> {
        PointLight::try_new(intensity, range, color).map(Self::Point)
    }

    /// Создать направленный источник света
    pub fn try_new_directional(intensity: f32, color: Srgb<u8>) -> SimpleResult<Self> {
        DirectionalLight::try_new(intensity, color).map(Self::Directional)
    }

    /// Создать прожектор
    pub fn try_new_spot(
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
        range: f32,
        color: Srgb<u8>,
    ) -> SimpleResult<Self> {
        SpotLight::try_new(intensity, inner_angle, outer_angle, range, color).map(Self::Spot)
    }
}
