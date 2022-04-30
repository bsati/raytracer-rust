use crate::math::Vector3;
use serde::{Deserialize, Deserializer};

use super::{
    image::Color,
    mesh::{load_obj, Mesh},
    raytrace::Ray,
};

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

#[derive(Deserialize)]
pub struct Light {
    pub position: Vector3,
    pub color: Color,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Object {
    Sphere(Sphere),
    Plane(Plane),
    Mesh(Mesh),
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: serde_yaml::Value = serde_yaml::Value::deserialize(deserializer).unwrap();
        let path = std::path::Path::new(val.get("path").unwrap().as_str().unwrap());
        let meshes = load_obj(path);
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
            point,
            normal,
            material,
            t,
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
            Object::Mesh(mesh) => mesh.intersect(ray),
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

/// Calculates the determinant of a matrix represented by three column vectors.
///
/// Following the formula of:
/// | a b c |
/// | d e f |
/// | g h i |
/// det = (aei + bfg + cdh) - (ceg + bdi + afh)
///
/// which leads to
/// det = (v1.x * v2.y * v3.z + v2.x * v3.y * v1.z + v3.x * v1.y * v2.z) - (v3.x * v2.y * v1.z + v2.x * v1.y * v3.z + v1.x * v3.y * v2.z)
///
/// # Arguments
///
/// * `v1` Vector representing the left column
/// * `v2` Vector representing the middle column
/// * `v3` Vector representing the right column
#[inline]
fn calculate_determinant(v1: &Vector3, v2: &Vector3, v3: &Vector3) -> f64 {
    (v1.x * v2.y * v3.z + v2.x * v3.y * v1.z + v3.x * v1.y * v2.z)
        - (v3.x * v2.y * v1.z + v2.x * v1.y * v3.z + v1.x * v3.y * v2.z)
}

impl Intersectable for Mesh {
    /// Intersection testing of a mesh happens in two steps:
    /// - test the AABB of the mesh (TODO)
    /// - test each triangle of the mesh and find the closest intersection (if any exist)
    ///
    /// Triangle intersection is implemented via barycentric coordinates.
    /// For a triangle constructed by the points `a`, `b`, `c` and a ray with origin `o` and direction `d`
    /// the equation `o + td = alpha * a + beta * b + (1 - alpha - beta) * c` has to be solved.
    /// This is done by using Cramers-Rule after rearranging the equation to:
    /// `[ d | (b-a) | (c-a) ] = (-t, alpha, beta)^T`
    /// The Matrix on the left hand side is represented as three column vectors.
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let mut result: Option<IntersectionInfo> = None;
        for triangle in &self.triangles {
            let pos_idx = triangle.vertex_idx;
            let a = self.vertex_positions[pos_idx[0]];
            let b = self.vertex_positions[pos_idx[1]];
            let c = self.vertex_positions[pos_idx[2]];
            let ab = b - a;
            let ac = c - a;

            let res = ray.origin - a;
            let det_m = calculate_determinant(&ray.direction, &ab, &ac);
            let det_m_t = calculate_determinant(&res, &ab, &ac);
            let det_m_a = calculate_determinant(&ray.direction, &res, &ac);
            let det_m_b = calculate_determinant(&ray.direction, &ab, &res);

            let a = det_m_a / det_m;
            let b = det_m_b / det_m;
            let t = -(det_m_t / det_m);

            if a < 0.0 || b < 0.0 || a + b > 1.0 || t < 0.0 {
                continue;
            }
            let normal = ab.cross(&ac).normalized();
            if result.is_none() || result.unwrap().t > t {
                result = Some(IntersectionInfo::new(
                    ray.at_timestep(t),
                    normal,
                    self.materials[triangle.material_idx],
                    t,
                ));
            }
        }

        result
    }
}
