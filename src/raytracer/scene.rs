use crate::math::Vector3;
use crate::raytracer::image::Color;
use crate::raytracer::raytrace::Ray;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SceneConfig {
    pub image: ImageConfig,
    pub camera: CameraConfig,
    pub scene: Scene,
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
pub struct Scene {
    pub ambient_light: Color,
    pub lights: Vec<Light>,
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
            let lp_vec = l.position - *point;
            let lp_vec_normalized = lp_vec.normalized();
            if self.should_color(point, &lp_vec, &lp_vec_normalized) {
                let r = lp_vec_normalized.mirror(normal);
                let dot_l = normal.dot(&lp_vec_normalized);
                if dot_l >= 0.0 {
                    c += l.color * (material.diffuse_color * dot_l);

                    let dot_r = view.dot(&r);
                    if dot_r >= 0.0 {
                        let shininess = dot_r.powf(material.shininess);
                        c += material.specular_color * l.color * shininess;
                    }
                }
            }
        }

        c
    }
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

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Material {
    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub shininess: f64,
    pub mirror: f64,
}

/// Information about a ray-object intersection.
/// Contains the intersection point, normal, material of the intersected object and the `t` for which the intersection occurs.
#[derive(Clone, Copy)]
pub struct IntersectionInfo {
    pub point: Vector3,
    pub normal: Vector3,
    pub material: Material,
    pub t: f64,
}

impl IntersectionInfo {
    fn new(point: Vector3, normal: Vector3, material: Material, t: f64) -> IntersectionInfo {
        IntersectionInfo {
            point: point,
            normal: normal,
            material: material,
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
                self.material,
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
            self.material,
            intersection_t,
        ))
    }
}
