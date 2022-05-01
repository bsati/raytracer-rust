use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Deserializer};

use crate::{
    math::Vector3,
    raytracer::{
        image::Color,
        mesh::{self, Mesh},
        raytrace::Ray,
    },
};

use super::intersections::{Intersectable, IntersectionInfo};

#[derive(Deserialize)]
pub struct SceneConfig {
    pub image: ImageConfig,
    pub camera: CameraConfig,
    pub scene: Scene,
}

#[derive(Deserialize)]
pub struct ImageConfig {
    pub width: usize,
    pub height: usize,
    pub background: Color,
}

#[derive(Deserialize)]
pub struct CameraConfig {
    pub eye: Vector3,
    pub look_at: Vector3,
    pub up: Vector3,
    pub fovy: f64,
}

#[derive(Deserialize)]
pub struct Scene {
    pub ambient_light: Color,
    lights: Vec<Light>,
    pub objects: Vec<Object>,
}

impl Scene {
    /// Returns the closest intersection of the ray with an object of the scene if there is any.
    ///
    /// # Arguments
    ///
    /// * `ray` the ray for which to check intersections
    pub fn get_closest_interesection(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let mut info: Option<IntersectionInfo> = None;

        for o in &self.objects {
            let intersection = o.intersect(ray);
            if let Some(intersection_info) = intersection {
                match info {
                    Some(i) => {
                        if i.t > intersection_info.t {
                            info = Some(intersection_info)
                        }
                    }
                    None => info = Some(intersection_info),
                }
            }
        }
        info
    }

    /// Returns whether a given point should be colored with diffuse and specular color.
    ///
    /// Depends on whether the point is being shadowed by another object or not.
    /// For a light `l` and point `p` the ray is constructed as `origin = p` and `direction = ||l.position - p||`.
    /// If `p` is being shadowed there has to be an intersection `i` with object `o` where `||l.position - p|| > ||l.position - i.position||`
    ///
    /// # Arguments
    ///
    /// * `point` the point to check
    /// * `lp_vec` vector from point to light
    /// * `lp_vec_normalized` `lp_vec` normalized
    #[inline]
    fn should_color(&self, point: &Vector3, lp_vec: &Vector3, lp_vec_normalized: &Vector3) -> bool {
        let ray = Ray::new(*point, *lp_vec_normalized);
        let shadow_intersection = self.get_closest_interesection(&ray);
        match shadow_intersection {
            Some(info) => {
                let len = (info.point - *point).sqr_len();
                len < 1e-4 || len > lp_vec.sqr_len()
            }
            None => true,
        }
    }

    /// Computes the color of a point on an object from the given view via the Phong Lighting Model.
    ///
    /// # Arguments
    ///
    /// * `point` Point in space to calculate the color for
    /// * `normal` Normal of the object intersection
    /// * `view` Position where the point / object is being viewed from
    /// * `material` Material of the hit object
    pub fn compute_phong_lighting(
        &self,
        point: &Vector3,
        normal: &Vector3,
        view: &Vector3,
        material: &Material,
    ) -> Color {
        let mut c = material.ambient_color * self.ambient_light;

        for l in &self.lights {
            let mut l_color = l
                .samples
                .par_iter()
                .map(|l_vec| {
                    let mut light_color = Color::new(0.0, 0.0, 0.0);
                    let lp_vec = *l_vec - *point;
                    let lp_vec_normalized = lp_vec.normalized();
                    if self.should_color(point, &lp_vec, &lp_vec_normalized) {
                        let r = lp_vec_normalized.mirror(normal);
                        let dot_l = normal.dot(&lp_vec_normalized);
                        if dot_l >= 0.0 {
                            light_color += l.color * (material.diffuse_color * dot_l);

                            let dot_r = view.dot(&r);
                            if dot_r >= 0.0 {
                                let shininess = dot_r.powf(material.shininess);
                                light_color += material.specular_color * l.color * shininess;
                            }
                        }
                    }

                    light_color
                })
                .reduce(|| Color::new(0.0, 0.0, 0.0), |a, b| a + b);
            l_color /= l.samples.len() as f64;
            c += l_color;
        }

        c
    }

    pub fn precompute(&mut self) {
        for l in &mut self.lights {
            l.compute_samples();
        }
        for m in &mut self.objects {
            if let Object::Mesh(mesh) = m {
                mesh.compute_aabb();
            }
        }
    }
}

#[derive(Deserialize)]
struct Light {
    #[serde(skip_deserializing)]
    samples: Vec<Vector3>,
    color: Color,
    light_info: LightInfo,
}

impl Light {
    fn compute_samples(&mut self) {
        self.samples = self.light_info.sample();
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum LightInfo {
    Point(PointLight),
    Area(AreaLight),
    Sphere(SphereLight),
}

impl LightInfo {
    fn sample(&self) -> Vec<Vector3> {
        match self {
            LightInfo::Point(pl) => vec![pl.position],
            LightInfo::Area(area_light) => {
                let resolution = area_light.grid_resolution;
                let mut result = Vec::with_capacity(resolution * resolution);
                for i in 0..resolution {
                    for j in 0..resolution {
                        result.push(
                            area_light.corner
                                + (area_light.u / i as f64)
                                + (area_light.v / j as f64),
                        );
                    }
                }
                result
            }
            LightInfo::Sphere(sphere_light) => {
                vec![]
            }
        }
    }
}

#[derive(Deserialize)]
struct PointLight {
    position: Vector3,
}

#[derive(Deserialize)]
struct AreaLight {
    corner: Vector3,
    u: Vector3,
    v: Vector3,
    grid_resolution: usize,
}

#[derive(Deserialize)]
struct SphereLight {}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Object {
    Sphere(Sphere),
    Plane(Plane),
    Mesh(Mesh),
}

impl Intersectable for Object {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        match self {
            Object::Sphere(sphere) => sphere.intersect(ray),
            Object::Plane(plane) => plane.intersect(ray),
            Object::Mesh(mesh) => mesh.intersect(ray),
        }
    }
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: serde_yaml::Value = serde_yaml::Value::deserialize(deserializer).unwrap();
        let path = std::path::Path::new(val.get("path").unwrap().as_str().unwrap());
        let meshes = mesh::load_obj(path);
        Ok(meshes[0].to_owned())
    }
}

#[derive(Deserialize)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Material,
}

#[derive(Deserialize)]
pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
    pub material: Material,
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Material {
    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub shininess: f64,
    pub mirror: f64,
}

impl Material {
    pub fn default() -> Material {
        Material {
            ambient_color: Color::default(),
            diffuse_color: Color::default(),
            specular_color: Color::default(),
            shininess: -1.0,
            mirror: 0.0,
        }
    }
}
