use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, path::Path};

use crate::{
    math::Vector3,
    raytracer::{
        image::{self, Color},
        raytrace::Ray,
    },
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
                for mat in &mut mesh.materials {
                    if let Material::Texture(tm) = mat {
                        let (pixels, width, height) =
                            image::read_image(Path::new(&tm.texture_path));
                        tm.pixel_colors = pixels;
                        tm.width = width as f64;
                        tm.height = height as f64;
                    }
                }
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
    pub sample_points: Vec<Vector3>,
}

impl Light {
    fn new(sample_points: Vec<Vector3>) -> Light {
        Light { sample_points }
    }
}

impl From<&Object> for Light {
    fn from(o: &Object) -> Self {
        match o {
            Object::Plane(p) => Light::new(vec![p.center]),
            Object::Sphere(s) => Light::new(vec![s.center]),
            Object::Mesh(m) => {
                let mut color = Color::new(0.0, 0.0, 0.0);
                let mut sample_positions = Vec::new();
                for triangle in &m.triangles {
                    if let Material::Emissive(e) = &m.materials[triangle.material_idx] {
                        color += e.color;
                        let interpolated = (m.vertex_positions[triangle.vertex_idx[0]]
                            + m.vertex_positions[triangle.vertex_idx[1]]
                            + m.vertex_positions[triangle.vertex_idx[2]])
                            / 3.0;
                        sample_positions.push(interpolated);
                    }
                }
                Light::new(sample_positions)
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
            raytrace::Ray,
            scene::{
                materials::{EmissiveMaterial, LambertianMaterial, Material},
                mesh::{Mesh, Triangle},
                Object,
            },
        },
    };

    use super::{Light, Plane, Scene, Sphere};

    #[test]
    fn test_closest_intersection() {
        let mut scene = Scene {
            background: Color::new(0.0, 0.0, 0.0),
            camera: super::CameraConfig {
                eye: Vector3::new(0.0, 0.0, 0.0),
                look_at: Vector3::new(0.0, 0.0, 0.0),
                up: Vector3::new(0.0, 0.0, 0.0),
                fovy: 0.0,
            },
            height: 10,
            width: 10,
            lights: Vec::new(),
            objects: Vec::new(),
        };
        let material = Material::Emissive(EmissiveMaterial::new(Color::new(0.0, 0.0, 0.0)));
        let sphere1 = Object::Sphere(Sphere {
            center: Vector3::new(5.0, 0.0, 0.0),
            radius: 1.0,
            material: material.clone(),
        });
        scene.objects.push(sphere1);
        let sphere2 = Object::Sphere(Sphere {
            center: Vector3::new(5.0, 0.0, 0.0),
            radius: 1.5,
            material: material.clone(),
        });
        scene.objects.push(sphere2);
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        let intersection = scene.get_closest_interesection(&ray);

        assert!(intersection.is_some());
        if let Some(intersection) = intersection {
            assert_eq!(intersection.t, 3.5);
        }
    }

    fn create_test_objects(material: &Material) -> (Object, Object, Object) {
        let sphere = Object::Sphere(Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 0.0,
            material: material.clone(),
        });
        let plane = Object::Plane(Plane {
            center: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            material: material.clone(),
        });
        let mut mesh = Mesh::new();
        mesh.vertex_positions.push(Vector3::new(3.0, 0.0, 0.0));
        mesh.vertex_positions.push(Vector3::new(0.0, 3.0, 0.0));
        mesh.vertex_positions.push(Vector3::new(0.0, 0.0, 3.0));
        mesh.materials.push(material.clone());
        let triangle = Triangle::new([0, 1, 2], 0);
        mesh.triangles.push(triangle);
        let mesh = Object::Mesh(mesh);
        (sphere, plane, mesh)
    }

    #[test]
    fn test_is_lights() {
        let material = Material::Emissive(EmissiveMaterial::new(Color::new(0.0, 0.0, 0.0)));
        let material_neg = Material::Lambertian(LambertianMaterial::new(Color::new(0.0, 0.0, 0.0)));

        let (sphere, plane, mesh) = create_test_objects(&material);
        let (sphere_neg, plane_neg, mesh_neg) = create_test_objects(&material_neg);

        assert!(sphere.is_light());
        assert!(plane.is_light());
        assert!(mesh.is_light());
        assert!(!sphere_neg.is_light());
        assert!(!plane_neg.is_light());
        assert!(!mesh_neg.is_light());
    }

    #[test]
    fn test_light_from_object() {
        let material = Material::Emissive(EmissiveMaterial::new(Color::new(0.0, 0.0, 0.0)));
        let (sphere, plane, mesh) = create_test_objects(&material);

        let sphere = Light::from(&sphere);
        let plane = Light::from(&plane);
        let mesh = Light::from(&mesh);

        assert_eq!(sphere.sample_points.len(), 1);
        assert_eq!(sphere.sample_points[0], Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(plane.sample_points.len(), 1);
        assert_eq!(plane.sample_points[0], Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(mesh.sample_points.len(), 1);
        assert_eq!(mesh.sample_points[0], Vector3::new(1.0, 1.0, 1.0));
    }
}
