use crate::{
    math::Vector3,
    raytracer::{image::Color, raytrace::Ray},
};

use super::intersections::IntersectionInfo;

pub trait Scatter {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)>;
}

pub enum Material {
    Lambertian(LambertianMaterial),
    Metal(MetalMaterial),
    Dieletrics(DielectricsMaterial),
    //Texture,
    Light(LightMaterial),
}

impl Scatter for Material {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        match self {
            Material::Lambertian(l) => l.scatter(ray, intersection),
            Material::Metal(m) => m.scatter(ray, intersection),
            Material::Dieletrics(d) => d.scatter(ray, intersection),
            Material::Light(l) => l.scatter(ray, intersection),
        }
    }
}

pub struct LambertianMaterial {
    albedo: Color,
}

impl Scatter for LambertianMaterial {
    fn scatter(&self, _ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let mut scatter_direction = intersection.normal + Vector3::random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = intersection.normal;
        }

        let scattered = Ray::new(intersection.point, scatter_direction);

        let attentuation = self.albedo;

        Some((Some(scattered), attentuation))
    }
}

pub struct LightMaterial {
    color: Color,
}

impl Scatter for LightMaterial {
    fn scatter(
        &self,
        _ray: &Ray,
        _intersection: &IntersectionInfo,
    ) -> Option<(Option<Ray>, Color)> {
        return Some((None, self.color));
    }
}

pub struct DielectricsMaterial {
    tint: Color,
    refraction_index: f64,
}

impl DielectricsMaterial {
    fn refract(&self, direction: &Vector3, normal: &Vector3, refraction_ratio: f64) -> Vector3 {
        let cos_theta = f64::min(normal.dot(&-*direction), 1.0);
        let r_out_perp = (*direction + *normal * cos_theta) * refraction_ratio;
        let r_out_parallel = *normal * -f64::sqrt(f64::abs(1.0 - r_out_perp.sqr_len()));
        return r_out_perp + r_out_parallel;
    }
}

impl Scatter for DielectricsMaterial {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let attentuation = self.tint;
        let refraction_ratio = 1.0 / self.refraction_index; // intersection.front_face ? 1.0 / ir : ir
        let unit_direction = ray.direction.normalized();
        let refracted = self.refract(&unit_direction, &intersection.normal, refraction_ratio);

        let scattered = Ray::new(intersection.point, refracted);

        Some((Some(scattered), attentuation))
    }
}

pub struct MetalMaterial {
    albedo: Color,
    fuzziness: f64,
}

impl Scatter for MetalMaterial {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let reflected = ray.direction.normalized().reflect(&intersection.normal);
        let scattered = Ray::new(
            intersection.point,
            reflected + Vector3::random_in_unit_sphere() * self.fuzziness,
        );
        let attentuation = self.albedo;

        if scattered.direction.dot(&intersection.normal) > 0.0 {
            return Some((Some(scattered), attentuation));
        }

        None
    }
}
