//! Сцена

use super::super::geometry::Ray3d;
use super::light::{BackgroundLight, LightSource, SolidBackgroundLight};
use super::material::AIR;
use super::material::{OpticalMedium, SurfaceData};
use super::object::{HitData, Object};
use palette::Srgb;

const MAX_DEPTH: i32 = 7;

/// Сцена
pub struct Scene {
    /// Источники света
    lights: Vec<LightSource>,
    /// Объекты на сцене
    objects: Vec<Box<dyn Object + Sync + Send>>,
    /// Настройки фонового освещения
    background: Box<dyn BackgroundLight + Sync + Send>,
    /// Цвет фонового освещения
    ambient: Srgb<f32>,
}

impl Default for Scene {
    /// Вручную задаем начальное состояние сцены
    fn default() -> Self {
        Self {
            lights: Vec::new(),
            objects: Vec::new(),
            ambient: Srgb::new(0.2, 0.2, 0.2),
            // Задаем дефолтный сплошной цвет фона (например, ваш оригинальный 0.0, 0.5, 0.5)
            background: Box::new(SolidBackgroundLight::new(Srgb::new(0.0, 0.5, 0.5))),
        }
    }
}

impl Scene {
    /// Добавить объект на сцену
    pub fn add_object(&mut self, object: Box<dyn Object + Sync + Send>) {
        self.objects.push(object);
    }

    /// Добавить источник освещения на сцену
    pub fn add_light(&mut self, light: LightSource) {
        self.lights.push(light);
    }

    pub fn intersect(&self, r: &Ray3d, maybe_epsilon: Option<f32>) -> Option<HitData> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, obj)| {
                obj.intersect(r, maybe_epsilon).map(|mut hit| {
                    hit.object_index = index;
                    hit
                })
            })
            // Безопасное сравнение float через partial_cmp, игнорирующее некорректные значения
            .min_by(|hit_a, hit_b| {
                hit_a
                    .t
                    .partial_cmp(&hit_b.t)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn trace(
        &self,
        cur_medium: OpticalMedium,
        weight: f32,
        depth: i32,
        r: &Ray3d,
        maybe_epsilon: Option<f32>,
    ) -> Srgb<f32> {
        if depth > MAX_DEPTH {
            return self.get_background(r);
        }

        if let Some(hit) = self.intersect(r, maybe_epsilon) {
            // Безопасно получаем объект по индексу из вектора сцены
            let target_object = &self.objects[hit.object_index];
            let mut surface = target_object.get_surface(&hit);

            let mut color = self.shade(
                cur_medium,
                weight,
                depth,
                r,
                &hit,
                &mut surface,
                maybe_epsilon,
            );

            if cur_medium.absorption > 1e-4 {
                color = color * f32::exp(-hit.t * cur_medium.absorption);
            }
            color
        } else {
            self.get_background(r)
        }
    }

    pub fn shade(
        &self,
        cur_medium: OpticalMedium,
        weight: f32,
        depth: i32,
        r: &Ray3d,
        hit: &HitData,
        surface: &mut SurfaceData,
        maybe_epsilon: Option<f32>,
    ) -> Srgb<f32> {
        // Вектор направления луча взгляда (уже нормализован в камере)
        let v = r.direction;

        // Направление нормали. Если луч заходит изнутри объекта, оригинальный код инвертирует нормаль
        if hit.invert_normal {
            surface.normal = -surface.normal;
        }

        let mut vn = v.dot(surface.normal);

        // Если угол между лучом взгляда и нормалью острый, значит мы смотрим на внутреннюю сторону
        let mut entering = true;
        if vn > 0.0 {
            surface.normal = -surface.normal;
            vn = -vn;
            entering = false;
        }

        // 1. Считаем Ambient (фоновое) освещение: ambient * surface_color * ka
        // В palette умножение цвета на цвет и цвета на число перегружено
        let mut final_color =
            self.ambient * surface.material.color * surface.material.ambient_coeff;

        // Добавляем собственное свечение материала (emission)
        final_color = final_color + surface.material.emission;

        // 2. Цикл по источникам света (Прямое освещение)
        for light in &self.lights {
            // Запрашиваем тень и направление к источнику света
            // Передаем self (ссылку на Scene), точку пересечения и None для дефолтного epsilon
            let (shadow_factor, l) = light.shadow(self, hit.position, None);

            if shadow_factor > 1e-4 {
                let ln = l.dot(surface.normal);

                // Если источник света находится с лицевой стороны поверхности
                if ln > 1e-4 {
                    // Диффузная составляющая (Diffuse)
                    if surface.material.diffuse_coeff > 1e-4 {
                        let diffuse = light.color
                            * surface.material.color
                            * (surface.material.diffuse_coeff * shadow_factor * ln);
                        final_color = final_color + diffuse;
                    }

                    // Зеркальный блик Блинна-Фонга (Specular)
                    if surface.material.specular_coeff > 1e-4 {
                        // Полувектор между направлением на свет и направлением К камере (-v)
                        let h = (l - v).normalize();
                        let ndoth = f32::max(0.0, surface.normal.dot(h));

                        let specular = light.color
                            * (surface.material.specular_coeff
                                * shadow_factor
                                * ndoth.powf(surface.material.shininess));
                        final_color = final_color + specular;
                    }
                }
            }
        }

        // 3. Идеальное зеркальное отражение (Reflection)
        let r_weight = weight * surface.material.reflection_coeff;
        if r_weight > 1e-4 && depth < MAX_DEPTH {
            // Формула отражения вектора: r = v - 2 * (v . n) * n
            let reflect_dir = v - surface.normal * (2.0 * vn);

            let reflect_ray = Ray3d {
                origin: hit.position,
                direction: reflect_dir.normalize(),
            };

            // Рекурсивный вызов трассировки для отраженного луча
            let reflected_color =
                self.trace(cur_medium, r_weight, depth + 1, &reflect_ray, maybe_epsilon);
            final_color = final_color + reflected_color * surface.material.reflection_coeff;
        }

        // 4. Преломление света / Прозрачность (Transmission)
        let t_weight = weight * surface.material.transmission_coeff;
        if t_weight > 1e-4 && depth < MAX_DEPTH {
            // Относительный коэффициент преломления
            let eta = cur_medium.refraction
                / if entering {
                    surface.material.medium.refraction
                } else {
                    1.0
                };
            let ci = -vn; // Косинус угла падения

            // Квадрат косинуса угла преломления (закон Снеллиуса в векторном виде)
            let ct_sq = 1.0 + eta * eta * (ci * ci - 1.0);

            // Если ct_sq <= 0, происходит Полное Внутреннее Отражение (свет не может выйти)
            if ct_sq > 1e-4 {
                let transmit_dir = v * eta + surface.normal * (eta * ci - f32::sqrt(ct_sq));

                let transmit_ray = Ray3d {
                    origin: hit.position,
                    direction: transmit_dir.normalize(),
                };

                // При входе переходим в материал объекта, при выходе — в воздух.
                let next_medium = if entering {
                    surface.material.medium
                } else {
                    AIR
                };

                let transmitted_color = self.trace(
                    next_medium,
                    t_weight,
                    depth + 1,
                    &transmit_ray,
                    maybe_epsilon,
                );
                final_color = final_color + transmitted_color * surface.material.transmission_coeff;
            } else {
                // При полном внутреннем отражении вся доля transmission отражается.
                let reflect_dir = v - surface.normal * (2.0 * vn);
                let reflect_ray = Ray3d {
                    origin: hit.position,
                    direction: reflect_dir.normalize(),
                };
                let reflected_color =
                    self.trace(cur_medium, t_weight, depth + 1, &reflect_ray, maybe_epsilon);
                final_color = final_color + reflected_color * surface.material.transmission_coeff;
            }
        }

        final_color
    }

    pub fn get_background(&self, ray: &Ray3d) -> Srgb<f32> {
        self.background.get_background(ray)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ray_tracing::material::{GLASS, MaterialProps};
    use glam::Vec3;

    #[test]
    fn total_internal_reflection_redirects_transmitted_energy() {
        let scene = Scene::default();
        let material = MaterialProps {
            color: Srgb::new(0.0, 0.0, 0.0),
            emission: Srgb::new(0.0, 0.0, 0.0),
            ambient_coeff: 0.0,
            diffuse_coeff: 0.0,
            specular_coeff: 0.0,
            reflection_coeff: 0.0,
            transmission_coeff: 1.0,
            medium: GLASS,
            ..MaterialProps::default()
        };
        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Vec3::new(0.9, 0.0, 0.4358899).normalize(),
        };
        let hit = HitData {
            position: Vec3::ZERO,
            normal: Vec3::Z,
            t: 1.0,
            invert_normal: false,
            object_index: 0,
            material,
        };
        let mut surface = SurfaceData {
            position: hit.position,
            normal: hit.normal,
            material,
        };

        let color = scene.shade(GLASS, 1.0, 1, &ray, &hit, &mut surface, None);

        assert!(color.green > 0.4);
        assert!(color.blue > 0.4);
    }
}
