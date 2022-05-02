use rand::Rng;
use serde::Deserialize;

use crate::{
    math::Vector3,
    raytracer::{image::Color, raytrace::Ray},
};

use super::intersections::IntersectionInfo;

pub trait Scatter {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)>;
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Material {
    Lambertian(LambertianMaterial),
    Metal(MetalMaterial),
    Dieletrics(DielectricsMaterial),
    //Texture,
    Emissive(EmissiveMaterial),
}

impl Scatter for Material {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        match self {
            Material::Lambertian(l) => l.scatter(ray, intersection),
            Material::Metal(m) => m.scatter(ray, intersection),
            Material::Dieletrics(d) => d.scatter(ray, intersection),
            Material::Emissive(l) => l.scatter(ray, intersection),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LambertianMaterial {
    albedo: Color,
    roughness: f64,
}

impl LambertianMaterial {
    pub fn new(albedo: Color, roughness: f64) -> LambertianMaterial {
        LambertianMaterial { albedo, roughness }
    }
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

#[derive(Clone, Debug, Deserialize)]
pub struct EmissiveMaterial {
    pub color: Color,
}

impl EmissiveMaterial {
    pub fn new(color: Color) -> EmissiveMaterial {
        EmissiveMaterial { color }
    }
}

impl Scatter for EmissiveMaterial {
    fn scatter(
        &self,
        _ray: &Ray,
        _intersection: &IntersectionInfo,
    ) -> Option<(Option<Ray>, Color)> {
        return Some((None, self.color));
    }
}

#[derive(Clone, Debug, Deserialize)]
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

    #[inline]
    fn reflectance(&self, cosine: f64, ref_idx: f64) -> f64 {
        let r0 = f64::powf((1.0 - ref_idx) / (1.0 + ref_idx), 2.0);
        r0 + (1.0 - r0) * f64::powf(1.0 - cosine, 5.0)
    }
}

impl Scatter for DielectricsMaterial {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let attentuation = self.tint;
        let refraction_ratio = 1.0 / self.refraction_index; // intersection.front_face ? 1.0 / ir : ir
        let unit_direction = ray.direction.normalized();

        let cos_theta = f64::min((-unit_direction).dot(&intersection.normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let mut rng = rand::thread_rng();

        let mut direction = self.refract(&unit_direction, &intersection.normal, refraction_ratio);
        if cannot_refract || self.reflectance(cos_theta, refraction_ratio) > rng.gen_range(0.0..1.0)
        {
            direction = unit_direction.reflect(&intersection.normal);
        }

        let scattered = Ray::new(intersection.point, direction);

        Some((Some(scattered), attentuation))
    }
}

#[derive(Clone, Debug, Deserialize)]
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
