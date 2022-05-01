use rand::Rng;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Deserializer};

use crate::{
    math::Vector3,
    raytracer::{image::Color, raytrace::Ray},
};

use super::{
    intersections::{Intersectable, IntersectionInfo},
    mesh::{self, Mesh},
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

    /// Returns whether a given point is affected by the light at `light_pos` and should be colored with diffuse and specular lighting.
    ///
    /// Depends on whether the point is being shadowed by another object.
    /// For a light `l` and point `p` the ray is constructed as `origin = p` and `direction = ||l.position - p||`.
    /// If `p` is being shadowed there has to be an intersection `i` with object `o` where `||l.position - p|| > ||l.position - i.position||`
    ///
    /// # Arguments
    ///
    /// * `point` the point to check
    /// * `light_pos` position of the light
    #[inline]
    fn should_color(&self, point: &Vector3, light_pos: &Vector3) -> bool {
        let lp_vec = *light_pos - *point;
        let ray = Ray::new(*point, lp_vec);
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
    pub fn compute_phong_shading(
        &self,
        point: &Vector3,
        normal: &Vector3,
        view: &Vector3,
        material: &Material,
    ) -> Color {
        let mut c = material.ambient_color * self.ambient_light;

        for l in &self.lights {
            let indices: Vec<usize> = (0..l.samples.len())
                .into_par_iter()
                .filter(|idx| {
                    let light_point = &l.samples[*idx];
                    self.should_color(point, light_point)
                })
                .collect();
            let intensity = indices.len() as f64 / l.samples.len() as f64;
            let mut l_color = indices
                .par_iter()
                .map(|idx| {
                    let l_vec = &l.samples[*idx];
                    let mut light_color = Color::new(0.0, 0.0, 0.0);
                    let lp_vec = *l_vec - *point;
                    let lp_vec_normalized = lp_vec.normalized();
                    let r = lp_vec_normalized.mirror(normal);
                    let dot_l = normal.dot(&lp_vec_normalized);
                    if dot_l >= 0.0 {
                        light_color += l.color * intensity * (material.diffuse_color * dot_l);

                        let dot_r = view.dot(&r);
                        if dot_r >= 0.0 {
                            let shininess = dot_r.powf(material.shininess);
                            light_color +=
                                material.specular_color * l.color * intensity * shininess;
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
}

impl LightInfo {
    fn sample(&self) -> Vec<Vector3> {
        match self {
            LightInfo::Point(pl) => vec![pl.position],
            LightInfo::Area(area_light) => {
                let resolution = area_light.grid_resolution;
                let mut result = Vec::with_capacity(resolution * resolution);
                let mut rng = rand::thread_rng();
                let resolution_f = resolution as f64;
                let u_step = area_light.u / resolution_f;
                let v_step = area_light.v / resolution_f;
                for i in 0..resolution {
                    for j in 0..resolution {
                        let i_f = i as f64;
                        let j_f = j as f64;
                        if area_light.deterministic {
                            result.push(
                                area_light.corner + u_step * (i_f + 0.5) + v_step * (j_f + 0.5),
                            );
                        } else {
                            result.push(
                                area_light.corner
                                    + u_step * (i_f + rng.gen_range(0.0..1.0))
                                    + v_step * (j_f + rng.gen_range(0.0..1.0)),
                            )
                        }
                    }
                }
                result
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
    deterministic: bool,
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

#[cfg(test)]
mod test {
    use crate::{math::Vector3, raytracer::image::Color};

    use super::{AreaLight, LightInfo, Material, Object, Plane, Scene};

    #[test]
    fn test_should_color() {
        let light_source = AreaLight {
            corner: Vector3::new(5.0, 0.0, 0.0),
            u: Vector3::new(0.0, 0.0, 5.0),
            v: Vector3::new(0.0, 5.0, 0.0),
            grid_resolution: 2,
            deterministic: true,
        };
        let samples = LightInfo::Area(light_source).sample();
        let plane = Plane {
            center: Vector3::new(5.0, 0.0, 2.5),
            normal: Vector3::new(0.0, 0.0, -1.0),
            material: Material::default(),
        };
        let point = Vector3::new(0.0, 0.0, 0.0);

        let scene = Scene {
            ambient_light: Color::default(),
            lights: Vec::new(),
            objects: vec![Object::Plane(plane)],
        };

        let mut negative_count = 0;
        for s in samples {
            if !scene.should_color(&point, &s) {
                negative_count += 1;
            }
        }

        assert_eq!(negative_count, 2);
    }
}
