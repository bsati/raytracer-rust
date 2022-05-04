use rand::Rng;
use serde::Deserialize;

use crate::{
    math::Vector3,
    raytracer::{image::Color, raytrace::Ray},
};

use super::intersections::IntersectionInfo;

/// Trait for all Materials to provide Scattering for raytracing.
/// Implementing materials can either return a new so called scattered ray that bounces from the intersection point
/// and / or a color value for the intersection point.
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
}

impl LambertianMaterial {
    #[cfg(test)]
    fn new(albedo: Color) -> LambertianMaterial {
        LambertianMaterial { albedo }
    }
}

impl Scatter for LambertianMaterial {
    fn scatter(&self, _ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let mut scatter_direction = intersection.normal + Vector3::random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = intersection.normal;
        }

        let scattered = Ray::new(intersection.point, scatter_direction);

        let attenuation = self.albedo;

        Some((Some(scattered), attenuation))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmissiveMaterial {
    pub color: Color,
}

impl EmissiveMaterial {
    #[cfg(test)]
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
    #[cfg(test)]
    fn new(tint: Color, refraction_index: f64) -> DielectricsMaterial {
        DielectricsMaterial {
            tint,
            refraction_index,
        }
    }

    fn refract(direction: &Vector3, normal: &Vector3, refraction_ratio: f64) -> Vector3 {
        let cos_theta = normal.dot(&-*direction).min(1.0);
        let r_out_perp = (*direction + *normal * cos_theta) * refraction_ratio;
        let r_out_parallel = *normal * -(1.0 - r_out_perp.sqr_len()).abs().sqrt();
        r_out_perp + r_out_parallel
    }

    #[inline]
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Scatter for DielectricsMaterial {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let attenuatin = self.tint;
        let front_face = intersection.normal.dot(&ray.direction) <= 0.0;
        let refraction_ratio = if front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };
        let normal = if front_face {
            intersection.normal
        } else {
            -intersection.normal
        };
        let unit_direction = ray.direction;

        let cos_theta = f64::min((-unit_direction).dot(&normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let mut rng = rand::thread_rng();

        let mut direction =
            DielectricsMaterial::refract(&unit_direction, &normal, refraction_ratio);
        if cannot_refract
            || DielectricsMaterial::reflectance(cos_theta, refraction_ratio) > rng.gen::<f64>()
        {
            direction = unit_direction.reflect(&normal);
        }

        let scattered = Ray::new(intersection.point, direction);

        Some((Some(scattered), attenuatin))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetalMaterial {
    albedo: Color,
    fuzziness: f64,
}

impl MetalMaterial {
    #[cfg(test)]
    fn new(albedo: Color, fuzziness: f64) -> MetalMaterial {
        MetalMaterial { albedo, fuzziness }
    }
}

impl Scatter for MetalMaterial {
    fn scatter(&self, ray: &Ray, intersection: &IntersectionInfo) -> Option<(Option<Ray>, Color)> {
        let reflected = ray.direction.normalized().reflect(&intersection.normal);
        let scattered = Ray::new(
            intersection.point,
            reflected + Vector3::random_in_unit_sphere() * self.fuzziness,
        );
        let attenuation = self.albedo;

        if scattered.direction.dot(&intersection.normal) > 0.0 {
            return Some((Some(scattered), attenuation));
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::{
        math::Vector3,
        raytracer::{
            image::Color,
            raytrace::Ray,
            scene::{
                intersections::IntersectionInfo,
                materials::{LambertianMaterial, MetalMaterial},
            },
        },
    };

    use super::{DielectricsMaterial, EmissiveMaterial, Material, Scatter};

    #[test]
    fn test_refract() {
        let v = Vector3::new(1.0, 0.0, 0.0);
        let n = Vector3::new(0.0, 1.0, 0.0);
        let idx = 1.0;

        let refracted = DielectricsMaterial::refract(&v, &n, idx);

        assert_eq!(refracted, v);
    }

    #[test]
    fn test_reflectance() {
        let reflectance = DielectricsMaterial::reflectance(0.0, 1.0);

        assert_eq!(reflectance, 1.0);
    }

    #[test]
    fn test_emissive_scatter() {
        let material = EmissiveMaterial::new(Color::new(1.0, 0.5, 0.0));
        let mat_wrapper = Material::Emissive(material.clone());

        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let intersection = IntersectionInfo::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            &mat_wrapper,
            0.0,
        );

        let result = material.scatter(&ray, &intersection);
        assert!(result.is_some());
        if let Some((r, c)) = result {
            assert!(r.is_none());
            assert_eq!(c, Color::new(1.0, 0.5, 0.0));
        }
    }

    #[test]
    fn test_lambertian_scatter() {
        let material = LambertianMaterial::new(Color::new(1.0, 0.5, 0.0));
        let mat_wrapper = Material::Lambertian(material.clone());

        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let intersection = IntersectionInfo::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            &mat_wrapper,
            0.0,
        );

        let result = material.scatter(&ray, &intersection);
        assert!(result.is_some());
        if let Some((r, c)) = result {
            assert!(r.is_some());
            assert_eq!(c, Color::new(1.0, 0.5, 0.0));
        }
    }

    #[test]
    fn test_metal_scatter() {
        let material = MetalMaterial::new(Color::new(1.0, 0.5, 0.0), 0.5);
        let mat_wrapper = Material::Metal(material.clone());

        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let intersection = IntersectionInfo::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            &mat_wrapper,
            0.0,
        );

        let result = material.scatter(&ray, &intersection);
        assert!(result.is_some());
        if let Some((r, c)) = result {
            assert!(r.is_some());
            assert_eq!(c, Color::new(1.0, 0.5, 0.0));
        }
    }

    #[test]
    fn test_dieletrics_scatter() {
        let material = DielectricsMaterial::new(Color::new(1.0, 0.5, 0.0), 0.5);
        let mat_wrapper = Material::Dieletrics(material.clone());

        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        let intersection = IntersectionInfo::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            &mat_wrapper,
            0.0,
        );

        let result = material.scatter(&ray, &intersection);
        assert!(result.is_some());
        if let Some((r, c)) = result {
            assert!(r.is_some());
            assert_eq!(c, Color::new(1.0, 0.5, 0.0));
        }
    }
}
