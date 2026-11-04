use super::super::canvas::Canvas;
use super::super::geometry::Ray3d;
use super::material::AIR;
use super::scene::Scene;
use glam::Vec3;
use palette::Srgb;
use std::io::Result;
use std::path::Path;
use rayon::prelude::*;

/// Камера
#[derive(Debug, Clone)]
pub struct Camera {
    /// Позиция камеры
    pub position: Vec3,
    /// Базис камеры - направление взгляда
    pub view: Vec3,
    /// Базис камеры - направление наверх
    pub up: Vec3,
    /// Базис камеры - направление вправо
    pub right: Vec3,
    /// Ширина кадра камеры в пикселях
    pub width: usize,
    /// Высота кадра камеры в пикселях
    pub height: usize,
    /// Угол обзора по оси X (в радианах)
    pub field_of_view_x: f32,
    /// Угол обзора по оси Y (в радианах)
    pub field_of_view_y: f32,
    /// Виртуальный "холст" для рисования
    pub canvas: Canvas,
}

/// Параметры фабричного метода "try_new" для создания камеры: то, что нельзя вычислить по другим полям
#[derive(Debug, Clone, Copy)]
pub struct NewCameraParams {
    /// Позиция камеры
    pub position: Vec3,
    /// Базис камеры - направление взгляда
    pub view: Vec3,
    /// Базис камеры - направление наверх
    pub up: Vec3,
    /// Угол обзора по оси X (в радианах)
    pub field_of_view_x: f32,
    /// Ширина кадра камеры (в пикселях)
    pub width: usize,
    /// Высота кадра камеры (в пикселях)
    pub height: usize,
}

impl Camera {
    /// Создать новую камеру
    pub fn try_new(
        NewCameraParams {
            position,
            view,
            up,
            field_of_view_x,
            width,
            height,
        }: NewCameraParams,
    ) -> Result<Self> {
        let right = view.cross(up);
        let field_of_view_y = field_of_view_x * (height as f32 / width as f32);
        let canvas = Canvas::try_new(width, height)?;
        Ok(Camera {
            position,
            view,
            up,
            right,
            width,
            height,
            field_of_view_x,
            field_of_view_y,
            canvas,
        })
    }

    pub fn trace_frame<P: AsRef<Path>>(&mut self, scene: &Scene, file_name: P, maybe_epsilon: Option<f32>) -> Result<()> {
        use std::f32::consts::PI;
        use std::time::Instant;

        let start = Instant::now();

        let tangent_fov_x = (self.field_of_view_x * PI / 180.0 / 2.0).tan();
        let tangent_fov_y = (self.field_of_view_y * PI / 180.0 / 2.0).tan();

        let width_f = self.width as f32;
        let height_f = self.height as f32;
        let half_width = width_f / 2.0;
        let half_height = height_f / 2.0;

        let width = self.width;

        // МАГИЯ RUST: Параллельный итератор по всем пикселям холста
        self.canvas.pixels_mut()
            .par_iter_mut() // Каждым пикселем занимается свободное ядро процессора
            .enumerate()
            .for_each(|(idx, pixel)| {
                // Восстанавливаем координаты x и y из плоского индекса
                let i = idx % width;
                let j = idx / width;

                let x = (i as f32 - half_width) * tangent_fov_x * 2.0 / width_f;
                let y = (j as f32 - half_height) * tangent_fov_y * 2.0 / height_f;

                let ray = Ray3d {
                    origin: self.position,
                    direction: (self.view + x * self.right + y * self.up).normalize(),
                };

                let color = scene.trace(AIR, 1.0, 1, &ray, maybe_epsilon);
                *pixel = color.into_format(); // Безопасно пишем результат прямо в пиксель
            });

        let duration = start.elapsed();
        println!("Время ПАРАЛЛЕЛЬНОЙ отрисовки кадра: {} миллисекунд", duration.as_millis());

        self.canvas.write_to_file(file_name)
    }

    /// Распределенная трассировка кадра (Суперсэмплинг для сглаживания)
    /// `n1` — количество подпикселей по горизонтали, `n2` — по вертикали.
    pub fn trace_frame_distributed<P: AsRef<Path>>(
        &mut self,
        scene: &Scene,
        file_name: P,
        n1: usize,
        n2: usize,
        maybe_epsilon: Option<f32>,
    ) -> Result<()> {
        use std::f32::consts::PI;
        use std::time::Instant;

        let start = Instant::now();

        let tangent_fov_x = (self.field_of_view_x * PI / 180.0 / 2.0).tan();
        let tangent_fov_y = (self.field_of_view_y * PI / 180.0 / 2.0).tan();

        let width_f = self.width as f32;
        let height_f = self.height as f32;
        let half_width = width_f / 2.0;
        let half_height = height_f / 2.0;

        let hx = tangent_fov_x * 2.0 / width_f;
        let hy = tangent_fov_y * 2.0 / height_f;

        let hx_sub = hx / n1 as f32;
        let hy_sub = hy / n2 as f32;
        let num_samples = (n1 * n2) as f32;

        let width = self.width;

        // Параллелим суперсэмплинг кадра
        self.canvas.pixels_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, pixel)| {
                let i = idx % width;
                let j = idx / width;

                let x = (i as f32 - half_width) * hx;
                let y = (j as f32 - half_height) * hy;

                let x1 = x - 0.5 * hx;
                let y1 = y - 0.5 * hy;

                let mut pixel_color = Srgb::new(0.0, 0.0, 0.0);

                for i_sub in 0..n1 {
                    for j_sub in 0..n2 {
                        let rand_x: f32 = rand::random();
                        let rand_y: f32 = rand::random();

                        let xt = x1 + hx_sub * (i_sub as f32 + rand_x);
                        let yt = y1 + hy_sub * (j_sub as f32 + rand_y);

                        let ray = Ray3d {
                            origin: self.position,
                            direction: (self.view + xt * self.right + yt * self.up).normalize(),
                        };

                        let sample_color = scene.trace(AIR, 1.0, 1, &ray, maybe_epsilon);
                        pixel_color = pixel_color + sample_color;
                    }
                }

                pixel_color = pixel_color * (1.0 / num_samples);
                *pixel = pixel_color.into_format();
            });

        let duration = start.elapsed();
        println!(
            "Время ПАРАЛЛЕЛЬНОЙ распределенной отрисовки кадра ({} сэмплов): {} мс",
            n1 * n2,
            duration.as_millis()
        );

        self.canvas.write_to_file(file_name)
    }
}
