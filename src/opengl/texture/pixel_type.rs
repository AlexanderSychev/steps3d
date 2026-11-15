//! Подсистема вывода типов данных OpenGL для текстур из типов данных Rust

use std::marker::PhantomData;

/// Типаж, определяющий упакованную раскладку бит для конкретного скаляра.
/// Если нет реализации этого типажа — код не скомпилируется.
pub trait PixelLayout<S> {
    /// Константа соответствующего типа OpenGL
    const OPENGL_TYPE: u32;
}

// ------------------------------------
// - Маркерные типы для раскладок бит -
// ------------------------------------

/// Без раскладки
pub struct NoLayout;

/// Раскладка "3+3+2":
///
/// | 3 бита  | 3 бита  | 2 бита |
/// |---------|---------|--------|
/// | Красный | Зеленый | Синий  |
pub struct Layout332;

/// Инвертированная раскладка "3+3+2":
///
/// | 2 бита | 3 бита  | 3 бита  |
/// |--------|---------|---------|
/// | Синий  | Зеленый | Красный |
pub struct Layout233Rev;

/// Раскладка "5+6+5":
///
/// | 5 бит   | 6 бит   | 5 бит  |
/// |---------|---------|--------|
/// | Красный | Зеленый | Синий  |
pub struct Layout565;

/// Инвертированная раскладка "5+6+5":
///
/// | 5 бит   | 6 бит   | 5 бит    |
/// |---------|---------|----------|
/// |  Синий  | Зеленый | Красный  |
pub struct Layout565Rev;

/// Раскладка "4+4+4+4":
///
/// | 4 бита  | 4 бита  | 4 бита | 4 бита |
/// |---------|---------|--------|--------|
/// | Красный | Зеленый | Синий  | Альфа  |
pub struct Layout4444;

/// Инвертированная раскладка "4+4+4+4":
///
/// | 4 бита | 4 бита | 4 бита  | 4 бита  |
/// |--------|--------|---------|---------|
/// | Альфа  | Синий  | Зеленый | Красный |
pub struct Layout4444Rev;

/// Раскладка "5+5+5+1":
///
/// | 5 бит   | 5 бит   | 5 бит  | 1 бит |
/// |---------|---------|--------|-------|
/// | Красный | Зеленый | Синий  | Альфа |
pub struct Layout5551;

/// Инвертированная раскладка "5+5+5+1":
///
/// | 1 бит | 5 бит  | 5 бит   | 5 бит   |
/// |-------|--------|---------|---------|
/// | Альфа | Синий  | Зеленый | Красный |
pub struct Layout5551Rev;

/// Раскладка "8+8+8+8":
///
/// | 8 бит   | 8 бит   | 8 бит  | 8 бит |
/// |---------|---------|--------|-------|
/// | Красный | Зеленый | Синий  | Альфа |
pub struct Layout8888;

/// Инвертированная раскладка "8+8+8+8":
///
/// | 8 бит | 8 бит  | 8 бит   | 8 бит   |
/// |-------|--------|---------|---------|
/// | Альфа | Синий  | Зеленый | Красный |
pub struct Layout8888Rev;

/// Раскладка "10+10+10+2":
///
/// | 10 бит  | 10 бит  | 10 бит | 2 бита |
/// |---------|---------|--------|--------|
/// | Красный | Зеленый | Синий  | Альфа  |
pub struct Layout1010102;

/// Инвертированная раскладка "10+10+10+2":
///
/// | 2 бита | 10 бит | 10 бит  | 10 бит  |
/// |--------|--------|---------|---------|
/// | Альфа  | Синий  | Зеленый | Красный |
pub struct Layout1010102Rev;

// ----------------------------------------------------------
// - Валидация на этапе компиляции через реализации трейтов -
// ----------------------------------------------------------

// Валидация для скалярных типов с "упаковкой"
impl PixelLayout<i8> for NoLayout {
    const OPENGL_TYPE: u32 = gl::BYTE;
}
impl PixelLayout<u8> for NoLayout {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_BYTE;
}
impl PixelLayout<i16> for NoLayout {
    const OPENGL_TYPE: u32 = gl::SHORT;
}
impl PixelLayout<u16> for NoLayout {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT;
}
impl PixelLayout<i32> for NoLayout {
    const OPENGL_TYPE: u32 = gl::INT;
}
impl PixelLayout<u32> for NoLayout {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_INT;
}
impl PixelLayout<u8> for Layout332 {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_BYTE_3_3_2;
}
impl PixelLayout<u8> for Layout233Rev {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_BYTE_2_3_3_REV;
}
impl PixelLayout<u16> for Layout565 {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT_5_6_5;
}
impl PixelLayout<u16> for Layout565Rev {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT_5_6_5_REV;
}
impl PixelLayout<u16> for Layout4444 {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT_4_4_4_4;
}
impl PixelLayout<u16> for Layout4444Rev {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT_4_4_4_4_REV;
}
impl PixelLayout<u16> for Layout5551 {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT_5_5_5_1;
}
impl PixelLayout<u16> for Layout5551Rev {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_SHORT_1_5_5_5_REV;
}
impl PixelLayout<u32> for Layout8888 {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_INT_8_8_8_8;
}
impl PixelLayout<u32> for Layout8888Rev {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_INT_8_8_8_8_REV;
}
impl PixelLayout<u32> for Layout1010102 {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_INT_10_10_10_2;
}
impl PixelLayout<u32> for Layout1010102Rev {
    const OPENGL_TYPE: u32 = gl::UNSIGNED_INT_2_10_10_10_REV;
}

/// Ортогональный и безопасный тип пикселя.
/// `S` - базовый скаляр (u8, u16...).
/// `L` - раскладка бит (по умолчанию без упаковки `NoLayout`).
pub struct PixelType<Scalar, Layout = NoLayout> {
    _scalar: PhantomData<Scalar>,
    _layout: PhantomData<Layout>,
}

// Ограничение накладываем только там, где нам нужна константа или создание типа
impl<Scalar, Layout: PixelLayout<Scalar>> PixelType<Scalar, Layout> {
    pub const fn new() -> Self {
        Self {
            _scalar: PhantomData,
            _layout: PhantomData,
        }
    }

    /// Выести тип данных OpenGL, соответствующий этому типу пикселя
    pub const fn to_gl(&self) -> u32 {
        Layout::OPENGL_TYPE
    }
}
