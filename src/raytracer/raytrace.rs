use crate::math;
use crate::raytracer::camera;
use crate::raytracer::image;
use crate::raytracer::scene;
use serde_yaml;
use std::fs;
use std::path;

pub struct Ray {
    origin: math::Vector3,
    direction: math::Vector3,
}

impl Ray {
    pub fn new(origin: math::Vector3, direction: math::Vector3) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
        }
    }

    fn at_timestep(&self, t: f64) -> math::Vector3 {
        self.origin + self.direction * t
    }

    fn trace(&self) {}
}

pub fn compute_image(scene_path: &path::Path, output_path: &path::Path) {
    let scene_file = fs::File::open(scene_path).unwrap();
    let scene: scene::Scene = serde_yaml::from_reader(scene_file).unwrap();

    let mut img = image::Image::new(scene.image.width, scene.image.height);
    let camera = camera::Camera::new(
        math::Vector3::new(0.0, 0.0, 0.0),
        math::Vector3::new(0.0, 0.0, -5.0),
        math::Vector3::new(0.0, 1.0, 0.0),
        50.0,
        scene.image.width,
        scene.image.height,
    );
    for i in 0..scene.image.width {
        for j in 0..scene.image.height {
            let mut ray = camera.spawn_ray(i as f64, j as f64);
        }
    }
}
