use super::color::TgaImage;
use palette::Srgb;
use std::io::Result;
use std::path::Path;

/// Виртуальный «холст» для рисования.
///
/// Координаты пикселей начинаются с `(0, 0)` в левом верхнем углу, ось Y
/// направлена вниз. Пиксели за пределами холста при рисовании молча
/// пропускаются. Это осознанное отличие от `putPixel` из C++-версии книги:
/// там проверки границ нет, и выход за пределы изображения — ошибка.
///
/// В C++ `TgaImage` координата `y = 0` соответствует нижней строке, поэтому
/// при сравнении с книгой картинка будет зеркальна по вертикали.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    image: TgaImage,
}

impl Canvas {
    /// Создать холст с проверкой размеров.
    pub fn try_new(width: usize, height: usize) -> Result<Canvas> {
        let image = TgaImage::try_new(width, height)?;
        Ok(Canvas { image })
    }

    /// Ширина холста в пикселях.
    pub fn width(&self) -> usize {
        self.image.get_width()
    }

    /// Высота холста в пикселях.
    pub fn height(&self) -> usize {
        self.image.get_height()
    }

    /// Изображение, лежащее в основе холста.
    pub fn image(&self) -> &TgaImage {
        &self.image
    }

    /// Заполнить холст заданным цветом.
    pub fn clear_with(&mut self, color: Srgb<u8>) {
        self.image.clear_with(color);
    }

    /// Очистить холст в чёрный цвет.
    pub fn clear(&mut self) {
        self.image.clear();
    }

    /// Нарисовать отрезок целочисленным алгоритмом Брезенхейма.
    ///
    /// Порт `drawLine` из `Code/Chapter-7/line.cpp`. Алгоритм работает во всех
    /// октантах: отрезок можно рисовать в любом направлении, включая обратное.
    /// Оба конца отрезка включаются в результат.
    ///
    /// Как и в оригинале, при `d == 0` выбор следующего пикселя зависит от
    /// направления обхода, поэтому обратный отрезок в редких случаях может
    /// дать другой (тоже корректный) набор пикселей.
    ///
    /// Пиксели, оказавшиеся за пределами холста, пропускаются.
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Srgb<u8>) {
        // Внутри работаем со знаком, чтобы поддерживать шаг -1 при движении
        // справа налево или снизу вверх. Преобразование saturating: для
        // реальных координат холста значения заведомо помещаются в i64.
        let xa = i64::try_from(x0).unwrap_or(i64::MAX);
        let ya = i64::try_from(y0).unwrap_or(i64::MAX);
        let xb = i64::try_from(x1).unwrap_or(i64::MAX);
        let yb = i64::try_from(y1).unwrap_or(i64::MAX);

        let dx = (xb - xa).abs();
        let dy = (yb - ya).abs();
        let sx = if xb >= xa { 1 } else { -1 };
        let sy = if yb >= ya { 1 } else { -1 };

        if dy <= dx {
            // Более пологий отрезок: на каждом шаге двигаемся по x.
            let mut d = (dy << 1) - dx;
            let d1 = dy << 1;
            let d2 = (dy - dx) << 1;
            let mut x = xa + sx;
            let mut y = ya;

            self.plot(xa, ya, color);

            for _ in 1..=dx {
                if d > 0 {
                    d += d2;
                    y += sy;
                } else {
                    d += d1;
                }

                self.plot(x, y, color);
                x += sx;
            }
        } else {
            // Более крутой отрезок: на каждом шаге двигаемся по y.
            let mut d = (dx << 1) - dy;
            let d1 = dx << 1;
            let d2 = (dx - dy) << 1;
            let mut x = xa;
            let mut y = ya + sy;

            self.plot(xa, ya, color);

            for _ in 1..=dy {
                if d > 0 {
                    d += d2;
                    x += sx;
                } else {
                    d += d1;
                }

                self.plot(x, y, color);
                y += sy;
            }
        }
    }

    /// Нарисовать окружность
    ///
    /// Порт `drawCircle2` из `Code/Chapter-7/circle.cpp`. Используется
    /// целочисленный алгоритм Брезенхейма и восемь симметричных точек.
    /// Пиксели, оказавшиеся за пределами холста, пропускаются.
    pub fn draw_circle(
        &mut self,
        center_x: usize,
        center_y: usize,
        radius: usize,
        color: Srgb<u8>,
    ) {
        let xc = i64::try_from(center_x).unwrap_or(i64::MAX);
        let yc = i64::try_from(center_y).unwrap_or(i64::MAX);
        let r = i64::try_from(radius).unwrap_or(i64::MAX);

        let mut x = 0;
        let mut y = r;
        let mut d = 1 - r;
        let mut delta1 = 3;
        let mut delta2 = -2 * r + 5;

        self.draw_circle_points(xc, yc, x, y, color);

        while y > x {
            if d < 0 {
                d += delta1;
                delta1 += 2;
                delta2 += 2;
                x += 1;
            } else {
                d += delta2;
                delta1 += 2;
                delta2 += 4;
                x += 1;
                y -= 1;
            }

            self.draw_circle_points(xc, yc, x, y, color);
        }
    }

    /// Нарисовать восемь симметричных точек окружности.
    fn draw_circle_points(&mut self, xc: i64, yc: i64, x: i64, y: i64, color: Srgb<u8>) {
        self.plot(xc + x, yc + y, color);
        self.plot(xc + y, yc + x, color);
        self.plot(xc + y, yc - x, color);
        self.plot(xc + x, yc - y, color);
        self.plot(xc - x, yc - y, color);
        self.plot(xc - y, yc - x, color);
        self.plot(xc - y, yc + x, color);
        self.plot(xc - x, yc + y, color);
    }

    /// Сохранить содержимое холста в файл.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.image.write_to_file(path)
    }

    /// Поставить пиксель, если координаты попадают на холст.
    fn plot(&mut self, x: i64, y: i64, color: Srgb<u8>) {
        let Ok(x) = usize::try_from(x) else {
            return;
        };
        let Ok(y) = usize::try_from(y) else {
            return;
        };

        if x < self.width() && y < self.height() {
            self.image.set_pixel(x, y, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white() -> Srgb<u8> {
        Srgb::new(255u8, 255, 255)
    }

    fn painted_pixels(canvas: &Canvas, color: Srgb<u8>) -> Vec<(usize, usize)> {
        let mut pixels = Vec::new();

        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                if canvas.image().get_pixel(x, y) == color {
                    pixels.push((x, y));
                }
            }
        }

        pixels
    }

    fn assert_painted(canvas: &Canvas, expected: &[(usize, usize)]) {
        let mut actual = painted_pixels(canvas, white());
        actual.sort_unstable();

        let mut expected = expected.to_vec();
        expected.sort_unstable();

        assert_eq!(actual, expected);
    }

    #[test]
    fn draws_shallow_line() {
        let mut canvas = Canvas::try_new(6, 3).unwrap();

        canvas.draw_line(0, 0, 5, 2, white());

        assert_painted(&canvas, &[(0, 0), (1, 0), (2, 1), (3, 1), (4, 2), (5, 2)]);
    }

    #[test]
    fn draws_steep_line() {
        let mut canvas = Canvas::try_new(3, 6).unwrap();

        canvas.draw_line(0, 0, 2, 5, white());

        assert_painted(&canvas, &[(0, 0), (0, 1), (1, 2), (1, 3), (2, 4), (2, 5)]);
    }

    #[test]
    fn draws_horizontal_and_vertical_lines() {
        let mut canvas = Canvas::try_new(5, 5).unwrap();

        canvas.draw_line(0, 2, 4, 2, white());
        assert_painted(&canvas, &[(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)]);

        canvas.clear();
        canvas.draw_line(2, 0, 2, 4, white());
        assert_painted(&canvas, &[(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)]);
    }

    #[test]
    fn draws_single_point() {
        let mut canvas = Canvas::try_new(3, 3).unwrap();

        canvas.draw_line(1, 1, 1, 1, white());

        assert_painted(&canvas, &[(1, 1)]);
    }

    #[test]
    fn reverse_direction_draws_inclusive_endpoints() {
        let mut canvas = Canvas::try_new(11, 11).unwrap();

        // При d == 0 оригинальный алгоритм выбирает шаг по-разному в
        // зависимости от направления, поэтому здесь проверяем именно
        // последовательность для обратного отрезка, а не равенство картинок.
        canvas.draw_line(9, 7, 1, 2, white());

        assert_painted(
            &canvas,
            &[
                (9, 7),
                (8, 6),
                (7, 6),
                (6, 5),
                (5, 5),
                (4, 4),
                (3, 3),
                (2, 3),
                (1, 2),
            ],
        );
    }

    #[test]
    fn draws_all_octants() {
        let center = (5usize, 5usize);
        let endpoints = [
            (10usize, 5usize),
            (9, 8),
            (7, 9),
            (4, 10),
            (0, 5),
            (1, 2),
            (3, 1),
            (6, 0),
        ];

        for (x, y) in endpoints {
            let mut canvas = Canvas::try_new(11, 11).unwrap();

            canvas.draw_line(center.0, center.1, x, y, white());

            assert_eq!(canvas.image().get_pixel(center.0, center.1), white());
            assert_eq!(canvas.image().get_pixel(x, y), white());
        }
    }

    #[test]
    fn skips_pixels_outside_canvas() {
        let mut canvas = Canvas::try_new(3, 3).unwrap();

        canvas.draw_line(0, 0, 5, 5, white());

        assert_painted(&canvas, &[(0, 0), (1, 1), (2, 2)]);
    }

    #[test]
    fn skips_offscreen_prefix() {
        let mut canvas = Canvas::try_new(3, 3).unwrap();

        canvas.draw_line(5, 1, 0, 1, white());

        assert_painted(&canvas, &[(0, 1), (1, 1), (2, 1)]);
    }

    #[test]
    fn draws_circle_with_expected_points() {
        let mut canvas = Canvas::try_new(11, 11).unwrap();

        canvas.draw_circle(5, 5, 5, white());

        let pixels = painted_pixels(&canvas, white());
        assert_eq!(pixels.len(), 28);

        for point in [(5, 0), (10, 5), (5, 10), (0, 5), (8, 9), (9, 8)] {
            assert!(pixels.contains(&point), "missing circle point {point:?}");
        }
    }

    #[test]
    fn circle_with_zero_radius_draws_center() {
        let mut canvas = Canvas::try_new(5, 5).unwrap();

        canvas.draw_circle(2, 2, 0, white());

        assert_painted(&canvas, &[(2, 2)]);
    }

    #[test]
    fn circle_skips_pixels_outside_canvas() {
        let mut canvas = Canvas::try_new(5, 5).unwrap();

        canvas.draw_circle(0, 0, 2, white());

        assert_painted(&canvas, &[(0, 2), (2, 0), (1, 2), (2, 1)]);
    }

    #[test]
    fn clear_fills_canvas() {
        let mut canvas = Canvas::try_new(3, 2).unwrap();
        let color = Srgb::new(1u8, 2, 3);

        canvas.clear_with(color);

        assert_eq!(painted_pixels(&canvas, color).len(), 6);
    }
}
