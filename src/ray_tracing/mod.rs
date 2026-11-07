//! Пример трассировки лучей из учебника.
//! Возможно, потом часть логики уйдет в модуль "light".

mod camera;
mod csg;
mod light;
mod material;
mod object;
mod scene;

pub use camera::*;
pub use csg::*;
pub use light::*;
pub use material::*;
pub use object::*;
pub use scene::*;

/// Примеры трассировки лучей
pub mod examples {
    use std::io::Result;

    /// Базовый пример. Возвращает отрендеренную камеру, чтобы кадр можно было показать в SDL.
    pub fn base_example() -> Result<super::Camera> {
        use super::{
            Camera, GLASS, LightSource, LightSourceKind, MaterialProps, NewCameraParams, Plane,
            Scene, Sphere,
        };
        use glam::Vec3;
        use palette::Srgb;

        println!("Запуск примера trace-1...");

        // 1. Инициализируем камеру (640x480, FOV 60, позиция (0,0,0), взгляд на (0,0,1))
        let camera_params = NewCameraParams {
            position: Vec3::new(0.0, 0.0, 0.0),
            view: Vec3::new(0.0, 0.0, 1.0), // В книге ось взгляда направлена вперед по Z
            up: Vec3::new(0.0, 1.0, 0.0),
            field_of_view_x: 60.0,
            width: 640,
            height: 480,
        };

        let mut camera = Camera::try_new(camera_params)?;

        // 2. Инициализируем сцену с дефолтными настройками
        let mut scene = Scene::default();

        // 3. Настраиваем базовый материал (для сферы s1: желтый цвет, стеклянная среда)
        let s1_material = MaterialProps {
            color: Srgb::new(1.0, 1.0, 0.0), // Желтый (1, 1, 0)
            emission: Srgb::new(0.0, 0.0, 0.0),
            shininess: 30.0,         // p = 30
            ambient_coeff: 0.2,      // ka = 0.2
            diffuse_coeff: 0.5,      // kd = 0.5
            specular_coeff: 0.6,     // ks = 0.6
            reflection_coeff: 0.3,   // kr = 0.3
            transmission_coeff: 0.0, // kt = 0.0
            medium: GLASS,           // medium = glass
        };

        // Создаем первую сферу (желтую, в точке (0, 1, 8), радиус 1.5)
        let s1 = Sphere::new(Vec3::new(0.0, 1.0, 8.0), 1.5, s1_material);

        // 4. Настраиваем материал для второй сферы на основе первого (копирование структуры)
        let s2_material = MaterialProps {
            color: Srgb::new(1.0, 0.0, 0.0), // Меняем цвет на красный (1, 0, 0)
            reflection_coeff: 0.0,           // kr = 0
            ..s1_material                    // Все остальные коэффициенты копируем из s1
        };

        // Создаем вторую сферу (красную, в точке (-3, 0, 8), радиус 1.0)
        let s2 = Sphere::new(Vec3::new(-3.0, 0.0, 8.0), 1.0, s2_material);

        // 5. Настраиваем материал для плоскости пола p1 на основе s1_material
        let p1_material = MaterialProps {
            color: Srgb::new(0.0, 0.0, 1.0), // Синий цвет (0, 0, 1)
            ambient_coeff: 0.1,              // ka = 0.1
            specular_coeff: 0.4,             // ks = 0.4
            diffuse_coeff: 0.5,              // kd = 0.5
            reflection_coeff: 0.4,           // kr = 0.4
            ..s1_material                    // Остальное без изменений
        };

        // Создаем плоскость пола (нормаль вверх (0,1,0), смещение d = 1)
        let p1 = Plane::new(Vec3::new(0.0, 1.0, 0.0), 1.0, p1_material);

        // 6. Создаем точечный источник света l1 (позиция (10, 5, -10), цвет белый (1,1,1))
        let l1 = LightSource {
            color: Srgb::new(1.0, 1.0, 1.0),
            kind: LightSourceKind::Point {
                position: Vec3::new(10.0, 5.0, -10.0),
            },
        };

        // 7. Добавляем всю заготовленную геометрию и свет на сцену
        scene.add_object(Box::new(s1));
        scene.add_object(Box::new(s2));
        scene.add_object(Box::new(p1));
        scene.add_light(l1);

        camera.trace_frame(&scene, "generated/ray-tracing-base-example.tga", None)?;

        println!(
            "Рендеринг завершен, файл 'generated/ray-tracing-base-example.tga' успешно сохранен."
        );

        Ok(camera)
    }

    /// Пример с CSG (конструктивная сплошная геометрия)
    pub fn csg_example() -> Result<()> {
        use glam::Vec3;
        use palette::Srgb;
        use std::io;
        // Предполагаем, что все типы реэкспортированы из корня вашего крейта
        use super::{
            Camera, CsgObject, CsgOperation, GLASS, LightSource, LightSourceKind, MaterialProps,
            NewCameraParams, Plane, Scene,
        };

        println!("Запуск примера trace-2 (CSG рендеринг)...");

        // 1. Инициализируем камеру (1280x960, FOV 60, позиция (0,0,0), взгляд на (0,0,1))
        let camera_params = NewCameraParams {
            position: Vec3::new(0.0, 0.0, 0.0),
            view: Vec3::new(0.0, 0.0, 1.0), // Взгляд по оси +Z
            up: Vec3::new(0.0, 1.0, 0.0),
            field_of_view_x: 60.0,
            width: 640 * 2,  // 1280
            height: 480 * 2, // 960
        };

        let mut camera = Camera::try_new(camera_params)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // 2. Инициализируем сцену с дефолтными настройками
        let mut scene = Scene::default();

        // 3. Настраиваем базовый материал для первой сферы (желтый, стеклянная среда)
        let s1_material = MaterialProps {
            color: Srgb::new(1.0, 1.0, 0.0), // Желтый (1, 1, 0)
            emission: Srgb::new(0.0, 0.0, 0.0),
            shininess: 50.0,         // p = 50
            ambient_coeff: 0.2,      // ka = 0.2
            diffuse_coeff: 0.5,      // kd = 0.5
            specular_coeff: 0.5,     // ks = 0.5
            reflection_coeff: 0.1,   // kr = 0.1
            transmission_coeff: 0.0, // kt = 0.0
            medium: GLASS,           // medium = glass
        };

        // Материал для второй сферы (красный, остальные свойства как у первой)
        let s2_material = MaterialProps {
            color: Srgb::new(1.0, 0.0, 0.0), // Красный (1, 0, 0)
            ..s1_material
        };

        // 4. Создаем композитный CSG объект: s1 - s2 (вычитание сфер)
        // В Rust мы просто конструируем вложенную структуру дерева
        let csg_object = CsgObject::Composite {
            op1: Box::new(CsgObject::Sphere {
                center: Vec3::new(-1.0, 1.0, 10.0),
                radius: 2.0,
                radius_sq: 2.0 * 2.0,
                material: s1_material,
            }),
            op2: Box::new(CsgObject::Sphere {
                center: Vec3::new(0.0, 1.0, 9.5),
                radius: 2.1,
                radius_sq: 2.1 * 2.1,
                material: s2_material,
            }),
            operation: CsgOperation::Sub, // Операция вычитания
        };

        // 5. Настраиваем материал для плоскости пола p1
        let p1_material = MaterialProps {
            color: Srgb::new(0.0, 0.0, 1.0), // Синий цвет (0, 0, 1)
            ambient_coeff: 0.1,              // ka = 0.1
            specular_coeff: 0.4,             // ks = 0.4
            diffuse_coeff: 0.5,              // kd = 0.5
            reflection_coeff: 0.4,           // kr = 0.4
            ..s1_material
        };

        // Создаем плоскость пола (нормаль вверх, смещение d = 1)
        let p1 = Plane::new(Vec3::new(0.0, 1.0, 0.0), 1.0, p1_material);

        // 6. Создаем точечный источник света l1 (позиция (10, 5, -10))
        let l1 = LightSource {
            color: Srgb::new(1.0, 1.0, 1.0),
            kind: LightSourceKind::Point {
                position: Vec3::new(10.0, 5.0, -10.0),
            },
        };

        // 7. Добавляем объекты и свет на сцену
        // Трейт Object реализован для CsgObject, поэтому он идеально заходит в вектор сцены
        scene.add_object(Box::new(csg_object));
        scene.add_object(Box::new(p1));
        scene.add_light(l1);

        // 8. Запускаем параллельный рендеринг кадра
        camera.trace_frame(&scene, "generated/ray-tracing-csg-example.tga", None)?;

        println!(
            "Рендеринг CSG завершен, файл 'generated/ray-tracing-csg-example.tga' успешно сохранен."
        );
        Ok(())
    }
}
