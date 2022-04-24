use crate::math::Vector3;
use crate::raytracer::image::Color;
use crate::raytracer::raytrace::Ray;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub image: ImageConfig,
    pub camera: CameraConfig,
    pub scene: SceneConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ImageConfig {
    pub width: usize,
    pub height: usize,
    pub background: Color,
}

#[derive(Serialize, Deserialize)]
pub struct CameraConfig {
    pub eye: Vector3,
    pub look_at: Vector3,
    pub up: Vector3,
    pub fovy: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SceneConfig {
    pub ambient_light: Color,
    pub lights: Vec<Light>,
    pub objects: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
pub struct Light {
    pub position: Vector3,
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Object {
    Sphere(Sphere),
    Plane(Plane),
}

#[derive(Serialize, Deserialize)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Material,
}

#[derive(Serialize, Deserialize)]
pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
    pub material: Material,
}

#[derive(Serialize, Deserialize)]
pub struct Material {
    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub shininess: f64,
    pub mirror: f64,
}

/// Information about a ray-object intersection.
/// Contains the intersection point, normal, diffuse color and the `t` for which the intersection occurs.
pub struct IntersectionInfo {
    pub point: Vector3,
    pub normal: Vector3,
    pub diffuse: Color,
    pub t: f64,
}

impl IntersectionInfo {
    fn new(point: Vector3, normal: Vector3, color: Color, t: f64) -> IntersectionInfo {
        IntersectionInfo {
            point: point,
            normal: normal,
            diffuse: color,
            t: t,
        }
    }
}

pub trait Intersectable {
    /// Checks if the ray intersects the object and returns the corresponding `IntersectionInfo` if it does
    /// or `None` otherwise
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo>;
}

impl Intersectable for Object {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        match self {
            Object::Sphere(sphere) => sphere.intersect(ray),
            Object::Plane(plane) => plane.intersect(ray),
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let dir = ray.direction;
        let oc = ray.origin - self.center;

        let a = dir.sqr_len();
        let b = 2.0 * dir.dot(&oc);
        let c = oc.sqr_len() - self.radius * self.radius;
        let mut d = b * b - 4.0 * a * c;

        // Check for intersection
        if d >= 0.0 {
            d = f64::sqrt(d);

            let t1 = (-b - d) / (2.0 * a);
            let t2 = (-b + d) / (2.0 * a);

            let mut intersection_t = f64::MAX;
            if t1 > 1e-5 && t1 < intersection_t {
                intersection_t = t1;
            }
            if t2 > 1e-5 && t2 < intersection_t {
                intersection_t = t2;
            }

            if intersection_t == f64::MAX {
                return None;
            }
            let intersection_point = ray.at_timestep(intersection_t);
            let intersection_normal = (intersection_point - self.center) / self.radius;

            return Some(IntersectionInfo::new(
                intersection_point,
                intersection_normal,
                self.material.diffuse_color,
                intersection_t,
            ));
        }
        None
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let dot_nd = self.normal.dot(&ray.direction);
        if f64::abs(dot_nd) < 1e-6 {
            return None;
        }

        let intersection_t = (self.center - ray.origin).dot(&self.normal) / dot_nd;

        if intersection_t < 1e-5 {
            return None;
        }

        let intersection_point = ray.at_timestep(intersection_t);
        let intersection_normal = self.normal;
        Some(IntersectionInfo::new(
            intersection_point,
            intersection_normal,
            self.material.diffuse_color,
            intersection_t,
        ))
    }
}
