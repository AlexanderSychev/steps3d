use super::axis_aligned_bounding_box_3d::AxisAlignedBoundingBox3d;
use super::oriented_bounding_box::OrientedBoundingBox;
use super::ray_3d::Ray3d;
use super::sphere::Sphere;
use super::triangle_3d::Triangle3d;

/// Объект, вокруг которого можно построить ограничивающий AABB.
pub trait Bounded {
    /// Вернуть плотный AABB объекта в мировых координатах.
    fn bounds(&self) -> AxisAlignedBoundingBox3d;
}

impl Bounded for AxisAlignedBoundingBox3d {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        *self
    }
}

impl Bounded for Sphere {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        let radius = glam::Vec3::splat(self.radius);
        AxisAlignedBoundingBox3d::new(self.center - radius, self.center + radius)
    }
}

impl Bounded for Triangle3d {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        AxisAlignedBoundingBox3d::from_vertices(&[self.a, self.b, self.c])
    }
}

impl Bounded for OrientedBoundingBox {
    fn bounds(&self) -> AxisAlignedBoundingBox3d {
        // Полуразмер AABB по каждой мировой оси — сумма модулей проекций
        // локальных полуосей OBB на эту ось.
        let half_extents = self.orientation.x_axis.abs() * self.extents.x
            + self.orientation.y_axis.abs() * self.extents.y
            + self.orientation.z_axis.abs() * self.extents.z;

        AxisAlignedBoundingBox3d::new(self.center - half_extents, self.center + half_extents)
    }
}

/// Объект, с которым можно найти пересечение луча.
pub trait RayIntersectable {
    /// Вернуть расстояние `t` вдоль луча до ближайшей точки пересечения
    /// или `None`, если луч не пересекает объект.
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32>;
}

impl RayIntersectable for AxisAlignedBoundingBox3d {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        let (t_enter, t_exit) = self.ray_intersection_interval(ray, maybe_epsilon)?;

        (t_exit >= 0.0).then(|| t_enter.max(0.0))
    }
}

impl RayIntersectable for Sphere {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        Sphere::ray_intersection(self, ray, maybe_epsilon)
    }
}

impl RayIntersectable for Triangle3d {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        self.get_intersection_barycentric(ray, maybe_epsilon)
            .map(|intersection| intersection.t)
    }
}

impl RayIntersectable for OrientedBoundingBox {
    fn ray_intersection(&self, ray: &Ray3d, maybe_epsilon: Option<f32>) -> Option<f32> {
        OrientedBoundingBox::ray_intersection(self, ray, maybe_epsilon)
    }
}
