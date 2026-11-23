//! Параметры текстуры для OpenGL

use super::super::limits::Limits;
use super::super::typings::{gl_enum_to_i32, impl_gl_enum_traits};
use super::guards::TextureBindingGuard;
use super::target::{SupportsMipmaps, SupportsTextureParameters, TextureTarget};
use palette::{LinSrgba, Srgba};
use simple_error::{SimpleError, SimpleResult, simple_error};
use std::collections::HashMap;
use std::num::NonZeroU32;
use strum::FromRepr;

/// Макрос, реализующий передачу перечисляемого целого значения в OpenGL.
macro_rules! impl_enum_tex_param {
    ($type:ty, $bound:path, $gl_param:expr) => {
        impl<T: $bound> ApplyTextureParameter<T> for $type {
            fn apply(&self, target: T) {
                unsafe {
                    gl::TexParameteri(
                        target.into() as u32,
                        $gl_param,
                        gl_enum_to_i32(*self as u32),
                    );
                }
            }
        }
    };
}

/// Типаж, описывающий передачу текстурного параметра в OpenGL
pub trait ApplyTextureParameter<T: Into<TextureTarget>> {
    /// Безопасная установка параметра для привязанной текстуры
    fn apply(&self, target: T);
}

/// Режим работы с трафаретом и глубиной текстуры (параметр `TextureParameterName::DepthStencilTextureMode`)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum DepthStencilTextureMode {
    DepthComponent = gl::DEPTH_COMPONENT,
    StencilIndex = gl::STENCIL_INDEX,
}
impl_enum_tex_param!(
    DepthStencilTextureMode,
    SupportsTextureParameters,
    gl::DEPTH_STENCIL_TEXTURE_MODE
);

/// Дополнительный цвет для пикселей на "краях" текстуры
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureBorderColor {
    color: LinSrgba<f32>,
}
impl Default for TextureBorderColor {
    fn default() -> Self {
        Self {
            color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}
impl From<LinSrgba<f32>> for TextureBorderColor {
    fn from(color: LinSrgba<f32>) -> Self {
        Self { color }
    }
}
impl From<Srgba<u8>> for TextureBorderColor {
    fn from(value: Srgba<u8>) -> Self {
        let color: LinSrgba<f32> = value.into_linear();
        Self { color }
    }
}
impl<T: SupportsTextureParameters> ApplyTextureParameter<T> for TextureBorderColor {
    fn apply(&self, target: T) {
        let flat = [
            self.color.red,
            self.color.green,
            self.color.blue,
            self.color.alpha,
        ];
        unsafe {
            gl::TexParameterfv(
                target.into() as u32,
                gl::TEXTURE_BORDER_COLOR,
                flat.as_ptr(),
            );
        }
    }
}

/// Операторы сравнения для текстурного параметра `TextureParameterName::TextureCompareFunc`.
///
/// В общем виде из можно представить в виде формулы <code>result = TextureCompareFunc(r, D<sub>t</sub>)</code>, где:
///   - `r` это текущая интерполированная текстурная координата;
///   - <code>D<sub>t</sub></code> это значение глубины из текущей привязанной текстуры глубины;
///
/// Значение `result` записывается в красный канал.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureCompareFunc {
    /// Если `r` меньше либо равно <code>D<sub>t</sub></code>, то `result` равняется `1.0`.
    /// В ином случае, `result` равняется `0.0`.
    LessThanOrEqual = gl::LEQUAL,
    /// Если `r` больше либо равно <code>D<sub>t</sub></code>, то `result` равняется `1.0`.
    /// В ином случае, `result` равняется `0.0`.
    GreaterThanOrEqual = gl::GEQUAL,
    /// Если `r` меньше <code>D<sub>t</sub></code>, то `result` равняется `1.0`.
    /// В ином случае, `result` равняется `0.0`
    LessThan = gl::LESS,
    /// Если `r` больше <code>D<sub>t</sub></code>, то `result` равняется `1.0`.
    /// В ином случае, `result` равняется `0.0`
    GreaterThan = gl::GREATER,
    /// Если `r` равно <code>D<sub>t</sub></code>, то `result` равняется `1.0`.
    /// В ином случае, `result` равняется `0.0`
    Equal = gl::EQUAL,
    /// Если `r` не равно <code>D<sub>t</sub></code>, то `result` равняется `1.0`.
    /// В ином случае, `result` равняется `0.0`
    NotEqual = gl::NOTEQUAL,
    /// `result` всегда равен `1.0`
    Always = gl::ALWAYS,
    /// `result` всегда равен `0.0`
    Never = gl::NEVER,
}
impl_enum_tex_param!(
    TextureCompareFunc,
    SupportsTextureParameters,
    gl::TEXTURE_COMPARE_FUNC
);

/// Режимы сравнений для текущей текстуры глубины
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureCompareMode {
    CompareRefToTexture = gl::COMPARE_REF_TO_TEXTURE,
    None = gl::NONE,
}
impl_enum_tex_param!(
    TextureCompareMode,
    SupportsTextureParameters,
    gl::TEXTURE_COMPARE_MODE
);

/// Cмещение уровня детализации текстуры
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureLodBias {
    /// Значение смещения
    bias: f32,
}
impl TextureLodBias {
    pub fn try_new(bias: f32, limits: &Limits) -> SimpleResult<TextureLodBias> {
        if !bias.is_finite() {
            return Err(simple_error!("Bias value must be finite float value"));
        }

        let max_texture_lod_bias = limits.get_max_texture_lod_bias();
        if bias.abs() > max_texture_lod_bias {
            return Err(simple_error!(
                "LOD bias {} exceeds GPU limit [-{}, {}]",
                bias,
                max_texture_lod_bias,
                max_texture_lod_bias
            ));
        }

        Ok(TextureLodBias { bias })
    }
}
impl Default for TextureLodBias {
    fn default() -> Self {
        Self { bias: 0.0 }
    }
}
impl<T: SupportsMipmaps> ApplyTextureParameter<T> for TextureLodBias {
    fn apply(&self, target: T) {
        unsafe {
            gl::TexParameterf(target.into() as u32, gl::TEXTURE_LOD_BIAS, self.bias);
        }
    }
}

/// Базовые режимы фильтрации для сжатия текстуры
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum BaseTextureMinFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
}
impl TryFrom<MipmappedTextureMinFilter> for BaseTextureMinFilter {
    type Error = SimpleError;

    fn try_from(value: MipmappedTextureMinFilter) -> Result<Self, Self::Error> {
        match value {
            MipmappedTextureMinFilter::Nearest => Ok(Self::Nearest),
            MipmappedTextureMinFilter::Linear => Ok(Self::Linear),
            _ => Err(simple_error!(
                "Cannot convert {:?} into \"BaseTextureMinFilter\" value",
                value
            )),
        }
    }
}
impl_enum_tex_param!(
    BaseTextureMinFilter,
    SupportsTextureParameters,
    gl::TEXTURE_MIN_FILTER
);

/// Режимы фильтрации для сжатия текстуры, поддерживающей MIP-уровни
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum MipmappedTextureMinFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
    NearestMipmapNearest = gl::NEAREST_MIPMAP_NEAREST,
    LinearMipmapNearest = gl::LINEAR_MIPMAP_NEAREST,
    NearestMipmapLinear = gl::NEAREST_MIPMAP_LINEAR,
    LinearMipmapLinear = gl::LINEAR_MIPMAP_LINEAR,
}
impl From<BaseTextureMinFilter> for MipmappedTextureMinFilter {
    fn from(value: BaseTextureMinFilter) -> Self {
        match value {
            BaseTextureMinFilter::Nearest => Self::Nearest,
            BaseTextureMinFilter::Linear => Self::Linear,
        }
    }
}
impl_enum_tex_param!(
    MipmappedTextureMinFilter,
    SupportsMipmaps,
    gl::TEXTURE_MIN_FILTER
);

/// Режимы фильтрации для растяжения текстуры
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureMagFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
}
impl_enum_tex_param!(
    TextureMagFilter,
    SupportsTextureParameters,
    gl::TEXTURE_MAG_FILTER
);

/// Диапазон детализации текстуры
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureLodRange {
    /// Минимальный уровень детализации текстуры
    min: f32,
    /// Максимальный уровень детализации текстуры
    max: f32,
}
impl TextureLodRange {
    /// Попробовать создать новый диапазон детализации текстуры
    pub fn try_new(min: f32, max: f32) -> SimpleResult<Self> {
        if !min.is_finite() {
            return Err(simple_error!("Minimum value must be finite float value"));
        }
        if !max.is_finite() {
            return Err(simple_error!("Maximum value must be finite float value"));
        }
        if min > max {
            return Err(simple_error!("Minimum value must lower than maximum value"));
        }

        Ok(Self { min, max })
    }
}
impl Default for TextureLodRange {
    fn default() -> Self {
        Self {
            min: -1000.0,
            max: 1000.0,
        }
    }
}
impl<T: SupportsMipmaps> ApplyTextureParameter<T> for TextureLodRange {
    fn apply(&self, target: T) {
        unsafe {
            gl::TexParameterf(target.into() as u32, gl::TEXTURE_MIN_LOD, self.min);
            gl::TexParameterf(target.into() as u32, gl::TEXTURE_MAX_LOD, self.max);
        }
    }
}

/// Диапазон MIP-уровней текстуры
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureMipLevelRange {
    /// Минимальный MIP-уровень текстуры
    min: i32,
    /// Максимальный MIP-уровень текстуры
    max: i32,
}
impl TextureMipLevelRange {
    /// Попробовать создать новый диапазон MIP-уровней текстуры
    pub fn try_new(min: u32, max: u32) -> SimpleResult<TextureMipLevelRange> {
        if min > max {
            return Err(simple_error!("Minimum value must not exceed maximum value"));
        }

        let min = i32::try_from(min).map_err(|err| {
            simple_error!(
                "Minimum value cannot be safely transformed into \"i32\": {}",
                err.to_string()
            )
        })?;
        let max = i32::try_from(max).map_err(|err| {
            simple_error!(
                "Maximum value cannot be safely transformed into \"i32\": {}",
                err.to_string()
            )
        })?;

        Ok(TextureMipLevelRange { min, max })
    }
}
impl Default for TextureMipLevelRange {
    fn default() -> Self {
        Self { min: 0, max: 1000 }
    }
}
impl<T: SupportsMipmaps> ApplyTextureParameter<T> for TextureMipLevelRange {
    fn apply(&self, target: T) {
        unsafe {
            gl::TexParameteri(target.into() as u32, gl::TEXTURE_BASE_LEVEL, self.min);
            gl::TexParameteri(target.into() as u32, gl::TEXTURE_MAX_LEVEL, self.max);
        }
    }
}

/// Компонента цвета, которая будет применена к компоненте цвета текселя перед его отправкой в шейдер
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureSwizzleSource {
    /// Красный канал
    Red = gl::RED,
    /// Зеленый канал
    Green = gl::GREEN,
    /// Синий канал
    Blue = gl::BLUE,
    /// Альфа-канал
    Alpha = gl::ALPHA,
    /// Всегда ноль
    Zero = gl::ZERO,
    /// Всегда единица
    One = gl::ONE,
}

/// Цветовая компонента текселя, в которуб будет записана компонента цвета
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureSwizzleTarget {
    /// Красный канал
    Red = gl::TEXTURE_SWIZZLE_R,
    /// Зеленый канал
    Green = gl::TEXTURE_SWIZZLE_G,
    /// Синий канал
    Blue = gl::TEXTURE_SWIZZLE_B,
    /// Альфа-канал
    Alpha = gl::TEXTURE_SWIZZLE_A,
}

/// Правила переназначения каналов текстуры, определяемые до загрузки текстуры во фрагментный шейдер.
/// Полезны для обработки некоторых устаревших форматов и различных эффектов (например, черно-белая окраска).
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TextureSwizzle {
    /// Текстурная координата
    coordinate: TextureSwizzleTarget,
    /// Цветовая компонента
    component: TextureSwizzleSource,
}
impl TextureSwizzle {
    pub fn new(coordinate: TextureSwizzleTarget, component: TextureSwizzleSource) -> Self {
        Self {
            coordinate,
            component,
        }
    }
}
impl<T: SupportsTextureParameters> ApplyTextureParameter<T> for TextureSwizzle {
    fn apply(&self, target: T) {
        unsafe {
            gl::TexParameteri(
                target.into() as u32,
                self.coordinate as u32,
                gl_enum_to_i32(self.component as u32),
            );
        }
    }
}

/// Набор правил переназначения каналов текстуры.
/// В отличие от `TextureSwizzle`, позволяет задать их за один вызов OpenGL.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TextureSwizzleMask {
    /// Набор
    mask: [TextureSwizzleSource; 4],
}
impl TextureSwizzleMask {
    /// Задать правила переназначения канала текстуры
    pub fn set_swizzle(&mut self, target: TextureSwizzleTarget, source: TextureSwizzleSource) {
        self.mask[Self::get_target_index(target)] = source;
    }

    /// Получить индекс канала текстуры в массиве
    const fn get_target_index(target: TextureSwizzleTarget) -> usize {
        match target {
            TextureSwizzleTarget::Red => 0,
            TextureSwizzleTarget::Green => 1,
            TextureSwizzleTarget::Blue => 2,
            TextureSwizzleTarget::Alpha => 3,
        }
    }
}
impl Default for TextureSwizzleMask {
    fn default() -> Self {
        let mask = [
            TextureSwizzleSource::Red,
            TextureSwizzleSource::Green,
            TextureSwizzleSource::Blue,
            TextureSwizzleSource::Alpha,
        ];
        Self { mask }
    }
}
impl From<HashMap<TextureSwizzleTarget, TextureSwizzleSource>> for TextureSwizzleMask {
    fn from(mask: HashMap<TextureSwizzleTarget, TextureSwizzleSource>) -> Self {
        let mut result = Self::default();
        for (target, source) in mask {
            result.set_swizzle(target, source);
        }
        result
    }
}
impl<T: SupportsTextureParameters> ApplyTextureParameter<T> for TextureSwizzleMask {
    fn apply(&self, target: T) {
        let flat = [
            gl_enum_to_i32(self.mask[0] as u32),
            gl_enum_to_i32(self.mask[1] as u32),
            gl_enum_to_i32(self.mask[2] as u32),
            gl_enum_to_i32(self.mask[3] as u32),
        ];

        unsafe {
            gl::TexParameteriv(
                target.into() as u32,
                gl::TEXTURE_SWIZZLE_RGBA,
                flat.as_ptr(),
            );
        }
    }
}

/// Режим наложения текстурных координат для прямоугольной текстуры без MIP-уровней (`gl::TEXTURE_RECTANGLE`)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum RectangleTextureWrapMode {
    /// Текстура обрезается по краю
    ClampToEdge = gl::CLAMP_TO_EDGE,
    /// Текстура обрезается по краю, а внешняя область заполняется цветом границы
    ClampToBorder = gl::CLAMP_TO_BORDER,
    /// Текстура обрезается по краю, а внешняя область зеркально отражается
    MirrorClampToEdge = gl::MIRROR_CLAMP_TO_EDGE,
}
impl TryFrom<MipmappedTextureWrapMode> for RectangleTextureWrapMode {
    type Error = SimpleError;

    fn try_from(value: MipmappedTextureWrapMode) -> SimpleResult<RectangleTextureWrapMode> {
        match value {
            MipmappedTextureWrapMode::ClampToEdge => Ok(Self::ClampToEdge),
            MipmappedTextureWrapMode::ClampToBorder => Ok(Self::ClampToBorder),
            MipmappedTextureWrapMode::MirrorClampToEdge => Ok(Self::MirrorClampToEdge),
            _ => Err(simple_error!(
                "Cannot convert {:?} to \"RectangleTextureWrapMode\" value",
                value
            )),
        }
    }
}

/// Режим наложения текстурных координат для текстур с MIP-уровнями
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum MipmappedTextureWrapMode {
    /// Текстура бесконечно повторяется
    Repeat = gl::REPEAT,
    /// Текстура повторяется, но на каждом нечетном интервале зеркально отражается
    MirroredRepeat = gl::MIRRORED_REPEAT,
    /// Текстура обрезается по краю
    ClampToEdge = gl::CLAMP_TO_EDGE,
    /// Текстура обрезается по краю, а внешняя область заполняется цветом границы
    ClampToBorder = gl::CLAMP_TO_BORDER,
    /// Текстура обрезается по краю, а внешняя область зеркально отражается
    MirrorClampToEdge = gl::MIRROR_CLAMP_TO_EDGE,
}
impl From<RectangleTextureWrapMode> for MipmappedTextureWrapMode {
    fn from(value: RectangleTextureWrapMode) -> Self {
        match value {
            RectangleTextureWrapMode::ClampToEdge => Self::ClampToEdge,
            RectangleTextureWrapMode::ClampToBorder => Self::ClampToBorder,
            RectangleTextureWrapMode::MirrorClampToEdge => Self::MirrorClampToEdge,
        }
    }
}

/// Ось текстурных координат для наложения
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum TextureWrapAxis {
    /// Ось `S` (горизонталь)
    S = gl::TEXTURE_WRAP_S,
    /// Ось `T` (вертикаль)
    T = gl::TEXTURE_WRAP_T,
    /// Ось `R` (глубина)
    R = gl::TEXTURE_WRAP_R,
}

/// Правила наложения текстурных координат для прямоугольных текстур без MIP-уровней (`gl::TEXTURE_RECTANGLE`)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct RectangleTextureWrap {
    /// Ось текстурных координат
    axis: TextureWrapAxis,
    /// Режим наложения текстурных координат
    mode: RectangleTextureWrapMode,
}
impl RectangleTextureWrap {
    /// Создать новые правила наложения текстурных координат
    pub fn new(axis: TextureWrapAxis, mode: RectangleTextureWrapMode) -> Self {
        Self { axis, mode }
    }
}
impl<T: SupportsTextureParameters> ApplyTextureParameter<T> for RectangleTextureWrap {
    fn apply(&self, target: T) {
        unsafe {
            gl::TexParameteri(
                target.into() as u32,
                self.axis as u32,
                gl_enum_to_i32(self.mode as u32),
            );
        }
    }
}

/// Правила наложения текстурных координат для текстур с MIP-уровнями
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MipmappedTextureWrap {
    /// Ось текстурных координат
    axis: TextureWrapAxis,
    /// Режим наложения текстурных координат
    mode: MipmappedTextureWrapMode,
}
impl MipmappedTextureWrap {
    /// Создать новые правила наложения текстурных координат
    pub fn new(axis: TextureWrapAxis, mode: MipmappedTextureWrapMode) -> Self {
        Self { axis, mode }
    }
}
impl<T: SupportsMipmaps> ApplyTextureParameter<T> for MipmappedTextureWrap {
    fn apply(&self, target: T) {
        unsafe {
            gl::TexParameteri(
                target.into() as u32,
                self.axis as u32,
                gl_enum_to_i32(self.mode as u32),
            );
        }
    }
}

impl_gl_enum_traits!(
    DepthStencilTextureMode,
    TextureCompareFunc,
    TextureCompareMode,
    BaseTextureMinFilter,
    MipmappedTextureMinFilter,
    TextureMagFilter,
    TextureSwizzleSource,
    TextureSwizzleTarget,
    RectangleTextureWrapMode,
    MipmappedTextureWrapMode,
    TextureWrapAxis,
);

/// "Мастер" базовых настроек текстуры
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct BaseTextureParametersWizard<T: SupportsTextureParameters> {
    /// Назначение текстуры
    target: T,
    /// Идентификатор текстуры в OpenGL
    opengl_id: NonZeroU32,
}
impl<T: SupportsTextureParameters> BaseTextureParametersWizard<T> {
    /// Создать новый мастер настройки текстуры
    pub fn new(target: T, opengl_id: NonZeroU32) -> Self {
        Self { target, opengl_id }
    }

    /// Применить параметр к текстуре
    pub fn apply<P: ApplyTextureParameter<T>>(&self, parameter: P) {
        // Делаем текстуру текущей (на время выполнения метода)
        let _binding_guard = TextureBindingGuard::new(self.target.into(), self.opengl_id);
        parameter.apply(self.target);
    }
}

/// "Мастер" настроек текстуры с MIP-уровнями
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct MipmappedTextureParametersWizard<T: SupportsMipmaps> {
    /// Назначение текстуры
    target: T,
    /// Идентификатор текстуры в OpenGL
    opengl_id: NonZeroU32,
}
impl<T: SupportsMipmaps> MipmappedTextureParametersWizard<T> {
    /// Создать новый мастер настройки текстуры
    pub fn new(target: T, opengl_id: NonZeroU32) -> Self {
        Self { target, opengl_id }
    }

    /// Применить параметр к текстуре
    pub fn apply<P: ApplyTextureParameter<T>>(&self, parameter: P) {
        // Делаем текстуру текущей (на время выполнения метода)
        let _binding_guard = TextureBindingGuard::new(self.target.into(), self.opengl_id);
        parameter.apply(self.target);
    }
}
