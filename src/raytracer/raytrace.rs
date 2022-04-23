use crate::math;
use crate::raytracer::camera;
use crate::raytracer::image;

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
}

pub fn compute_image(width: usize, height: usize) {
    let mut img = image::Image::new(width, height);
    let camera = camera::Camera::new(
        math::Vector3::new(0.0, 0.0, 0.0),
        math::Vector3::new(0.0, 0.0, -5.0),
        math::Vector3::new(0.0, 1.0, 0.0),
        50.0,
        width,
        height,
    );
    for i in 0..width {
        for j in 0..height {
            let mut ray = camera.spawn_ray(i as f64, j as f64);
        }
    }
}
