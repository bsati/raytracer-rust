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

pub struct Intersection {
    pub point: Vector3,
    pub normal: Vector3,
    pub diffuse: Vector3,
    pub t: f64,
}

pub trait Intersectable {
    fn intersect(ray: &Ray) -> Option<Intersection>;
}
