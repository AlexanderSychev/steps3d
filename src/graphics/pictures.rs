//! Работа с изображениями

use image::{DynamicImage, ImageReader, Rgba, RgbaImage, imageops::FilterType};
use palette::Srgba;
use simple_error::{SimpleResult, simple_error};
use std::path::Path;

/// Изображение в памяти
#[derive(Debug, Clone)]
pub struct Picture {
    /// Ширина в пикселях
    width: usize,
    /// Высота в пикселях
    height: usize,
    /// Пиксели
    pixels: Vec<Srgba<u8>>,
}
impl Picture {
    /// Попробовать создать изображение заданного размера.
    /// Можно задать цвет заливки (черный по-умолчанию).
    pub fn try_new(
        width: usize,
        height: usize,
        maybe_fill: Option<Srgba<u8>>,
    ) -> SimpleResult<Picture> {
        if width == 0 {
            return Err(simple_error!("Zero width is not allowed"));
        }
        if height == 0 {
            return Err(simple_error!("Zero height is not allowed"));
        }

        // Безопасный подсчет общего числа пикселей
        let count = width
            .checked_mul(height)
            .ok_or_else(|| simple_error!("Pixels count overflow"))?;
        let pixels = vec![maybe_fill.unwrap_or(Srgba::new(0, 0, 0, 255)); count];
        Ok(Picture {
            width,
            height,
            pixels,
        })
    }

    /// Попробовать создать изображение из файла
    pub fn try_from_file<P: AsRef<Path>>(path: P) -> SimpleResult<Picture> {
        let reader = ImageReader::open(path).map_err(|err| simple_error!(err.to_string()))?;
        let picture = reader
            .decode()
            .map_err(|err| simple_error!(err.to_string()))?;

        // Берем высоту и ширину с безопасным преобразованием типов
        let width =
            usize::try_from(picture.width()).map_err(|err| simple_error!(err.to_string()))?;
        let height =
            usize::try_from(picture.height()).map_err(|err| simple_error!(err.to_string()))?;

        // Безопасный подсчет общего числа пикселей для проверки на переполнение
        let _ = width
            .checked_mul(height)
            .ok_or_else(|| simple_error!("Pixels count overflow"))?;

        let rgba = picture.to_rgba8();
        let pixels = rgba
            .pixels()
            .map(|p| Srgba::new(p[0], p[1], p[2], p[3]))
            .collect();

        Ok(Picture {
            width,
            height,
            pixels,
        })
    }

    /// Получить ширину изображения
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Получить высоту изображения
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Получить число пикселей изображения
    pub fn get_pixels_count(&self) -> usize {
        self.pixels.len()
    }

    /// Получить пиксели для чтения
    pub fn get_pixels(&self) -> &[Srgba<u8>] {
        self.pixels.as_slice()
    }

    /// Установить новый цвет пикселя по координатам
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Srgba<u8>) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = y * self.width + x;
        self.pixels[index] = color;
    }
}

/// Генератор MIP-уровней
#[derive(Debug, Clone)]
enum MipMappedTextureGenerator {
    /// Исходное изображение во внутреннем формате пакета "image"
    Image(DynamicImage),
    /// Исходное изображение в виде массива пикселей
    Pixels(Vec<Srgba<u8>>),
}
impl MipMappedTextureGenerator {
    /// Сгенерировать MIP-уровни из исходного изображения
    fn generate_mip_levels(
        &self,
        width: usize,
        height: usize,
        filter: FilterType,
    ) -> Vec<Vec<Srgba<u8>>> {
        let mut current_picture = match self {
            MipMappedTextureGenerator::Image(image) => image.clone(),
            MipMappedTextureGenerator::Pixels(pixels) => {
                DynamicImage::from(RgbaImage::from_fn(width as u32, height as u32, |x, y| {
                    let index = y * (width as u32) + x;
                    let color = pixels[index as usize];
                    Rgba::<u8>::from([color.red, color.green, color.blue, color.alpha])
                }))
            }
        };
        let mut pixels: Vec<Vec<Srgba<u8>>> = match self {
            MipMappedTextureGenerator::Image(image) => {
                let rgba = image.clone().to_rgba8();
                let origin = rgba
                    .pixels()
                    .map(|p| Srgba::new(p[0], p[1], p[2], p[3]))
                    .collect();
                vec![origin]
            }
            MipMappedTextureGenerator::Pixels(pixels) => vec![pixels.clone()],
        };

        // Если исходное изображение уже единичного размера, то MIP-уровни не нужны
        if current_picture.width() == 1 && current_picture.height() == 1 {
            return pixels;
        }

        loop {
            let next_width = (current_picture.width() / 2).max(1);
            let next_height = (current_picture.height() / 2).max(1);

            let next_level = current_picture.resize_exact(next_width, next_height, filter);
            let next_rgba = next_level.to_rgba8();
            let next_pixels = next_rgba
                .pixels()
                .map(|p| Srgba::new(p[0], p[1], p[2], p[3]))
                .collect();
            pixels.push(next_pixels);
            current_picture = next_level;

            // Если оба размера стали единичным, то этот MIP-уровень последний
            if next_width == 1 && next_height == 1 {
                break;
            }
        }

        pixels
    }
}

/// Текстура с предварительно рассчитанными MIP-уровнями
#[derive(Debug, Clone)]
pub struct MipMappedTexture {
    /// Ширина в пикселях
    width: usize,
    /// Высота в пикселях
    height: usize,
    /// Фильтр, используемый при создании MIP-уровней
    filter: FilterType,
    /// Пиксели по MIP-уровням (от оригинального размера к минимальному)
    pixels: Vec<Vec<Srgba<u8>>>,
}
impl MipMappedTexture {
    /// Попробовать создать текстуру из файла с автоматическим созданием MIP-уровней
    pub fn try_from_file<P: AsRef<Path>>(
        path: P,
        maybe_filter: Option<FilterType>,
    ) -> SimpleResult<MipMappedTexture> {
        let filter = maybe_filter.unwrap_or(FilterType::Lanczos3);
        let reader = ImageReader::open(path).map_err(|err| simple_error!(err.to_string()))?;
        let picture = reader
            .decode()
            .map_err(|err| simple_error!(err.to_string()))?;

        // Берем высоту и ширину с безопасным преобразованием типов
        let width =
            usize::try_from(picture.width()).map_err(|err| simple_error!(err.to_string()))?;
        let height =
            usize::try_from(picture.height()).map_err(|err| simple_error!(err.to_string()))?;

        // Безопасный подсчет общего числа пикселей для проверки на переполнение
        let _ = width
            .checked_mul(height)
            .ok_or_else(|| simple_error!("Pixels count overflow"))?;

        // Генерируем MIP-уровни
        let pixels =
            MipMappedTextureGenerator::Image(picture).generate_mip_levels(width, height, filter);

        Ok(MipMappedTexture {
            width,
            height,
            filter,
            pixels,
        })
    }

    /// Получить ширину текстуры
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Получить высоту текстуры
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Получить число пикселей текстуры
    pub fn get_pixels_count(&self) -> usize {
        // Первый MIP-уровень это оригинальное изображение, его "длина" и будет числом пикселей
        self.pixels[0].len()
    }

    /// Получить число MIP-уровней текстуры
    pub fn get_mip_levels_count(&self) -> usize {
        self.pixels.len()
    }

    /// Установить новый цвет пикселя по координатам
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Srgba<u8>) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = y * self.width + x;
        let mut pixels = self.pixels[0].clone();
        pixels[index] = color;

        self.pixels = MipMappedTextureGenerator::Pixels(pixels).generate_mip_levels(
            self.width,
            self.height,
            self.filter,
        );
    }

    /// Установить новые цвета для нескольких пикселей.
    ///
    /// Данный метод предпочтителен для "массовых" перекрасок, так как пересчитывает MIP-уровни один раз.
    pub fn bulk_set_pixel(&mut self, updates: &[(usize, usize, Srgba<u8>)]) {
        if updates.len() == 0 {
            return;
        }

        let mut pixels = self.pixels[0].clone();
        for &(x, y, color) in updates {
            if x >= self.width || y >= self.height {
                continue;
            }

            let index = y * self.width + x;
            pixels[index] = color;
        }
        self.pixels = MipMappedTextureGenerator::Pixels(pixels).generate_mip_levels(
            self.width,
            self.height,
            self.filter,
        );
    }

    /// Получить пиксели для MIP-уровня
    pub fn get_pixels(&self, level: usize) -> SimpleResult<&[Srgba<u8>]> {
        let pixels = self
            .pixels
            .get(level)
            .ok_or(simple_error!("Unknown MIP-level: {}", level))?;
        Ok(pixels.as_slice())
    }
}
