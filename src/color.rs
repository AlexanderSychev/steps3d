//! Утилиты для связи библиотек `palette` (цвета) и `glam` (векторная алгебра),
//! а также простой writer для 24-битного TGA-изображения.
//!
//! В учебных примерах TGA используется как простой способ сохранить результат
//! растеризации. Для «боевого» кода лучше взять готовый крейт для изображений.

use glam::{Vec3, Vec4};
use palette::{IntoColor, LinSrgb, Srgb};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// Конвертирует цвет в линейный RGB `f32` и возвращает `glam::Vec3`.
///
/// Для 8-битных sRGB-цветов используйте [`srgb_u8_to_linear`]: `palette`
/// не реализует прямое преобразование `Srgb<u8>` в `LinSrgb<f32>` через
/// `IntoColor`.
pub trait IntoGlam {
    /// Конвертирует цвет в линейный RGB `f32`.
    fn to_glam3(&self) -> Vec3;

    /// Конвертирует цвет в линейный RGB `f32` с указанной альфой.
    ///
    /// Альфа ограничивается диапазоном `0..=1`.
    fn to_glam4(&self, alpha: f32) -> Vec4 {
        self.to_glam3().extend(alpha.clamp(0.0, 1.0))
    }
}

impl<C> IntoGlam for C
where
    C: IntoColor<LinSrgb<f32>> + Clone,
{
    fn to_glam3(&self) -> Vec3 {
        let linear: LinSrgb<f32> = self.clone().into_color();
        let (red, green, blue) = linear.into_components();

        Vec3::new(red, green, blue)
    }
}

/// Переводит 8-битный sRGB-цвет в линейный RGB `f32`.
///
/// Это два отдельных шага: `u8 -> f32` в пространстве sRGB, затем снятие
/// гамма-кривой sRGB.
pub fn srgb_u8_to_linear(color: Srgb<u8>) -> LinSrgb<f32> {
    Srgb::<f32>::from_format(color).into_linear()
}

/// Простое изображение в формате TGA: sRGB, 8 бит на канал, 24 бита на пиксель.
///
/// Строки хранятся сверху вниз, а в заголовке выставляется признак top-left
/// origin. Это отличается от C++-версии из книги, которая пишет строки снизу
/// вверх с bottom-left origin, но визуальный результат одинаковый.
#[derive(Debug, Clone, PartialEq)]
pub struct TgaImage {
    /// Пиксели в виде плоского списка; строка за строкой, сверху вниз.
    pixels: Vec<Srgb<u8>>,
    /// Ширина в пикселях.
    width: usize,
    /// Высота в пикселях.
    height: usize,
}

impl TgaImage {
    /// Создать изображение заданного размера, заполненное чёрным цветом.
    ///
    /// Паникует, если размеры невалидны. Если нужна обработка ошибки —
    /// используйте [`TgaImage::try_new`].
    pub fn new(width: usize, height: usize) -> Self {
        Self::try_new(width, height).expect("invalid TGA image dimensions")
    }

    /// Создать изображение с проверкой размеров.
    ///
    /// TGA хранит ширину и высоту в 16-битных полях, поэтому допустимы
    /// значения `1..=65535`.
    pub fn try_new(width: usize, height: usize) -> io::Result<Self> {
        validate_dimensions(width, height)?;

        let len = width
            .checked_mul(height)
            .ok_or_else(|| invalid_dimensions(width, height))?;

        let mut pixels = Vec::new();
        pixels.try_reserve_exact(len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::OutOfMemory,
                "not enough memory for TGA image",
            )
        })?;
        pixels.resize(len, Srgb::new(0u8, 0, 0));

        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    /// Ширина изображения.
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Высота изображения.
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Все пиксели изображения.
    pub fn pixels(&self) -> &[Srgb<u8>] {
        &self.pixels
    }

    /// Получить пиксель по координатам. `(0, 0)` — верхний левый угол.
    ///
    /// Паникует, если координаты выходят за пределы изображения.
    pub fn get_pixel(&self, x: usize, y: usize) -> Srgb<u8> {
        self.pixels[self.pixel_index(x, y)]
    }

    /// Задать цвет пикселя по координатам. `(0, 0)` — верхний левый угол.
    ///
    /// Паникует, если координаты выходят за пределы изображения: молчаливая
    /// запись в соседнюю строку при `x == width` была бы опасной ошибкой.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Srgb<u8>) {
        let index = self.pixel_index(x, y);
        self.pixels[index] = color;
    }

    /// Заполнить изображение заданным цветом.
    pub fn clear_with(&mut self, color: Srgb<u8>) {
        self.pixels.fill(color);
    }

    /// Очистить изображение в чёрный цвет.
    pub fn clear(&mut self) {
        self.clear_with(Srgb::new(0u8, 0, 0));
    }

    /// Записать линейный цвет, преобразовав его в sRGB и 8-битные компоненты.
    pub fn set_pixel_linear(&mut self, x: usize, y: usize, color: LinSrgb<f32>) {
        let encoded_f32: Srgb<f32> = color.into_color();
        let encoded_u8: Srgb<u8> = encoded_f32.into_format();

        self.set_pixel(x, y, encoded_u8);
    }

    /// Записать линейный цвет, заданный `glam::Vec3`.
    pub fn set_pixel_vec3(&mut self, x: usize, y: usize, color: Vec3) {
        self.set_pixel_linear(x, y, LinSrgb::new(color.x, color.y, color.z));
    }

    /// Получить пиксель как линейный цвет.
    pub fn get_pixel_linear(&self, x: usize, y: usize) -> LinSrgb<f32> {
        srgb_u8_to_linear(self.get_pixel(x, y))
    }

    /// Записать изображение в произвольный [`Write`].
    ///
    /// Полезно для тестов: можно писать в `Vec<u8>` или `Cursor<Vec<u8>>`.
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        self.validate_dimensions()?;

        let header = self.make_header();
        writer.write_all(&header)?;

        // Пишем построчно, чтобы не держать в памяти второй полный BGR-буфер.
        let mut row = Vec::with_capacity(self.width * 3);

        for y in 0..self.height {
            row.clear();

            for x in 0..self.width {
                let pixel = &self.pixels[y * self.width + x];
                let (red, green, blue) = pixel.into_components();

                // TGA для 24-битного true-color ожидает BGR.
                row.push(blue);
                row.push(green);
                row.push(red);
            }

            writer.write_all(&row)?;
        }

        writer.flush()
    }

    /// Сохранить изображение в файл.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        // Проверяем размеры до создания файла, чтобы не оставлять пустой файл
        // при заведомо невалидных данных.
        self.validate_dimensions()?;

        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        self.write_to(writer)
    }

    fn pixel_index(&self, x: usize, y: usize) -> usize {
        assert!(
            x < self.width && y < self.height,
            "pixel ({x}, {y}) outside {}x{} image",
            self.width,
            self.height
        );

        y * self.width + x
    }

    fn validate_dimensions(&self) -> io::Result<()> {
        validate_dimensions(self.width, self.height)
    }

    fn make_header(&self) -> [u8; 18] {
        let mut header = [0u8; 18];

        header[2] = 2; // Uncompressed true-color image.

        // Ширина и высота — 16-битные little-endian значения.
        header[12..14].copy_from_slice(&(self.width as u16).to_le_bytes());
        header[14..16].copy_from_slice(&(self.height as u16).to_le_bytes());

        header[16] = 24; // 8 бит на канал, BGR.
        header[17] = 0x20; // Top-left origin; alpha bits = 0.

        header
    }
}

fn validate_dimensions(width: usize, height: usize) -> io::Result<()> {
    if width == 0 || height == 0 || width > u16::MAX as usize || height > u16::MAX as usize {
        return Err(invalid_dimensions(width, height));
    }

    Ok(())
}

fn invalid_dimensions(width: usize, height: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "TGA width and height must be in 1..={}, got {width}x{height}",
            u16::MAX
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette::LinSrgb;

    #[test]
    fn rejects_invalid_dimensions() {
        assert!(TgaImage::try_new(0, 1).is_err());
        assert!(TgaImage::try_new(1, 0).is_err());
        assert!(TgaImage::try_new(u16::MAX as usize + 1, 1).is_err());
        assert!(TgaImage::try_new(1, u16::MAX as usize + 1).is_err());
    }

    #[test]
    fn writes_valid_header_and_bgr_pixels() {
        let mut image = TgaImage::new(2, 1);
        image.set_pixel(0, 0, Srgb::new(255u8, 0, 0));
        image.set_pixel(1, 0, Srgb::new(0u8, 255, 0));

        let mut bytes = Vec::new();
        image.write_to(&mut bytes).unwrap();

        assert_eq!(bytes.len(), 18 + 2 * 3);
        assert_eq!(bytes[2], 2); // uncompressed true-color
        assert_eq!(u16::from_le_bytes([bytes[12], bytes[13]]), 2);
        assert_eq!(u16::from_le_bytes([bytes[14], bytes[15]]), 1);
        assert_eq!(bytes[16], 24);
        assert_eq!(bytes[17], 0x20); // top-left origin

        // Красный и зелёный пиксели должны быть записаны как BGR.
        assert_eq!(&bytes[18..], &[0, 0, 255, 0, 255, 0]);
    }

    #[test]
    fn top_left_pixel_round_trip() {
        let mut image = TgaImage::new(2, 2);
        let red = Srgb::new(255u8, 0, 0);
        let green = Srgb::new(0u8, 255, 0);

        image.set_pixel(0, 0, red);
        image.set_pixel(1, 1, green);

        assert_eq!(image.get_pixel(0, 0), red);
        assert_eq!(image.get_pixel(1, 1), green);
        assert_eq!(image.get_width(), 2);
        assert_eq!(image.get_height(), 2);
        assert_eq!(image.pixels().len(), 4);
    }

    #[test]
    #[should_panic]
    fn rejects_x_equal_to_width() {
        let mut image = TgaImage::new(2, 2);
        image.set_pixel(2, 0, Srgb::new(0u8, 0, 0));
    }

    #[test]
    fn clear_with_fills_all_pixels() {
        let mut image = TgaImage::new(3, 2);
        let color = Srgb::new(1u8, 2, 3);

        image.clear_with(color);

        assert!(image.pixels().iter().all(|&pixel| pixel == color));
    }

    #[test]
    fn linear_color_is_saved_as_srgb() {
        let mut image = TgaImage::new(1, 1);
        image.set_pixel_linear(0, 0, LinSrgb::new(0.5f32, 0.25, 0.0));

        let (red, green, blue) = image.get_pixel(0, 0).into_components();

        // linear 0.5 -> sRGB ~0.735 -> ~188; linear 0.25 -> sRGB ~0.537 -> ~137.
        assert!((red as i16 - 188).abs() <= 1);
        assert!((green as i16 - 137).abs() <= 1);
        assert_eq!(blue, 0);
    }

    #[test]
    fn linear_pixel_round_trip_is_close() {
        let mut image = TgaImage::new(1, 1);
        image.set_pixel_linear(0, 0, LinSrgb::new(0.5f32, 0.25, 0.0));

        let linear = image.get_pixel_linear(0, 0);

        assert!((linear.red - 0.5).abs() < 0.01);
        assert!((linear.green - 0.25).abs() < 0.01);
        assert!(linear.blue.abs() < 0.01);
    }

    #[test]
    fn srgb_u8_conversion_decodes_red() {
        let linear = srgb_u8_to_linear(Srgb::new(255u8, 0, 0));

        assert!((linear.red - 1.0).abs() < 1e-6);
        assert!(linear.green.abs() < 1e-6);
        assert!(linear.blue.abs() < 1e-6);
    }

    #[test]
    fn into_glam_converts_srgb_to_linear() {
        let color = Srgb::new(0.5f32, 0.5, 0.5);
        let linear = color.to_glam3();

        // 0.5 в sRGB примерно соответствует 0.214 в линейном RGB.
        assert!((linear.x - 0.214).abs() < 0.001);
        assert!((linear.y - 0.214).abs() < 0.001);
        assert!((linear.z - 0.214).abs() < 0.001);
    }

    #[test]
    fn to_glam4_clamps_alpha() {
        let color = Srgb::new(1.0f32, 0.0, 0.0);

        assert_eq!(color.to_glam4(2.0).w, 1.0);
        assert_eq!(color.to_glam4(-1.0).w, 0.0);
    }
}
