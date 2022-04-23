use crate::math;
use crate::raytracer::raytrace::Ray;

pub struct Camera {
    eye: math::Vector3,
    horizontal: math::Vector3,
    vertical: math::Vector3,
    lower_left: math::Vector3,
}

impl Camera {
    pub fn new(
        eye: math::Vector3,
        look_at: math::Vector3,
        up: math::Vector3,
        fovy: f64,
        width: usize,
        height: usize,
    ) -> Camera {
        let view_vec = look_at - eye;
        let distance = view_vec.len();
        let view = view_vec / distance;

        let image_height = 2.0 * distance * (0.5 * fovy / 180.0 * std::f64::consts::PI).tan();
        let image_width = width as f64 / height as f64 * image_height;

        let horizontal = view.cross(&up).normalized() * image_width / width as f64;
        let vertical = horizontal.cross(&view).normalized() * image_height / height as f64;

        let lower_left =
            look_at - horizontal * (0.5 * width as f64) - vertical * (0.5 * height as f64);

        Camera {
            eye: eye,
            horizontal: horizontal,
            vertical: vertical,
            lower_left: lower_left,
        }
    }

    pub fn spawn_ray(&self, x: f64, y: f64) -> Ray {
        Ray::new(
            self.eye,
            self.lower_left + self.horizontal * x + self.vertical * y - self.eye,
        )
    }
}
