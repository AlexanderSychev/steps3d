//! Объекты сцены

use super::super::geometry::DEFAULT_GEOMETRY_EPSILON;
use super::super::geometry::Ray3d;
use super::material::{MaterialProps, SurfaceData};
use glam::Vec3;

/// Кэшируемые данные пересечений
#[derive(Debug, Clone, Copy)]
pub enum HitCache {
    Sphere { t1: f32, t2: f32 },
    Plane { vd: f32 },
    Triangle,
    Aabb,
}

/// Информация о пересечении
#[derive(Debug, Clone, Copy)]
pub struct HitData {
    /// Точка пересечения
    pub position: Vec3,
    /// Параметр вдоль луча
    pub t: f32,
    /// Нужно ли перевернуть нормаль
    pub invert_normal: bool,
    /// Кэшированные значения для конкретной геометрии
    pub cache: HitCache,
    /// Индекс объекта в векторе сцены, с которым произошло пересечение
    pub object_index: usize,
}

/// Интерфейс геометрического объекта сцены
pub trait Object {
    /// Проверить, пересекает ли луч объект.
    /// Внутри реализации объект пока не знает свой индекс,
    /// поэтому мы можем временно ставить туда 0, а сцена перезапишет его.
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData>;

    /// Получить данные поверхности в точке пересечения
    fn get_surface(&self, hit_data: &HitData) -> SurfaceData;
}

/// Сфера
#[derive(Debug, Clone)]
pub struct Sphere {
    /// Центр сферы
    pub center: Vec3,
    /// Радиус сферы
    pub radius: f32,
    /// Квадрат радиуса сферы (для оптимизации расчетов)
    pub radius_sq: f32,
    /// Дефолтные свойства материала этой сферы
    pub def_surface: MaterialProps,
}

impl Sphere {
    /// Создать новую сферу с предрасчитанным квадратом радиуса
    pub fn new(center: Vec3, radius: f32, def_surface: MaterialProps) -> Self {
        Self {
            center,
            radius,
            radius_sq: radius * radius,
            def_surface,
        }
    }
}

impl Object for Sphere {
    /// Проверить пересечение луча со сферой
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Вектор направленный от начала луча к центру сферы ("l" в C++)
        let l = self.center - ray.origin;

        // Квадрат расстояния от начала луча до центра сферы
        let l2oc = l.dot(l);

        // Проекция вектора l на направление луча (ближайшая точка к центру сферы вдоль луча)
        let tca = l.dot(ray.direction);

        // Квадрат расстояния от проекции до точки пересечения (по теореме Пифагора)
        let t2hc = self.radius_sq - l2oc + tca * tca;

        // Если t2hc меньше нуля, луч проходит мимо сферы
        if t2hc <= 0.0 {
            return None;
        }

        let t2hc_sqrt = f32::sqrt(t2hc);
        let t1: f32;
        let t2: f32;

        // Проверяем, находится ли начало луча внутри сферы или снаружи
        if tca < t2hc_sqrt {
            // Луч внутри сферы
            t1 = tca + t2hc_sqrt;
            t2 = tca - t2hc_sqrt;
        } else {
            // Луч снаружи сферы
            t1 = tca - t2hc_sqrt;
            t2 = tca + t2hc_sqrt;
        }

        // Обработка касания или очень близких точек через "epsilon"
        let mut final_t = t1;
        if f32::abs(final_t) < epsilon {
            final_t = t2;
        }

        // Если точка пересечения находится впереди по ходу луча (t > epsilon)
        if final_t > epsilon {
            let position = ray.point_at(final_t); // Используем метод ray для получения точки

            Some(HitData {
                position,
                t: final_t,
                invert_normal: false,
                // Заполняем наш безопасный типизированный кэш вместо массива float
                cache: HitCache::Sphere { t1, t2 },
                // Индекс объекта будет проставлен сценой динамически во время итерации
                object_index: 0,
            })
        } else {
            None
        }
    }

    /// Получить параметры поверхности сферы в точке пересечения
    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        // Вычисляем нормаль: вектор от центра сферы к точке пересечения, деленный на радиус
        let normal = (hit_data.position - self.center).normalize();

        SurfaceData {
            position: hit_data.position,
            normal,
            material: self.def_surface,
        }
    }
}

/// Плоскость
#[derive(Debug, Clone)]
pub struct Plane {
    /// Нормаль к плоскости (уравнение плоскости: (p, n) + d = 0)
    pub normal: Vec3,
    /// Смещение плоскости относительно начала координат
    pub d: f32,
    /// Дефолтные свойства материала этой плоскости
    pub def_surface: MaterialProps,
}

impl Plane {
    /// Создать новую плоскость
    pub fn new(normal: Vec3, d: f32, def_surface: MaterialProps) -> Self {
        Self {
            normal: normal.normalize(), // Гарантируем, что нормаль единичной длины
            d,
            def_surface,
        }
    }
}

impl Object for Plane {
    /// Проверить пересечение луча с плоскостью
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Скалярное произведение нормали плоскости и направления луча ("vd" в C++)
        let vd = self.normal.dot(ray.direction);

        // Если vd близко к нулю, луч параллелен плоскости (или лежит в ней) — пересечения нет
        if vd > -epsilon && vd < epsilon {
            return None;
        }

        // Вычисляем дистанцию до точки пересечения t
        // Формула: t = -( (n, origin) + d ) / vd
        let t = -(self.normal.dot(ray.origin) + self.d) / vd;

        // Пересечение должно быть впереди по ходу луча (t > EPSILON)
        if t > epsilon {
            // Вычисляем точку пересечения вручную, так как у Ray3d нет методов
            let position = ray.origin + ray.direction * t;

            Some(HitData {
                position,
                t,
                invert_normal: false,
                // Заполняем кэш для плоскости
                cache: HitCache::Plane { vd },
                // Будет перезаписано сценой
                object_index: 0,
            })
        } else {
            None
        }
    }

    /// Получить параметры поверхности плоскости в точке пересечения
    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        SurfaceData {
            position: hit_data.position,
            // Для плоскости нормаль всегда одинакова в любой точке
            normal: self.normal,
            material: self.def_surface,
        }
    }
}

/// Треугольник
#[derive(Debug, Clone)]
pub struct Tri {
    /// Вершины треугольника
    pub p: [Vec3; 3],
    /// Геометрическая нормаль к треугольнику
    pub normal: Vec3,
    /// Дефолтные свойства материала этого треугольника
    pub def_surface: MaterialProps,
}

impl Tri {
    /// Создать новый треугольник по трем вершинам
    pub fn new(a: Vec3, b: Vec3, c: Vec3, def_surface: MaterialProps) -> Self {
        // Вычисляем нормаль через кросс-произведение двух ребер
        let edge1 = b - a;
        let edge2 = c - a;
        let normal = edge1.cross(edge2).normalize();

        Self {
            p: [a, b, c],
            normal,
            def_surface,
        }
    }
}

impl Object for Tri {
    /// Проверить пересечение луча с треугольником (Алгоритм Моллера-Трумбора)
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        let edge1 = self.p[1] - self.p[0];
        let edge2 = self.p[2] - self.p[0];

        // Вычисляем вектор pvec
        let pvec = ray.direction.cross(edge2);
        let det = edge1.dot(pvec);

        // Если определитель близок к нулю, луч параллелен плоскости треугольника
        if det.abs() < epsilon {
            return None;
        }

        let inv_det = 1.0 / det;

        // Вычисляем расстояние от первой вершины до начала луча
        let tvec = ray.origin - self.p[0];

        // Вычисляем барицентрическую координату u и проверяем границы
        let u = tvec.dot(pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        // Вычисляем вектор qvec
        let qvec = tvec.cross(edge1);

        // Вычисляем барицентрическую координату v и проверяем границы
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // Вычисляем t — дистанцию вдоль луча до точки пересечения
        let t = edge2.dot(qvec) * inv_det;

        // Проверяем, что пересечение находится перед лучом
        if t > epsilon {
            let position = ray.origin + ray.direction * t;

            Some(HitData {
                position,
                t,
                invert_normal: false,
                // Для треугольника кэш можно оставить пустым, так как u и v
                // в базовом коде книги больше нигде не используются
                cache: HitCache::Triangle,
                object_index: 0,
            })
        } else {
            None
        }
    }

    /// Получить параметры поверхности треугольника в точке пересечения
    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        SurfaceData {
            position: hit_data.position,
            // Используем предрассчитанную геометрическую нормаль треугольника
            normal: self.normal,
            material: self.def_surface,
        }
    }
}

/// Выровненная по осям координат коробка (Axis-Aligned Bounding Box)
#[derive(Debug, Clone)]
pub struct Aabb {
    /// Минимальная точка коробки (ближний нижний левый угол)
    pub min: Vec3,
    /// Максимальная точка коробки (дальний верхний правый угол)
    pub max: Vec3,
    /// Дефолтные свойства материала коробки
    pub def_surface: MaterialProps,
}

impl Aabb {
    /// Создать новую коробку по двум противоположным точкам
    pub fn new(p1: Vec3, p2: Vec3, def_surface: MaterialProps) -> Self {
        Self {
            // Гарантируем, что min действительно содержит минимальные компоненты, а max — максимальные
            min: Vec3::new(
                f32::min(p1.x, p2.x),
                f32::min(p1.y, p2.y),
                f32::min(p1.z, p2.z),
            ),
            max: Vec3::new(
                f32::max(p1.x, p2.x),
                f32::max(p1.y, p2.y),
                f32::max(p1.z, p2.z),
            ),
            def_surface,
        }
    }
}

impl Object for Aabb {
    /// Проверить пересечение луча с коробкой методом Slabs
    fn intersect(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

        // Используем инвертированные компоненты направления направления луча для исключения деления на ноль.
        // Если направление по оси равно 0, Rust вернет Infinity, что корректно отработает в сравнениях.
        let inv_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        // Расчет пересечений по оси X
        let mut tmin = (self.min.x - ray.origin.x) * inv_dir.x;
        let mut tmax = (self.max.x - ray.origin.x) * inv_dir.x;
        if inv_dir.x < 0.0 {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        // Расчет пересечений по оси Y
        let mut tymin = (self.min.y - ray.origin.y) * inv_dir.y;
        let mut tymax = (self.max.y - ray.origin.y) * inv_dir.y;
        if inv_dir.y < 0.0 {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        // Проверяем, пересекаются ли интервалы X и Y
        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        // Расчет пересечений по оси Z
        let mut tzmin = (self.min.z - ray.origin.z) * inv_dir.z;
        let mut tzmax = (self.max.z - ray.origin.z) * inv_dir.z;
        if inv_dir.z < 0.0 {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        // Проверяем, пересекается ли интервал Z с накопленным интервалом
        if (tmin > tzmax) || (tzmin > tmax) {
            return None;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        // tmin — это точка входа в коробку, tmax — точка выхода.
        // Выбираем правильное расстояние перед лучом.
        let mut final_t = tmin;
        if final_t < epsilon {
            final_t = tmax; // Если мы внутри коробки, то точка пересечения — это выход из неё
            if final_t < epsilon {
                return None; // Коробка осталась позади луча
            }
        }

        let position = ray.origin + ray.direction * final_t;

        Some(HitData {
            position,
            t: final_t,
            invert_normal: false,
            cache: HitCache::Aabb,
            object_index: 0,
        })
    }

    /// Получить нормаль к коробке в точке пересечения
    fn get_surface(&self, hit_data: &HitData) -> SurfaceData {
        let p = hit_data.position;
        let mut normal = Vec3::ZERO;
        let bias = 1e-4; // Маленькое смещение для определения, на какой грани лежит точка

        // Определяем, к какой из 6 граней коробки ближе всего находится точка пересечения
        if f32::abs(p.x - self.min.x) < bias {
            normal.x = -1.0;
        } else if f32::abs(p.x - self.max.x) < bias {
            normal.x = 1.0;
        } else if f32::abs(p.y - self.min.y) < bias {
            normal.y = -1.0;
        } else if f32::abs(p.y - self.max.y) < bias {
            normal.y = 1.0;
        } else if f32::abs(p.z - self.min.z) < bias {
            normal.z = -1.0;
        } else if f32::abs(p.z - self.max.z) < bias {
            normal.z = 1.0;
        } else {
            // На всякий случай дефолтная нормаль, если из-за погрешности f32 точка «зависла»
            normal.y = 1.0;
        }

        SurfaceData {
            position: hit_data.position,
            normal: normal.normalize(),
            material: self.def_surface,
        }
    }
}
