//! Камера

use glam::Mat4;
use glam::camera::lh::proj::opengl::perspective;
use simple_error::{SimpleResult, simple_error};
use std::f32::consts::PI;
use std::fmt::Debug;

/// Название ширины для сообщений об ошибках
const WIDTH_NAME: &str = "Width";

/// Название высоты для сообщений об ошибках
const HEIGHT_NAME: &str = "Height";

/// Область просмотра
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Viewport {
    /// Смещение начала области просмотра по ширине относительно начала окна (левый верхний угол).
    /// Смещение может быть отрицательным числом.
    shift_x: i32,
    /// Смещение начала области просмотра по высоте относительно начала окна (левый верхний угол)
    /// Смещение может быть отрицательным числом.
    shift_y: i32,
    /// Ширина в пикселях
    width: u32,
    /// Высота в пикселях
    height: u32,
}
impl Viewport {
    /// Создаёт новую область просмотра с проверкой размеров
    pub fn try_new(
        width: u32,
        height: u32,
        maybe_shift_x: Option<i32>,
        maybe_shift_y: Option<i32>,
    ) -> SimpleResult<Viewport> {
        Self::validate_size(width, height).map(|()| Viewport {
            width,
            height,
            shift_x: maybe_shift_x.unwrap_or(0),
            shift_y: maybe_shift_y.unwrap_or(0),
        })
    }

    /// Получить cмещение начала области просмотра по ширине
    pub fn get_shift_x(&self) -> i32 {
        self.shift_x
    }

    /// Изменить cмещение начала области просмотра по ширине
    pub fn set_shift_x(&mut self, shift_x: i32) {
        self.shift_x = shift_x
    }

    /// Получить cмещение начала области просмотра по высоте
    pub fn get_shift_y(&self) -> i32 {
        self.shift_y
    }

    /// Изменить cмещение начала области просмотра по высоте
    pub fn set_shift_y(&mut self, shift_y: i32) {
        self.shift_y = shift_y
    }

    /// Получить ширину в пикселях
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Задать ширину в пикселях
    pub fn set_width(&mut self, width: u32) -> SimpleResult<()> {
        Self::validate_size_value(width, WIDTH_NAME)?;
        self.width = width;
        Ok(())
    }

    /// Получить высоту в пикселях
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Задать высоту в пикселях
    pub fn set_height(&mut self, height: u32) -> SimpleResult<()> {
        Self::validate_size_value(height, HEIGHT_NAME)?;
        self.height = height;
        Ok(())
    }

    /// Задать новые размеры области просмотра
    pub fn resize(&mut self, width: u32, height: u32) -> SimpleResult<()> {
        Self::validate_size(width, height)?;
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Получить соотношение сторон (ширина деленная на высоту)
    pub fn get_aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Получить обратное соотношение сторон (высота деленная на ширину)
    pub fn get_aspect_ratio_inversed(&self) -> f32 {
        self.height as f32 / self.width as f32
    }

    /// Провалидировать размеры области просмотра
    fn validate_size(width: u32, height: u32) -> SimpleResult<()> {
        Self::validate_size_value(width, WIDTH_NAME)?;
        Self::validate_size_value(height, HEIGHT_NAME)
    }

    /// Провалидировать ширину или высоту области просмотра
    fn validate_size_value(value: u32, name: &'static str) -> SimpleResult<()> {
        if value == 0 {
            return Err(simple_error!("{} cannot be zero", name));
        }
        Ok(())
    }
}

/// Название оси X для сообщений об ошибках
const X_AXIS_NAME: &str = "X-axis";

/// Название оси Y для сообщений об ошибках
const Y_AXIS_NAME: &str = "Y-axis";

/// Ось, угол обзора по которой надо оставить прежним в поле зрения при смене области просмотра
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FieldOfViewKeepAxis {
    /// Оставить угол по оси X
    KeepX,
    /// Оставить угол по оси Y
    KeepY,
}

/// Поле зрения: углы обзора по осям X и Y
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldOfView {
    /// Область просмотра камеры
    viewport: Viewport,
    /// Угол обзора по оси X (в радианах)
    x_angle: f32,
    /// Угол обзора по оси Y (в радианах)
    y_angle: f32,
}
impl FieldOfView {
    /// Создать поле зрения по углу обзора X и области просмотра
    pub fn try_new_from_x_angle(x_angle: f32, viewport: Viewport) -> SimpleResult<Self> {
        Self::validate_angle(x_angle, X_AXIS_NAME)?;

        let y_angle = Self::compute_y_angle(x_angle, &viewport);
        Self::validate_angle(y_angle, Y_AXIS_NAME)?;

        Ok(Self {
            x_angle,
            y_angle,
            viewport,
        })
    }

    /// Создать поле зрения по углу обзора Y и области просмотра
    pub fn try_new_from_y_angle(y_angle: f32, viewport: Viewport) -> SimpleResult<Self> {
        Self::validate_angle(y_angle, Y_AXIS_NAME)?;

        let x_angle = Self::compute_x_angle(y_angle, &viewport);
        Self::validate_angle(x_angle, X_AXIS_NAME)?;

        Ok(Self {
            x_angle,
            y_angle,
            viewport,
        })
    }

    /// Получить область просмотра
    pub fn get_viewport(&self) -> Viewport {
        self.viewport
    }

    /// Изменить область просмотра. Второй аргумент (необязательный) позволяет настроить ось,
    /// угол которой останется прежним (по-умолчанию это X).
    pub fn set_viewport(
        &mut self,
        viewport: Viewport,
        maybe_keep_axis: Option<FieldOfViewKeepAxis>,
    ) -> SimpleResult<()> {
        let keep_axis = maybe_keep_axis.unwrap_or(FieldOfViewKeepAxis::KeepX);

        let (x_angle, y_angle) = match keep_axis {
            FieldOfViewKeepAxis::KeepX => {
                let y_angle = Self::compute_y_angle(self.x_angle, &viewport);
                Self::validate_angle(y_angle, Y_AXIS_NAME).map(|()| (self.x_angle, y_angle))
            }
            FieldOfViewKeepAxis::KeepY => {
                let x_angle = Self::compute_x_angle(self.y_angle, &viewport);
                Self::validate_angle(x_angle, X_AXIS_NAME).map(|()| (x_angle, self.y_angle))
            }
        }?;

        self.viewport = viewport;
        self.x_angle = x_angle;
        self.y_angle = y_angle;

        Ok(())
    }

    /// Получить угол обзора по оси X (в радианах)
    pub fn get_x_angle(&self) -> f32 {
        self.x_angle
    }

    /// Изменить угол обзора по оси X (в радианах).
    /// Угол обзора по оси Y будет пересчитан автоматически.
    pub fn set_x_angle(&mut self, x_angle: f32) -> SimpleResult<()> {
        Self::validate_angle(x_angle, X_AXIS_NAME)?;

        let y_angle = Self::compute_y_angle(x_angle, &self.viewport);
        Self::validate_angle(y_angle, Y_AXIS_NAME)?;

        self.x_angle = x_angle;
        self.y_angle = y_angle;

        Ok(())
    }

    /// Получить угол обзора по оси Y
    pub fn get_y_angle(&self) -> f32 {
        self.y_angle
    }

    /// Изменить угол обзора по оси Y
    /// Угол обзора по оси X будет пересчитан автоматически.
    pub fn set_y_angle(&mut self, y_angle: f32) -> SimpleResult<()> {
        Self::validate_angle(y_angle, Y_AXIS_NAME)?;

        let x_angle = Self::compute_x_angle(y_angle, &self.viewport);
        Self::validate_angle(x_angle, X_AXIS_NAME)?;

        self.x_angle = x_angle;
        self.y_angle = y_angle;

        Ok(())
    }

    /// Провалидировать угол обзора
    fn validate_angle(value: f32, axis_prefix: &'static str) -> SimpleResult<()> {
        if !value.is_finite() {
            return Err(simple_error!(
                "{} field of view angle must be finite value",
                axis_prefix
            ));
        }
        if value <= 0.0 || value >= PI {
            return Err(simple_error!(
                "{} field of view angle must be in range (0, π)",
                axis_prefix
            ));
        }

        Ok(())
    }

    /// Вычислить угол обзора по оси Y (в радианах) по углу обзора по оси X (тоже в радианах) и размерам области посмотра
    fn compute_y_angle(x_angle: f32, viewport: &Viewport) -> f32 {
        let aspect_ratio_inversed = viewport.get_aspect_ratio_inversed();
        2.0 * ((x_angle * 0.5).tan() * aspect_ratio_inversed).atan()
    }

    /// Вычислить угол обзора по оси X (в радианах) по углу обзора по оси Y (тоже в радианах) и размерам области посмотра
    fn compute_x_angle(y_angle: f32, viewport: &Viewport) -> f32 {
        let aspect_ratio = viewport.get_aspect_ratio();
        2.0 * ((y_angle * 0.5).tan() * aspect_ratio).atan()
    }
}

/// Плоскости отсечения (по оси Z)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippingPlanes {
    /// Ближняя
    near: f32,
    /// Дальняя
    far: f32,
}
impl ClippingPlanes {
    pub fn try_new(near: f32, far: f32) -> SimpleResult<Self> {
        Self::validate_planes(near, far)?;
        Ok(Self { near, far })
    }

    /// Получить ближнюю плоскость отсечения
    pub fn get_near(&self) -> f32 {
        self.near
    }

    /// Задать ближнюю плоскость отсечения
    pub fn set_near(&mut self, near: f32) -> SimpleResult<()> {
        Self::validate_planes(near, self.far)?;
        self.near = near;
        Ok(())
    }

    /// Получить дальнюю плоскость отсечения
    pub fn get_far(&self) -> f32 {
        self.far
    }

    /// Задать дальнюю плоскость отсечения
    pub fn set_far(&mut self, far: f32) -> SimpleResult<()> {
        Self::validate_planes(self.near, far)?;
        self.far = far;
        Ok(())
    }

    /// Провалидировать плоскости отсечения
    fn validate_planes(near: f32, far: f32) -> SimpleResult<()> {
        if !near.is_finite() {
            return Err(simple_error!("Near clipping plane value must be finite"));
        }
        if near <= 0.0 {
            return Err(simple_error!("Near clipping plane value must be positive"));
        }
        if !far.is_finite() {
            return Err(simple_error!("Far clipping plane value must be finite"));
        }
        if far <= near {
            return Err(simple_error!(
                "Far clipping plane value must be greater than near"
            ));
        }

        Ok(())
    }
}
impl Default for ClippingPlanes {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Перспективная камера
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCamera {
    /// Поле зрения
    field_of_view: FieldOfView,
    /// Плоскости отсечения
    clipping: ClippingPlanes,
}
impl PerspectiveCamera {
    /// Создать камеру со стандартным базисом
    pub fn new(field_of_view: FieldOfView, clipping: ClippingPlanes) -> Self {
        Self {
            field_of_view,
            clipping,
        }
    }

    /// Получить поле зрения камеры для чтения
    pub fn get_field_of_view(&self) -> FieldOfView {
        self.field_of_view
    }

    /// Получить поле зрения камеры для записи
    pub fn get_field_of_view_mut(&mut self) -> &mut FieldOfView {
        &mut self.field_of_view
    }

    /// Задать новое поле зрения камеры для записи
    pub fn set_field_of_view(&mut self, field_of_view: FieldOfView) {
        self.field_of_view = field_of_view;
    }

    /// Получить отсекающие плоскости для чтения
    pub fn get_clipping(&self) -> ClippingPlanes {
        self.clipping
    }

    /// Получить отсекающие плоскости для записи
    pub fn get_clipping_mut(&mut self) -> &mut ClippingPlanes {
        &mut self.clipping
    }

    /// Задать новые отсекающие плоскости
    pub fn set_clipping(&mut self, clipping: ClippingPlanes) {
        self.clipping = clipping
    }

    /// Возвращает матрицу проекции (Projection Matrix) для OpenGL
    pub fn get_projection_matrix(&self) -> Mat4 {
        perspective(
            self.field_of_view.get_y_angle(),
            self.field_of_view.get_viewport().get_aspect_ratio(),
            self.clipping.near,
            self.clipping.far,
        )
    }
}
