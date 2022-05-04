use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

use crate::{
    math::Vector3,
    raytracer::{image::Color, raytrace::Ray},
};

use super::{
    intersections::{Intersectable, IntersectionInfo},
    materials::Material,
    mesh::{self, Mesh},
};

#[derive(Deserialize)]
pub struct Scene {
    pub camera: CameraConfig,
    pub width: usize,
    pub height: usize,
    pub background: Color,
    #[serde(skip_deserializing)]
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

    pub fn precompute(&mut self) {
        for o in &mut self.objects {
            if let Object::Mesh(mesh) = o {
                mesh.compute_aabb();
            }
            if o.is_light() {
                self.lights.push(Light::from(&*o));
            }
        }
    }
}

#[derive(Deserialize)]
pub struct CameraConfig {
    pub eye: Vector3,
    pub look_at: Vector3,
    pub up: Vector3,
    pub fovy: f64,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Object {
    Sphere(Sphere),
    Plane(Plane),
    Mesh(Mesh),
}

impl Object {
    fn is_light(&self) -> bool {
        match self {
            Object::Sphere(sphere) => match sphere.material {
                Material::Emissive(_) => true,
                _ => false,
            },
            Object::Plane(plane) => match plane.material {
                Material::Emissive(_) => true,
                _ => false,
            },
            Object::Mesh(mesh) => {
                for material in &mesh.materials {
                    if let Material::Emissive(_) = material {
                        return true;
                    }
                }
                false
            }
        }
    }
}

pub struct Light {
    pub color: Color,
    pub sample_points: Vec<Vector3>,
}

impl Light {
    fn new(color: Color, sample_points: Vec<Vector3>) -> Light {
        Light {
            color,
            sample_points,
        }
    }
}

impl From<&Object> for Light {
    fn from(o: &Object) -> Self {
        match o {
            Object::Plane(p) => match &p.material {
                Material::Emissive(e) => Light::new(e.color, vec![p.center]),
                _ => Light::new(Color::new(1.0, 1.0, 1.0), vec![p.center]),
            },
            Object::Sphere(s) => match &s.material {
                Material::Emissive(e) => {
                    let mut sample1 = s.center.clone();
                    sample1[0] += s.radius;
                    let mut sample2 = s.center.clone();
                    sample2[0] -= s.radius;
                    Light::new(e.color, vec![s.center, sample1, sample2])
                }
                _ => Light::new(Color::new(1.0, 1.0, 1.0), vec![s.center]),
            },
            Object::Mesh(m) => {
                let mut color = Color::new(0.0, 0.0, 0.0);
                let mut sample_positions = Vec::new();
                for triangle in &m.triangles {
                    if let Material::Emissive(e) = &m.materials[triangle.material_idx] {
                        color += e.color;
                        sample_positions.push(m.vertex_positions[triangle.vertex_idx[0]]);
                    }
                }
                Light::new(color, sample_positions)
            }
        }
    }
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
        let materials = val.get("materials").unwrap();

        let materials: HashMap<String, Material> =
            serde_yaml::from_value(materials.clone()).unwrap();
        let meshes = mesh::load_obj(path, &materials);
        Ok(meshes[0].to_owned())
    }
}

#[derive(Deserialize, Clone)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Material,
}

#[derive(Deserialize, Clone)]
pub struct Plane {
    pub center: Vector3,
    pub normal: Vector3,
    pub material: Material,
}

#[cfg(test)]
mod test {
    use crate::{
        math::Vector3,
        raytracer::{
            image::Color,
            scene::materials::{LambertianMaterial, Material},
        },
    };

    use super::{Object, Plane, Scene};

    #[test]
    fn test_should_color() {
        // let light_source = AreaLight {
        //     corner: Vector3::new(5.0, 0.0, 0.0),
        //     u: Vector3::new(0.0, 0.0, 5.0),
        //     v: Vector3::new(0.0, 5.0, 0.0),
        //     grid_resolution: 2,
        //     deterministic: true,
        // };
        // let samples = LightInfo::Area(light_source).sample();
        // let plane = Plane {
        //     center: Vector3::new(5.0, 0.0, 2.5),
        //     normal: Vector3::new(0.0, 0.0, -1.0),
        //     material: Material::Lambertian(LambertianMaterial::new(Color::new(0.0, 0.0, 0.0), 0.5)),
        // };
        // let point = Vector3::new(0.0, 0.0, 0.0);

        // let scene = Scene {
        //     ambient_light: Color::default(),
        //     lights: Vec::new(),
        //     objects: vec![Object::Plane(plane)],
        // };

        // let mut negative_count = 0;
        // for s in samples {
        //     if !scene.should_color(&point, &s) {
        //         negative_count += 1;
        //     }
        // }

        // assert_eq!(negative_count, 2);
    }
}
