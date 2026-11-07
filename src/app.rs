use palette::Srgb;
use sdl2::{
    EventPump, Sdl, init,
    pixels::PixelFormatEnum,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use simple_error::{SimpleResult, simple_error};
use std::time::Instant;

/// SDL-окно с CPU-буфером кадра.
pub struct Application {
    /// Холст SDL для вывода готового кадра.
    canvas: Canvas<Window>,
    /// Ширина экрана
    width: u32,
    /// Высота экрана
    height: u32,
    /// Пиксели в формате RGB24, построчно сверху вниз.
    pixels: Vec<u8>,
    /// Контекст библиотеки SDL должен жить дольше окна и холста.
    context: Sdl,
}

impl Application {
    /// Попробовать создать новое оконное приложение.
    pub fn try_new(title: &str, width: u32, height: u32) -> SimpleResult<Application> {
        let pixel_count = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|count| count.checked_mul(3))
            .ok_or_else(|| simple_error!("Размер буфера кадра слишком велик"))?;

        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(pixel_count)
            .map_err(|err| simple_error!(err.to_string()))?;
        pixels.resize(pixel_count, 0);

        let context = init().map_err(|msg| simple_error!(msg))?;
        let video_subsystem = context.video().map_err(|msg| simple_error!(msg))?;

        let window = video_subsystem
            .window(title, width, height)
            .resizable()
            .position_centered()
            .build()
            .map_err(|err| simple_error!(err.to_string()))?;

        let canvas = window
            .into_canvas()
            .build()
            .map_err(|err| simple_error!(err.to_string()))?;

        Ok(Application {
            context,
            canvas,
            width,
            height,
            pixels,
        })
    }

    /// Ширина буфера кадра в пикселях.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Высота буфера кадра в пикселях.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Создать creator для streaming-текстуры, используемой в [`present`].
    ///
    /// Creator и созданную из него текстуру следует хранить рядом с приложением,
    /// чтобы не создавать новую текстуру при каждом выводе кадра. Текстуру следует
    /// создать с [`pixel_format`](Self::pixel_format), [`width`](Self::width) и
    /// [`height`](Self::height) этого приложения.
    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    /// Заполнить CPU-буфер заданным цветом.
    pub fn clear_with(&mut self, color: Srgb<u8>) {
        for pixel in self.pixels.chunks_exact_mut(3) {
            pixel[0] = color.red;
            pixel[1] = color.green;
            pixel[2] = color.blue;
        }
    }

    /// Очистить CPU-буфер в чёрный цвет.
    pub fn clear(&mut self) {
        self.clear_with(Srgb::new(0, 0, 0));
    }

    /// Скопировать в CPU-буфер готовый кадр из массива sRGB-пикселей.
    pub fn set_pixels(&mut self, pixels: &[Srgb<u8>]) -> SimpleResult<()> {
        let expected_count = self.pixels.len() / 3;
        if pixels.len() != expected_count {
            return Err(simple_error!(format!(
                "Ожидалось {expected_count} пикселей, получено {}",
                pixels.len()
            )));
        }

        for (target, source) in self.pixels.chunks_exact_mut(3).zip(pixels) {
            target[0] = source.red;
            target[1] = source.green;
            target[2] = source.blue;
        }
        Ok(())
    }

    /// Поставить пиксель в CPU-буфер. Отрицательные координаты и координаты за
    /// пределами буфера молча пропускаются.
    pub fn draw_pixel(&mut self, x: i64, y: i64, color: Srgb<u8>) -> SimpleResult<()> {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return Ok(());
        };
        let (Ok(width), Ok(height)) = (usize::try_from(self.width), usize::try_from(self.height))
        else {
            return Ok(());
        };
        if x >= width || y >= height {
            return Ok(());
        }

        let offset = (y * width + x) * 3;
        self.pixels[offset] = color.red;
        self.pixels[offset + 1] = color.green;
        self.pixels[offset + 2] = color.blue;
        Ok(())
    }

    /// Загрузить буфер в SDL streaming-текстуру и показать кадр.
    pub fn present(&mut self, texture: &mut Texture<'_>) -> SimpleResult<()> {
        let pitch = usize::try_from(self.width)
            .ok()
            .and_then(|width| width.checked_mul(3))
            .ok_or_else(|| simple_error!("Ширина буфера кадра слишком велика"))?;

        texture
            .update(None, &self.pixels, pitch)
            .map_err(|err| simple_error!(err.to_string()))?;
        self.canvas
            .copy(texture, None, None)
            .map_err(|err| simple_error!(err))?;
        self.canvas.present();
        Ok(())
    }

    /// Формат текстуры, необходимый для буфера RGB24.
    pub const fn pixel_format() -> PixelFormatEnum {
        PixelFormatEnum::RGB24
    }

    /// Получить обработчик событий SDL
    pub fn event_pump(&self) -> SimpleResult<EventPump> {
        self.context.event_pump().map_err(|msg| simple_error!(msg))
    }
}

/// Утилита для измерения времени между кадрами.
/// Может быть использована для ограничений анимации и скорости
#[derive(Debug, Clone, Copy)]
pub struct FrameDelta {
    /// Clock measurement instance
    clock: Instant,
    /// Computed time delta in seconds
    delta_time: f32,
    /// Last
    last_frame: f32,
}

impl FrameDelta {
    /// Compute next time difference. This method must be called at the end of game tick.
    pub fn tick(&mut self) {
        let current_frame = self.clock.elapsed().as_secs_f32();
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;
    }

    /// Returns current time difference
    pub fn get_delta(&self) -> f32 {
        self.delta_time
    }
}

impl Default for FrameDelta {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            last_frame: 0.0,
            clock: Instant::now(),
        }
    }
}
