use super::super::canvas::Canvas;
use super::super::geometry::Ray3d;
use super::material::AIR;
use super::scene::Scene;
use glam::{Vec2, Vec3};
use palette::Srgb;
use rayon::prelude::*;
use std::io::Result;
use std::path::Path;

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
    /// Угол обзора по оси X (в градусах)
    pub field_of_view_x: f32,
    /// Угол обзора по оси Y (в градусах)
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
    /// Угол обзора по оси X (в градусах)
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
        use std::io::{Error, ErrorKind};

        if width == 0 || height == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "размер камеры должен быть ненулевым",
            ));
        }
        if !field_of_view_x.is_finite() || !(0.0..180.0).contains(&field_of_view_x) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "горизонтальный FOV должен быть в диапазоне (0°, 180°)",
            ));
        }
        if !position.is_finite() || !view.is_finite() || !up.is_finite() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "базис камеры должен быть конечным",
            ));
        }

        let view_length = view.length();
        if view_length <= f32::EPSILON {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "направление взгляда не должно быть нулевым",
            ));
        }
        let view = view / view_length;
        let right = up.cross(view);
        if right.length_squared() <= f32::EPSILON {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "up не должен быть параллелен view",
            ));
        }
        let right = right.normalize();
        let up = view.cross(right).normalize();
        let field_of_view_y = (2.0
            * ((field_of_view_x.to_radians() * 0.5).tan() * height as f32 / width as f32).atan())
        .to_degrees();
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

    fn direction_at(
        basis: (Vec3, Vec3, Vec3),
        dimensions: (usize, usize),
        pixel: Vec2,
        tangent_fov: Vec2,
    ) -> Vec3 {
        let (view, right, up) = basis;
        let (width, height) = dimensions;
        let x = (pixel.x / width as f32 * 2.0 - 1.0) * tangent_fov.x;
        let y = (1.0 - pixel.y / height as f32 * 2.0) * tangent_fov.y;
        (view + x * right + y * up).normalize()
    }

    pub fn trace_frame<P: AsRef<Path>>(
        &mut self,
        scene: &Scene,
        file_name: P,
        maybe_epsilon: Option<f32>,
    ) -> Result<()> {
        use std::time::Instant;

        let start = Instant::now();

        let tangent_fov_x = (self.field_of_view_x.to_radians() * 0.5).tan();
        let tangent_fov_y = (self.field_of_view_y.to_radians() * 0.5).tan();
        let width = self.width;
        let height = self.height;
        let view = self.view;
        let right = self.right;
        let up = self.up;

        // МАГИЯ RUST: Параллельный итератор по всем пикселям холста
        self.canvas
            .pixels_mut()
            .par_iter_mut() // Каждым пикселем занимается свободное ядро процессора
            .enumerate()
            .for_each(|(idx, pixel)| {
                // Восстанавливаем координаты x и y из плоского индекса
                let i = idx % width;
                let j = idx / width;

                let ray = Ray3d {
                    origin: self.position,
                    direction: Self::direction_at(
                        (view, right, up),
                        (width, height),
                        Vec2::new(i as f32 + 0.5, j as f32 + 0.5),
                        Vec2::new(tangent_fov_x, tangent_fov_y),
                    ),
                };

                let color = scene.trace(AIR, 1.0, 1, &ray, maybe_epsilon);
                *pixel = color.into_format(); // Безопасно пишем результат прямо в пиксель
            });

        let duration = start.elapsed();
        println!(
            "Время ПАРАЛЛЕЛЬНОЙ отрисовки кадра: {} миллисекунд",
            duration.as_millis()
        );

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
        use std::io::{Error, ErrorKind};
        use std::time::Instant;

        if n1 == 0 || n2 == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "число подпикселей по каждой оси должно быть положительным",
            ));
        }
        let num_samples = n1
            .checked_mul(n2)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "слишком много сэмплов"))?;
        let start = Instant::now();

        let tangent_fov_x = (self.field_of_view_x.to_radians() * 0.5).tan();
        let tangent_fov_y = (self.field_of_view_y.to_radians() * 0.5).tan();
        let width = self.width;
        let height = self.height;
        let view = self.view;
        let right = self.right;
        let up = self.up;

        // Параллелим суперсэмплинг кадра
        self.canvas
            .pixels_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, pixel)| {
                let i = idx % width;
                let j = idx / width;

                let mut pixel_color = Srgb::new(0.0, 0.0, 0.0);

                for i_sub in 0..n1 {
                    for j_sub in 0..n2 {
                        let rand_x: f32 = rand::random();
                        let rand_y: f32 = rand::random();
                        let pixel_x = i as f32 + (i_sub as f32 + rand_x) / n1 as f32;
                        let pixel_y = j as f32 + (j_sub as f32 + rand_y) / n2 as f32;

                        let ray = Ray3d {
                            origin: self.position,
                            direction: Self::direction_at(
                                (view, right, up),
                                (width, height),
                                Vec2::new(pixel_x, pixel_y),
                                Vec2::new(tangent_fov_x, tangent_fov_y),
                            ),
                        };

                        let sample_color = scene.trace(AIR, 1.0, 1, &ray, maybe_epsilon);
                        pixel_color = pixel_color + sample_color;
                    }
                }

                pixel_color = pixel_color * (1.0 / num_samples as f32);
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3};

    fn camera(width: usize, height: usize) -> Camera {
        Camera::try_new(NewCameraParams {
            position: Vec3::ZERO,
            view: Vec3::Z,
            up: Vec3::Y,
            field_of_view_x: 60.0,
            width,
            height,
        })
        .unwrap()
    }

    #[test]
    fn camera_basis_points_right_and_up_for_positive_screen_axes() {
        let camera = camera(2, 2);

        assert_eq!(camera.right, Vec3::X);
        assert_eq!(camera.up, Vec3::Y);
        let upper_left = Camera::direction_at(
            (camera.view, camera.right, camera.up),
            (camera.width, camera.height),
            Vec2::new(0.5, 0.5),
            Vec2::ONE,
        );
        assert!(upper_left.x < 0.0);
        assert!(upper_left.y > 0.0);
    }

    #[test]
    fn center_pixel_ray_points_along_view_direction() {
        let camera = camera(1, 1);
        let direction = Camera::direction_at(
            (camera.view, camera.right, camera.up),
            (camera.width, camera.height),
            Vec2::splat(0.5),
            Vec2::new(
                (camera.field_of_view_x.to_radians() * 0.5).tan(),
                (camera.field_of_view_y.to_radians() * 0.5).tan(),
            ),
        );

        assert!(direction.abs_diff_eq(Vec3::Z, 1e-6));
    }

    #[test]
    fn vertical_fov_matches_horizontal_fov_and_aspect_ratio() {
        let camera = camera(640, 480);
        let expected = 2.0 * ((30.0_f32.to_radians().tan() * 0.75).atan()).to_degrees();

        assert!((camera.field_of_view_y - expected).abs() < 1e-5);
    }

    #[test]
    fn distributed_renderer_rejects_zero_samples() {
        let mut camera = camera(2, 2);
        let scene = Scene::default();

        let error = camera
            .trace_frame_distributed(&scene, "target/should-not-be-created.tga", 0, 1, None)
            .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }
}
